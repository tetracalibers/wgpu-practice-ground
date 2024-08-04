use crate::{
  color::Color,
  geometry_value::{Bounds, Corners},
};

struct Rect {
  color: Color,
  bounds: Bounds<f32>,
  corners: Corners<f32>,
  sigma: f32,
}

pub struct UiRenderer {
  device: wgpu::Device,
  target_view: wgpu::TextureView,
}

impl UiRenderer {
  pub fn new(device: &wgpu::Device, target_view: wgpu::TextureView) -> Self {
    todo!()
  }

  pub fn rectangle(&self, rect: Rect) {
    todo!()
  }

  pub fn render(&self) {
    todo!()
  }
}
