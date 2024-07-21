use std::sync::Arc;

use winit::{
  application::ApplicationHandler, event_loop::ActiveEventLoop, window::Window,
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
    window_id: winit::window::WindowId,
    event: winit::event::WindowEvent,
  ) {
    todo!()
  }
}
