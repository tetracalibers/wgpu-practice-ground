use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

#[derive(Debug)]
pub struct WgpuContext<'a> {
  pub instance: wgpu::Instance,
  pub adapter: wgpu::Adapter,
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  pub surface: Option<wgpu::Surface<'a>>,
  pub config: Option<wgpu::SurfaceConfiguration>,
  pub size: winit::dpi::PhysicalSize<u32>,
  pub format: wgpu::TextureFormat,
  pub sample_count: u32,
}

impl<'a> WgpuContext<'a> {
  pub async fn new(
    window: Arc<Window>,
    sample_count: u32,
    limits: Option<wgpu::Limits>,
  ) -> Self {
    let limits_device = limits.unwrap_or(wgpu::Limits::default());

    let size = window.inner_size();
    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(window).unwrap();

    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
      })
      .await
      .expect("Failed to find an appropriate adapter");

    let (device, queue) = adapter
      .request_device(
        &wgpu::DeviceDescriptor {
          label: None,
          required_features: wgpu::Features::default()
            | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
          required_limits: limits_device,
          ..Default::default()
        },
        None,
      )
      .await
      .expect("Failed to create device");

    let surface_caps = surface.get_capabilities(&adapter);
    let format = surface_caps.formats[0];

    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format,
      width: size.width,
      height: size.height,
      present_mode: wgpu::PresentMode::Fifo,
      alpha_mode: surface_caps.alpha_modes[0],
      view_formats: vec![],
      desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    Self {
      instance,
      surface: Some(surface),
      adapter,
      device,
      queue,
      config: Some(config),
      format,
      size,
      sample_count,
    }
  }

  pub async fn new_without_surface(
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    sample_count: u32,
  ) -> Self {
    let size = PhysicalSize::new(width, height);

    let instance = wgpu::Instance::default();

    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions::default())
      .await
      .unwrap();

    let (device, queue) = adapter
      .request_device(&wgpu::DeviceDescriptor::default(), None)
      .await
      .unwrap();

    Self {
      instance,
      surface: None,
      adapter,
      device,
      queue,
      config: None,
      size,
      format,
      sample_count,
    }
  }
}

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

pub fn create_bind_group_layout(
  device: &wgpu::Device,
  binding_types: &[wgpu::BufferBindingType],
  shader_stages: &[wgpu::ShaderStages],
) -> wgpu::BindGroupLayout {
  let entries = shader_stages.iter().enumerate().map(|(i, stage)| {
    wgpu::BindGroupLayoutEntry {
      binding: i as u32,
      visibility: *stage,
      ty: wgpu::BindingType::Buffer {
        ty: binding_types[i],
        has_dynamic_offset: false,
        min_binding_size: None,
      },
      count: None,
    }
  });

  device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("Bind Group Layout"),
    entries: entries.collect::<Vec<_>>().as_slice(),
  })
}

pub fn create_bind_group(
  device: &wgpu::Device,
  layout: &wgpu::BindGroupLayout,
  resources: &[wgpu::BindingResource],
) -> wgpu::BindGroup {
  let entries =
    resources.iter().enumerate().map(|(i, resource)| wgpu::BindGroupEntry {
      binding: i as u32,
      resource: resource.clone(),
    });

  device.create_bind_group(&wgpu::BindGroupDescriptor {
    label: Some("Bind Group"),
    layout: &layout,
    entries: &entries.collect::<Vec<_>>(),
  })
}

pub fn create_uniform_bind_group_layout(
  device: &wgpu::Device,
  shader_stages: Vec<wgpu::ShaderStages>,
) -> wgpu::BindGroupLayout {
  let entries = shader_stages
    .iter()
    .enumerate()
    .map(|(i, stage)| wgpu::BindGroupLayoutEntry {
      binding: i as u32,
      visibility: *stage,
      ty: wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
      },
      count: None,
    })
    .collect::<Vec<_>>();

  device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("Uniform Bind Group Layout"),
    entries: entries.as_slice(),
  })
}

pub fn create_uniform_bind_group(
  device: &wgpu::Device,
  shader_stages: Vec<wgpu::ShaderStages>,
  resources: &[wgpu::BindingResource],
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
  let entries = resources
    .iter()
    .enumerate()
    .map(|(i, resource)| wgpu::BindGroupEntry {
      binding: i as u32,
      resource: resource.clone(),
    })
    .collect::<Vec<_>>();

  let layout = create_uniform_bind_group_layout(device, shader_stages);
  let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    label: Some("Uniform Bind Group"),
    layout: &layout,
    entries: &entries,
  });

  (layout, bind_group)
}

pub fn create_color_attachment(
  texture_view: &wgpu::TextureView,
) -> wgpu::RenderPassColorAttachment {
  wgpu::RenderPassColorAttachment {
    view: texture_view,
    resolve_target: None,
    ops: wgpu::Operations {
      load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
      store: wgpu::StoreOp::Store,
    },
  }
}

pub fn create_msaa_texture_view(init: &WgpuContext) -> wgpu::TextureView {
  let msaa_texture = init.device.create_texture(&wgpu::TextureDescriptor {
    label: None,
    size: wgpu::Extent3d {
      width: init.size.width,
      height: init.size.height,
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: init.sample_count,
    dimension: wgpu::TextureDimension::D2,
    format: init.format,
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    view_formats: &[],
  });

  msaa_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub fn create_msaa_color_attachment<'a>(
  texture_view: &'a wgpu::TextureView,
  msaa_view: &'a wgpu::TextureView,
) -> wgpu::RenderPassColorAttachment<'a> {
  wgpu::RenderPassColorAttachment {
    view: msaa_view,
    resolve_target: Some(texture_view),
    ops: wgpu::Operations {
      load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
      store: wgpu::StoreOp::Store,
    },
  }
}

pub fn create_depth_view(init: &WgpuContext) -> wgpu::TextureView {
  let depth_texture = init.device.create_texture(&wgpu::TextureDescriptor {
    label: None,
    size: wgpu::Extent3d {
      width: init.size.width,
      height: init.size.height,
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: init.sample_count,
    dimension: wgpu::TextureDimension::D2,
    format: wgpu::TextureFormat::Depth24Plus,
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    view_formats: &[],
  });

  depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub fn create_depth_stencil_attachment(
  depth_view: &wgpu::TextureView,
) -> wgpu::RenderPassDepthStencilAttachment {
  wgpu::RenderPassDepthStencilAttachment {
    view: depth_view,
    depth_ops: Some(wgpu::Operations {
      load: wgpu::LoadOp::Clear(1.0),
      store: wgpu::StoreOp::Discard,
    }),
    stencil_ops: None,
  }
}
