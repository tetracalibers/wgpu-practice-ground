use std::{future::Future, iter, mem, sync::Arc, time};

use anyhow::Result;
use winit::{
  application::ApplicationHandler,
  dpi::PhysicalSize,
  event::{ElementState, KeyEvent, WindowEvent},
  event_loop::{ActiveEventLoop, EventLoop},
  keyboard::{KeyCode, PhysicalKey},
  window::{Window, WindowId},
};

use crate::context::WgpuContext;

pub struct DrawOutput<'a> {
  pub surface_texture: Option<wgpu::SurfaceTexture>,
  pub encoder: &'a wgpu::CommandEncoder,
}

pub enum RenderTarget<'a> {
  Surface(&'a wgpu::Surface<'a>),
  Texture(&'a wgpu::Texture),
}

#[allow(opaque_hidden_inferred_bound, unused_variables)]
pub trait Render<'a> {
  type DrawData;
  type InitialState;

  fn new(
    ctx: &WgpuContext<'a>,
    draw_data: &Self::DrawData,
    initial_state: &Self::InitialState,
  ) -> impl Future<Output = Self>;
  fn resize(&mut self, ctx: &WgpuContext, size: Option<PhysicalSize<u32>>) {
    let size = size.unwrap_or(ctx.size);

    if size.width > 0 && size.height > 0 {
      if let Some(surface) = &ctx.surface {
        surface.configure(&ctx.device, &ctx.config.as_ref().unwrap());
      }
    }
  }
  fn process_event(&mut self, event: &WindowEvent) -> bool {
    false
  }
  fn update(&mut self, ctx: &WgpuContext, dt: time::Duration) {}
  fn draw(
    &mut self,
    encoder: &mut wgpu::CommandEncoder,
    target: RenderTarget,
    sample_count: u32,
  ) -> Result<Option<wgpu::SurfaceTexture>, wgpu::SurfaceError>;
  fn submit(
    &self,
    queue: &wgpu::Queue,
    encoder: wgpu::CommandEncoder,
    frame: Option<wgpu::SurfaceTexture>,
  ) {
    queue.submit(iter::once(encoder.finish()));

    if let Some(frame) = frame {
      frame.present();
    }
  }
}

pub struct Gif<'a, R>
where
  R: Render<'a>,
{
  renderer: R,
  size: u32,
  sample_count: u32,
  ctx: WgpuContext<'a>,
}

impl<'a, R> Gif<'a, R>
where
  R: Render<'a>,
{
  pub async fn new(
    size: u32,
    draw_data: R::DrawData,
    initial_state: R::InitialState,
    sample_count: Option<u32>,
  ) -> Self {
    let sample_count = sample_count.unwrap_or(1);

    let ctx = WgpuContext::new_without_surface(
      size,
      size,
      wgpu::TextureFormat::Rgba8UnormSrgb,
      sample_count,
    )
    .await;

    let renderer = R::new(&ctx, &draw_data, &initial_state).await;

    Self {
      renderer,
      size,
      sample_count,
      ctx,
    }
  }

  fn save_gif(
    &self,
    file_path: &str,
    frames: &mut Vec<Vec<u8>>,
    speed: i32,
    size: u16,
  ) -> anyhow::Result<()> {
    use gif::{Encoder, Frame, Repeat};

    let mut image = std::fs::File::create(file_path)?;
    let mut encoder = Encoder::new(&mut image, size, size, &[])?;
    encoder.set_repeat(Repeat::Infinite)?;

    for mut frame in frames {
      encoder
        .write_frame(&Frame::from_rgba_speed(size, size, &mut frame, speed))?;
    }

    Ok(())
  }

  pub async fn export(
    &mut self,
    file_path: &str,
    scene_count: usize,
    speed: i32,
  ) -> Result<()> {
    let texture_desc = wgpu::TextureDescriptor {
      size: wgpu::Extent3d {
        width: self.size,
        height: self.size,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1, // コピー先のテクスチャでは 1 でよい
      dimension: wgpu::TextureDimension::D2,
      format: self.ctx.format,
      usage: wgpu::TextureUsages::COPY_SRC
        | wgpu::TextureUsages::RENDER_ATTACHMENT,
      label: None,
      view_formats: &[],
    };
    let texture = self.ctx.device.create_texture(&texture_desc);

    let pixel_size = mem::size_of::<[u8; 4]>() as u32;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let unpadded_bytes_per_row = pixel_size * self.size;
    let padding = (align - unpadded_bytes_per_row % align) % align;
    let padded_bytes_per_row = unpadded_bytes_per_row + padding;

    let buffer_size = (padded_bytes_per_row * self.size) as wgpu::BufferAddress;
    let buffer_desc = wgpu::BufferDescriptor {
      size: buffer_size,
      usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
      label: Some("Output Buffer"),
      mapped_at_creation: false,
    };
    let output_buffer = self.ctx.device.create_buffer(&buffer_desc);

    let mut frames = Vec::new();
    let render_start_time = time::Instant::now();

    for _ in 0..scene_count {
      let mut command_encoder = self.ctx.device.create_command_encoder(
        &wgpu::CommandEncoderDescriptor { label: None },
      );

      let now = time::Instant::now();
      let dt = now - render_start_time;
      self.renderer.update(&self.ctx, dt);

      self.renderer.draw(
        &mut command_encoder,
        RenderTarget::Texture(&texture),
        self.sample_count,
      )?;

      command_encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
          texture: &texture,
          mip_level: 0,
          origin: wgpu::Origin3d::ZERO,
          aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
          buffer: &output_buffer,
          layout: wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(padded_bytes_per_row),
            rows_per_image: Some(self.size),
          },
        },
        texture_desc.size,
      );

      self.renderer.submit(&self.ctx.queue, command_encoder, None);

      let buffer_slice = output_buffer.slice(..);
      let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
      buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
      });
      self.ctx.device.poll(wgpu::Maintain::Wait);

      match rx.receive().await {
        Some(Ok(())) => {
          let padded_data = buffer_slice.get_mapped_range();
          let data = padded_data
            .chunks(padded_bytes_per_row as _)
            .map(|chunk| &chunk[..unpadded_bytes_per_row as _])
            .flatten()
            .map(|x| *x)
            .collect::<Vec<_>>();
          drop(padded_data);
          output_buffer.unmap();
          frames.push(data);
        }
        _ => eprintln!("Something went wrong"),
      }
    }

    self.save_gif(file_path, &mut frames, speed, self.size as u16)?;

    Ok(())
  }
}

