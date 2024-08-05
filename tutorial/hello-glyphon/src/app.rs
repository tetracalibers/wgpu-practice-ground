use std::sync::Arc;

use winit::{
  application::ApplicationHandler,
  dpi::LogicalSize,
  event::{ElementState, KeyEvent, WindowEvent},
  keyboard::{KeyCode, PhysicalKey},
  window::Window,
};

use crate::state::WindowState;

#[derive(Default)]
pub struct Application {
  window_state: Option<WindowState>,
}

impl ApplicationHandler for Application {
  fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
    if self.window_state.is_some() {
      return;
    }

    let (width, height) = (800, 600);
    let window_attributes = Window::default_attributes()
      .with_inner_size(LogicalSize::new(width as f64, height as f64))
      .with_title("Hello, glyphon!");
    let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

    self.window_state = Some(pollster::block_on(WindowState::new(window)));
  }

  fn window_event(
    &mut self,
    event_loop: &winit::event_loop::ActiveEventLoop,
    _window_id: winit::window::WindowId,
    event: winit::event::WindowEvent,
  ) {
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
      _ => {}
    }
  }
}
