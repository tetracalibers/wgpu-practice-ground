use std::{marker::PhantomData, ops::Deref};

use wgpu::util::DeviceExt;

pub struct BufferBuilder<'a, C: bytemuck::Pod> {
  usages: wgpu::BufferUsages,
  label: wgpu::Label<'a>,
  _ty: PhantomData<C>,
}

impl<'a, C: bytemuck::Pod> BufferBuilder<'a, C> {
  pub fn new() -> Self {
    Self {
      usages: wgpu::BufferUsages::empty(),
      label: None,
      _ty: PhantomData,
    }
  }

  ///
  /// Set the VERTEX usage.
  ///
  pub fn vertex(mut self) -> Self {
    self.usages |= wgpu::BufferUsages::VERTEX;
    self
  }
  ///
  /// Set the INDEX usage.
  ///
  pub fn index(mut self) -> Self {
    self.usages |= wgpu::BufferUsages::INDEX;
    self
  }

  ///
  /// Set the STORAGE usage.
  ///
  pub fn storage(mut self) -> Self {
    self.usages |= wgpu::BufferUsages::STORAGE;
    self
  }

  ///
  /// Set the UNIFORM usage.
  ///
  pub fn uniform(mut self) -> Self {
    self.usages |= wgpu::BufferUsages::UNIFORM;
    self
  }

  ///
  /// Set the COPY_DST usage.
  ///
  pub fn copy_dst(mut self) -> Self {
    self.usages |= wgpu::BufferUsages::COPY_DST;
    self
  }

  ///
  /// Set the COPY_SRC usage.
  ///
  pub fn copy_src(mut self) -> Self {
    self.usages |= wgpu::BufferUsages::COPY_SRC;
    self
  }
  ///
  /// Set the MAP_READ usage.
  ///
  pub fn read(mut self) -> Self {
    self.usages |= wgpu::BufferUsages::MAP_READ;
    self
  }
  ///
  /// Set the MAP_WRITE usage.
  ///
  pub fn write(mut self) -> Self {
    self.usages |= wgpu::BufferUsages::MAP_WRITE;
    self
  }

  ///
  /// Set buffer usages for the buffer.
  ///
  pub fn set_usage(mut self, usage: wgpu::BufferUsages) -> Self {
    self.usages = usage;
    self
  }

  ///
  /// Set the label of the buffer.
  ///
  pub fn set_label(mut self, label: &'a str) -> Self {
    self.label = Some(label);
    self
  }

  ///
  /// Build a buffer with data.
  ///
  pub fn build(&self, device: &wgpu::Device, data: &[C]) -> Buffer<C> {
    Buffer::<C>::new(device, self.usages, self.label, data)
  }

  ///
  /// Build a buffer with length. Data in the buffer is undefined.
  ///
  pub fn build_empty(&self, device: &wgpu::Device, len: usize) -> Buffer<C> {
    Buffer::<C>::new_empty(device, self.usages, self.label, len)
  }
}

///
/// A typesafe wrapper for wgpu::Buffer.
///
#[allow(unused)]
pub struct Buffer<C: bytemuck::Pod> {
  pub buffer: wgpu::Buffer,
  len: usize,
  usage: wgpu::BufferUsages,
  label: Option<String>,
  _pd: PhantomData<C>,
}

impl<C: bytemuck::Pod> Buffer<C> {
  pub fn new_empty(
    device: &wgpu::Device,
    usage: wgpu::BufferUsages,
    label: wgpu::Label,
    len: usize,
  ) -> Self {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label,
      size: (len * std::mem::size_of::<C>()) as u64,
      usage,
      mapped_at_creation: false,
    });

    let label = label.map(|x| x.to_string());

    Self {
      buffer,
      len,
      usage,
      label,
      _pd: PhantomData,
    }
  }

  pub fn new(
    device: &wgpu::Device,
    usage: wgpu::BufferUsages,
    label: wgpu::Label,
    data: &[C],
  ) -> Self {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label,
      contents: bytemuck::cast_slice(data),
      usage,
    });

    let label = label.map(|x| x.to_string());

    Self {
      buffer,
      len: data.len(),
      usage,
      label,
      _pd: PhantomData,
    }
  }

  pub fn write_buffer(
    &mut self,
    queue: &wgpu::Queue,
    offset: usize,
    data: &[C],
  ) {
    queue.write_buffer(
      &self.buffer,
      (offset * std::mem::size_of::<C>()) as u64,
      bytemuck::cast_slice(data),
    );
  }
}

impl<C: bytemuck::Pod> Deref for Buffer<C> {
  type Target = wgpu::Buffer;

  fn deref(&self) -> &Self::Target {
    &self.buffer
  }
}

impl<C: bytemuck::Pod> Into<wgpu::Buffer> for Buffer<C> {
  fn into(self) -> wgpu::Buffer {
    self.buffer
  }
}
