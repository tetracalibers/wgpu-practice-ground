mod vertex;

use std::error::Error;
use std::time;

use rand::Rng;
use vertex::{Vertex, VERTICES};
use wgpu::util::DeviceExt;
use wgpu_helper::context as helper_util;
use wgpu_helper::{
  context::WgpuContext,
  framework::with_gif::{App, Gif, Render, RenderTarget},
};

// グリッドの縦方向と横方向にそれぞれいくつのセルが存在するか
// 整数値で十分だが、シェーダー側でのキャストが面倒なので最初から浮動小数点値で定義
const GRID_SIZE: f32 = 32.0;

// simulation.wgslの@workgroup_sizeと一致させる必要がある
const WORKGROUP_SIZE: f32 = 8.0;

fn setup() -> Initial {
  Initial {}
}

pub fn run() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  let initial = setup();

  let mut app: App<State> = App::new("with_gif/life_game", initial)
    .with_window_size(512, 512)
    .with_update_interval(time::Duration::from_millis(150));
  app.run()?;

  Ok(())
}

pub async fn export_gif() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  let initial = setup();

  let mut gif = Gif::<State>::new(512, initial, false).await;
  gif.export("export/with_gif-lige_game-3.gif", 30, 10).await?;

  Ok(())
}

struct Initial {}

struct State {
  //
  // pipelines
  //
  render_pipeline: wgpu::RenderPipeline,
  simulation_pipeline: wgpu::ComputePipeline,

  //
  // model data
  //
  vertex_buffer: wgpu::Buffer,
  num_vertices: u32,
  num_instances: u32,

  //
  // for Ping-Pong patter
  //
  ping_pong_bind_groups: Vec<wgpu::BindGroup>,
  step: usize,
}

impl<'a> Render<'a> for State {
  type Initial = Initial;

  async fn new(ctx: &WgpuContext<'a>, _initial: &Self::Initial) -> Self {
    //
    // shader
    //
    let render_shader = ctx
      .device
      .create_shader_module(wgpu::include_wgsl!("./shader/render.wgsl"));
    let simulation_shader = ctx
      .device
      .create_shader_module(wgpu::include_wgsl!("./shader/simulation.wgsl"));

    //
    // vertex
    //

    let vertex_buffer =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Cell vertices"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      });

    let num_vertices = VERTICES.len() as u32;

    //
    // instance variables
    //

    let grid_size = GRID_SIZE as u32;
    let num_instances = grid_size * grid_size;

    //
    // uniform buffer
    //

    let grid_uniform_buffer =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Grid uniforms"),
        contents: bytemuck::cast_slice(&[GRID_SIZE, GRID_SIZE]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      });

    //
    // storage buffer
    //

    let mut rng = rand::thread_rng();
    let cell_state: Vec<u32> = (0..grid_size * grid_size)
      .map(|_| if rng.gen::<f32>() > 0.6 { 1 } else { 0 })
      .collect();

    let cell_state_storage_buffer_1 =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Cell State 1"),
        contents: bytemuck::cast_slice(cell_state.as_slice()),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      });
    let cell_state_storage_buffer_2 =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Cell State 2"),
        contents: bytemuck::cast_slice(cell_state.as_slice()),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      });

    //
    // bind group layout
    //

    let bind_group_layout = helper_util::create_bind_group_layout(
      &ctx.device,
      &[
        wgpu::BufferBindingType::Uniform,
        wgpu::BufferBindingType::Storage { read_only: true },
        wgpu::BufferBindingType::Storage { read_only: false },
      ],
      &[
        wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE,
        wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE,
        wgpu::ShaderStages::COMPUTE,
      ],
    );

    let bind_group_1 = helper_util::create_bind_group(
      &ctx.device,
      &bind_group_layout,
      &[
        grid_uniform_buffer.as_entire_binding(),
        cell_state_storage_buffer_1.as_entire_binding(), // input (1)
        cell_state_storage_buffer_2.as_entire_binding(), // output (2)
      ],
    );
    let bind_group_2 = helper_util::create_bind_group(
      &ctx.device,
      &bind_group_layout,
      &[
        grid_uniform_buffer.as_entire_binding(),
        cell_state_storage_buffer_2.as_entire_binding(), // input (2)
        cell_state_storage_buffer_1.as_entire_binding(), // output (1)
      ],
    );

    //
    // pipeline
    //

    let pipeline_layout =
      ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
      });

    let render_pipeline =
      ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
          module: &render_shader,
          entry_point: "vs_main",
          buffers: &[Vertex::layout()],
          compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
          module: &render_shader,
          entry_point: "fs_main",
          targets: &[Some(wgpu::ColorTargetState {
            format: ctx.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
          })],
          compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
          topology: wgpu::PrimitiveTopology::TriangleList,
          cull_mode: Some(wgpu::Face::Back),
          ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
      });

    let simulation_pipeline =
      ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Simulation pipeline"),
        layout: Some(&pipeline_layout),
        module: &simulation_shader,
        entry_point: "cp_main",
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
      });

    Self {
      render_pipeline,
      simulation_pipeline,
      vertex_buffer,
      num_vertices,
      num_instances,
      ping_pong_bind_groups: vec![bind_group_1, bind_group_2],
      step: 0,
    }
  }

  fn draw(
    &mut self,
    encoder: &mut wgpu::CommandEncoder,
    target: RenderTarget,
    _sample_count: u32,
  ) -> anyhow::Result<Option<wgpu::SurfaceTexture>, wgpu::SurfaceError> {
    //
    // computing process
    //

    let mut compute_pass =
      encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("Compute Pass"),
        timestamp_writes: None,
      });

    compute_pass.set_pipeline(&self.simulation_pipeline);
    compute_pass.set_bind_group(
      0,
      &self.ping_pong_bind_groups[self.step % 2],
      &[],
    );

    let workgroup_count = (GRID_SIZE / WORKGROUP_SIZE).ceil() as u32;
    compute_pass.dispatch_workgroups(workgroup_count, workgroup_count, 1);

    drop(compute_pass);

    //
    // swap: コンピューティングパイプラインの出力バッファをレンダリングパイプラインの入力バッファにする
    //

    self.step += 1;

    //
    // get render target
    //
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

    let color_attach = wgpu::RenderPassColorAttachment {
      view: &view,
      resolve_target: None,
      ops: wgpu::Operations {
        load: wgpu::LoadOp::Clear(wgpu::Color {
          r: 0.0,
          g: 0.0,
          b: 0.2,
          a: 1.0,
        }),
        store: wgpu::StoreOp::Store,
      },
    };

    //
    // rendering process
    //

    let mut render_pass =
      encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(color_attach)],
        ..Default::default()
      });

    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(
      0,
      &self.ping_pong_bind_groups[self.step % 2],
      &[],
    );
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass.draw(0..self.num_vertices, 0..self.num_instances);

    drop(render_pass);

    //
    // return
    //

    Ok(frame)
  }
}
