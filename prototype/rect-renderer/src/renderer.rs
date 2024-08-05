use wgpu::include_wgsl;

use crate::{
  color::Color,
  geometry_value::{Bounds, Corners},
};

struct Rect {
  color: Color,
  bounds: Bounds<f32>,
  corners: Corners<f32>,
  sigma: f32,
}

pub struct UiRenderer {
  rectangle_pipeline: wgpu::RenderPipeline,
}

impl UiRenderer {
  pub fn new(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
  ) -> Self {
    let rectangle_module =
      device.create_shader_module(include_wgsl!("./shader/rect.wgsl"));

    let rectangle_pipeline_layout =
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("rectangle pipeline layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
      });

    let rectangle_pipeline =
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("blurred rectangles"),
        layout: Some(&rectangle_pipeline_layout),
        vertex: wgpu::VertexState {
          module: &rectangle_module,
          entry_point: "vs_main",
          buffers: &[],
          compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
          module: &rectangle_module,
          entry_point: "fs_main",
          targets: &[Some(wgpu::ColorTargetState {
            format: target_format,
            blend: Some(wgpu::BlendState::REPLACE), // TODO: update
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
          count: 1, // TODO: update
          mask: !0,
          alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
      });

    Self { rectangle_pipeline }
  }

  pub fn rectangle(&self, rect: Rect) {
    todo!()
  }

  pub fn render(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
  ) {
    let mut render_pass =
      encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view,
          resolve_target: None, // TODO: update
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

    render_pass.set_pipeline(&self.rectangle_pipeline);
    render_pass.draw(0..3, 0..1);
  }
}
