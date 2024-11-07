use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

use crate::surface_cfg::SurfaceConfigBuilder;

#[derive(Debug, Clone, Copy)]
pub struct Size {
  pub width: u32,
  pub height: u32,
}

impl Size {
  pub fn new(width: u32, height: u32) -> Self {
    Self { width, height }
  }
}

impl Into<Size> for PhysicalSize<u32> {
  fn into(self) -> Size {
    Size {
      width: self.width,
      height: self.height,
    }
  }
}

#[derive(Debug)]
pub struct SurfaceDrawingContext<'a> {
  pub surface: wgpu::Surface<'a>,
  pub config: wgpu::SurfaceConfiguration,
  pub size: Size,
  pub dpi: u32,
}

#[derive(Debug)]
pub struct TextureDrawingContext {
  pub format: wgpu::TextureFormat,
  pub size: Size,
}

#[derive(Debug)]
pub enum DrawingContextType<'a> {
  Surface(SurfaceDrawingContext<'a>),
  Texture(TextureDrawingContext),
}

#[derive(Debug)]
pub struct DrawingContext<'a> {
  pub ty: DrawingContextType<'a>,
  pub instance: wgpu::Instance,
  pub adapter: wgpu::Adapter,
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  pub sample_count: u32,
}

impl<'a> DrawingContext<'a> {
  pub async fn new_for_texture(
    size: Size,
    format: wgpu::TextureFormat,
  ) -> Self {
    let instance = wgpu::Instance::default();

    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions::default())
      .await
      .unwrap();

    let (device, queue) = adapter
      .request_device(&wgpu::DeviceDescriptor::default(), None)
      .await
      .unwrap();

    Self {
      instance,
      adapter,
      device,
      queue,
      ty: DrawingContextType::Texture(TextureDrawingContext { format, size }),
      sample_count: 1,
    }
  }

  pub async fn new_for_surface(
    window: Arc<Window>,
    cfg_builder: &SurfaceConfigBuilder<'a>,
  ) -> Self {
    let size = window.inner_size();
    let dpi = window.scale_factor();

    let instance = wgpu::Instance::default();
    let surface =
      instance.create_surface(window).expect("Failed to create surface");

    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
      })
      .await
      .expect("Failed to find an appropriate adapter");

    let (device, queue) = adapter
      .request_device(
        &wgpu::DeviceDescriptor {
          required_features: wgpu::Features::default()
            | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
          ..Default::default()
        },
        None,
      )
      .await
      .expect("Failed to create device");

    let config = cfg_builder.build(&adapter, &surface, size.width, size.height);
    surface.configure(&device, &config);

    Self {
      instance,
      adapter,
      device,
      queue,
      ty: DrawingContextType::Surface(SurfaceDrawingContext {
        surface,
        config,
        size: size.into(),
        dpi: dpi as u32,
      }),
      sample_count: 1,
    }
  }

  pub fn with_sample_count(mut self, sample_count: u32) -> Self {
    self.sample_count = sample_count;
    self
  }

  pub fn format(&'a self) -> wgpu::TextureFormat {
    match &self.ty {
      DrawingContextType::Surface(ctx) => ctx.config.format,
      DrawingContextType::Texture(ctx) => ctx.format,
    }
  }

  pub fn surface(&self) -> Option<&wgpu::Surface> {
    match &self.ty {
      DrawingContextType::Surface(ctx) => Some(&ctx.surface),
      DrawingContextType::Texture(_) => None,
    }
  }

  pub fn size(&self) -> &Size {
    match &self.ty {
      DrawingContextType::Surface(ctx) => &ctx.size,
      DrawingContextType::Texture(ctx) => &ctx.size,
    }
  }

  pub fn resolution(&self) -> Size {
    match &self.ty {
      DrawingContextType::Surface(ctx) => {
        let logical_size = Size {
          width: ctx.size.width / ctx.dpi,
          height: ctx.size.height / ctx.dpi,
        };
        logical_size
      }
      DrawingContextType::Texture(ctx) => ctx.size,
    }
  }

  pub fn aspect_ratio(&self) -> f32 {
    let Size { width, height } = self.size();
    *width as f32 / *height as f32
  }

  pub fn resize(&mut self, size: Size) {
    match &mut self.ty {
      DrawingContextType::Surface(ctx) => ctx.resize(&self.device, size),
      DrawingContextType::Texture(ctx) => ctx.resize(size),
    }
  }
}

impl<'a> SurfaceDrawingContext<'a> {
  pub fn resize(&mut self, device: &wgpu::Device, size: Size) {
    self.size = size;
    self.config.width = self.size.width;
    self.config.height = self.size.height;
    self.surface.configure(device, &self.config);
  }
}

impl TextureDrawingContext {
  pub fn resize(&mut self, size: Size) {
    self.size = size;
  }
}
