use std::sync::Arc;

use winit::{
  application::ApplicationHandler,
  event::{ElementState, KeyEvent, WindowEvent},
  event_loop::ActiveEventLoop,
  keyboard::{KeyCode, PhysicalKey},
  window::{Window, WindowId},
};

use crate::state::State;

#[derive(Default)]
pub struct Application<'w> {
  state: Option<State<'w>>,
}

impl<'w> ApplicationHandler for Application<'w> {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let window_attributes =
      Window::default_attributes().with_title("Hello, World!");

    let window = event_loop
      .create_window(window_attributes)
      .expect("Failed to create window");

    let state =
      pollster::block_on(async { State::new(Arc::new(window)).await });
    self.state = Some(state);
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    _window_id: WindowId,
    event: WindowEvent,
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
      WindowEvent::Resized(physical_size) => {
        if let Some(ref mut state) = self.state {
          state.resize(physical_size);
        }
      }
      _ => {}
    }
  }
}
