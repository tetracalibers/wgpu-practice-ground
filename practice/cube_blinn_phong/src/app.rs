use std::{sync::Arc, time};

use anyhow::Result;
use winit::{
  application::ApplicationHandler,
  event::{ElementState, KeyEvent, WindowEvent},
  event_loop::{ActiveEventLoop, EventLoop},
  keyboard::{KeyCode, PhysicalKey},
  window::{Window, WindowId},
};

use crate::render::Render;

pub struct App<'a, R>
where
  R: Render,
{
  window: Option<Arc<Window>>,
  window_title: &'a str,
  draw_data: R::DrawData,
  initial_state: R::InitialState,
  renderer: Option<R>,
  render_start_time: Option<time::Instant>,
}

impl<'a, R: Render> App<'a, R> {
  pub fn new(
    window_title: &'a str,
    draw_data: R::DrawData,
    initial_state: R::InitialState,
  ) -> Self {
    Self {
      window: None,
      window_title,
      draw_data,
      initial_state,
      renderer: None,
      render_start_time: None,
    }
  }

  pub fn run(&mut self) -> Result<()> {
    let event_loop = EventLoop::builder().build()?;
    event_loop.run_app(self)?;

    Ok(())
  }

  fn window(&self) -> Option<&Window> {
    match &self.window {
      Some(window) => Some(window.as_ref()),
      None => None,
    }
  }
}

impl<'a, R: Render> ApplicationHandler for App<'a, R> {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let window_attributes =
      Window::default_attributes().with_title(self.window_title);
    let window = event_loop.create_window(window_attributes).unwrap();
    self.window = Some(Arc::new(window));

    let renderer = R::new(
      Arc::clone(self.window.as_ref().unwrap()),
      &self.draw_data,
      &self.initial_state,
    );
    let renderer = pollster::block_on(renderer);
    self.renderer = Some(renderer);

    self.render_start_time = Some(time::Instant::now());
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    window_id: WindowId,
    event: WindowEvent,
  ) {
    let binding = self.window();
    let window = match &binding {
      Some(window) => window,
      None => return,
    };
    if window.id() != window_id {
      return;
    }

    let renderer = match &mut self.renderer {
      Some(renderer) => renderer,
      None => return,
    };
    if renderer.process_event(&event) {
      return;
    }

    match event {
      WindowEvent::Resized(size) => {
        renderer.resize(size);
      }
      WindowEvent::RedrawRequested => {
        let now = time::Instant::now();
        let dt = now - self.render_start_time.unwrap_or(now);
        renderer.update(dt);

        match renderer.draw() {
          Ok(()) => {}
          Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.get_size()),
          Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
          Err(e) => eprintln!("{:?}", e),
        }
      }
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      WindowEvent::KeyboardInput {
        event:
          KeyEvent {
            physical_key: PhysicalKey::Code(KeyCode::Escape),
            state: ElementState::Pressed,
            ..
          },
        ..
      } => {
        event_loop.exit();
      }
      _ => {}
    }
  }

  fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    let binding = self.window();
    let window = match &binding {
      Some(window) => window,
      None => return,
    };
    window.request_redraw();
  }
}
