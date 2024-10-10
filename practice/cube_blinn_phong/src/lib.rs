use std::{future::Future, sync::Arc, time};
use std::{iter, mem};

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use cgmath::*;
use enum_rotate::EnumRotate;
use wgpu::util::DeviceExt;
use wgpu_helper::transforms as wt;
use wgpu_helper::vertex_data as vd;
use wgpu_helper::vertex_data::cube::Cube;
use wgpu_helper::wgpu_simplified as ws;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

fn create_vertices() -> (Vec<Vertex>, Vec<u16>, Vec<u16>) {
  let Cube {
    positions,
    normals,
    indices,
    indices_wireframe,
    ..
  } = vd::cube::create_cube_data(2.);

  let data = (0..positions.len())
    .map(|i| Vertex {
      position: positions[i],
      normal: normals[i],
    })
    .collect::<Vec<Vertex>>();

  (data, indices, indices_wireframe)
}

pub fn run(title: &str) -> Result<()> {
  env_logger::init();

  let args = std::env::args().collect::<Vec<String>>();
  let sample_count = args.get(2).map(|s| s.parse::<u32>()).transpose()?;
  let sample_count = sample_count.unwrap_or(1);

  let (vertex_data, index_data_1, index_data_2) = create_vertices();

  let inputs = Inputs {
    vertex_data,
    index_data_1,
    index_data_2,
    sample_count,
  };

  let initial = Initial {
    camera_position: Point3::new(3., 1.5, 3.),
    look_direction: Point3::new(0., 0., 0.),
    up_direction: Vector3::unit_y(),
    specular_color: [1., 1., 1.],
    object_color: [1., 0., 0.],
    wireframe_color: [1., 1., 0.],
    material: IMaterial::default(),
    plot_mode: PlotMode::Both,
    rotation_speed: 1.,
  };

  let mut app: App<State> = App::new(title, inputs, initial);
  app.run()?;

  Ok(())
}

struct App<'a, R>
where
  R: Render,
{
  window: Option<Arc<Window>>,
  window_title: &'a str,
  inputs: Inputs,
  initial: Initial,
  handler: Option<R>,
  render_start_time: Option<time::Instant>,
}

impl<'a, R: Render> App<'a, R> {
  pub fn new(window_title: &'a str, inputs: Inputs, initial: Initial) -> Self {
    Self {
      window: None,
      window_title,
      inputs,
      initial,
      handler: None,
      render_start_time: None,
    }
  }

  pub fn run(&mut self) -> Result<()> {
    let event_loop = EventLoop::builder().build()?;
    event_loop.run_app(self)?;

    Ok(())
  }

  fn window(&self) -> Option<&Window> {
    match &self.window {
      Some(window) => Some(window.as_ref()),
      None => None,
    }
  }
}

impl<'a, R: Render> ApplicationHandler for App<'a, R> {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let window_attributes =
      Window::default_attributes().with_title(self.window_title);
    let window = event_loop.create_window(window_attributes).unwrap();
    self.window = Some(Arc::new(window));

    let handler = R::new(
      Arc::clone(self.window.as_ref().unwrap()),
      &self.inputs,
      &self.initial,
    );
    let handler = pollster::block_on(handler);
    self.handler = Some(handler);

    self.render_start_time = Some(time::Instant::now());
  }

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    window_id: WindowId,
    event: WindowEvent,
  ) {
    let binding = self.window();
    let window = match &binding {
      Some(window) => window,
      None => return,
    };
    if window.id() != window_id {
      return;
    }

    let handler = match &mut self.handler {
      Some(handler) => handler,
      None => return,
    };
    if handler.process_event(&event) {
      return;
    }

    match event {
      WindowEvent::Resized(size) => {
        handler.resize(size);
      }
      WindowEvent::RedrawRequested => {
        let now = time::Instant::now();
        let dt = now - self.render_start_time.unwrap_or(now);
        handler.update(dt);

        match handler.draw() {
          Ok(()) => {}
          Err(wgpu::SurfaceError::Lost) => handler.resize(handler.get_size()),
          Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
          Err(e) => eprintln!("{:?}", e),
        }
      }
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      WindowEvent::KeyboardInput {
        event:
          KeyEvent {
            physical_key: PhysicalKey::Code(KeyCode::Escape),
            state: ElementState::Pressed,
            ..
          },
        ..
      } => {
        event_loop.exit();
      }
      _ => {}
    }
  }

  fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    let binding = self.window();
    let window = match &binding {
      Some(window) => window,
      None => return,
    };
    window.request_redraw();
  }
}

