use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

pub struct State {
  window: Arc<Window>,
}

impl State {
  pub async fn new(window: Window) -> Self {
    let window = Arc::new(window);

    let instance = Self::create_gpu_instance();
    // instanceによって、コードで描画を行うためのテクスチャ（surface texture）が提供される
    // - テクスチャとは、WebGPUが画像データを保存するために使用するオブジェクト
    // surface = 描画先（Webではcanvasに相当する、ここではwindow）
    let surface = instance.create_surface(Arc::clone(&window)).unwrap();
    // WebGPU内でデバイスの特定のGPUハードウェアを表現したもの
    // OSのネイティブグラフィックスAPIからWebGPUへの変換レイヤー
    let adapter = Self::create_adapter(&instance, &surface).await;
    // device
    // - 論理デバイス（自分だけの仮想的なGPU）
    // - 他のアプリケーションが使うテクスチャの内容などが読めないように、GPUを多重化したもの
    // - GPU とのほとんどのやり取りを行うための主なインターフェースとなる
    // queue
    // - GPUに仕事を投げ込むためのキュー
    let (device, queue) = Self::create_device(&adapter).await;

    let size = window.inner_size();
    let surface_caps = surface.get_capabilities(&adapter);
    // デバイスで使用するSurfaceの構成
    let config = Self::create_surface_config(size, &surface_caps);

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
  ) -> wgpu::Adapter {
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
      .unwrap()
  }

  async fn create_device(
    adapter: &wgpu::Adapter,
  ) -> (wgpu::Device, wgpu::Queue) {
    adapter
      .request_device(
        &wgpu::DeviceDescriptor {
          label: None,
          // 欲しい追加機能を指定できるが、ここでは余計な機能は使わない
          required_features: wgpu::Features::empty(),
          // 作成できるリソースの種類の上限を記述する
          // ここでは、ほとんどのデバイスをサポートできるように、デフォルトを使用
          required_limits: wgpu::Limits::default(),
          ..Default::default()
        },
        None,
      )
      .await
      .unwrap()
  }

  fn create_surface_config(
    size: PhysicalSize<u32>,
    surface_caps: &wgpu::SurfaceCapabilities,
  ) -> wgpu::SurfaceConfiguration {
    let surface_format = surface_caps
      .formats
      .iter()
      // なるべくリニアsRGB（ガンマ補正後）のサーフェスが作られるようにする
      // 注）ブラウザのWebGPU環境などでは、リニアsRGBのサーフェスを作ることができず、結果として最終的に出力される色が暗くなることがある
      .find(|format| format.is_srgb())
      .copied()
      .unwrap_or(surface_caps.formats[0]);

    wgpu::SurfaceConfiguration {
      // surface textureの使用方法を記述する
      // RENDER_ATTACHMENTは、テクスチャがスクリーンへの書き込みに使用されることを指定する
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      // surface textureがGPUにどのように保存されるかを定義する
      // 各テクスチャには一定の形式があり、この形式により、GPUのメモリ内でのデータの展開方法が規定される
      // サポートされているフォーマットは、SurfaceCapabilitiesから取得できる
      format: surface_format,
      // surface textureのピクセル単位の幅と高さ
      // 注）0だとアプリがクラッシュする
      width: size.width,
      height: size.height,
      // PresentMode::Fifoはディスプレイのフレームレートに表示レートを制限する
      present_mode: wgpu::PresentMode::Fifo,
      // ここでは、最初に利用可能なオプションを選択しておく
      alpha_mode: surface_caps.alpha_modes[0],
      // TextureViewsを作成するときに使用できるTextureFormatsのリスト
      view_formats: vec![],
      desired_maximum_frame_latency: 2,
    }
  }
}
