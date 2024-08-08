use etagere::{size2, Allocation, AtlasAllocator, BucketedAtlasAllocator};

struct Atlas {
  packer: BucketedAtlasAllocator,
  texture_view: wgpu::TextureView,
  // TODO: cache
}

impl Atlas {
  const INITIAL_SIZE: u32 = 256;

  pub fn new(
    device: &wgpu::Device,
    texture_format: wgpu::TextureFormat,
  ) -> Self {
    let size = Self::INITIAL_SIZE; // TODO: max device.limits()

    let packer = BucketedAtlasAllocator::new(size2(size as i32, size as i32));

    let texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("font atlas texture"),
      size: wgpu::Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: texture_format, // TODO: calc texture format from kind
      usage: wgpu::TextureUsages::TEXTURE_BINDING
        | wgpu::TextureUsages::COPY_DST,
      view_formats: &[],
    });

    let texture_view =
      texture.create_view(&wgpu::TextureViewDescriptor::default());

    Self {
      packer,
      texture_view,
    }
  }

  pub fn try_allocate(
    &mut self,
    width: usize,
    height: usize,
  ) -> Option<Allocation> {
    let size = size2(width as i32, height as i32);
    self.packer.allocate(size)

    // TODO: for failed allocation
  }
}
