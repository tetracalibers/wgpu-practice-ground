use std::error::Error;
use std::f32::consts::PI;
use std::sync::Arc;
use std::{iter, mem, time};

use bytemuck::{Pod, Zeroable};
use cgmath::*;
use wgpu::util::DeviceExt;
use wgpu_helper::framework::v1::{App, Render};
use wgpu_helper::transforms as wt;
use wgpu_helper::vertex_data as vd;
use wgpu_helper::vertex_data::cube::Cube;
use wgpu_helper::wgpu_simplified as ws;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;

pub fn run(title: &str) -> Result<(), Box<dyn Error>> {
  env_logger::init();

  let (vertex_data, index_data) = create_vertices();

  let model = Model {
    vertex_data,
    index_data,
    sample_count: 1,
  };

  let initial = Initial {
    camera_position: Point3::new(3., 1.5, 3.),
    look_direction: Point3::new(0., 0., 0.),
    up_direction: Vector3::unit_y(),
    specular_color: [1., 1., 1.],
    object_color: [0.855, 0.792, 0.969],
    material: Material::default(),
    rotation_speed: 1.,
  };

  let mut app: App<State> = App::new(title, model, initial);
  app.run()?;

  Ok(())
}

struct Model {
  pub vertex_data: Vec<Vertex>,
  pub index_data: Vec<u16>,
  pub sample_count: u32,
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
  } = vd::cube::create_cube_data(2.);

  let data = (0..positions.len())
    .map(|i| Vertex {
      position: positions[i],
      normal: normals[i],
    })
    .collect::<Vec<Vertex>>();

  (data, indices)
}

struct Initial {
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
impl Material {
  fn as_array(&self) -> [f32; 4] {
    [
      self.ambient_intensity,
      self.diffuse_intensity,
      self.specular_intensity,
      self.specular_shininess,
    ]
  }
}

struct State<'a> {
  /// drawing context
  init: ws::WgpuContext<'a>,
  pipeline: wgpu::RenderPipeline,

  /// drawing data
  vertex_buffer: wgpu::Buffer,
  index_buffer: wgpu::Buffer,
  indices_len: u32,
  uniform_bind_groups: Vec<wgpu::BindGroup>,
  uniform_buffers: Vec<wgpu::Buffer>,
  msaa_texture_view: wgpu::TextureView,
  depth_texture_view: wgpu::TextureView,

  /// transformation matrices
  view_mat: Matrix4<f32>,
  project_mat: Matrix4<f32>,

  /// rendering settings
  rotation_speed: f32,
}

impl<'a> Render for State<'a> {
  type DrawData = Model;
  type InitialState = Initial;

