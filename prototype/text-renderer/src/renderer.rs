use wgpu::{include_wgsl, util::DeviceExt};

const TEXT_STRUCT_SIZE: usize = 16; // TODO: 内訳を明記
const MAX_TEXT_COUNT: usize = 1024;
const TEXT_BUFFER_SIZE: usize = TEXT_STRUCT_SIZE * MAX_TEXT_COUNT;

const FULL_SCREEN_QUAD_VERTICES: [f32; 12] =
  [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];

pub struct Text<'a> {
  content: &'a str,
  position: [f32; 2], // TODO: replace Size struct
  font_size: f32,
  color: [f32; 4], // TODO: replace Color struct
}

pub struct FontData {
  text_bind_group: wgpu::BindGroup,
}

pub struct UiRenderer {
  viewport: (f32, f32),
  sampler: wgpu::Sampler,
  vertex_buffer: wgpu::Buffer,
  text_buffer: wgpu::Buffer,
  text_bind_group_layout: wgpu::BindGroupLayout,
  text_pipeline: wgpu::RenderPipeline,
  glyph_data: Vec<f32>,
  glyph_count: usize,
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

    let text_module =
      device.create_shader_module(include_wgsl!("./shader/text.wgsl"));

    let text_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("text"),
      size: TEXT_BUFFER_SIZE as u64,
      usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      mipmap_filter: wgpu::FilterMode::Linear,
      ..Default::default()
    });

    let text_bind_group_layout =
      device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("text bind group layout"),
        entries: &[
          wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
              ty: wgpu::BufferBindingType::Storage { read_only: true },
              has_dynamic_offset: false,
              min_binding_size: None,
            },
            count: None,
          },
          wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), // TODO: 要確認
            count: None,
          },
          wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
              sample_type: wgpu::TextureSampleType::Float { filterable: true }, // TODO: 要確認
              view_dimension: wgpu::TextureViewDimension::D2,
              multisampled: false, // TODO: MSAA?
            },
            count: None,
          },
        ],
      });

    let text_pipeline_layout =
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("text pipeline layout"),
        bind_group_layouts: &[&text_bind_group_layout],
        push_constant_ranges: &[],
      });

    let text_pipeline =
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("text"),
        layout: Some(&text_pipeline_layout),
        vertex: wgpu::VertexState {
          module: &text_module,
          entry_point: "vs_main",
          buffers: &[vertex_buffer_layout],
          compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
          module: &text_module,
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
          count: 1, // TODO: MSAA
          mask: !0,
          alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
      });

    Self {
      viewport: (target_config.width as f32, target_config.height as f32),
      sampler,
      vertex_buffer,
      text_buffer,
      text_bind_group_layout,
      text_pipeline,
      glyph_data: vec![],
      glyph_count: 0,
    }
  }

  pub fn init_font(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    (font_atlas_width, font_atlas_height): (u32, u32),
    font_atlas_data: &Vec<u8>,
  ) -> FontData {
    let font_atlas_texture_desc = wgpu::TextureDescriptor {
      label: Some("font atlas"),
      size: wgpu::Extent3d {
        width: font_atlas_width,
        height: font_atlas_height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::R8Unorm,
      usage: wgpu::TextureUsages::TEXTURE_BINDING
        | wgpu::TextureUsages::COPY_DST
        | wgpu::TextureUsages::RENDER_ATTACHMENT,
      view_formats: &[],
    };

    let font_atlas_texture = device.create_texture_with_data(
      queue,
      &font_atlas_texture_desc,
      wgpu::util::TextureDataOrder::default(),
      &font_atlas_data,
    );

    FontData {
      text_bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("text"),
        layout: &self.text_bind_group_layout,
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: self.text_buffer.as_entire_binding(),
          },
          wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Sampler(&self.sampler),
          },
          wgpu::BindGroupEntry {
            binding: 2,
            resource: wgpu::BindingResource::TextureView(
              &font_atlas_texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
            ),
          },
        ],
      }),
    }
  }

  pub fn set_viewport_size(&mut self, width: u32, height: u32) {
    self.viewport = (width as f32, height as f32);
  }

  pub fn text(
    &mut self,
    char_rects: &Vec<(f32, f32, f32, f32)>,
    [origin_x, origin_y]: [f32; 2],
    font_size: f32,
    [color_r, color_g, color_b, color_a]: [f32; 4],
    uvs: &Vec<[f32; 4]>,
  ) {
    for (i, (x, y, w, h)) in char_rects.iter().enumerate() {
      let (shape_x, shape_y) = (x + origin_x, y + origin_y);
      let [uv_x, uv_y, uv_z, uv_w] = uvs[i];
      let (viewport_w, viewport_h) = self.viewport;

      let new_text = vec![
        shape_x, shape_y, 0., font_size, color_r, color_g, color_b, color_a,
        *w, *h, uv_x, uv_y, uv_z, uv_w, viewport_w, viewport_h,
      ];

      self.glyph_data.extend(new_text);
      self.glyph_count += 1;
    }
  }

  pub fn render(
    &mut self,
    encoder: &mut wgpu::CommandEncoder,
    queue: &wgpu::Queue,
    view: &wgpu::TextureView,
    font_data: &FontData,
  ) {
    let mut render_pass =
      encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view,                 // TODO: change to MASS texture
          resolve_target: None, // TODO: change to Some(view) for MSAA
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
            store: wgpu::StoreOp::Store,
          },
        })],
        ..Default::default()
      });

    render_pass.set_viewport(
      0.0,
      0.0,
      self.viewport.0,
      self.viewport.1,
      0.0,
      1.0,
    );

    queue.write_buffer(
      &self.text_buffer,
      0,
      bytemuck::cast_slice(self.glyph_data.as_slice()),
    );

    render_pass.set_pipeline(&self.text_pipeline);
    render_pass.set_bind_group(0, &font_data.text_bind_group, &[]);
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    render_pass.draw(0..6, 0..self.glyph_count as u32);

    //self.glyph_count = 0;
    //self.glyph_data.clear();
  }
}
