use std::{error::Error, sync::Arc};

use winit::window::Window;

use crate::text::{RenderTarget, Text, TextShape};

pub struct WindowState<'a> {
  pub window: Arc<Window>,
  device: wgpu::Device,
  queue: wgpu::Queue,
  surface: wgpu::Surface<'a>,
  surface_config: wgpu::SurfaceConfiguration,
  text_renderer: Text,
  text_buffer: glyphon::Buffer,
}

impl<'a> WindowState<'a> {
  pub async fn new(window: Arc<Window>) -> Self {
    let physical_size = window.inner_size();
    let scale_factor = window.scale_factor();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
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

    let swapchain_format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let surface_config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: swapchain_format,
      width: physical_size.width,
      height: physical_size.height,
      present_mode: wgpu::PresentMode::Fifo,
      alpha_mode: wgpu::CompositeAlphaMode::Opaque,
      view_formats: vec![],
      desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &surface_config);

    let mut text_renderer = Text::new(&device, &queue, swapchain_format);
    let text_buffer = text_renderer.init_buffer(TextShape {
      content: "Hello world! 👋\nThis is rendered with 🦅 glyphon 🦁\nThe text below should be partially clipped.\na b c d e f g h i j k l m n o p q r s t u v w x y z",
      attributes: glyphon::Attrs::new().family(glyphon::Family::SansSerif),
      metrics: glyphon::Metrics::new(30.0, 42.2),
    }, RenderTarget {
      width: physical_size.width,
      height: physical_size.height,
      scale_factor: scale_factor as f32,
    });

    WindowState {
      window,
      device,
      queue,
      surface,
      surface_config,
      text_renderer,
      text_buffer,
    }
  }

  pub fn update_view(&mut self) -> Result<(), Box<dyn Error>> {
    self.text_renderer.prepare(
      &self.device,
      &self.queue,
      glyphon::Resolution {
        width: self.surface_config.width,
        height: self.surface_config.height,
      },
      [glyphon::TextArea {
        buffer: &self.text_buffer,
        left: 10.0,
        top: 10.0,
        scale: 1.0,
        bounds: glyphon::TextBounds {
          left: 0,
          top: 0,
          right: 600,
          bottom: 44 * 4,
        },
        default_color: glyphon::Color::rgb(110, 133, 183),
      }],
    )?;

    let frame = self.surface.get_current_texture().unwrap();
    let view =
      frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: None,
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &view,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
          store: wgpu::StoreOp::Store,
        },
      })],
      depth_stencil_attachment: None,
      timestamp_writes: None,
      occlusion_query_set: None,
    });
    self.text_renderer.render(&mut pass)?;
    drop(pass);

    self.queue.submit(Some(encoder.finish()));
    frame.present();

    self.text_renderer.clean_for_next();

    Ok(())
  }
}
