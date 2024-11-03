use std::error::Error;
use std::f32::consts::PI;
use std::{mem, time};

use bytemuck::{Pod, Zeroable};
use cgmath::*;
use wgpu::util::DeviceExt;
use wgsim::app::App;
use wgsim::ctx::{DrawingContext, Size};
use wgsim::export::Gif;
use wgsim::geometry::generator as ge;
use wgsim::geometry::Cube;
use wgsim::matrix;
use wgsim::ppl::RenderPipelineBuilder;
use wgsim::render::{Render, RenderTarget};
use wgsim::util;

pub fn run(title: &str) -> Result<(), Box<dyn Error>> {
  env_logger::init();

  let (vertex_data, index_data) = create_vertices();

  let initial = Initial {
    vertex_data,
    index_data,

    camera_position: Point3::new(3., 1.5, 3.),
    look_direction: Point3::new(0., 0., 0.),
    up_direction: Vector3::unit_y(),

    specular_color: [1., 1., 1.],
    object_color: [0.855, 0.792, 0.969],
    material: Material::default(),

    rotation_speed: 1.,
  };

  let mut app: App<State> = App::new(title, initial).with_msaa();
  app.run()?;

  Ok(())
}

pub async fn export_gif() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  let (vertex_data, index_data) = create_vertices();

  let initial = Initial {
    vertex_data,
    index_data,

    camera_position: Point3::new(3., 1.5, 3.),
    look_direction: Point3::new(0., 0., 0.),
    up_direction: Vector3::unit_y(),

    specular_color: [1., 1., 1.],
    object_color: [0.855, 0.792, 0.969],
    material: Material::default(),

    rotation_speed: 2.5,
  };

  let mut gif = Gif::<State>::new(1024, initial, true).await;
  gif.export("export/with_gif-msaa-5.gif", 50, 1).await?;

  Ok(())
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
  pub position: [f32; 3],
  pub normal: [f32; 3],
}
fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
  let Cube {
    positions,
    normals,
    indices,
    ..
  } = ge::create_cube_data(2.);

  let data = (0..positions.len())
    .map(|i| Vertex {
      position: positions[i],
      normal: normals[i],
    })
    .collect::<Vec<Vertex>>();

  (data, indices)
}

struct Initial {
  pub vertex_data: Vec<Vertex>,
  pub index_data: Vec<u16>,

  pub camera_position: Point3<f32>,
  pub look_direction: Point3<f32>,
  pub up_direction: Vector3<f32>,
  pub specular_color: [f32; 3],
  pub object_color: [f32; 3],
  pub material: Material,
  pub rotation_speed: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Material {
  ambient_intensity: f32,
  diffuse_intensity: f32,
  specular_intensity: f32,
  specular_shininess: f32,
}
impl Default for Material {
  fn default() -> Self {
    Self {
      ambient_intensity: 0.2,
      diffuse_intensity: 0.8,
      specular_intensity: 0.4,
      specular_shininess: 30.,
    }
  }
}

struct State {
  /// drawing context
  pipeline: wgpu::RenderPipeline,

  /// model data
  vertex_buffer: wgpu::Buffer,
  index_buffer: wgpu::Buffer,
  indices_len: u32,

  /// uniforms
  uniform_bind_groups: Vec<wgpu::BindGroup>,
  matrix_uniform_buffer: wgpu::Buffer,

  /// textures
  msaa_texture_view: wgpu::TextureView,
  depth_texture_view: wgpu::TextureView,

  /// transformation matrices
  view_mat: Matrix4<f32>,
  project_mat: Matrix4<f32>,

  /// rendering settings
  rotation_speed: f32,
}

impl<'a> Render<'a> for State {
  type Initial = Initial;

  async fn new(ctx: &DrawingContext<'a>, initial: &Initial) -> Self {
    let vs_shader = ctx
      .device
      .create_shader_module(wgpu::include_wgsl!("./shader-vert.wgsl"));
    let fs_shader = ctx
      .device
      .create_shader_module(wgpu::include_wgsl!("./shader-frag.wgsl"));

    let aspect = ctx.aspect_ratio();
    let view_mat = matrix::create_view_mat(
      initial.camera_position,
      initial.look_direction,
      initial.up_direction,
    );
    let project_mat =
      matrix::create_perspective_mat(Rad(2. * PI / 5.), aspect, 1., 1000.);

    let matrix_uniform_buffer =
      ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Matrix Uniform Buffer"),
        size: (mem::size_of::<[f32; 16]>() * 3) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      });

    let light_position: &[f32; 3] = initial.camera_position.as_ref();
    let eye_position: &[f32; 3] = initial.camera_position.as_ref();

