use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

pub struct App<'a> {
  window: Option<Arc<Window>>,
  window_title: &'a str,
  window_size: Option<LogicalSize<u32>>,
}

impl<'a> App<'a> {
  pub fn new(window_title: &'a str) -> Self {
    Self {
      window: None,
      window_title,
      window_size: None,
    }
  }

  pub fn with_window_size(mut self, width: u32, height: u32) -> Self {
    self.window_size = Some(LogicalSize::new(width, height));
    self
  }

  fn window(&self) -> Option<&Window> {
    match &self.window {
      Some(window) => Some(window.as_ref()),
      None => None,
    }
  }
}

impl<'a> ApplicationHandler for App<'a> {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let mut window_attributes =
      Window::default_attributes().with_title(self.window_title);

    if let Some(window_size) = self.window_size {
      window_attributes = window_attributes.with_max_inner_size(window_size);
    }

    let window = event_loop.create_window(window_attributes).unwrap();
    self.window = Some(Arc::new(window));
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    window_id: WindowId,
    event: WindowEvent,
  ) {
    let window = self.window();
    let window = match &window {
      Some(window) => window,
      None => return,
    };
    if window.id() != window_id {
      return;
    }

    match event {
      WindowEvent::Resized(_new_size) => {
        // resize
      }
      WindowEvent::RedrawRequested => {
        // update & render
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
    let window = self.window();
    let window = match &window {
      Some(window) => window,
      None => return,
    };
    window.request_redraw();
  }
}
