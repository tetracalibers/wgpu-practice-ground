use wgpu::{Device, Queue, Surface, SurfaceConfiguration, SurfaceError};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

struct State<'window> {
  surface: Surface<'window>,
  device: Device,
  queue: Queue,
  config: SurfaceConfiguration,
  size: PhysicalSize<u32>,
  window: &'window Window,
}

impl<'w> State<'w> {
  async fn new(window: &Window) -> State {
    todo!()
  }

  pub fn window(&self) -> &Window {
    todo!()
  }

  fn resize(&mut self, new_size: PhysicalSize<u32>) {
    todo!()
  }

  fn input(&mut self, event: &WindowEvent) -> bool {
    todo!()
  }

  fn update(&mut self) {
    todo!()
  }

  fn render(&mut self) -> Result<(), SurfaceError> {
    todo!()
  }
}
