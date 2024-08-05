use winit::{
  application::ApplicationHandler,
  dpi::LogicalSize,
  event::{ElementState, KeyEvent, WindowEvent},
  event_loop::ActiveEventLoop,
  keyboard::{KeyCode, PhysicalKey},
  window::{Window, WindowId},
};

use crate::state::GfxState;

#[derive(Default)]
pub struct Application<'a> {
  state: Option<GfxState<'a>>,
}

impl<'a> ApplicationHandler for Application<'a> {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let window_attributes = Window::default_attributes()
      .with_title("Prototype: Rect Renderer")
      .with_inner_size(LogicalSize::new(400.0, 300.0));

    let window = event_loop
      .create_window(window_attributes)
      .expect("Failed to create window");

    let state = pollster::block_on(async { GfxState::new(window).await });

    self.state = Some(state);
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    _window_id: WindowId,
    event: WindowEvent,
  ) {
    let state = match &mut self.state {
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
    if let Some(state) = &self.state {
      state.window().request_redraw();
    }
  }
}
