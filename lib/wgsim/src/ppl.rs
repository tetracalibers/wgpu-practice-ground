use crate::ctx::DrawingContext;

pub struct RenderPipelineBuilder<'a> {
  ctx: &'a DrawingContext<'a>,
  pipeline_layout: Option<&'a wgpu::PipelineLayout>,

  depth_stencil: Option<wgpu::DepthStencilState>,

  vs_shader: Option<&'a wgpu::ShaderModule>,
  vs_entry: &'a str,
  vertex_buffer_layout: &'a [wgpu::VertexBufferLayout<'a>],

  fs_shader: Option<&'a wgpu::ShaderModule>,
  fs_entry: &'a str,
  targets: Vec<Option<wgpu::ColorTargetState>>,

  primitive: wgpu::PrimitiveState,
}

impl<'a> RenderPipelineBuilder<'a> {
  pub fn new(ctx: &'a DrawingContext) -> Self {
    Self {
      ctx,
      depth_stencil: None,
      pipeline_layout: None,
      vs_shader: None,
      vs_entry: "vs_main",
      fs_shader: None,
      fs_entry: "fs_main",
      vertex_buffer_layout: &[],
      targets: vec![Some(ctx.format().into())],
      primitive: wgpu::PrimitiveState::default(),
    }
  }

  pub fn enable_depth_stencil(
    mut self,
    custom_depth_stencil: Option<wgpu::DepthStencilState>,
  ) -> Self {
    if let Some(depth_stencil) = custom_depth_stencil {
      self.depth_stencil = Some(depth_stencil);
    } else {
      self.depth_stencil = Some(wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth24Plus,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::LessEqual,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
      });
    }
    self
  }

  pub fn pipeline_layout(mut self, layout: &'a wgpu::PipelineLayout) -> Self {
    self.pipeline_layout = Some(layout);
    self
  }

  pub fn vs_shader(
    mut self,
    module: &'a wgpu::ShaderModule,
    entry: &'a str,
  ) -> Self {
    self.vs_shader = Some(module);
    self.vs_entry = entry;
    self
  }

  pub fn fs_shader(
    mut self,
    module: &'a wgpu::ShaderModule,
    entry: &'a str,
  ) -> Self {
    self.fs_shader = Some(module);
    self.fs_entry = entry;
    self
  }

  pub fn vertex_buffer_layout(
    mut self,
    layouts: &'a [wgpu::VertexBufferLayout<'a>],
  ) -> Self {
    self.vertex_buffer_layout = layouts;
    self
  }

  pub fn primitive(mut self, primitive: wgpu::PrimitiveState) -> Self {
    self.primitive = primitive;
    self
  }

  pub fn build(&self) -> wgpu::RenderPipeline {
    self.ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: self.pipeline_layout,
      vertex: wgpu::VertexState {
        module: &self.vs_shader.unwrap(),
        entry_point: &self.vs_entry,
        buffers: &self.vertex_buffer_layout,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
      },
      fragment: self.fs_shader.map(|fs_shader| wgpu::FragmentState {
        module: fs_shader,
        entry_point: self.fs_entry,
        targets: self.targets.as_slice(),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
      }),
      primitive: self.primitive,
      depth_stencil: self.depth_stencil.clone(),
      multisample: wgpu::MultisampleState {
        count: self.ctx.sample_count,
        ..Default::default()
      },
      multiview: None,
      cache: None,
    })
  }
}

pub struct ComputePipelineBuilder<'a> {
  device: &'a wgpu::Device,

  pipeline_layout: Option<&'a wgpu::PipelineLayout>,

  cs_shader: Option<&'a wgpu::ShaderModule>,
  cs_entry: &'a str,
}

impl<'a> ComputePipelineBuilder<'a> {
  pub fn new(device: &'a wgpu::Device) -> Self {
    Self {
      device,
      pipeline_layout: None,
      cs_shader: None,
      cs_entry: "cs_main",
    }
  }

  pub fn pipeline_layout(mut self, layout: &'a wgpu::PipelineLayout) -> Self {
    self.pipeline_layout = Some(layout);
    self
  }

  pub fn cs_shader(
    mut self,
    module: &'a wgpu::ShaderModule,
    entry: &'a str,
  ) -> Self {
    self.cs_shader = Some(module);
    self.cs_entry = entry;
    self
  }

  pub fn build(&self) -> wgpu::ComputePipeline {
    self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
      label: Some("Compute Pipeline"),
      layout: self.pipeline_layout,

      module: &self.cs_shader.unwrap(),
      entry_point: &self.cs_entry,

      compilation_options: wgpu::PipelineCompilationOptions::default(),
      cache: None,
    })
  }
}
