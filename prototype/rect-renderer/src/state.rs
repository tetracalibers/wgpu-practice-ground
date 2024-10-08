use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

use crate::color::Color;
use crate::geometry_value::*;
use crate::renderer::{Rect, UiRenderer};

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
      self.ui.set_viewport_size(new_size.width, new_size.height);
    }
  }

  fn set_geometry(&mut self) {
    self.ui.rectangle(Rect {
      color: Color {
        r: 1.0,
        g: 0.5,
        b: 1.0,
        a: 1.0,
      },
      bounds: Bounds {
        origin: Point { x: 50.0, y: 100.0 },
        size: Size {
          width: 100.0,
          height: 100.0,
        },
      },
      sigma: 0.25,
      corners: Corners {
        top_left: 10.0,
        top_right: 10.0,
        bottom_right: 10.0,
        bottom_left: 10.0,
      },
    });

    self.ui.rectangle(Rect {
      color: Color {
        r: 1.0,
        g: 1.0,
        b: 0.5,
        a: 1.0,
      },
      bounds: Bounds {
        origin: Point { x: 450.0, y: 150.0 },
        size: Size {
          width: 100.0,
          height: 100.0,
        },
      },
      corners: Corners {
        top_left: 0.0,
        top_right: 0.0,
        bottom_right: 0.0,
        bottom_left: 0.0,
      },
      sigma: 20.0,
    });

    self.ui.rectangle(Rect {
      color: Color {
        r: 0.5,
        g: 0.25,
        b: 1.0,
        a: 1.0,
      },
      bounds: Bounds {
        origin: Point { x: 150.0, y: 300.0 },
        size: Size {
          width: 100.0,
          height: 100.0,
        },
      },
      corners: Corners {
        top_left: 0.0,
        top_right: 10.0,
        bottom_right: 20.0,
        bottom_left: 30.0,
      },
      sigma: 0.25,
    });

    self.ui.rectangle(Rect {
      color: Color {
        r: 1.0,
        g: 0.5,
        b: 0.25,
        a: 1.0,
      },
      bounds: Bounds {
        origin: Point { x: 250.0, y: 50.0 },
        size: Size {
          width: 100.0,
          height: 100.0,
        },
      },
      corners: Corners {
        top_left: 50.0,
        top_right: 50.0,
        bottom_right: 50.0,
        bottom_left: 50.0,
      },
      sigma: 0.25,
    });

    self.ui.rectangle(Rect {
      color: Color {
        r: 1.0,
        g: 0.5,
        b: 1.0,
        a: 1.0,
      },
      bounds: Bounds {
        origin: Point { x: 400.0, y: 400.0 },
        size: Size {
          width: 100.0,
          height: 100.0,
        },
      },
      corners: Corners {
        top_left: 10.0,
        top_right: 10.0,
        bottom_right: 10.0,
        bottom_left: 10.0,
      },
      sigma: 20.0,
    });

    self.ui.rectangle(Rect {
      color: Color {
        r: 0.5,
        g: 0.25,
        b: 0.5,
        a: 1.0,
      },
      bounds: Bounds {
        origin: Point { x: 400.0, y: 400.0 },
        size: Size {
          width: 100.0,
          height: 100.0,
        },
      },
      corners: Corners {
        top_left: 10.0,
        top_right: 10.0,
        bottom_right: 10.0,
        bottom_left: 10.0,
      },
      sigma: 0.25,
    });

    self.ui.rectangle(Rect {
      color: Color {
        r: 1.0,
        g: 0.5,
        b: 1.0,
        a: 1.0,
      },
      bounds: Bounds {
        origin: Point { x: 401.0, y: 401.0 },
        size: Size {
          width: 98.0,
          height: 98.0,
        },
      },
      corners: Corners {
        top_left: 9.0,
        top_right: 9.0,
        bottom_right: 9.0,
        bottom_left: 9.0,
      },
      sigma: 0.25,
    })
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

    self.set_geometry();
    self.ui.render(&mut encoder, &self.queue, &view);

    self.queue.submit(std::iter::once(encoder.finish()));
    surface_texture.present();

    Ok(())
  }
}
