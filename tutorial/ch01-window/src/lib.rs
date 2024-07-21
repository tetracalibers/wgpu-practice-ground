use std::error::Error;

use winit::{
  application::ApplicationHandler,
  event::WindowEvent,
  event_loop::{ActiveEventLoop, EventLoop},
  window::{Window, WindowId},
};

pub fn run() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  let event_loop = EventLoop::builder().build()?;
  let mut app = Application::default();

  event_loop.run_app(&mut app)?;

  Ok(())
}

#[derive(Default)]
struct Application {
  window: Option<Window>,
}

impl ApplicationHandler for Application {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let window_attributes =
      Window::default_attributes().with_title("Hello, World!");

    let window = event_loop
      .create_window(window_attributes)
      .expect("Failed to create window");

    self.window = Some(window);
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
      _ => {}
    }
  }
}
