use std::{error::Error, fs::File, io::BufWriter, path::Path};

use wgpu::BufferView;

use crate::util;

pub struct ComputePixel {
  img_size: u32,
  texture: wgpu::Texture,
  texture_data_buffer: wgpu::Buffer,
  device: wgpu::Device,
  queue: wgpu::Queue,
  compute_pipeline: wgpu::ComputePipeline,
  bind_group: wgpu::BindGroup,
}

impl ComputePixel {
  pub async fn new(
    module: wgpu::ShaderModuleDescriptor<'_>,
    entry_point: &str,
    tex_format: wgpu::TextureFormat,
    img_size: u32,
  ) -> Result<Self, Box<dyn Error>> {
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

    let compute_shader = device.create_shader_module(module);

    //
    // texture
    //

    let texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("compute output texture"),
      size: wgpu::Extent3d {
        width: img_size,
        height: img_size,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: tex_format,
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
      size: 4 * img_size as u64 * img_size as u64,
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
        format: tex_format,
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
        entry_point,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
      });

    Ok(Self {
      img_size,
      texture,
      texture_data_buffer,
      device,
      queue,
      compute_pipeline,
      bind_group,
    })
  }

  pub async fn compute(
    &self,
    workgroup_size_x: u32,
    workgroup_size_y: u32,
  ) -> Result<BufferView, Box<dyn Error>> {
    //
    // commands submission
    //

    let mut command_encoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let mut compute_pass_encoder =
      command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("Compute Pass"),
        timestamp_writes: None,
      });

    compute_pass_encoder.set_pipeline(&self.compute_pipeline);
    compute_pass_encoder.set_bind_group(0, &self.bind_group, &[]);
    compute_pass_encoder.dispatch_workgroups(
      self.img_size / workgroup_size_x,
      self.img_size / workgroup_size_y,
      1,
    );

    drop(compute_pass_encoder);

    //
    // copy texture to buffer
    //

    command_encoder.copy_texture_to_buffer(
      wgpu::ImageCopyTexture {
        texture: &self.texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      },
      wgpu::ImageCopyBuffer {
        buffer: &self.texture_data_buffer,
        layout: wgpu::ImageDataLayout {
          offset: 0,
          bytes_per_row: Some(4 * self.img_size),
          rows_per_image: Some(self.img_size),
        },
      },
      wgpu::Extent3d {
        width: self.img_size,
        height: self.img_size,
        depth_or_array_layers: 1,
      },
    );

    //
    // submit GPU commands
    //

    self.queue.submit(std::iter::once(command_encoder.finish()));

    //
    // read buffer
    //

    let buffer_slice = self.texture_data_buffer.slice(..);

    let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
      tx.send(result).unwrap();
    });
    self.device.poll(wgpu::Maintain::Wait);
    rx.receive().await.unwrap()?;

    let data_view = buffer_slice.get_mapped_range();

    Ok(data_view)
  }

  pub fn export_png(
    &self,
    path: &Path,
    px_data: &[u8],
  ) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let ref mut w = BufWriter::new(file);

    let mut png_encoder = png::Encoder::new(w, self.img_size, self.img_size);
    png_encoder.set_color(png::ColorType::Rgba);

    let mut writer = png_encoder.write_header()?;
    writer.write_image_data(&px_data)?;

    Ok(())
  }

  pub fn clean_up(&self, data_view: BufferView) {
    drop(data_view);
    self.texture_data_buffer.unmap();
  }
}
