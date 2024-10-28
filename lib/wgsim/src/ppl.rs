use crate::ctx::WgpuContext;

pub struct RenderSet<'a> {
  pub shader: Option<&'a wgpu::ShaderModule>,
  pub vs_shader: Option<&'a wgpu::ShaderModule>,
  pub fs_shader: Option<&'a wgpu::ShaderModule>,
  pub vertex_buffer_layout: &'a [wgpu::VertexBufferLayout<'a>],
  pub pipeline_layout: Option<&'a wgpu::PipelineLayout>,
  pub topology: wgpu::PrimitiveTopology,
  pub strip_index_format: Option<wgpu::IndexFormat>,
  pub cull_mode: Option<wgpu::Face>,
  pub is_depth_stencil: bool,
  pub vs_entry: &'a str,
  pub fs_entry: &'a str,
}

impl<'a> Default for RenderSet<'a> {
  fn default() -> Self {
    Self {
      shader: None,
      vs_shader: None,
      fs_shader: None,
      vertex_buffer_layout: &[],
      pipeline_layout: None,
      topology: wgpu::PrimitiveTopology::TriangleList,
      strip_index_format: None,
      cull_mode: None,
      is_depth_stencil: true,
      vs_entry: "vs_main",
      fs_entry: "fs_main",
    }
  }
}

impl RenderSet<'_> {
  pub fn new(&mut self, init: &WgpuContext) -> wgpu::RenderPipeline {
    if self.shader.is_some() {
      self.vs_shader = self.shader;
      self.fs_shader = self.shader;
    }

    let depth_stencil = if self.is_depth_stencil {
      Some(wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth24Plus,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::LessEqual,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
      })
    } else {
      None
    };

    init.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: self.pipeline_layout,
      vertex: wgpu::VertexState {
        module: &self.vs_shader.unwrap(),
        entry_point: &self.vs_entry,
        buffers: &self.vertex_buffer_layout,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
      },
      fragment: Some(wgpu::FragmentState {
        module: &self.fs_shader.unwrap(),
        entry_point: &self.fs_entry,
        targets: &[Some(init.format.into())],
        compilation_options: wgpu::PipelineCompilationOptions::default(),
      }),
      primitive: wgpu::PrimitiveState {
        topology: self.topology,
        strip_index_format: self.strip_index_format,
        ..Default::default()
      },
      depth_stencil,
      multisample: wgpu::MultisampleState {
        count: init.sample_count,
        ..Default::default()
      },
      multiview: None,
      cache: None,
    })
  }
}
