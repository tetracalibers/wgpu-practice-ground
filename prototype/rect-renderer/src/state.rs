use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

use crate::renderer::UiRenderer;

pub struct GfxState<'a> {
  window: Arc<Window>,
  size: PhysicalSize<u32>,
  surface: wgpu::Surface<'a>,
  config: wgpu::SurfaceConfiguration,
  device: wgpu::Device,
  queue: wgpu::Queue,
  ui: UiRenderer,
}

impl<'a> GfxState<'a> {
  pub async fn new(window: Window) -> Self {
    let window = Arc::new(window);

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

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
      .formats
      .iter()
      .find(|format| format.is_srgb())
      .copied()
      .unwrap_or(surface_caps.formats[0]);

    let size = window.inner_size();

    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width,
      height: size.height,
      present_mode: wgpu::PresentMode::Fifo,
      alpha_mode: wgpu::CompositeAlphaMode::Opaque,
      view_formats: vec![],
      desired_maximum_frame_latency: 2,
    };

    surface.configure(&device, &config);

    let ui = UiRenderer::new(&device, config.format);

    Self {
      window,
      size,
      surface,
      config,
      device,
      queue,
      ui,
    }
  }

  pub fn window(&self) -> &Window {
    &self.window
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
    }
  }

  pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
    let surface_texture = self.surface.get_current_texture()?;
    let view = surface_texture
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder =
      self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("render blurred rectangles"),
      });

    self.ui.render(&mut encoder, &view);

    self.queue.submit(std::iter::once(encoder.finish()));
    surface_texture.present();

    Ok(())
  }
}