    let light_uniform_buffer =
      ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Light Uniform Buffer 1"),
        size: (mem::size_of::<[f32; 4]>() * 4) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      });
    ctx.queue.write_buffer(
      &light_uniform_buffer,
      4 * 4 * 0,
      bytemuck::cast_slice(light_position),
    );
    ctx.queue.write_buffer(
      &light_uniform_buffer,
      4 * 4 * 1,
      bytemuck::cast_slice(eye_position),
    );
    ctx.queue.write_buffer(
      &light_uniform_buffer,
      4 * 4 * 2,
      bytemuck::cast_slice(initial.specular_color.as_ref()),
    );
    ctx.queue.write_buffer(
      &light_uniform_buffer,
      4 * 4 * 3,
      bytemuck::cast_slice(initial.object_color.as_ref()),
    );

    let material_uniform_buffer =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Material Uniform Buffer"),
        contents: bytemuck::cast_slice(&[initial.material]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      });

    let vert_bind_group_layout = util::create_bind_group_layout_for_buffer(
      &ctx.device,
      &[wgpu::BufferBindingType::Uniform],
      &[wgpu::ShaderStages::VERTEX],
    );
    let vert_bind_group = util::create_bind_group(
      &ctx.device,
      &vert_bind_group_layout,
      &[matrix_uniform_buffer.as_entire_binding()],
    );

    let frag_bind_group_layout = util::create_bind_group_layout_for_buffer(
      &ctx.device,
      &[
        wgpu::BufferBindingType::Uniform,
        wgpu::BufferBindingType::Uniform,
      ],
      &[wgpu::ShaderStages::FRAGMENT, wgpu::ShaderStages::FRAGMENT],
    );
    let frag_bind_group = util::create_bind_group(
      &ctx.device,
      &frag_bind_group_layout,
      &[
        light_uniform_buffer.as_entire_binding(),
        material_uniform_buffer.as_entire_binding(),
      ],
    );

    let vertex_buffer_layout = [wgpu::VertexBufferLayout {
      array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
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

    let msaa_texture_view = util::create_msaa_texture_view(&ctx);
    let depth_texture_view = util::create_depth_view(&ctx);

    let vertex_buffer =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&initial.vertex_data),
        usage: wgpu::BufferUsages::VERTEX,
      });

    let index_buffer =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&initial.index_data),
        usage: wgpu::BufferUsages::INDEX,
      });

    Self {
      pipeline,
      vertex_buffer: vertex_buffer.into(),
      index_buffer: index_buffer.into(),
      uniform_bind_groups: vec![vert_bind_group, frag_bind_group],
      matrix_uniform_buffer,
      view_mat,
      project_mat,
      msaa_texture_view,
      depth_texture_view,
      indices_len: initial.index_data.len() as u32,
      rotation_speed: initial.rotation_speed,
    }
  }

  fn resize(&mut self, ctx: &mut DrawingContext<'_>, size: Size) {
    if size.width > 0 && size.height > 0 {
      ctx.resize(size.into());

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

  fn update(&mut self, ctx: &DrawingContext, dt: time::Duration) {
    let dt = self.rotation_speed * dt.as_secs_f32();

    let model_mat =
      matrix::create_model_mat_with_rotation([dt.sin(), dt.cos(), 0.]);
    let view_proj_mat = self.project_mat * self.view_mat;
    let normal_mat = (model_mat.invert().unwrap()).transpose();

    let model_ref: &[f32; 16] = model_mat.as_ref();
    let view_proj_ref: &[f32; 16] = view_proj_mat.as_ref();
    let normal_ref: &[f32; 16] = normal_mat.as_ref();

    ctx.queue.write_buffer(
      &self.matrix_uniform_buffer,
      16 * 4 * 0,
      bytemuck::cast_slice(view_proj_ref),
    );
    ctx.queue.write_buffer(
      &self.matrix_uniform_buffer,
      16 * 4 * 1,
      bytemuck::cast_slice(model_ref),
    );
    ctx.queue.write_buffer(
      &self.matrix_uniform_buffer,
      16 * 4 * 2,
      bytemuck::cast_slice(normal_ref),
    );
  }

  fn draw(
    &mut self,
    encoder: &mut wgpu::CommandEncoder,
    target: RenderTarget,
    sample_count: u32,
  ) -> anyhow::Result<Option<wgpu::SurfaceTexture>, wgpu::SurfaceError> {
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
        timestamp_writes: None,
        occlusion_query_set: None,
      });

    render_pass.set_pipeline(&self.pipeline);
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass
      .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    render_pass.set_bind_group(0, &self.uniform_bind_groups[0], &[]);
    render_pass.set_bind_group(1, &self.uniform_bind_groups[1], &[]);
    render_pass.draw_indexed(0..self.indices_len, 0, 0..1);

    drop(render_pass);

    Ok(frame)
  }
}
