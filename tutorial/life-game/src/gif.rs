use std::{iter, mem};

use crate::renderer::Renderer;

const SCENE_COUNT: usize = 2;

pub async fn export() {
  let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
    backends: wgpu::Backends::all(),
    ..Default::default()
  });

  let adapter = instance
    .request_adapter(&wgpu::RequestAdapterOptions::default())
    .await
    .unwrap();

  let (device, queue) = adapter
    .request_device(&wgpu::DeviceDescriptor::default(), None)
    .await
    .unwrap();

  //
  // レンダリング先のテクスチャ
  // surface.get_current_texture()を使って描画するテクスチャを取得する代わりに、テクスチャを自分で作成する
  //
  let texture_size = 512;
  let texture_desc = wgpu::TextureDescriptor {
    size: wgpu::Extent3d {
      width: texture_size,
      height: texture_size,
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
  let texture = device.create_texture(&texture_desc);
  let texture_view = texture.create_view(&Default::default());

  //
  // wgpuはテクスチャからバッファへのコピーがCOPY_BYTES_PER_ROW_ALIGNMENTを使用して整列されることを要求する
  // このため、padded_bytes_per_rowとunpadded_bytes_per_rowの両方を保存する必要がある
  //
  let pixel_size = mem::size_of::<[u8; 4]>() as u32;
  let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
  let unpadded_bytes_per_row = pixel_size * texture_size;
  let padding = (align - unpadded_bytes_per_row % align) % align;
  let padded_bytes_per_row = unpadded_bytes_per_row + padding;

  //
  // 出力をコピーするバッファを作成する
  //
  // 最終的に、テクスチャからバッファにデータをコピーしてファイルに保存する
  // データを保存するためには、十分な大きさのバッファが必要
  //
  let buffer_size =
    (padded_bytes_per_row * texture_size) as wgpu::BufferAddress;
  let buffer_desc = wgpu::BufferDescriptor {
    size: buffer_size,
    // BufferUsages::MAP_READは、wpguにこのバッファをCPUから読み込みたいことを伝える
    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    label: Some("Output Buffer"),
    mapped_at_creation: false,
  };
  let output_buffer = device.create_buffer(&buffer_desc);

  let mut renderer =
    Renderer::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb).await;

  // フレームをレンダリングし、そのフレームをVec<u8>にコピーする
  let mut frames = Vec::new();

  for _ in 0..SCENE_COUNT {
    let mut encoder = device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    // テクスチャに描画する
    renderer.draw(&mut encoder, &texture_view);

    // テクスチャの内容をバッファにコピーする
    encoder.copy_texture_to_buffer(
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
          rows_per_image: Some(texture_size),
        },
      },
      texture_desc.size,
    );

    queue.submit(iter::once(encoder.finish()));

    //
    // バッファからデータを取り出すには、まずバッファをマップし、バッファビューを取得して、それを&[u8]のように扱う必要がある
    //
    let buffer_slice = output_buffer.slice(..);
    let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
      tx.send(result).unwrap();
    });
    device.poll(wgpu::Maintain::Wait);

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

  save_gif("export/life-game.gif", &mut frames, 10, texture_size as u16)
    .unwrap();
}

fn save_gif(
  path: &str,
  frames: &mut Vec<Vec<u8>>,
  speed: i32,
  size: u16,
) -> anyhow::Result<()> {
  use gif::{Encoder, Frame, Repeat};

  let mut image = std::fs::File::create(path)?;
  let mut encoder = Encoder::new(&mut image, size, size, &[])?;
  encoder.set_repeat(Repeat::Infinite)?;

  for mut frame in frames {
    encoder
      .write_frame(&Frame::from_rgba_speed(size, size, &mut frame, speed))?;
  }

  Ok(())
}