  async fn new(window: Arc<Window>, model: &Model, initial: &Initial) -> Self {
    let init = ws::WgpuContext::new(window, model.sample_count, None).await;

    let vs_shader = init
      .device
      .create_shader_module(wgpu::include_wgsl!("./shader-vert.wgsl"));
    let fs_shader = init
      .device
      .create_shader_module(wgpu::include_wgsl!("./shader-frag.wgsl"));

    let aspect = init.config.width as f32 / init.config.height as f32;
    let view_mat = wt::create_view_mat(
      initial.camera_position,
      initial.look_direction,
      initial.up_direction,
    );
    let project_mat =
      wt::create_perspective_mat(Rad(2. * PI / 5.), aspect, 1., 1000.);

    let matrix_uniform_buffer =
      init.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Matrix Uniform Buffer"),
        size: (mem::size_of::<[f32; 16]>() * 3) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      });

    let light_position: &[f32; 3] = initial.camera_position.as_ref();
    let eye_position: &[f32; 3] = initial.camera_position.as_ref();

    let light_uniform_buffer =
      init.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Light Uniform Buffer"),
        size: (mem::size_of::<[f32; 4]>() * 4) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      });
    init.queue.write_buffer(
      &light_uniform_buffer,
      4 * 4 * 0,
      bytemuck::cast_slice(light_position),
    );
    init.queue.write_buffer(
      &light_uniform_buffer,
      4 * 4 * 1,
      bytemuck::cast_slice(eye_position),
    );
    init.queue.write_buffer(
      &light_uniform_buffer,
      4 * 4 * 2,
      bytemuck::cast_slice(initial.specular_color.as_ref()),
    );
    init.queue.write_buffer(
      &light_uniform_buffer,
      4 * 4 * 3,
      bytemuck::cast_slice(initial.object_color.as_ref()),
    );

    let material_uniform_buffer =
      init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Material Uniform Buffer"),
        contents: bytemuck::cast_slice(initial.material.as_array().as_ref()),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      });
    init.queue.write_buffer(
      &material_uniform_buffer,
      0,
      bytemuck::cast_slice(&initial.material.as_array()),
    );

    let (vert_bind_group_layout, vert_bind_group) =
      ws::create_uniform_bind_group(
        &init.device,
        vec![wgpu::ShaderStages::VERTEX],
        &[matrix_uniform_buffer.as_entire_binding()],
      );

    let (frag_bind_group_layout, frag_bind_group) =
      ws::create_uniform_bind_group(
        &init.device,
        vec![wgpu::ShaderStages::FRAGMENT, wgpu::ShaderStages::FRAGMENT],
        &[
          light_uniform_buffer.as_entire_binding(),
          material_uniform_buffer.as_entire_binding(),
        ],
      );

    let vertex_buffer_layout = wgpu::VertexBufferLayout {
      array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
    };
    let pipeline_layout =
      init.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&vert_bind_group_layout, &frag_bind_group_layout],
        push_constant_ranges: &[],
      });
    let mut render = ws::RenderSet {
      vs_shader: Some(&vs_shader),
      fs_shader: Some(&fs_shader),
      pipeline_layout: Some(&pipeline_layout),
      vertex_buffer_layout: &[vertex_buffer_layout],
      ..Default::default()
    };
    let pipeline = render.new(&init);

    let msaa_texture_view = ws::create_msaa_texture_view(&init);
    let depth_texture_view = ws::create_depth_view(&init);

    let vertex_buffer =
      init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&model.vertex_data),
        usage: wgpu::BufferUsages::VERTEX,
      });

    let index_buffer =
      init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&model.index_data),
        usage: wgpu::BufferUsages::INDEX,
      });

    Self {
      init,
      pipeline,
      vertex_buffer,
      index_buffer,
      uniform_bind_groups: vec![vert_bind_group, frag_bind_group],
      uniform_buffers: vec![
        matrix_uniform_buffer,
        light_uniform_buffer,
        material_uniform_buffer,
      ],
      view_mat,
      project_mat,
      msaa_texture_view,
      depth_texture_view,
      indices_len: model.index_data.len() as u32,
      rotation_speed: initial.rotation_speed,
    }
  }

  fn get_size(&self) -> PhysicalSize<u32> {
    self.init.size
  }

  fn resize(&mut self, size: PhysicalSize<u32>) {
    if size.width > 0 && size.height > 0 {
      self.init.size = size;
      self.init.config.width = size.width;
      self.init.config.height = size.height;
      self.init.surface.configure(&self.init.device, &self.init.config);

      self.project_mat =
        wt::create_projection_mat(size.width as f32 / size.height as f32, true);

      self.depth_texture_view = ws::create_depth_view(&self.init);

      if self.init.sample_count > 1 {
        self.msaa_texture_view = ws::create_msaa_texture_view(&self.init);
      }
    }
  }

  fn process_event(&mut self, _event: &WindowEvent) -> bool {
    false
  }

  fn update(&mut self, dt: time::Duration) {
    let dt = self.rotation_speed * dt.as_secs_f32();

    let model_mat =
      wt::create_model_mat_with_rotation([dt.sin(), dt.cos(), 0.]);
    let view_proj_mat = self.project_mat * self.view_mat;

    let normal_mat = (model_mat.invert().unwrap()).transpose();

    let model_ref: &[f32; 16] = model_mat.as_ref();
    let view_proj_ref: &[f32; 16] = view_proj_mat.as_ref();
    let normal_ref: &[f32; 16] = normal_mat.as_ref();

    self.init.queue.write_buffer(
      &self.uniform_buffers[0],
      16 * 4 * 0,
      bytemuck::cast_slice(view_proj_ref),
    );
    self.init.queue.write_buffer(
      &self.uniform_buffers[0],
      16 * 4 * 1,
      bytemuck::cast_slice(model_ref),
    );
    self.init.queue.write_buffer(
      &self.uniform_buffers[0],
      16 * 4 * 2,
      bytemuck::cast_slice(normal_ref),
    );
  }

  fn draw(&mut self) -> anyhow::Result<(), wgpu::SurfaceError> {
    let frame = self.init.surface.get_current_texture()?;
    let view =
      frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = self.init.device.create_command_encoder(
      &wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
      },
    );

    let color_attach = ws::create_color_attachment(&view);
    let msaa_attach =
      ws::create_msaa_color_attachment(&view, &self.msaa_texture_view);
    let color_attachment = if self.init.sample_count == 1 {
      color_attach
    } else {
      msaa_attach
    };
    let depth_attachment =
      ws::create_depth_stencil_attachment(&self.depth_texture_view);

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

    self.init.queue.submit(iter::once(encoder.finish()));
    frame.present();

    Ok(())
  }
}
