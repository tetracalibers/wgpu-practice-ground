use std::{error::Error, fs::File, io::BufWriter, iter, path::Path};

use wgsim::util;

pub async fn run() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  //
  // define constants
  //

  const IMG_SIZE: u32 = 128;

  //
  // init wgpu
  //

  let instance = wgpu::Instance::default();

  let adapter = instance
    .request_adapter(&wgpu::RequestAdapterOptions::default())
    .await
    .unwrap();

  let (device, queue) =
    adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await?;

  //
  // compile shader
  //

  let compute_shader =
    device.create_shader_module(wgpu::include_wgsl!("./compute.wgsl"));

  //
  // texture
  //

  let texture = device.create_texture(&wgpu::TextureDescriptor {
    label: Some("output grid texture"),
    size: wgpu::Extent3d {
      width: IMG_SIZE,
      height: IMG_SIZE,
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: wgpu::TextureFormat::Rgba8Unorm,
    usage: wgpu::TextureUsages::COPY_DST
      | wgpu::TextureUsages::COPY_SRC
      | wgpu::TextureUsages::TEXTURE_BINDING
      | wgpu::TextureUsages::STORAGE_BINDING,
    view_formats: &[],
  });

  //
  // staging buffer
  //

  let texture_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("staging buffer for texture data"),
    size: 4 * IMG_SIZE as u64 * IMG_SIZE as u64,
    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    mapped_at_creation: false,
  });

  //
  // create bind_group
  //

  let bind_group_layout = util::create_bind_group_layout(
    &device,
    &[wgpu::BindingType::StorageTexture {
      access: wgpu::StorageTextureAccess::WriteOnly,
      format: wgpu::TextureFormat::Rgba8Unorm,
      view_dimension: wgpu::TextureViewDimension::D2,
    }],
    &[wgpu::ShaderStages::COMPUTE],
  );

  let bind_group = util::create_bind_group(
    &device,
    &bind_group_layout,
    &[wgpu::BindingResource::TextureView(
      &texture.create_view(&wgpu::TextureViewDescriptor::default()),
    )],
  );

  //
  // create compute_pipeline
  //

  let pipeline_layout =
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Pipeline Layout"),
      bind_group_layouts: &[&bind_group_layout],
      push_constant_ranges: &[],
    });

  let compute_pipeline =
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
      label: Some("Compute pipeline"),
      layout: Some(&pipeline_layout),
      module: &compute_shader,
      entry_point: "cs_main",
      compilation_options: wgpu::PipelineCompilationOptions::default(),
      cache: None,
    });

  //
  // commands submission
  //

  let mut command_encoder = device
    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

  let mut compute_pass_encoder =
    command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
      label: Some("Compute Pass"),
      timestamp_writes: None,
    });

  compute_pass_encoder.set_pipeline(&compute_pipeline);
  compute_pass_encoder.set_bind_group(0, &bind_group, &[]);
  compute_pass_encoder.dispatch_workgroups(16, 16, 1);

  drop(compute_pass_encoder);

  //
  // copy texture to buffer
  //

  command_encoder.copy_texture_to_buffer(
    wgpu::ImageCopyTexture {
      texture: &texture,
      mip_level: 0,
      origin: wgpu::Origin3d::ZERO,
      aspect: wgpu::TextureAspect::All,
    },
    wgpu::ImageCopyBuffer {
      buffer: &texture_data_buffer,
      layout: wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: Some(4 * IMG_SIZE),
        rows_per_image: Some(IMG_SIZE),
      },
    },
    wgpu::Extent3d {
      width: IMG_SIZE,
      height: IMG_SIZE,
      depth_or_array_layers: 1,
    },
  );

  //
  // submit GPU commands
  //

  queue.submit(iter::once(command_encoder.finish()));

  //
  // read buffer
  //

  let buffer_slice = texture_data_buffer.slice(..);

  let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
  buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
    tx.send(result).unwrap();
  });
  device.poll(wgpu::Maintain::Wait);
  rx.receive().await.unwrap()?;

  let px_data = buffer_slice.get_mapped_range();

  //
  // create png
  //

  let path = Path::new("export/compute_visualize_workgroup_local.png");
  let file = File::create(path)?;
  let ref mut w = BufWriter::new(file);

  let mut png_encoder = png::Encoder::new(w, IMG_SIZE, IMG_SIZE);
  png_encoder.set_color(png::ColorType::Rgba);

  let mut writer = png_encoder.write_header()?;
  writer.write_image_data(&px_data)?;

  //
  // clean up
  //

  drop(px_data);
  texture_data_buffer.unmap();

  Ok(())
}
