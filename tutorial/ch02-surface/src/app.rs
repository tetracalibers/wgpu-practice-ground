use std::collections::HashMap;

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
  windows: HashMap<WindowId, State<'w>>,
}

impl<'w> ApplicationHandler for Application<'w> {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let window_attributes =
      Window::default_attributes().with_title("Hello, World!");

    let window = event_loop
      .create_window(window_attributes)
      .expect("Failed to create window");

    let state = pollster::block_on(async { State::new(window).await });

    let window_id = state.window().id();
    self.windows.insert(window_id, state);
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    window_id: WindowId,
    event: WindowEvent,
  ) {
    let window_state = match self.windows.get_mut(&window_id) {
      Some(state) => state,
      None => return,
    };

    if window_state.input(&event) {
      return;
    }

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
        window_state.resize(physical_size);
      }
      _ => {}
    }
  }
}
