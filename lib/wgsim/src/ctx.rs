use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

#[derive(Debug)]
pub struct WgpuContext<'a> {
  pub instance: wgpu::Instance,
  pub adapter: wgpu::Adapter,
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  pub surface: Option<wgpu::Surface<'a>>,
  pub config: Option<wgpu::SurfaceConfiguration>,
  pub size: PhysicalSize<u32>,
  pub format: wgpu::TextureFormat,
  pub sample_count: u32,
}

impl<'a> WgpuContext<'a> {
  pub async fn new(
    window: Arc<Window>,
    sample_count: u32,
    limits: Option<wgpu::Limits>,
  ) -> Self {
    let limits_device = limits.unwrap_or(wgpu::Limits::default());

    let size = window.inner_size();
    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(window).unwrap();

    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
      })
      .await
      .expect("Failed to find an appropriate adapter");

    let (device, queue) = adapter
      .request_device(
        &wgpu::DeviceDescriptor {
          label: None,
          required_features: wgpu::Features::default()
            | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
          required_limits: limits_device,
          ..Default::default()
        },
        None,
      )
      .await
      .expect("Failed to create device");

    let surface_caps = surface.get_capabilities(&adapter);
    let format = surface_caps.formats[0];

    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format,
      width: size.width,
      height: size.height,
      present_mode: wgpu::PresentMode::Fifo,
      alpha_mode: surface_caps.alpha_modes[0],
      view_formats: vec![],
      desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    Self {
      instance,
      surface: Some(surface),
      adapter,
      device,
      queue,
      config: Some(config),
      format,
      size,
      sample_count,
    }
  }

  pub async fn new_without_surface(
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    sample_count: u32,
  ) -> Self {
    let size = PhysicalSize::new(width, height);

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
      surface: None,
      adapter,
      device,
      queue,
      config: None,
      size,
      format,
      sample_count,
    }
  }
}
