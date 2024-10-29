mod instance_defs;
mod light_defs;

use std::error::Error;

use bytemuck::cast_slice;
use cgmath::{Matrix4, Point3, Vector3};
use instance_defs::{Matrices, Shapes, Vertex};
use light_defs::DirectionLight;
use wgpu::util::DeviceExt;
use wgsim::app::App;
use wgsim::ctx::WgpuContext;
use wgsim::export::Gif;
use wgsim::matrix;
use wgsim::ppl::RenderPipelineBuilder;
use wgsim::render::{Render, RenderTarget};
use wgsim::util;
use winit::dpi::PhysicalSize;

const NUM_CUBES: u32 = 50;
const NUM_SPHERES: u32 = 50;
const NUM_TORI: u32 = 50;

fn setup(animation_speed: f32) -> Initial {
  Initial {
    camera_position: Point3::new(8., 8., 16.),
    look_direction: Point3::new(0., 0., 0.),
    up_direction: Vector3::unit_y(),

    light: DirectionLight {
      direction: Point3::new(0.2, 1., 0.3).into(),
      color: Point3::new(1., 1., 1.).into(),
    },

    animation_speed,
  }
}

pub fn run() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  let initial = setup(1.);

  let mut app: App<State> =
    App::new("instanced_cube_sphere_torus - direction_light_2", initial)
      .with_msaa();
  app.run()?;

  Ok(())
}

pub async fn export_gif() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  let initial = setup(3.);

  let mut gif = Gif::<State>::new(1024, initial, true).await;
  gif
    .export(
      "export/instanced-cube-sphere-torus_direction-light-diffuse-ambient.gif",
      50,
      30,
    )
    .await?;

  Ok(())
}

struct Initial {
  pub camera_position: Point3<f32>,
  pub look_direction: Point3<f32>,
  pub up_direction: Vector3<f32>,

  pub light: DirectionLight,

  pub animation_speed: f32,
}

struct State {
  pipeline: wgpu::RenderPipeline,

  shapes: Shapes,

  vert_bind_group: wgpu::BindGroup,
  frag_bind_group: wgpu::BindGroup,

  msaa_texture_view: wgpu::TextureView,
  depth_texture_view: wgpu::TextureView,

  light_uniform_buffer: wgpu::Buffer,
  vp_uniform_buffer: wgpu::Buffer,

  view_mat: Matrix4<f32>,
  project_mat: Matrix4<f32>,

  animation_speed: f32,
}

impl<'a> Render<'a> for State {
  type Initial = Initial;

  async fn new(ctx: &WgpuContext<'a>, initial: &Self::Initial) -> Self {
    //
    // shader
    //

    let vs_shader = ctx
      .device
      .create_shader_module(wgpu::include_wgsl!("./shader-vert.wgsl"));
    let fs_shader = ctx
      .device
      .create_shader_module(wgpu::include_wgsl!("./shader-frag.wgsl"));

    //
    // matrix
    //

    let objects_count = NUM_CUBES + NUM_SPHERES + NUM_TORI;
    let aspect = ctx.size.width as f32 / ctx.size.height as f32;

    let Matrices {
      model_mat,
      normal_mat,
      color_vec,
    } = instance_defs::create_transform_mat_color(objects_count, true);

    let view_mat = matrix::create_view_mat(
      initial.camera_position,
      initial.look_direction,
      initial.up_direction,
    );
    let project_mat = matrix::create_projection_mat(aspect, true);
    let vp_mat = project_mat * view_mat;

    //
    // uniform
    //

    let vp_uniform_buffer =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("View-Projection Buffer"),
        contents: cast_slice(vp_mat.as_ref() as &[f32; 16]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      });

