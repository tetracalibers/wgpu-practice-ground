use crate::ctx::DrawingContext;

pub fn create_bind_group_layout_for_buffer(
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

pub fn create_bind_group_layout(
  device: &wgpu::Device,
  binding_types: &[wgpu::BindingType],
  shader_stages: &[wgpu::ShaderStages],
) -> wgpu::BindGroupLayout {
  let entries = shader_stages.iter().enumerate().map(|(i, stage)| {
    wgpu::BindGroupLayoutEntry {
      binding: i as u32,
      visibility: *stage,
      ty: binding_types[i],
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

pub fn create_msaa_texture_view(init: &DrawingContext) -> wgpu::TextureView {
  let size = init.size();

  let msaa_texture = init.device.create_texture(&wgpu::TextureDescriptor {
    label: None,
    size: wgpu::Extent3d {
      width: size.width,
      height: size.height,
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count: init.sample_count,
    dimension: wgpu::TextureDimension::D2,
    format: init.format(),
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

pub fn create_depth_view(init: &DrawingContext) -> wgpu::TextureView {
  let size = init.size();

  let depth_texture = init.device.create_texture(&wgpu::TextureDescriptor {
    label: None,
    size: wgpu::Extent3d {
      width: size.width,
      height: size.height,
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
