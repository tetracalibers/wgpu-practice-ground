use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

use crate::renderer::Renderer;

pub struct State<'w> {
  window: Arc<Window>,
  size: PhysicalSize<u32>,
  surface: wgpu::Surface<'w>,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  renderer: Renderer,
}

impl<'w> State<'w> {
  pub async fn new(window: Window) -> Self {
    let window = Arc::new(window);

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
      // allとすることですべてのバックエンドから必要なものを選択してくれる
      backends: wgpu::Backends::all(),
      ..Default::default()
    });

    // instanceによって、コードで描画を行うためのテクスチャ（surface texture）が提供される
    // - テクスチャとは、WebGPUが画像データを保存するために使用するオブジェクト
    // surface = 描画先（Webではcanvasに相当する、ここではwindow）
    let surface = instance.create_surface(Arc::clone(&window)).unwrap();

    // WebGPU内でデバイスの特定のGPUハードウェアを表現したもの
    // OSのネイティブグラフィックスAPIからWebGPUへの変換レイヤー
    let adapter = instance
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
      .unwrap();

    // device
    // - 論理デバイス（自分だけの仮想的なGPU）
    // - 他のアプリケーションが使うテクスチャの内容などが読めないように、GPUを多重化したもの
    // - GPU とのほとんどのやり取りを行うための主なインターフェースとなる
    // queue
    // - GPUに仕事を投げ込むためのキュー
    let (device, queue) = adapter
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
      .unwrap();

    let size = window.inner_size();

    // for SurfaceConfiguration
    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
      .formats
      .iter()
      // なるべくリニアsRGB（ガンマ補正後）のサーフェスが作られるようにする
      // 注）ブラウザのWebGPU環境などでは、リニアsRGBのサーフェスを作ることができず、結果として最終的に出力される色が暗くなることがある
      .find(|format| format.is_srgb())
      .copied()
      .unwrap_or(surface_caps.formats[0]);

    // デバイスで使用するSurfaceの構成
    let config = wgpu::SurfaceConfiguration {
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
    };
    surface.configure(&device, &config);

    let renderer = Renderer::new(&device, config.format).await;

    Self {
      window,
      size,
      surface,
      device,
      queue,
      config,
      renderer,
    }
  }

  pub fn window(&self) -> &Window {
    &self.window
  }

  pub fn size(&self) -> PhysicalSize<u32> {
    self.size
  }

  pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      // windowのサイズが変わるたびにsurfaceを再設定する必要がある
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
    }
  }

  pub fn update(&mut self) {
    self.renderer.update();
  }

  pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
    //
    // 描画先のコンテンツを更新するたびに、get_current_texture()を呼び出してレンダリングパスのためのテクスチャを新たに取得し、新しいコマンドバッファを記録して送信する必要がある
    //

    // レンダリング先のsurfaceが新しいSurfaceTextureを提供するのを待機し、取得する
    // windowのwidth/heightに一致するピクセル幅と高さ、そしてconfigで指定したformatを持つテクスチャを得る
    let surface_texture = self.surface.get_current_texture()?;
    // レンダリングパスでは、TextureではなくTextureViewを渡して、テクスチャのどの部分にレンダリングするか指定する必要がある
    // デフォルト設定のTextureViewは、テクスチャ全体を意味する
    let view = surface_texture
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());

    // GPUに処理内容を指示するコマンドを送信する必要がある
    // これを行うには、GPUコマンドを記録するインターフェースとなるCommandEncoderをデバイスで作成する
    // - 最近のグラフィックフレームワークのほとんどは、GPUに送信する前にコマンドがコマンドバッファに格納されることを期待している
    // - CommandEncoderはコマンドバッファを構築し、それをGPUに送ることができる
    let mut encoder =
      self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
      });

    self.renderer.draw(&mut encoder, &view);

    //
    // ここまでで、GPUが後で実行するコマンドが記録される
    // 以下で、GPUで実際に処理を実行する
    //

    // 1. encoder.finish()でコマンドバッファを作成する
    // - コマンドバッファは、記録されたコマンドをラップして詳細を隠すためのハンドル
    //
    // 2. queueを使用して、GPUにコマンドバッファを送信する
    // - queueにより、すべてのGPUコマンドが順番通り、かつ適切に同期をとりながら実行される
    // - ここでは1つのコマンドバッファのみを渡す
    //
    // 注）
    // - コマンドバッファを送信（submit）すると、そのコマンドバッファは再利用できなくなる
    // - さらにコマンドを送信する場合は、別のコマンドバッファを作成する必要がある
    self.queue.submit(std::iter::once(encoder.finish()));

    // テクスチャがsurfaceに表示されるようにスケジュール
    surface_texture.present();

    Ok(())
  }
}
