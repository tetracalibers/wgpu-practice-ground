use std::sync::Arc;

use winit::{
  application::ApplicationHandler,
  dpi::LogicalSize,
  event::{ElementState, KeyEvent, WindowEvent},
  event_loop::ActiveEventLoop,
  keyboard::{KeyCode, PhysicalKey},
  window::Window,
};

use crate::state::WindowState;

#[derive(Default)]
pub struct Application<'a> {
  window_state: Option<WindowState<'a>>,
  font_atlas_size: (u32, u32),
  font_atlas_data: Vec<u8>,
  char_rects: Vec<(f32, f32, f32, f32)>,
  origin: [f32; 2],
  color: [f32; 4],
  uvs: Vec<[f32; 4]>,
}

impl<'a> Application<'a> {
  pub fn new(
    font_atlas_size: (u32, u32),
    font_atlas_data: Vec<u8>,
    char_rects: Vec<(f32, f32, f32, f32)>,
    origin: [f32; 2],
    color: [f32; 4],
    uvs: Vec<[f32; 4]>,
  ) -> Self {
    Application {
      window_state: None,
      font_atlas_size,
      font_atlas_data,
      char_rects,
      origin,
      color,
      uvs,
    }
  }
}

impl<'a> ApplicationHandler for Application<'a> {
  fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
    if self.window_state.is_some() {
      return;
    }

    let (width, height) = (800, 600);
    let window_attributes = Window::default_attributes()
      .with_inner_size(LogicalSize::new(width as f64, height as f64))
      .with_title("Prototype: Text Renderer");
    let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

    let mut state = pollster::block_on(WindowState::new(window));
    state.set_font(self.font_atlas_size, &self.font_atlas_data);
    state.set_geometry(&self.char_rects, self.origin, self.color, &self.uvs);

    self.window_state = Some(state);
  }

  fn window_event(
    &mut self,
    event_loop: &winit::event_loop::ActiveEventLoop,
    _window_id: winit::window::WindowId,
    event: winit::event::WindowEvent,
  ) {
    let state = match &mut self.window_state {
      Some(state) => state,
      None => return,
    };

    match event {
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
      WindowEvent::RedrawRequested => match state.render() {
        Ok(_) => {}
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
          state.resize(state.size());
        }
        Err(wgpu::SurfaceError::OutOfMemory) => {
          println!("Out of memory");
          event_loop.exit();
        }
        Err(wgpu::SurfaceError::Timeout) => {
          println!("Surface timeout");
        }
      },
      _ => {}
    }
  }

  fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    if let Some(state) = &self.window_state {
      state.window.request_redraw();
    }
  }
}
