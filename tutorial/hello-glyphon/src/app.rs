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
}

impl<'a> ApplicationHandler for Application<'a> {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window_state.is_some() {
      return;
    }

    let (width, height) = (400, 200);
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
    let Some(state) = &mut self.window_state else {
      return;
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
      WindowEvent::RedrawRequested => state.update_view().unwrap(),
      _ => {}
    }
  }

  fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    let Some(state) = &mut self.window_state else {
      return;
    };
    let WindowState { window, .. } = state;
    window.request_redraw();
  }
}
