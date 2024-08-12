use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

use crate::renderer::{FontData, UiRenderer};

pub struct WindowState<'a> {
  pub window: Arc<Window>,
  size: PhysicalSize<u32>,
  surface: wgpu::Surface<'a>,
  config: wgpu::SurfaceConfiguration,
  device: wgpu::Device,
  queue: wgpu::Queue,
  ui: UiRenderer,
  font_data: Option<FontData>,
}

impl<'a> WindowState<'a> {
  pub async fn new(window: Arc<Window>) -> Self {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::all(),
      ..Default::default()
    });

    let surface = instance.create_surface(Arc::clone(&window)).unwrap();

    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
      })
      .await
      .unwrap();

    let (device, queue) = adapter
      .request_device(&wgpu::DeviceDescriptor::default(), None)
      .await
      .unwrap();

    let size = window.inner_size();

    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: wgpu::TextureFormat::Bgra8Unorm,
      width: size.width,
      height: size.height,
      present_mode: wgpu::PresentMode::Fifo,
      alpha_mode: wgpu::CompositeAlphaMode::Opaque,
      view_formats: vec![],
      desired_maximum_frame_latency: 2,
    };

    surface.configure(&device, &config);

    let ui = UiRenderer::new(&device, &config);

    WindowState {
      window,
      size,
      surface,
      config,
      device,
      queue,
      ui,
      font_data: None,
    }
  }

  pub fn size(&self) -> PhysicalSize<u32> {
    self.size
  }

  pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
      self.ui.set_viewport_size(new_size.width, new_size.height);
    }
  }

  pub fn set_font(
    &mut self,
    font_atlas_size: (u32, u32),
    font_atlas_data: &Vec<u8>,
  ) {
    let font_data = self.ui.init_font(
      &self.device,
      &self.queue,
      font_atlas_size,
      font_atlas_data,
    );
    self.font_data = Some(font_data);
  }

  pub fn set_geometry(
    &mut self,
    char_rects: &Vec<(f32, f32, f32, f32)>,
    origin: [f32; 2],
    font_size: f32,
    color: [f32; 4],
    uvs: &Vec<[f32; 4]>,
  ) {
    self.ui.text(char_rects, origin, font_size, color, uvs);
  }

  pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    let surface_texture = self.surface.get_current_texture()?;
    let view = surface_texture
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder =
      self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("render blurred rectangles"),
      });

    self.ui.render(
      &mut encoder,
      &self.queue,
      &view,
      &self.font_data.as_ref().unwrap(),
    );

    self.queue.submit(std::iter::once(encoder.finish()));
    surface_texture.present();

    Ok(())
  }
}
