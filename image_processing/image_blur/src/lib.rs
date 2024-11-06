use std::error::Error;

use bytemuck::cast_slice;
use image::GenericImageView;
use wgpu::util::DeviceExt;
use wgsim::app::App;
use wgsim::ctx::DrawingContext;
use wgsim::ppl::{ComputePipelineBuilder, RenderPipelineBuilder};
use wgsim::render::{Render, RenderTarget};
use wgsim::util;

const TILE_DIM: u32 = 128;
const BATCH: [u32; 2] = [4, 4];

fn setup() -> Initial {
  let img_bytes =
    include_bytes!("../../../assets/img/stained-glass_512x512.png");
  let image = image::load_from_memory(img_bytes).unwrap();
  let image_size = image.dimensions();

  Initial {
    image,
    image_size,
    filter_size: 15,
    iterations: 2,
  }
}

pub fn run() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  let initial = setup();

  let mut app: App<State> =
    App::new("instanced_cube_sphere_torus - base", initial);
  app.run()?;

  Ok(())
}

struct Initial {
  image: image::DynamicImage,
  image_size: (u32, u32),
  filter_size: u32,
  iterations: u32,
}

struct State {
  blur_pipeline: wgpu::ComputePipeline,
  fullscreen_quad_pipeline: wgpu::RenderPipeline,
  compute_constants_bind_group: wgpu::BindGroup,
  compute_bind_group_0: wgpu::BindGroup,
  compute_bind_group_1: wgpu::BindGroup,
  compute_bind_group_2: wgpu::BindGroup,
  show_result_bind_group: wgpu::BindGroup,

  image_size: (u32, u32),
  block_dim: u32,
  iterations: u32,
}

impl<'a> Render<'a> for State {
  type Initial = Initial;

