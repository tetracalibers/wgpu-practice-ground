use std::error::Error;

use crate::{
  ctx::WgpuContext,
  render::{Render, RenderTarget},
};

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
  pub async fn new(size: u32, initial: R::Initial, msaa: bool) -> Self {
    let sample_count = if msaa { 4 } else { 1 };

    let ctx = WgpuContext::new_without_surface(
      size,
      size,
      wgpu::TextureFormat::Rgba8UnormSrgb,
      sample_count,
    )
    .await;

    let renderer = R::new(&ctx, &initial).await;

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
  ) -> Result<(), Box<dyn Error>> {
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
  ) -> Result<(), Box<dyn Error>> {
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

    let pixel_size = std::mem::size_of::<[u8; 4]>() as u32;
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
    let render_start_time = std::time::Instant::now();

    for _ in 0..scene_count {
      let mut command_encoder = self.ctx.device.create_command_encoder(
        &wgpu::CommandEncoderDescriptor { label: None },
      );

      let now = std::time::Instant::now();
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
