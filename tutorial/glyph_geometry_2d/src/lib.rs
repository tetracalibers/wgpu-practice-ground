mod font_data;

use std::error::Error;

use bytemuck::cast_slice;
use enum_rotate::EnumRotate;
use font_data::FontSelection;
use wgpu::util::DeviceExt;
use wgsim::app::App;
use wgsim::ctx::{DrawingContext, Size};
use wgsim::ppl::RenderPipelineBuilder;
use wgsim::render::{Render, RenderTarget};
use wgsim::util;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

fn setup() -> Initial<'static> {
  Initial {
    text: "Hello, World!",
    font_selection: FontSelection::Lusitana,
    text_position: [0.0, 0.0],
    color: [1.0, 1.0, 1.0, 1.0],
    scale: 0.45,
  }
}

pub fn run() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  let initial = setup();

  let mut app: App<State> = App::new("glyph-geometry-2d", initial).with_msaa();
  app.run()?;

  Ok(())
}

struct Initial<'a> {
  font_selection: FontSelection,
  text: &'a str,
  text_position: [f32; 2],
  color: [f32; 4],
  scale: f32,
}

struct State {
  pipeline: wgpu::RenderPipeline,

  vertex_buffer: wgpu::Buffer,
  index_buffer: wgpu::Buffer,
  index_count: u32,

  frag_bind_group: wgpu::BindGroup,

  msaa_texture_view: wgpu::TextureView,

  data_changed: bool,
  font_selection: FontSelection,
  text: String,
  text_position: [f32; 2],
  scale: f32,
}

impl<'a> Render<'a> for State {
  type Initial = Initial<'a>;

  async fn new(ctx: &DrawingContext<'a>, initial: &Self::Initial) -> Self {
    //
    // shader
    //

    let shader =
      ctx.device.create_shader_module(wgpu::include_wgsl!("./glyph_2d.wgsl"));

    //
    // uniform
    //

    let color_uniform_buffer =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("color uniform buffer"),
        contents: cast_slice(&initial.color),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      });

    //
    // uniform bind group for fragment shader
    //

    let frag_bind_group_layout = util::create_bind_group_layout_for_buffer(
      &ctx.device,
      &[wgpu::BufferBindingType::Uniform],
      &[wgpu::ShaderStages::FRAGMENT],
    );

    let frag_bind_group = util::create_bind_group(
      &ctx.device,
      &frag_bind_group_layout,
      &[color_uniform_buffer.as_entire_binding()],
    );

    //
    // pipeline
    //

    let vertex_buffer_layout = [wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<f32>() as wgpu::BufferAddress * 2,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &wgpu::vertex_attr_array![0 => Float32x2],
    }];

    let pipeline_layout =
      ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&frag_bind_group_layout],
        push_constant_ranges: &[],
      });

    let pipeline_builder = RenderPipelineBuilder::new(&ctx)
      .vs_shader(&shader, "vs_main")
      .fs_shader(&shader, "fs_main")
      .pipeline_layout(&pipeline_layout)
      .vertex_buffer_layout(&vertex_buffer_layout);

    let pipeline = pipeline_builder.build();

    //
    // texture views
    //

    let msaa_texture_view = util::create_msaa_texture_view(&ctx);

    //
    // vertex and index buffers for objects
    //

    let geometry = font_data::get_text_vertices_2d(
      initial.font_selection,
      initial.text,
      initial.text_position,
      initial.scale,
      ctx.aspect_ratio(),
    );

    let vertex_buffer =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: geometry.vertices.as_slice(),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      });

    let index_buffer =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: geometry.indices.as_slice(),
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
      });

    Self {
      pipeline,

      vertex_buffer,
      index_buffer,
      index_count: geometry.indices_len,

      frag_bind_group,

      msaa_texture_view,

      data_changed: false,
      text: initial.text.to_string(),
      text_position: initial.text_position,
      font_selection: initial.font_selection,
      scale: initial.scale,
    }
  }

  fn resize(&mut self, ctx: &mut DrawingContext<'_>, size: Size) {
    if size.width > 0 && size.height > 0 {
      ctx.resize(size.into());

      if ctx.sample_count > 1 {
        self.msaa_texture_view = util::create_msaa_texture_view(&ctx);
      }

      self.data_changed = true;
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
        PhysicalKey::Code(KeyCode::Space) => {
          self.font_selection = self.font_selection.next();
          self.data_changed = true;
          true
        }
        _ => false,
      },
      _ => false,
    }
  }

  fn update(&mut self, ctx: &DrawingContext, _dt: std::time::Duration) {
    if !self.data_changed {
      return;
    }

    let geometry = font_data::get_text_vertices_2d(
      self.font_selection,
      &self.text,
      self.text_position,
      self.scale,
      ctx.aspect_ratio(),
    );

    self.vertex_buffer.destroy();
    self.index_buffer.destroy();
    self.vertex_buffer =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: geometry.vertices.as_slice(),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      });
    self.index_buffer =
      ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: geometry.indices.as_slice(),
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
      });
    self.index_count = geometry.indices_len;

    self.data_changed = false;
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

    let mut render_pass =
      encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(color_attachment)],
        ..Default::default()
      });

    render_pass.set_pipeline(&self.pipeline);
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass
      .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    render_pass.set_bind_group(0, &self.frag_bind_group, &[]);
    render_pass.draw_indexed(0..self.index_count, 0, 0..1);

    drop(render_pass);

    Ok(frame)
  }
}
