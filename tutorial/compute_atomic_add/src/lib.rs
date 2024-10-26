use std::{error::Error, iter};

use num_traits::FromBytes;
use wgpu::util::DeviceExt;
use wgpu_helper::context as helper_util;

pub async fn run() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  //
  // define constants
  //

  // std::mem::size_of::<i32>()で求められるが、ハードコーディングしてしまう
  let buffer_size = 4;

  //
  // init wgpu
  //

  let instance = wgpu::Instance::default();

  let adapter = instance
    .request_adapter(&wgpu::RequestAdapterOptions::default())
    .await
    .unwrap();

  let (device, queue) = adapter
    .request_device(&wgpu::DeviceDescriptor::default(), None)
    .await
    .unwrap();

  //
  // compile shader
  //

  let compute_shader =
    device.create_shader_module(wgpu::include_wgsl!("./compute.wgsl"));

  //
  // create a buffer to store data
  //

  let input_data_buffer =
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Storage Buffer for input data"),
      contents: bytemuck::cast_slice(&[4, 5, 6]),
      usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

  let non_atomic_result_data_buffer =
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Storage Buffer for result data (non atomic)"),
      contents: bytemuck::cast_slice(&[0, 0, 0]),
      usage: wgpu::BufferUsages::STORAGE
        | wgpu::BufferUsages::COPY_DST
        | wgpu::BufferUsages::COPY_SRC,
    });

  let atomic_result_data_buffer =
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Storage Buffer for result data (atomic)"),
      contents: bytemuck::cast_slice(&[0, 0, 0]),
      usage: wgpu::BufferUsages::STORAGE
        | wgpu::BufferUsages::COPY_DST
        | wgpu::BufferUsages::COPY_SRC,
    });

  //
  // create bind_group
  //

  let bind_group_layout = helper_util::create_bind_group_layout(
    &device,
    &[
      wgpu::BufferBindingType::Storage { read_only: true }, // input
      wgpu::BufferBindingType::Storage { read_only: false }, // result (non atomic)
      wgpu::BufferBindingType::Storage { read_only: false }, // result (atomic)
    ],
    &[
      wgpu::ShaderStages::COMPUTE,
      wgpu::ShaderStages::COMPUTE,
      wgpu::ShaderStages::COMPUTE,
    ],
  );

  let bind_group = helper_util::create_bind_group(
    &device,
    &bind_group_layout,
    &[
      input_data_buffer.as_entire_binding(),
      non_atomic_result_data_buffer.as_entire_binding(),
      atomic_result_data_buffer.as_entire_binding(),
    ],
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
  // create command encode
  //

  let mut command_encoder = device
    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

  //
  // encode compute pass
  //

  let mut compute_pass_encoder =
    command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
      label: Some("Compute Pass"),
      timestamp_writes: None,
    });

  compute_pass_encoder.set_pipeline(&compute_pipeline);
  compute_pass_encoder.set_bind_group(0, &bind_group, &[]);
  // シェーダーのローカルワークグループは `8`に設定されているため、必要なのは 1つのワークグループだけ
  compute_pass_encoder.dispatch_workgroups(1, 1, 1);

  drop(compute_pass_encoder);

  //
  // execute commands
  //

  queue.submit(iter::once(command_encoder.finish()));

  //
  // result
  //

  let non_atomic_result_data: Vec<i32> = get_gpu_buffer(
    &device,
    &queue,
    &non_atomic_result_data_buffer,
    buffer_size,
  )
  .await;

  let atomic_result_data: Vec<i32> =
    get_gpu_buffer(&device, &queue, &atomic_result_data_buffer, buffer_size)
      .await;

  // アトミックロックを使用した場合は正しい結果が得られる
  // アトミックロックを使用しない場合は誤った結果が出る可能性がある（まれに正しい結果が得られることもあるが、多くの場合は失敗する）
  println!("  no atomic array contents: {:?}", non_atomic_result_data);
  println!("with atomic array contents: {:?}", atomic_result_data);

  Ok(())
}

async fn get_gpu_buffer<T>(
  device: &wgpu::Device,
  queue: &wgpu::Queue,
  src_buffer: &wgpu::Buffer,
  buffer_size: u64,
) -> Vec<T>
where
  T: FromBytes<Bytes = [u8; 4]>,
{
  let tmp_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("tmp_buffer"),
    size: buffer_size,
    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    mapped_at_creation: false,
  });

  let mut command_encoder =
    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("command_encoder for tmp"),
    });

  //
  // Encode commands for copying buffer to buffer
  //

  command_encoder.copy_buffer_to_buffer(
    &src_buffer,
    0,
    &tmp_buffer,
    0,
    buffer_size,
  );

  //
  // Submit GPU commands
  //

  queue.submit(iter::once(command_encoder.finish()));

  //
  // Read buffer
  //

  let buffer_slice = tmp_buffer.slice(..);

  let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
  buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
    tx.send(result).unwrap();
  });
  device.poll(wgpu::Maintain::Wait);
  rx.receive().await.unwrap().unwrap();

  let data_view = buffer_slice.get_mapped_range();

  let data = data_view
    .chunks_exact(buffer_size as usize)
    .map(|b| FromBytes::from_ne_bytes(&b.try_into().unwrap()))
    .collect::<Vec<T>>();

  drop(data_view);
  tmp_buffer.unmap();

  data
}
