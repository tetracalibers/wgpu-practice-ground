use wgpu::{include_wgsl, util::DeviceExt};
use winit::dpi::PhysicalSize;

use crate::{
  color::Color,
  geometry_value::{Bounds, Corners},
};

pub struct Rect {
  pub color: Color,
  pub bounds: Bounds<f32>,
  pub corners: Corners<f32>,
  pub sigma: f32,
}

const RECTANGLE_STRUCT_SIZE: usize = 4 + 2 + 2 + 4 + 1 + 2; // color + origin + size + corners + sigma + window_size
const MAX_RECTANGLE_COUNT: usize = 1024;
const RECTANGLE_BUFFER_SIZE: usize =
  RECTANGLE_STRUCT_SIZE * MAX_RECTANGLE_COUNT;

const FULL_SCREEN_QUAD_VERTICES: [f32; 12] =
  [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];

const SAMPLE_COUNT: u32 = 4;

pub struct UiRenderer {
  msaa_texture_view: wgpu::TextureView,
  vertex_buffer: wgpu::Buffer,
  rectangle_buffer: wgpu::Buffer,
  rectangle_bind_group: wgpu::BindGroup,
  rectangle_pipeline: wgpu::RenderPipeline,
  rectangle_data: Vec<f32>,
  rectangle_count: usize,
}

impl UiRenderer {
  pub fn new(
    device: &wgpu::Device,
    target_config: &wgpu::SurfaceConfiguration,
  ) -> Self {
    let vertex_buffer_layout = wgpu::VertexBufferLayout {
      array_stride: 2 * std::mem::size_of::<f32>() as u64,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: 0,
        shader_location: 0,
      }],
    };

    let vertex_buffer =
      device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("full-screen-sized quad"),
        contents: bytemuck::cast_slice(&FULL_SCREEN_QUAD_VERTICES),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      });

    let rectangle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("rectangle"),
      size: RECTANGLE_BUFFER_SIZE as u64,
      usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    let rectangle_bind_group_layout =
      device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("rectangle bind group layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        }],
      });

    let rectangle_bind_group =
      device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("rectangles"),
        layout: &rectangle_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
          binding: 0,
          resource: rectangle_buffer.as_entire_binding(),
        }],
      });

    let rectangle_module =
      device.create_shader_module(include_wgsl!("./shader/rect.wgsl"));

    let rectangle_pipeline_layout =
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("rectangle pipeline layout"),
        bind_group_layouts: &[&rectangle_bind_group_layout],
        push_constant_ranges: &[],
      });

    let rectangle_pipeline =
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("blurred rectangles"),
        layout: Some(&rectangle_pipeline_layout),
        vertex: wgpu::VertexState {
          module: &rectangle_module,
          entry_point: "vs_main",
          buffers: &[vertex_buffer_layout],
          compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
          module: &rectangle_module,
          entry_point: "fs_main",
          targets: &[Some(wgpu::ColorTargetState {
            format: target_config.format,
            blend: Some(wgpu::BlendState {
              color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                ..Default::default()
              },
              alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                ..Default::default()
              },
            }),
            write_mask: wgpu::ColorWrites::ALL,
          })],
          compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
          topology: wgpu::PrimitiveTopology::TriangleList,
          ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
          count: SAMPLE_COUNT,
          mask: !0,
          alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
      });

    let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("color"),
      size: wgpu::Extent3d {
        width: target_config.width,
        height: target_config.height,
        ..Default::default()
      },
      mip_level_count: 1,
      sample_count: SAMPLE_COUNT,
      dimension: wgpu::TextureDimension::D2,
      format: target_config.format,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::COPY_SRC,
      view_formats: &[],
    });
    let msaa_texture_view = msaa_texture.create_view(&Default::default());

    Self {
      msaa_texture_view,
      vertex_buffer,
      rectangle_buffer,
      rectangle_bind_group,
      rectangle_pipeline,
      rectangle_count: 0,
      rectangle_data: vec![],
    }
  }

  pub fn rectangle(&mut self, window: PhysicalSize<u32>, rect: Rect) {
    if self.rectangle_count >= MAX_RECTANGLE_COUNT {
      return;
    }

    let new_rectangle = vec![
      rect.color.r,
      rect.color.g,
      rect.color.b,
      rect.color.a,
      rect.bounds.origin.x,
      rect.bounds.origin.y,
      0.0,
      rect.sigma,
      rect.corners.top_left,
      rect.corners.top_right,
      rect.corners.bottom_right,
      rect.corners.bottom_left,
      rect.bounds.size.width,
      rect.bounds.size.height,
      window.width as f32,
      window.height as f32,
    ];

    self.rectangle_data.extend(new_rectangle);
    self.rectangle_count += 1;
  }

  pub fn render(
    &mut self,
    encoder: &mut wgpu::CommandEncoder,
    queue: &wgpu::Queue,
    view: &wgpu::TextureView,
  ) {
    let mut render_pass =
      encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &self.msaa_texture_view,
          resolve_target: Some(view),
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color {
              r: 1.0,
              g: 1.0,
              b: 1.0,
              a: 1.0,
            }),
            store: wgpu::StoreOp::Store,
          },
        })],
        ..Default::default()
      });

    // TODO: 検討
    //render_pass.set_viewport(0, 0, w, h, 0, 1);

    queue.write_buffer(
      &self.rectangle_buffer,
      0,
      bytemuck::cast_slice(self.rectangle_data.as_slice()),
    );

    render_pass.set_pipeline(&self.rectangle_pipeline);
    render_pass.set_bind_group(0, &self.rectangle_bind_group, &[]);
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass.draw(0..6, 0..self.rectangle_count as u32);

    self.rectangle_count = 0;
    self.rectangle_data.clear();
  }
}
