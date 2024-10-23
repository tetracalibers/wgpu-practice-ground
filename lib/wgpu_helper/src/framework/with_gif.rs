use std::{future::Future, mem, sync::Arc, time};

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

#[allow(opaque_hidden_inferred_bound)]
pub trait Render<'a> {
  type DrawData;
  type InitialState;

  fn new(
    ctx: &WgpuContext<'a>,
    draw_data: &Self::DrawData,
    initial_state: &Self::InitialState,
  ) -> impl Future<Output = Self>;
  fn get_size(&self, ctx: &WgpuContext) -> PhysicalSize<u32> {
    ctx.size
  }
  fn resize(&mut self, ctx: &mut WgpuContext, size: PhysicalSize<u32>);
  fn process_event(&mut self, event: &WindowEvent) -> bool;
  fn update(&mut self, ctx: &WgpuContext, dt: time::Duration);
  fn draw(
    &mut self,
    encoder: wgpu::CommandEncoder,
    target: RenderTarget,
    sample_count: Option<u32>,
    before_submit_hook: impl FnOnce(&mut wgpu::CommandEncoder) -> (),
  ) -> Result<impl FnOnce(&wgpu::Queue) -> (), wgpu::SurfaceError>;
  // fn submit(&self, queue: &wgpu::Queue, output: DrawOutput);
}

pub struct Gif<'a, M, S, R>
where
  R: Render<'a, DrawData = M, InitialState = S>,
{
  renderer: R,
  size: u32,
  ctx: WgpuContext<'a>,
  _phantom_data: std::marker::PhantomData<&'a ()>,
}

impl<'a, M, S, R> Gif<'a, M, S, R>
where
  R: Render<'a, DrawData = M, InitialState = S>,
{
  pub async fn new(size: u32, draw_data: M, initial_state: S) -> Self {
    let ctx = WgpuContext::new_without_surface(
      size,
      size,
      wgpu::TextureFormat::Rgba8UnormSrgb,
      1,
    )
    .await;

    let renderer = R::new(&ctx, &draw_data, &initial_state).await;

    Self {
      renderer,
      size,
      ctx,
      _phantom_data: std::marker::PhantomData,
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
    //
    // レンダリング先のテクスチャ
    // surface.get_current_texture()を使って描画するテクスチャを取得する代わりに、テクスチャを自分で作成する
    //
    let texture_desc = wgpu::TextureDescriptor {
      size: wgpu::Extent3d {
        width: self.size,
        height: self.size,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8UnormSrgb,
      // TextureUsages::RENDER_ATTACHMENTを使って、wgpuがテクスチャにレンダリングできるようにする
      // TextureUsages::COPY_SRCはテクスチャからデータを取り出してファイルに保存できるようにするため
      usage: wgpu::TextureUsages::COPY_SRC
        | wgpu::TextureUsages::RENDER_ATTACHMENT,
      label: None,
      view_formats: &[],
    };
    let texture = self.ctx.device.create_texture(&texture_desc);

    //
    // wgpuはテクスチャからバッファへのコピーがCOPY_BYTES_PER_ROW_ALIGNMENTを使用して整列されることを要求する
    // このため、padded_bytes_per_rowとunpadded_bytes_per_rowの両方を保存する必要がある
    //
    let pixel_size = mem::size_of::<[u8; 4]>() as u32;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let unpadded_bytes_per_row = pixel_size * self.size;
    let padding = (align - unpadded_bytes_per_row % align) % align;
    let padded_bytes_per_row = unpadded_bytes_per_row + padding;

    //
    // 出力をコピーするバッファを作成する
    //
    // 最終的に、テクスチャからバッファにデータをコピーしてファイルに保存する
    // データを保存するためには、十分な大きさのバッファが必要
    //
    let buffer_size = (padded_bytes_per_row * self.size) as wgpu::BufferAddress;
    let buffer_desc = wgpu::BufferDescriptor {
      size: buffer_size,
      // BufferUsages::MAP_READは、wpguにこのバッファをCPUから読み込みたいことを伝える
      usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
      label: Some("Output Buffer"),
      mapped_at_creation: false,
    };
    let output_buffer = self.ctx.device.create_buffer(&buffer_desc);

    // フレームをレンダリングし、そのフレームをVec<u8>にコピーする
    let mut frames = Vec::new();

    for _ in 0..scene_count {
      let mut command_encoder = self.ctx.device.create_command_encoder(
        &wgpu::CommandEncoderDescriptor { label: None },
      );

      let copy_to_buffer = |command_encoder: &mut wgpu::CommandEncoder| {
        // テクスチャの内容をバッファにコピーする
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
      };

      // テクスチャに描画する
      let submit = self.renderer.draw(
        command_encoder,
        RenderTarget::Texture(&texture),
        None,
        copy_to_buffer,
      )?;

      submit(&self.ctx.queue);

      //
      // バッファからデータを取り出すには、まずバッファをマップし、バッファビューを取得して、それを&[u8]のように扱う必要がある
      //
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

    let ctx = match &mut self.ctx {
      Some(ctx) => ctx,
      None => return,
    };

    match event {
      WindowEvent::Resized(size) => {
        renderer.resize(ctx, size);
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

        let command_encoder =
          ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
          });

        match renderer.draw(
          command_encoder,
          RenderTarget::Surface(ctx.surface.as_ref().unwrap()),
          Some(self.sample_count),
          |_command_encoder| {},
        ) {
          Ok(submit) => {
            submit(&ctx.queue);
          }
          //Err(wgpu::SurfaceError::Lost) => {
          //  renderer.resize(ctx, renderer.get_size(&ctx))
          //}
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

  fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    let binding = self.window();
    let window = match &binding {
      Some(window) => window,
      None => return,
    };
    window.request_redraw();
  }
}
