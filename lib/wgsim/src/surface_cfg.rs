pub struct SurfaceConfigBuilder<'a> {
  usage: wgpu::TextureUsages,
  format: Option<wgpu::TextureFormat>,
  present_mode: wgpu::PresentMode,
  alpha_mode: Option<wgpu::CompositeAlphaMode>,
  view_formats: &'a [wgpu::TextureFormat],
  desired_maximum_frame_latency: u32,
}

impl<'a> SurfaceConfigBuilder<'a> {
  pub fn new() -> Self {
    Self {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: None,
      present_mode: wgpu::PresentMode::Fifo,
      alpha_mode: None,
      view_formats: &[],
      desired_maximum_frame_latency: 2,
    }
  }

  pub fn format(mut self, format: wgpu::TextureFormat) -> Self {
    self.format = Some(format);
    self
  }

  pub fn alpha_mode(mut self, mode: wgpu::CompositeAlphaMode) -> Self {
    self.alpha_mode = Some(mode);
    self
  }

  pub fn build(
    &self,
    adapter: &'a wgpu::Adapter,
    surface: &wgpu::Surface,
    width: u32,
    height: u32,
  ) -> wgpu::SurfaceConfiguration {
    let surface_caps = surface.get_capabilities(&adapter);

    wgpu::SurfaceConfiguration {
      usage: self.usage,
      format: self.format.unwrap_or(surface_caps.formats[0]),
      width,
      height,
      present_mode: self.present_mode,
      alpha_mode: self.alpha_mode.unwrap_or(surface_caps.alpha_modes[0]),
      view_formats: self.view_formats.to_vec(),
      desired_maximum_frame_latency: self.desired_maximum_frame_latency,
    }
  }
}
