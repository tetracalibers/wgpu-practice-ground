use std::{future::Future, sync::Arc, time};

use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

#[allow(opaque_hidden_inferred_bound)]
pub trait Render {
  type DrawData;
  type InitialState;

  fn new(
    window: Arc<Window>,
    draw_data: &Self::DrawData,
    initial_state: &Self::InitialState,
  ) -> impl Future<Output = Self>;
  fn get_size(&self) -> PhysicalSize<u32>;
  fn resize(&mut self, size: PhysicalSize<u32>);
  fn process_event(&mut self, event: &WindowEvent) -> bool;
  fn update(&mut self, dt: time::Duration);
  fn draw(&mut self) -> Result<(), wgpu::SurfaceError>;
}