pub struct App<'a, R>
where
  R: Render<'a>,
{
  window: Option<Arc<Window>>,
  window_title: &'a str,
  draw_data: R::DrawData,
  initial_state: R::InitialState,
  sample_count: u32,
  ctx: Option<WgpuContext<'a>>,
  renderer: Option<R>,
  render_start_time: Option<time::Instant>,
}

impl<'a, R: Render<'a>> App<'a, R> {
  pub fn new(
    window_title: &'a str,
    draw_data: R::DrawData,
    initial_state: R::InitialState,
    sample_count: Option<u32>,
  ) -> Self {
    Self {
      window: None,
      window_title,
      draw_data,
      initial_state,
      sample_count: sample_count.unwrap_or(1),
      ctx: None,
      renderer: None,
      render_start_time: None,
    }
  }

  pub fn run(&mut self) -> Result<()> {
    let event_loop = EventLoop::builder().build()?;
    event_loop.run_app(self)?;

    Ok(())
  }

  fn window(&self) -> Option<&Window> {
    match &self.window {
      Some(window) => Some(window.as_ref()),
      None => None,
    }
  }

  async fn init(&mut self, window: Arc<Window>) {
    let ctx = WgpuContext::new(window, self.sample_count, None).await;
    self.ctx = Some(ctx);

    let renderer = R::new(
      self.ctx.as_ref().unwrap(),
      &self.draw_data,
      &self.initial_state,
    )
    .await;
    self.renderer = Some(renderer);
  }
}

impl<'a, R: Render<'a>> ApplicationHandler for App<'a, R> {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let window_attributes =
      Window::default_attributes().with_title(self.window_title);
    let window = event_loop.create_window(window_attributes).unwrap();
    self.window = Some(Arc::new(window));

    pollster::block_on(self.init(self.window.as_ref().unwrap().clone()));

    self.render_start_time = Some(time::Instant::now());
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    window_id: WindowId,
    event: WindowEvent,
  ) {
    let binding = self.window();
    let window = match &binding {
      Some(window) => window,
      None => return,
    };
    if window.id() != window_id {
      return;
    }

    let renderer = match &mut self.renderer {
      Some(renderer) => renderer,
      None => return,
    };
    if renderer.process_event(&event) {
      return;
    }

    let ctx = match &self.ctx {
      Some(ctx) => ctx,
      None => return,
    };

    match event {
      WindowEvent::Resized(size) => {
        renderer.resize(ctx, Some(size));
      }
      WindowEvent::RedrawRequested => {
        let ctx = match &mut self.ctx {
          Some(ctx) => ctx,
          None => {
            eprintln!("Context is not initialized");
            return;
          }
        };

        let now = time::Instant::now();
        let dt = now - self.render_start_time.unwrap_or(now);
        renderer.update(ctx, dt);

        let mut command_encoder =
          ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
          });

        let result = renderer.draw(
          &mut command_encoder,
          RenderTarget::Surface(ctx.surface.as_ref().unwrap()),
          self.sample_count,
        );

        match result {
          Ok(frame) => renderer.submit(&ctx.queue, command_encoder, frame),
          Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
            renderer.resize(ctx, None)
          }
          Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
          Err(e) => eprintln!("{:?}", e),
        }
      }
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      WindowEvent::KeyboardInput {
        event:
          KeyEvent {
            physical_key: PhysicalKey::Code(KeyCode::Escape),
            state: ElementState::Pressed,
            ..
          },
        ..
      } => {
        event_loop.exit();
      }
      _ => {}
    }
  }

  // TODO: 更新間隔を指定できるようにする
  // - need_redrawフラグを追加
  // - 引数にupdate_interval: Option<Duration>を追加
  // - update_intervalがSomeの場合、set_control_flowによる待機処理を入れる
  // 参考: tutorial/life-game/src/app.rs
  fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    let binding = self.window();
    let window = match &binding {
      Some(window) => window,
      None => return,
    };
    window.request_redraw();
  }
}
