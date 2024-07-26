use std::sync::Arc;

use winit::window::Window;

pub struct State {
  window: Arc<Window>,
}

impl State {
  pub async fn new(window: Window) -> Self {
    let window = Arc::new(window);

    let instance = Self::create_gpu_instance();
    // 描画先
    let surface = instance.create_surface(Arc::clone(&window)).unwrap();

    Self { window }
  }

  pub fn window(&self) -> &Window {
    &self.window
  }

  fn create_gpu_instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor {
      // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
      // allとすることですべてのバックエンドから必要なものを選択してくれる
      backends: wgpu::Backends::all(),
      ..Default::default()
    })
  }
}