#[derive(Clone, Copy, EnumRotate)]
enum PlotMode {
  Shape,
  Wireframe,
  Both,
}

pub struct Inputs {
  pub vertex_data: Vec<Vertex>,
  pub index_data_1: Vec<u16>,
  pub index_data_2: Vec<u16>,
  pub sample_count: u32,
}

struct Initial {
  pub camera_position: Point3<f32>,
  pub look_direction: Point3<f32>,
  pub up_direction: Vector3<f32>,
  pub specular_color: [f32; 3],
  pub object_color: [f32; 3],
  pub wireframe_color: [f32; 3],
  pub material: IMaterial,
  pub plot_mode: PlotMode,
  pub rotation_speed: f32,
}

#[allow(opaque_hidden_inferred_bound)]
trait Render {
  fn new(
    window: Arc<Window>,
    inputs: &Inputs,
    initial: &Initial,
  ) -> impl Future<Output = Self>;
  fn get_size(&self) -> PhysicalSize<u32>;
  fn resize(&mut self, size: PhysicalSize<u32>);
  fn process_event(&mut self, event: &WindowEvent) -> bool;
  fn update(&mut self, dt: time::Duration);
  fn draw(&mut self) -> Result<(), wgpu::SurfaceError>;
}

struct State<'a> {
  init: ws::WgpuInit<'a>,
  pipelines: Vec<wgpu::RenderPipeline>,
  vertex_buffer: wgpu::Buffer,
  index_buffers: Vec<wgpu::Buffer>,
  uniform_bind_groups: Vec<wgpu::BindGroup>,
  uniform_buffers: Vec<wgpu::Buffer>,
  view_mat: Matrix4<f32>,
  project_mat: Matrix4<f32>,
  msaa_texture_view: wgpu::TextureView,
  depth_texture_view: wgpu::TextureView,
  indices_lens: Vec<u32>,
  plot_mode: PlotMode,
  rotation_speed: f32,

  ambient: f32,
  diffuse: f32,
  specular: f32,
  shininess: f32,
}