    let model_uniform_buffer =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Model Uniform Buffer"),
        contents: cast_slice(model_mat.as_slice()),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      });

    let normal_uniform_buffer =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Normal Uniform Buffer"),
        contents: cast_slice(normal_mat.as_slice()),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      });

    let color_uniform_buffer =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("color Uniform Buffer"),
        contents: cast_slice(color_vec.as_slice()),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      });

    let light_uniform_buffer =
      ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Light Uniform Buffer"),
        size: (std::mem::size_of::<[f32; 4]>() * 4) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      });
    ctx.queue.write_buffer(
      &light_uniform_buffer,
      4 * 4 * 0,
      cast_slice(&initial.light.direction),
    );
    ctx.queue.write_buffer(
      &light_uniform_buffer,
      4 * 4 * 1,
      cast_slice(&initial.light.color),
    );

    //
    // uniform bind group for vertex shader
    //

    let vert_bind_group_layout = util::create_bind_group_layout(
      &ctx.device,
      &[
        wgpu::BufferBindingType::Uniform,
        wgpu::BufferBindingType::Storage { read_only: true },
        wgpu::BufferBindingType::Storage { read_only: true },
        wgpu::BufferBindingType::Storage { read_only: true },
      ],
      &[
        wgpu::ShaderStages::VERTEX,
        wgpu::ShaderStages::VERTEX,
        wgpu::ShaderStages::VERTEX,
        wgpu::ShaderStages::VERTEX,
      ],
    );

    let vert_bind_group = util::create_bind_group(
      &ctx.device,
      &vert_bind_group_layout,
      &[
        vp_uniform_buffer.as_entire_binding(),
        model_uniform_buffer.as_entire_binding(),
        normal_uniform_buffer.as_entire_binding(),
        color_uniform_buffer.as_entire_binding(),
      ],
    );

    //
    // uniform bind group for fragment shader
    //

    let frag_bind_group_layout = util::create_bind_group_layout(
      &ctx.device,
      &[wgpu::BufferBindingType::Uniform],
      &[wgpu::ShaderStages::FRAGMENT],
    );

    let frag_bind_group = util::create_bind_group(
      &ctx.device,
      &frag_bind_group_layout,
      &[light_uniform_buffer.as_entire_binding()],
    );

    //
    // pipeline
    //

    let vertex_buffer_layout = [wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
    }];

    let pipeline_layout =
      ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&vert_bind_group_layout, &frag_bind_group_layout],
        push_constant_ranges: &[],
      });

    let pipeline_builder = RenderPipelineBuilder::new(&ctx)
      .vs_shader(&vs_shader, "vs_main")
      .fs_shader(&fs_shader, "fs_main")
      .pipeline_layout(&pipeline_layout)
      .vertex_buffer_layout(&vertex_buffer_layout);

    let pipeline = pipeline_builder.build();

    //
    // texture views
    //

    let msaa_texture_view = util::create_msaa_texture_view(&ctx);
    let depth_texture_view = util::create_depth_view(&ctx);

    //
    // vertex and index buffers for objects
    //

    let shapes = instance_defs::create_object_buffers(&ctx.device);

    Self {
      pipeline,
      shapes,
      vert_bind_group,
      frag_bind_group,
      msaa_texture_view,
      depth_texture_view,
      light_uniform_buffer,
      vp_uniform_buffer,
      view_mat,
      project_mat,
      animation_speed: initial.animation_speed,
    }
  }

  fn resize(&mut self, ctx: &WgpuContext<'_>, size: Option<PhysicalSize<u32>>) {
    let size = size.unwrap_or(ctx.size);

    if size.width > 0 && size.height > 0 {
      if let Some(surface) = &ctx.surface {
        surface.configure(&ctx.device, &ctx.config.as_ref().unwrap());
      }

      self.project_mat = matrix::create_projection_mat(
        size.width as f32 / size.height as f32,
        true,
      );

      self.depth_texture_view = util::create_depth_view(ctx);

      if ctx.sample_count > 1 {
        self.msaa_texture_view = util::create_msaa_texture_view(&ctx);
      }
    }
  }

  fn update(&mut self, ctx: &WgpuContext, dt: std::time::Duration) {
    let dt = self.animation_speed * dt.as_secs_f32();
    let sin = 10.0 * (0.5 + dt.sin());
    let cos = 10.0 * (0.5 + dt.cos());

    ctx.queue.write_buffer(
      &self.light_uniform_buffer,
      0,
      cast_slice([-0.2 * sin, -0.3 * cos, -1.0].as_ref()),
    );

    let view_project_mat = self.project_mat * self.view_mat;
    let view_projection_ref: &[f32; 16] = view_project_mat.as_ref();
    ctx.queue.write_buffer(
      &self.vp_uniform_buffer,
      0,
      cast_slice(view_projection_ref),
    );
  }

  fn draw(
    &mut self,
    encoder: &mut wgpu::CommandEncoder,
    target: RenderTarget,
    sample_count: u32,
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

    let color_attach = util::create_color_attachment(&view);
    let msaa_attach =
      util::create_msaa_color_attachment(&view, &self.msaa_texture_view);
    let color_attachment = if sample_count == 1 {
      color_attach
    } else {
      msaa_attach
    };
    let depth_attachment =
      util::create_depth_stencil_attachment(&self.depth_texture_view);

    let mut render_pass =
      encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(color_attachment)],
        depth_stencil_attachment: Some(depth_attachment),
        ..Default::default()
      });

    render_pass.set_pipeline(&self.pipeline);
    render_pass.set_bind_group(0, &self.vert_bind_group, &[]);
    render_pass.set_bind_group(1, &self.frag_bind_group, &[]);

    //
    // draw cubes
    //
    render_pass.set_vertex_buffer(0, self.shapes.cube.vertex_buffer.slice(..));
    render_pass.set_index_buffer(
      self.shapes.cube.index_buffer.slice(..),
      wgpu::IndexFormat::Uint16,
    );
    render_pass.draw_indexed(0..self.shapes.cube.index_count, 0, 0..NUM_CUBES);

    //
    // draw spheres
    //
    render_pass
      .set_vertex_buffer(0, self.shapes.sphere.vertex_buffer.slice(..));
    render_pass.set_index_buffer(
      self.shapes.sphere.index_buffer.slice(..),
      wgpu::IndexFormat::Uint16,
    );
    render_pass.draw_indexed(
      0..self.shapes.sphere.index_count,
      0,
      NUM_CUBES..NUM_CUBES + NUM_SPHERES,
    );

    //
    // draw tori
    //
    render_pass.set_vertex_buffer(0, self.shapes.torus.vertex_buffer.slice(..));
    render_pass.set_index_buffer(
      self.shapes.torus.index_buffer.slice(..),
      wgpu::IndexFormat::Uint16,
    );
    render_pass.draw_indexed(
      0..self.shapes.torus.index_count,
      0,
      NUM_CUBES + NUM_SPHERES..NUM_CUBES + NUM_SPHERES + NUM_TORI,
    );

    drop(render_pass);

    Ok(frame)
  }
}
