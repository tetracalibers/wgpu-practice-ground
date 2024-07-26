use winit::{
  application::ApplicationHandler,
  event::{ElementState, KeyEvent, WindowEvent},
  keyboard::{KeyCode, PhysicalKey},
  window::Window,
};

use crate::state::State;

#[derive(Default)]
pub struct Application {
  state: Option<State>,
}

impl ApplicationHandler for Application {
  fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
    let window_attributes =
      Window::default_attributes().with_title("Life Game");

    let window = event_loop
      .create_window(window_attributes)
      .expect("Failed to create window");

    let state = pollster::block_on(async { State::new(window).await });

    self.state = Some(state);
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
