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
    // WebGPU内でデバイスの特定のGPUハードウェアを表現したもの
    // OSのネイティブグラフィックスAPIからWebGPUへの変換レイヤー
    let adapter = Self::create_adapter(&instance, &surface).await.unwrap();

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

  async fn create_adapter<'w>(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'w>,
  ) -> Option<wgpu::Adapter> {
    instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        // マルチGPU搭載デバイス上で省電力のハードウェアを使用するか、または高パフォーマンスのハードウェアを使用するか
        // HighPerformanceオプション用のアダプタがない場合、LowPowerを優先する
        power_preference: wgpu::PowerPreference::default(),
        // wgpuに提供されたサーフェスに提示できるアダプターを見つけるように指示する
        compatible_surface: Some(&surface),
        // wgpuに全てのハードウェアで動作するアダプタを選択させる
        // これは通常、レンダリングバックエンドがGPUのようなハードウェアの代わりに "ソフトウェア "システムを使用することを意味する
        force_fallback_adapter: false,
        ..Default::default()
      })
      .await
  }
}
