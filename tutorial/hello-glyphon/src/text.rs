pub struct TextShape<'a> {
  pub content: &'a str,
  pub attributes: glyphon::Attrs<'a>,
  pub metrics: glyphon::Metrics,
}

pub struct RenderTarget {
  pub width: u32,
  pub height: u32,
  pub scale_factor: f32,
}

pub struct Text {
  renderer: glyphon::TextRenderer,
  font_system: glyphon::FontSystem,
  atlas: glyphon::TextAtlas,
  swash_cache: glyphon::SwashCache,
}

impl Text {
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    swapchain_format: wgpu::TextureFormat,
  ) -> Self {
    let font_system = glyphon::FontSystem::new();
    let swash_cache = glyphon::SwashCache::new();
    let mut atlas = glyphon::TextAtlas::new(device, &queue, swapchain_format);

    let renderer = glyphon::TextRenderer::new(
      &mut atlas,
      &device,
      wgpu::MultisampleState::default(),
      None,
    );

    Self {
      renderer,
      font_system,
      atlas,
      swash_cache,
    }
  }

  pub fn init_buffer(
    &mut self,
    text_shape: TextShape,
    render_target: RenderTarget,
  ) -> glyphon::Buffer {
    let TextShape {
      content,
      attributes,
      metrics,
    } = text_shape;
    let RenderTarget {
      width,
      height,
      scale_factor,
    } = render_target;

    let mut text_buffer = glyphon::Buffer::new(&mut self.font_system, metrics);

    let physical_width = width as f32 * scale_factor;
    let physical_height = height as f32 * scale_factor;

    text_buffer.set_size(
      &mut self.font_system,
      physical_width,
      physical_height,
    );
    text_buffer.set_text(
      &mut self.font_system,
      content,
      attributes,
      glyphon::Shaping::Advanced,
    );
    text_buffer.shape_until_scroll(&mut self.font_system);

    text_buffer
  }

  pub fn prepare<'a>(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    resolution: glyphon::Resolution,
    text_areas: impl IntoIterator<Item = glyphon::TextArea<'a>>,
  ) -> Result<(), glyphon::PrepareError> {
    self.renderer.prepare(
      device,
      queue,
      &mut self.font_system,
      &mut self.atlas,
      resolution,
      text_areas,
      &mut self.swash_cache,
    )?;
    Ok(())
  }

  pub fn render<'rpass>(
    &'rpass self,
    pass: &mut wgpu::RenderPass<'rpass>,
  ) -> Result<(), glyphon::RenderError> {
    self.renderer.render(&self.atlas, pass)?;
    Ok(())
  }

  pub fn clean_for_next(&mut self) {
    self.atlas.trim();
  }
}
