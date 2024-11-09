use std::{error::Error, iter};

use num_traits::FromBytes;
use rand::Rng;
use wgsim::{ctx::ComputingContext, ppl::ComputePipelineBuilder, util};

const ARRAY_SIZE: usize = 128;

pub async fn run() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  //
  // data
  //

  let rng = &mut rand::thread_rng();
  let random_numbers: Vec<i32> =
    (0..ARRAY_SIZE).map(|_| rng.gen_range(1..=100)).collect();

  println!("before: {:?}", random_numbers);

  //
  // wgpu
  //

  let ctx = ComputingContext::new().await?;

  //
  // shader
  //

  let compute_shader =
    ctx.device.create_shader_module(wgpu::include_wgsl!("./compute.wgsl"));

  //
  // buffer
  //

  let buffer_size = ARRAY_SIZE * std::mem::size_of::<i32>();

  let input_data_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("input_data_buffer"),
    size: buffer_size as u64,
    usage: wgpu::BufferUsages::STORAGE
      | wgpu::BufferUsages::COPY_DST
      | wgpu::BufferUsages::COPY_SRC,
    mapped_at_creation: false,
  });
  ctx.queue.write_buffer(
    &input_data_buffer,
    0,
    bytemuck::cast_slice(&random_numbers),
  );

  let result_data_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("result_data_buffer"),
    size: buffer_size as u64,
    usage: wgpu::BufferUsages::STORAGE
      | wgpu::BufferUsages::COPY_DST
      | wgpu::BufferUsages::COPY_SRC,
    mapped_at_creation: false,
  });

  let readback_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("readback_buffer"),
    size: buffer_size as u64,
    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    mapped_at_creation: false,
  });

  //
  // bind_group
  //

  let bind_group_layout = util::create_bind_group_layout_for_buffer(
    &ctx.device,
    &[wgpu::BufferBindingType::Storage { read_only: false }],
    &[wgpu::ShaderStages::COMPUTE],
  );

  let bind_group = util::create_bind_group(
    &ctx.device,
    &bind_group_layout,
    &[input_data_buffer.as_entire_binding()],
  );

  //
  // compute_pipeline
  //

  let pipeline_layout =
    ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Pipeline Layout"),
      bind_group_layouts: &[&bind_group_layout],
      push_constant_ranges: &[],
    });

  let compute_pipeline = ComputePipelineBuilder::new(&ctx.device)
    .cs_shader(&compute_shader, "cs_main")
    .pipeline_layout(&pipeline_layout)
    .build();

  //
  // compute command
  //

  let mut command_encoder = ctx
    .device
    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

  let mut compute_pass_encoder =
    command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
      label: Some("compute_pass_encoder"),
      timestamp_writes: None,
    });

  compute_pass_encoder.set_pipeline(&compute_pipeline);
  compute_pass_encoder.set_bind_group(0, &bind_group, &[]);

  let dispatch_dim = (ARRAY_SIZE / 64) as u32;
  compute_pass_encoder.dispatch_workgroups(dispatch_dim, 1, 1);

  drop(compute_pass_encoder);

  //
  // copy command
  //

  command_encoder.copy_buffer_to_buffer(
    &input_data_buffer,
    0,
    &result_data_buffer,
    0,
    buffer_size as u64,
  );

  command_encoder.copy_buffer_to_buffer(
    &result_data_buffer,
    0,
    &readback_buffer,
    0,
    buffer_size as u64,
  );

  //
  // execute commands
  //

  ctx.queue.submit(iter::once(command_encoder.finish()));

  //
  // read buffer
  //

  let data: Vec<i32> = read_gpu_buffer(&ctx.device, &readback_buffer).await?;

  println!("after: {:?}", data);

  Ok(())
}

async fn read_gpu_buffer<T>(
  device: &wgpu::Device,
  readback_buffer: &wgpu::Buffer,
) -> Result<Vec<T>, Box<dyn Error>>
where
  T: FromBytes<Bytes = [u8; 4]>,
{
  let buffer_slice = readback_buffer.slice(..);

  let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
  buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
    tx.send(result).unwrap();
  });
  device.poll(wgpu::Maintain::Wait);
  rx.receive().await.unwrap()?;

  let data_view = buffer_slice.get_mapped_range();

  let data = data_view
    .chunks_exact(std::mem::size_of::<T>())
    .map(|b| T::from_ne_bytes(&b.try_into().unwrap()))
    .collect::<Vec<T>>();

  drop(data_view);
  readback_buffer.unmap();

  Ok(data)
}
