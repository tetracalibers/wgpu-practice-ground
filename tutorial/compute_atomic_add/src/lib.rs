use std::{error::Error, iter};

use wgpu_helper::context as helper_util;

pub async fn run() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  //
  // define constants
  //

  // std::mem::size_of::<f32>()で求められるが、ハードコーディングしてしまう
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

  let store_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("Storage Buffer for store value"),
    size: buffer_size,
    usage: wgpu::BufferUsages::STORAGE // compute shaderからアクセスできるように
      | wgpu::BufferUsages::COPY_SRC // read可能
      | wgpu::BufferUsages::COPY_DST, // write可能
    mapped_at_creation: false,
  });

  //
  // create bind_group
  //

  let bind_group_layout = helper_util::create_bind_group_layout(
    &device,
    &[wgpu::BufferBindingType::Storage { read_only: false }],
    &[wgpu::ShaderStages::COMPUTE],
  );

  let bind_group = helper_util::create_bind_group(
    &device,
    &bind_group_layout,
    &[store_buffer.as_entire_binding()],
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
  // create command encoder
  //

  let mut encoder = device
    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

  //
  // encode compute pass
  //

  let mut compute_pass =
    encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
      label: Some("Compute Pass"),
      timestamp_writes: None,
    });

  compute_pass.set_pipeline(&compute_pipeline);
  compute_pass.set_bind_group(0, &bind_group, &[]);
  compute_pass.dispatch_workgroups(1, 1, 1);

  drop(compute_pass);

  //
  // copy buffer to GPU
  //

  // 計算が完了した後、メモリに直接アクセスすることはできない
  // まず、一時的なステージング領域にデータをコピーし、その後、CPUに戻す必要がある
  // この操作を行うためには、メモリバッファを作成する際にGPUBufferUsage.MAP_READフラグを指定する必要がある

  // Note:
  // バッファの使用用途にMAP_READフラグが含まれる場合、同時に使用できる他の用途はCOPY_DSTのみ
  // つまり、コンピュートシェーダーで使用するSTORAGEフラグとMAP_READフラグを同時に設定することはできない
  // この制約を回避するため、データをRust側に戻す際には、一時的なステージング用のメモリブロックを使用する必要がある

  let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("Storage Buffer for staging"),
    size: buffer_size,
    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    mapped_at_creation: false,
  });

  // コンピュートシェーダーの結果を一時的なステージングバッファにコピーする
  encoder.copy_buffer_to_buffer(
    &store_buffer,
    0,
    &staging_buffer,
    0,
    buffer_size,
  );

  //
  // execute commands
  //

  queue.submit(iter::once(encoder.finish()));

  //
  // map and read buffer
  //

  // データがステージングメモリに移動したら、map_asyncを使ってGPUメモリをロックし、GPUからCPUにコピーできるようにする
  // ref: https://sotrh.github.io/learn-wgpu/news/0.13/

  let buffer_slice = staging_buffer.slice(..);

  let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
  buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
    tx.send(result).unwrap();
  });
  device.poll(wgpu::Maintain::Wait);
  rx.receive().await.unwrap().unwrap();

  // コピーが完了したら、そのデータをデバッグ用のコンソールに出力する

  let data = buffer_slice.get_mapped_range();

  println!(
    "Value after computation: {:?}",
    data
      // 指定したサイズ（ここでは4バイト）ごとにスライスを分割
      // もしdataの長さが4の倍数でない場合、chunks_exactは余ったバイトを無視する
      .chunks_exact(buffer_size as usize)
      // neは「ネイティブエンディアン」の略で、実行環境のエンディアン（バイト順序）に依存して変換を行う
      // 引数は[u8; 4]型を要求するため、try_intoを使って変換する
      .map(|b| f32::from_ne_bytes(b.try_into().unwrap()))
      .collect::<Vec<f32>>()
  );

  drop(data);

  // マッピングを解除してメモリを解放
  staging_buffer.unmap();

  Ok(())
}
