use std::sync::Arc;

use wgpu::{
  Backends, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor,
  Limits, PowerPreference, PresentMode, Queue, RequestAdapterOptions, Surface,
  SurfaceConfiguration, SurfaceError, TextureUsages,
};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

pub struct State<'window> {
  surface: Surface<'window>,
  device: Device,
  queue: Queue,
  config: SurfaceConfiguration,
  size: PhysicalSize<u32>,
  window: Arc<Window>,
}

impl<'w> State<'w> {
  pub async fn new(window: Window) -> Self {
    let window = Arc::new(window);

    // wgpuのアプリケーションはinstanceという構造体と紐付けられる
    let instance = Instance::new(InstanceDescriptor {
      // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
      // allとすることですべてのバックエンドから必要なものを選択してくれる
      backends: Backends::all(),
      ..Default::default()
    });

    // Surface
    // - 描画先
    let surface = instance.create_surface(Arc::clone(&window)).unwrap();

    // Adapter:
    // - OSのネイティブグラフィックスAPIからWebGPUへの変換レイヤー
    // - 実際のグラフィックカードのハンドルであり、これを使用して、グラフィックスカードの名前や、アダプタが使用しているバックエンドなどの情報を取得できる
    let adapter = instance
      .request_adapter(&RequestAdapterOptions {
        // HighPerformanceオプション用のアダプタがない場合、LowPowerを優先する
        power_preference: PowerPreference::default(),
        // wgpuに提供されたサーフェスに提示できるアダプターを見つけるように指示する
        compatible_surface: Some(&surface),
        // wgpuに全てのハードウェアで動作するアダプタを選択させる
        // これは通常、レンダリングバックエンドがGPUのようなハードウェアの代わりに "ソフトウェア "システムを使用することを意味する
        force_fallback_adapter: false,
        ..Default::default()
      })
      .await
      .unwrap();

    // 論理デバイス
    // - 自分だけの仮想的なGPU
    // - 他のアプリケーションが使うテクスチャの内容などが読めないように、GPUを多重化したもの
    // キュー
    // - GPUに仕事を投げ込むためのキュー
    let (device, queue) = adapter
      .request_device(
        &DeviceDescriptor {
          label: None,
          // 欲しい追加機能を指定できるが、ここでは余計な機能は使わない
          required_features: Features::empty(),
          // 作成できるリソースの種類の上限を記述する
          // ここでは、ほとんどのデバイスをサポートできるように、デフォルトを使用
          required_limits: Limits::default(),
          ..Default::default()
        },
        None,
      )
      .await
      .unwrap();

    // for config
    let size = window.inner_size();

    // for config
    let surface_caps = surface.get_capabilities(&adapter);

    // for config
    let surface_format = surface_caps
      .formats
      .iter()
      // なるべくリニアsRGB（ガンマ補正後）のサーフェスが作られるようにする
      // 注）ブラウザのWebGPU環境などでは、リニアsRGBのサーフェスを作ることができず、結果として最終的に出力される色が暗くなることがある
      .find(|format| format.is_srgb())
      .copied()
      .unwrap_or(surface_caps.formats[0]);

    // SurfaceがどのようにSurfaceTextureを作成するかを定義する
    let config = SurfaceConfiguration {
      // SurfaceTexturesの使用方法を記述する
      // RENDER_ATTACHMENTは、テクスチャがスクリーンへの書き込みに使用されることを指定する
      usage: TextureUsages::RENDER_ATTACHMENT,
      // SurfaceTexturesがGPUにどのように保存されるかを定義する
      // サポートされているフォーマットは、SurfaceCapabilitiesから取得できる
      format: surface_format,
      // SurfaceTextureのピクセル単位の幅と高さ
      // 注）0だとアプリがクラッシュする
      width: size.width,
      height: size.height,
      // PresentMode::Fifoはディスプレイのフレームレートに表示レートを制限する
      present_mode: PresentMode::Fifo,
      // ここでは、最初に利用可能なオプションを選択しておく
      alpha_mode: surface_caps.alpha_modes[0],
      // TextureViewsを作成するときに使用できるTextureFormatsのリスト
      view_formats: vec![],
      desired_maximum_frame_latency: 2,
    };

    Self {
      surface,
      device,
      queue,
      config,
      size,
      window,
    }
  }

  pub fn window(&self) -> &Window {
    &self.window
  }

  pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      // ウィンドウのサイズが変わるたびにサーフェスを再設定する必要がある
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
    }
  }

  fn input(&mut self, event: &WindowEvent) -> bool {
    todo!()
  }

  fn update(&mut self) {
    todo!()
  }

  fn render(&mut self) -> Result<(), SurfaceError> {
    todo!()
  }
}