  async fn new(ctx: &DrawingContext<'a>, initial: &Self::Initial) -> Self {
    //
    // shader
    //

    let fullscreen_quad_shader = ctx.device.create_shader_module(
      wgpu::include_wgsl!("./fullscreen-textured-quad.wgsl"),
    );
    let blur_shader =
      ctx.device.create_shader_module(wgpu::include_wgsl!("./blur.wgsl"));

    //
    // texture & sampler
    //

    let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("sampler"),
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      ..Default::default()
    });

    let texture_desc = wgpu::TextureDescriptor {
      label: Some("texture"),
      size: wgpu::Extent3d {
        width: initial.image_size.0,
        height: initial.image_size.1,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8Unorm,
      usage: wgpu::TextureUsages::COPY_DST
        | wgpu::TextureUsages::STORAGE_BINDING
        | wgpu::TextureUsages::TEXTURE_BINDING,
      view_formats: &[],
    };

    let image_texture = ctx.device.create_texture(&texture_desc);
    ctx.queue.write_texture(
      image_texture.as_image_copy(),
      &initial.image.to_rgba8(),
      wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: Some(4 * initial.image_size.0),
        rows_per_image: Some(initial.image_size.1),
      },
      wgpu::Extent3d {
        width: initial.image_size.0,
        height: initial.image_size.1,
        depth_or_array_layers: 1,
      },
    );

    let textures = (0..=1)
      .map(|_| ctx.device.create_texture(&texture_desc))
      .collect::<Vec<_>>();

    //
    // uniform
    //

    let flip_0_uniform_buffer =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("flip uniform buffer with 0"),
        contents: cast_slice(&[0u32]),
        usage: wgpu::BufferUsages::UNIFORM,
      });

    let flip_1_uniform_buffer =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("flip uniform buffer with 1"),
        contents: cast_slice(&[1u32]),
        usage: wgpu::BufferUsages::UNIFORM,
      });

    let block_dim = TILE_DIM - (initial.filter_size - 1);
    let blur_params_uniform_buffer =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("blur params uniform buffer"),
        contents: cast_slice(&[initial.filter_size, block_dim]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      });

    //
    // uniform bind group
    //

    let sampler_binding_type =
      wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering);

    let uniform_binding_type = wgpu::BindingType::Buffer {
      ty: wgpu::BufferBindingType::Uniform,
      has_dynamic_offset: false,
      min_binding_size: None,
    };

    let texture_binding_type = wgpu::BindingType::Texture {
      sample_type: wgpu::TextureSampleType::Float { filterable: true },
      view_dimension: wgpu::TextureViewDimension::D2,
      multisampled: false,
    };

    let compute_constants_bind_group_layout = util::create_bind_group_layout(
      &ctx.device,
      &[sampler_binding_type, uniform_binding_type],
      &[wgpu::ShaderStages::COMPUTE, wgpu::ShaderStages::COMPUTE],
    );
    let compute_constants_bind_group = util::create_bind_group(
      &ctx.device,
      &compute_constants_bind_group_layout,
      &[
        wgpu::BindingResource::Sampler(&sampler),
        blur_params_uniform_buffer.as_entire_binding(),
      ],
    );

    let compute_bind_group_layout = util::create_bind_group_layout(
      &ctx.device,
      &[
        texture_binding_type,
        wgpu::BindingType::StorageTexture {
          access: wgpu::StorageTextureAccess::WriteOnly,
          format: texture_desc.format,
          view_dimension: wgpu::TextureViewDimension::D2,
        },
        uniform_binding_type,
      ],
      &[
        wgpu::ShaderStages::COMPUTE,
        wgpu::ShaderStages::COMPUTE,
        wgpu::ShaderStages::COMPUTE,
      ],
    );

    let compute_bind_group_0 = util::create_bind_group(
      &ctx.device,
      &compute_bind_group_layout,
      &[
        wgpu::BindingResource::TextureView(
          &image_texture.create_view(&wgpu::TextureViewDescriptor::default()),
        ),
        wgpu::BindingResource::TextureView(
          &textures[0].create_view(&wgpu::TextureViewDescriptor::default()),
        ),
        flip_0_uniform_buffer.as_entire_binding(),
      ],
    );

    let compute_bind_group_1 = util::create_bind_group(
      &ctx.device,
      &compute_bind_group_layout,
      &[
        wgpu::BindingResource::TextureView(
          &textures[0].create_view(&wgpu::TextureViewDescriptor::default()),
        ),
        wgpu::BindingResource::TextureView(
          &textures[1].create_view(&wgpu::TextureViewDescriptor::default()),
        ),
        flip_1_uniform_buffer.as_entire_binding(),
      ],
    );

    let compute_bind_group_2 = util::create_bind_group(
      &ctx.device,
      &compute_bind_group_layout,
      &[
        wgpu::BindingResource::TextureView(
          &textures[1].create_view(&wgpu::TextureViewDescriptor::default()),
        ),
        wgpu::BindingResource::TextureView(
          &textures[0].create_view(&wgpu::TextureViewDescriptor::default()),
        ),
        flip_0_uniform_buffer.as_entire_binding(),
      ],
    );

    let show_result_bind_group_layout = util::create_bind_group_layout(
      &ctx.device,
      &[sampler_binding_type, texture_binding_type],
      &[wgpu::ShaderStages::FRAGMENT, wgpu::ShaderStages::FRAGMENT],
    );
    let show_result_bind_group = util::create_bind_group(
      &ctx.device,
      &show_result_bind_group_layout,
      &[
        wgpu::BindingResource::Sampler(&sampler),
        wgpu::BindingResource::TextureView(
          &textures[1].create_view(&wgpu::TextureViewDescriptor::default()),
        ),
      ],
    );

    //
    // pipeline
    //

    let fullscreen_quad_pipeline_layout =
      ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Fullscreen Quad Pipeline Layout"),
        bind_group_layouts: &[&show_result_bind_group_layout],
        push_constant_ranges: &[],
      });
    let fullscreen_quad_pipeline = RenderPipelineBuilder::new(&ctx)
      .vs_shader(&fullscreen_quad_shader, "vs_main")
      .fs_shader(&fullscreen_quad_shader, "fs_main")
      .pipeline_layout(&fullscreen_quad_pipeline_layout)
      .build();

    let blur_pipeline_layout =
      ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Blur Pipeline Layout"),
        bind_group_layouts: &[
          &compute_constants_bind_group_layout,
          &compute_bind_group_layout,
        ],
        push_constant_ranges: &[],
      });
    let blur_pipeline = ComputePipelineBuilder::new(&ctx.device)
      .cs_shader(&blur_shader, "cs_main")
      .pipeline_layout(&blur_pipeline_layout)
      .build();

    Self {
      blur_pipeline,
      fullscreen_quad_pipeline,
      compute_constants_bind_group,
      compute_bind_group_0,
      compute_bind_group_1,
      compute_bind_group_2,
      show_result_bind_group,

      image_size: initial.image_size,
      iterations: initial.iterations,
      block_dim,
    }
  }

  fn draw(
    &mut self,
    encoder: &mut wgpu::CommandEncoder,
    target: RenderTarget,
    _sample_count: u32,
  ) -> Result<Option<wgpu::SurfaceTexture>, wgpu::SurfaceError> {
    let (view, frame) = match target {
      RenderTarget::Surface(surface) => {
        let frame = surface.get_current_texture()?;
        let view =
          frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        (view, Some(frame))
      }
      RenderTarget::Texture(texture) => {
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (view, None)
      }
    };

    let mut compute_pass =
      encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("compute pass"),
        ..Default::default()
      });

    compute_pass.set_pipeline(&self.blur_pipeline);
    compute_pass.set_bind_group(0, &self.compute_constants_bind_group, &[]);

    compute_pass.set_bind_group(1, &self.compute_bind_group_0, &[]);
    compute_pass.dispatch_workgroups(
      self.image_size.0 / self.block_dim,
      self.image_size.1 / BATCH[1],
      1,
    );

    compute_pass.set_bind_group(1, &self.compute_bind_group_1, &[]);
    compute_pass.dispatch_workgroups(
      self.image_size.1 / self.block_dim,
      self.image_size.0 / BATCH[1],
      1,
    );

    for _ in 0..self.iterations - 1 {
      compute_pass.set_bind_group(1, &self.compute_bind_group_2, &[]);
      compute_pass.dispatch_workgroups(
        self.image_size.0 / self.block_dim,
        self.image_size.1 / BATCH[1],
        1,
      );

      compute_pass.set_bind_group(1, &self.compute_bind_group_1, &[]);
      compute_pass.dispatch_workgroups(
        self.image_size.1 / self.block_dim,
        self.image_size.0 / BATCH[1],
        1,
      );
    }

    drop(compute_pass);

    let color_attachment = util::create_color_attachment(&view);
    let mut render_pass =
      encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(color_attachment)],
        ..Default::default()
      });

    render_pass.set_pipeline(&self.fullscreen_quad_pipeline);
    render_pass.set_bind_group(0, &self.show_result_bind_group, &[]);
    render_pass.draw(0..6, 0..1);

    drop(render_pass);

    Ok(frame)
  }
}