impl<'a> Render for State<'a> {
  async fn new(
    window: Arc<Window>,
    inputs: &Inputs,
    initial: &Initial,
  ) -> Self {
    let init = ws::WgpuInit::new(window, inputs.sample_count, None).await;

    let vs_shader = init
      .device
      .create_shader_module(wgpu::include_wgsl!("./shader-vert.wgsl"));
    let fs_shader = init
      .device
      .create_shader_module(wgpu::include_wgsl!("./blinn-phong-frag.wgsl"));

    let aspect = init.config.width as f32 / init.config.height as f32;
    let (view_mat, project_mat, _) = wt::create_vp_mat(
      initial.camera_position,
      initial.look_direction,
      initial.up_direction,
      aspect,
    );

    let matrix_uniform_buffer =
      init.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Matrix Uniform Buffer"),
        size: (mem::size_of::<[f32; 16]>() * 3) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      });

    let light_position: &[f32; 3] = initial.camera_position.as_ref();
    let eye_position: &[f32; 3] = initial.camera_position.as_ref();

    let light_uniform_buffer_1 =
      init.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Light Uniform Buffer 1"),
        size: (mem::size_of::<[f32; 4]>() * 4) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      });
    init.queue.write_buffer(
      &light_uniform_buffer_1,
      4 * 4 * 0,
      bytemuck::cast_slice(light_position),
    );
    init.queue.write_buffer(
      &light_uniform_buffer_1,
      4 * 4 * 1,
      bytemuck::cast_slice(eye_position),
    );
    init.queue.write_buffer(
      &light_uniform_buffer_1,
      4 * 4 * 2,
      bytemuck::cast_slice(initial.specular_color.as_ref()),
    );
    init.queue.write_buffer(
      &light_uniform_buffer_1,
      4 * 4 * 3,
      bytemuck::cast_slice(initial.object_color.as_ref()),
    );

    let light_uniform_buffer_2 =
      init.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Light Uniform Buffer 2 (for Wireframe)"),
        size: (mem::size_of::<[f32; 4]>() * 4) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      });
    init.queue.write_buffer(
      &light_uniform_buffer_2,
      4 * 4 * 0,
      bytemuck::cast_slice(light_position),
    );
    init.queue.write_buffer(
      &light_uniform_buffer_2,
      4 * 4 * 1,
      bytemuck::cast_slice(eye_position),
    );
    init.queue.write_buffer(
      &light_uniform_buffer_2,
      4 * 4 * 2,
      bytemuck::cast_slice(initial.specular_color.as_ref()),
    );
    init.queue.write_buffer(
      &light_uniform_buffer_2,
      4 * 4 * 3,
      bytemuck::cast_slice(initial.wireframe_color.as_ref()),
    );

    let material_uniform_buffer =
      init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Material Uniform Buffer"),
        contents: bytemuck::cast_slice(initial.material.as_array().as_ref()),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      });

    let (vert_bind_group_layout_1, vert_bind_group_1) =
      ws::create_uniform_bind_group(
        &init.device,
        vec![wgpu::ShaderStages::VERTEX],
        &[matrix_uniform_buffer.as_entire_binding()],
      );
    let (vert_bind_group_layout_2, vert_bind_group_2) =
      ws::create_uniform_bind_group(
        &init.device,
        vec![wgpu::ShaderStages::VERTEX],
        &[matrix_uniform_buffer.as_entire_binding()],
      );

    let (frag_bind_group_layout_1, frag_bind_group_1) =
      ws::create_uniform_bind_group(
        &init.device,
        vec![wgpu::ShaderStages::FRAGMENT, wgpu::ShaderStages::FRAGMENT],
        &[
          light_uniform_buffer_1.as_entire_binding(),
          material_uniform_buffer.as_entire_binding(),
        ],
      );
    let (frag_bind_group_layout_2, frag_bind_group_2) =
      ws::create_uniform_bind_group(
        &init.device,
        vec![wgpu::ShaderStages::FRAGMENT, wgpu::ShaderStages::FRAGMENT],
        &[
          light_uniform_buffer_2.as_entire_binding(),
          material_uniform_buffer.as_entire_binding(),
        ],
      );

    let vertex_buffer_layout_1 = wgpu::VertexBufferLayout {
      array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
    };
    let pipeline_layout_1 =
      init.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout 1"),
        bind_group_layouts: &[
          &vert_bind_group_layout_1,
          &frag_bind_group_layout_1,
        ],
        push_constant_ranges: &[],
      });
    let mut ppl_1 = ws::RenderSet {
      vs_shader: Some(&vs_shader),
      fs_shader: Some(&fs_shader),
      pipeline_layout: Some(&pipeline_layout_1),
      vertex_buffer_layout: &[vertex_buffer_layout_1],
      ..Default::default()
    };
    let pipeline_1 = ppl_1.new(&init);

    let vertex_buffer_layout_2 = wgpu::VertexBufferLayout {
      array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
    };
    let pipeline_layout_2 =
      init.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout 2 (for Wireframe)"),
        bind_group_layouts: &[
          &vert_bind_group_layout_2,
          &frag_bind_group_layout_2,
        ],
        push_constant_ranges: &[],
      });
    let mut ppl_2 = ws::RenderSet {
      topology: wgpu::PrimitiveTopology::LineList,
      vs_shader: Some(&vs_shader),
      fs_shader: Some(&fs_shader),
      pipeline_layout: Some(&pipeline_layout_2),
      vertex_buffer_layout: &[vertex_buffer_layout_2],
      ..Default::default()
    };
    let pipeline_2 = ppl_2.new(&init);

    let msaa_texture_view = ws::create_msaa_texture_view(&init);
    let depth_texture_view = ws::create_depth_view(&init);

    let vertex_buffer =
      init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&inputs.vertex_data),
        usage: wgpu::BufferUsages::VERTEX,
      });

    let index_buffer_1 =
      init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&inputs.index_data_1),
        usage: wgpu::BufferUsages::INDEX,
      });
    let index_buffer_2 =
      init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer 2 (for Wireframe)"),
        contents: bytemuck::cast_slice(&inputs.index_data_2),
        usage: wgpu::BufferUsages::INDEX,
      });

    Self {
      init,
      pipelines: vec![pipeline_1, pipeline_2],
      vertex_buffer,
      index_buffers: vec![index_buffer_1, index_buffer_2],
      uniform_bind_groups: vec![
        vert_bind_group_1,
        frag_bind_group_1,
        vert_bind_group_2,
        frag_bind_group_2,
      ],
      uniform_buffers: vec![
        matrix_uniform_buffer,
        light_uniform_buffer_1,
        material_uniform_buffer,
        light_uniform_buffer_2,
      ],
      view_mat,
      project_mat,
      msaa_texture_view,
      depth_texture_view,
      indices_lens: vec![
        inputs.index_data_1.len() as u32,
        inputs.index_data_2.len() as u32,
      ],
      plot_mode: initial.plot_mode,
      rotation_speed: initial.rotation_speed,
      ambient: initial.material.ambient_intensity,
      diffuse: initial.material.diffuse_intensity,
      specular: initial.material.specular_intensity,
      shininess: initial.material.specular_shininess,
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

  fn process_event(&mut self, event: &WindowEvent) -> bool {
    match event {
      WindowEvent::KeyboardInput {
        event:
          KeyEvent {
            physical_key,
            state: ElementState::Pressed,
            ..
          },
        ..
      } => match physical_key {
        PhysicalKey::Code(KeyCode::ControlLeft) => {
          let new_color: [f32; 3] =
            [rand::random(), rand::random(), rand::random()];
          self.change_shape_color(new_color);
          true
        }
        PhysicalKey::Code(KeyCode::AltLeft) => {
          let new_color: [f32; 3] =
            [rand::random(), rand::random(), rand::random()];
          self.change_wireframe_color(new_color);
          true
        }
        PhysicalKey::Code(KeyCode::Space) => {
          self.plot_mode = self.plot_mode.next();
          true
        }
        PhysicalKey::Code(KeyCode::KeyQ) => {
          self.ambient += 0.01;
          println!("ambient intensity = {}", self.ambient);
          true
        }
        PhysicalKey::Code(KeyCode::KeyA) => {
          self.ambient -= 0.05;
          if self.ambient < 0. {
            self.ambient = 0.;
          }
          println!("ambient intensity = {}", self.ambient);
          true
        }
        PhysicalKey::Code(KeyCode::KeyW) => {
          self.diffuse += 0.05;
          println!("diffuse intensity = {}", self.diffuse);
          true
        }
        PhysicalKey::Code(KeyCode::KeyS) => {
          self.diffuse -= 0.05;
          if self.diffuse < 0. {
            self.diffuse = 0.;
          }
          println!("diffuse intensity = {}", self.diffuse);
          true
        }
        PhysicalKey::Code(KeyCode::KeyE) => {
          self.specular += 0.05;
          println!("specular intensity = {}", self.specular);
          true
        }
        PhysicalKey::Code(KeyCode::KeyD) => {
          self.specular -= 0.05;
          if self.specular < 0. {
            self.specular = 0.;
          }
          println!("specular intensity = {}", self.specular);
          true
        }
        PhysicalKey::Code(KeyCode::KeyR) => {
          self.shininess += 5.;
          println!("specular shininess = {}", self.shininess);
          true
        }
        PhysicalKey::Code(KeyCode::KeyF) => {
          self.shininess -= 5.;
          if self.shininess < 0. {
            self.shininess = 0.;
          }
          println!("specular shininess = {}", self.shininess);
          true
        }
        PhysicalKey::Code(KeyCode::KeyT) => {
          self.rotation_speed += 0.1;
          true
        }
        PhysicalKey::Code(KeyCode::KeyG) => {
          self.rotation_speed -= 0.1;
          if self.rotation_speed < 0. {
            self.rotation_speed = 0.;
          }
          true
        }
        _ => false,
      },
      _ => false,
    }
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

    let material = [self.ambient, self.diffuse, self.specular, self.shininess];
    self.init.queue.write_buffer(
      &self.uniform_buffers[2],
      0,
      bytemuck::cast_slice(&material),
    );
  }

  fn draw(&mut self) -> Result<(), wgpu::SurfaceError> {
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

    match self.plot_mode {
      PlotMode::Shape => self.draw_shape(&mut render_pass),
      PlotMode::Wireframe => self.draw_wireframe(&mut render_pass),
      PlotMode::Both => {
        self.draw_shape(&mut render_pass);
        self.draw_wireframe(&mut render_pass);
      }
    }

    drop(render_pass);

    self.init.queue.submit(iter::once(encoder.finish()));
    frame.present();

    Ok(())
  }
}

impl State<'_> {
  fn change_shape_color(&self, color: [f32; 3]) {
    self.init.queue.write_buffer(
      &self.uniform_buffers[1],
      4 * 4 * 2,
      bytemuck::cast_slice(&color),
    );
  }

  fn change_wireframe_color(&self, color: [f32; 3]) {
    self.init.queue.write_buffer(
      &self.uniform_buffers[3],
      4 * 4 * 2,
      bytemuck::cast_slice(&color),
    );
  }

  fn draw_shape(&self, render_pass: &mut wgpu::RenderPass) {
    render_pass.set_pipeline(&self.pipelines[0]);
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass.set_index_buffer(
      self.index_buffers[0].slice(..),
      wgpu::IndexFormat::Uint16,
    );
    render_pass.set_bind_group(0, &self.uniform_bind_groups[0], &[]);
    render_pass.set_bind_group(1, &self.uniform_bind_groups[1], &[]);
    render_pass.draw_indexed(0..self.indices_lens[0], 0, 0..1);
  }

  fn draw_wireframe(&self, render_pass: &mut wgpu::RenderPass) {
    render_pass.set_pipeline(&self.pipelines[1]);
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass.set_index_buffer(
      self.index_buffers[1].slice(..),
      wgpu::IndexFormat::Uint16,
    );
    render_pass.set_bind_group(0, &self.uniform_bind_groups[2], &[]);
    render_pass.set_bind_group(1, &self.uniform_bind_groups[3], &[]);
    render_pass.draw_indexed(0..self.indices_lens[1], 0, 0..1);
  }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct IMaterial {
  ambient_intensity: f32,
  diffuse_intensity: f32,
  specular_intensity: f32,
  specular_shininess: f32,
}

impl Default for IMaterial {
  fn default() -> Self {
    Self {
      ambient_intensity: 0.2,
      diffuse_intensity: 0.8,
      specular_intensity: 0.4,
      specular_shininess: 30.,
    }
  }
}

impl IMaterial {
  fn as_array(&self) -> [f32; 4] {
    [
      self.ambient_intensity,
      self.diffuse_intensity,
      self.specular_intensity,
      self.specular_shininess,
    ]
  }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
  pub position: [f32; 3],
  pub normal: [f32; 3],
}
