use std::sync::Arc;

use wgpu::{
  Backends, CommandEncoderDescriptor, Device, DeviceDescriptor, Features,
  Instance, InstanceDescriptor, Limits, PowerPreference, PresentMode, Queue,
  RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceError,
  TextureUsages, TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

pub struct State<'window> {
  surface: Surface<'window>,
  device: Device,
  queue: Queue,
  config: SurfaceConfiguration,
  size: PhysicalSize<u32>,
  window: Arc<Window>,
  // Pipeline
  // - パイプラインは、あるデータセットに対してGPUが実行するすべてのアクションを記述する
  // - OpenGLでのシェーダープログラムのより堅牢なバージョンと考えることができる
  // ここでは、特にRenderPipelineを作成する
  render_pipeline: wgpu::RenderPipeline,
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

    // シェーダーをロードする
    // ShaderModuleDescriptorの代わりに、wgpu::include_wgsl!("shader.wgsl")を使用することもできる
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("Shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let render_pipeline_layout =
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
      });

    let render_pipeline =
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
          module: &shader,
          // シェーダー内のどの関数をエントリポイントにするかを指定する
          // @vertexでマークした関数
          entry_point: "vs_main",
          // 頂点シェーダに渡したい頂点の種類を伝える
          // 今回は頂点シェーダ自体で頂点を指定するので、ここは空にしておく
          buffers: &[],
          compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        // fragmentは技術的にはオプションなので、Some()でラップする必要がある
        // fragmentは色データをサーフェスに保存したい場合に必要になる
        fragment: Some(wgpu::FragmentState {
          module: &shader,
          // シェーダー内のどの関数をエントリポイントにするかを指定する
          // @fragmentでマークした関数
          entry_point: "fs_main",
          // 設定すべきカラー出力を指示する
          // 配列として複数指定できるが、今回はサーフェス用に1つだけ必要
          targets: &[Some(wgpu::ColorTargetState {
            // サーフェスへのコピーが簡単にできるように、サーフェスのフォーマットを使う
            format: config.format,
            // 古いピクセルデータを新しいデータに置き換えるだけでいいと指定
            blend: Some(wgpu::BlendState::REPLACE),
            // 赤、青、緑、アルファのすべての色に書き込むようにwgpuに指示
            write_mask: wgpu::ColorWrites::ALL,
          })],
          compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        // 頂点を三角形に変換する際の解釈方法を記述する
        primitive: wgpu::PrimitiveState {
          // PrimitiveTopology::TriangleListを使うと、3つの頂点が1つの三角形に対応することになる
          topology: wgpu::PrimitiveTopology::TriangleList,
          strip_index_format: None,
          // front_faceとcull_modeフィールド
          // - 与えられた三角形が正面を向いているかどうかを決定する方法をwgpuに伝える
          // FrontFace::Ccwは、頂点が反時計回りに配置されている場合、三角形が正面を向いていることを意味する
          front_face: wgpu::FrontFace::Ccw,
          // 正面を向いていないとみなされた三角形は、CullMode::Backで指定されたようにカリングされる（レンダリングに含まれない）
          cull_mode: Some(wgpu::Face::Back),
          polygon_mode: wgpu::PolygonMode::Fill,
          // Requires Features::DEPTH_CLIP_CONTROL
          unclipped_depth: false,
          // Requires Features::CONSERVATIVE_RASTERIZATION
          conservative: false,
        },
        // 今回は深度／ステンシル・バッファは使用しないので、depth_stencilはNoneのままにしておく
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
          // パイプラインが使用するサンプルの数を決定する
          count: 1,
          // どのサンプルをアクティブにするかを指定する
          // 今回はすべてのサンプルを使用する
          mask: !0, // !はビット単位の否定（NOT演算子）
          // アンチエイリアシングに関係する
          // 今回はアンチエイリアシングを取り上げないので、これはfalseのままにしておく
          alpha_to_coverage_enabled: false,
        },
        // レンダーアタッチメントがいくつの配列レイヤーを持つことができるかを示す
        // 今回は配列テクスチャにレンダリングしないので、Noneに設定する
        multiview: None,
        // wgpuにシェーダーのコンパイルデータをキャッシュさせるかどうかを指定する
        // Androidのビルドターゲットにのみ役に立つ
        cache: None,
      });

    Self {
      surface,
      device,
      queue,
      config,
      size,
      window,
      render_pipeline,
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
      // ウィンドウのサイズが変わるたびにサーフェスを再設定する必要がある
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
    }
  }

  // イベントが完全に処理されたかどうかを示すboolを返す
  // このメソッドがtrueを返した場合、メイン・ループはそれ以上イベントを処理しない
  pub fn input(&mut self, _event: &WindowEvent) -> bool {
    // 今は取り込みたいイベントがないので、falseを返すことにする
    false
  }

  pub fn update(&mut self) {
    // We don't have anything to update yet
  }

  pub fn render(&mut self) -> Result<(), SurfaceError> {
    // get_current_texture関数は、レンダリング先のサーフェスが新しいSurfaceTextureを提供するのを待機する
    let output = self.surface.get_current_texture()?;

    // デフォルト設定のTextureViewを作成する
    // TextureViewを介して、レンダリングコードがテクスチャとどのように相互作用するかを制御する
    let view = output.texture.create_view(&TextureViewDescriptor::default());

    // GPUに送信する実際のコマンドを作成するために、CommandEncoderを作成する必要がある
    // 最近のグラフィックフレームワークのほとんどは、GPUに送信する前にコマンドがコマンドバッファに格納されることを期待している
    // エンコーダーはコマンドバッファを構築し、それをGPUに送ることができる
    let mut encoder =
      self.device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Render Encoder"),
      });

    // begin_render_pass()はencoderをミュータブルに借用する
    // このミュータブルな借用を解放するまで、encoder.finish()を呼び出すことはできない
    // このブロックは、コードがそのスコープから出たときに、その中の変数をドロップするようにRustに指示する
    // ※）{}の代わりに、drop(render_pass)を使っても同じ効果が得られる
    {
      // CommandEncoderのbegin_render_passメソッドによって、描画パスRenderPassの構築を開始できる
      // RenderPassには実際の描画のためのすべてのメソッドがある
      // RenderPassが持つ各種のメソッドを呼ぶことで描画のためのコマンドを組み上げる
      let mut render_pass =
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
          label: Some("Render Pass"),
          // フラグメントシェーダーの結果の書き込み先として使用するテクスチャビューを指定する
          color_attachments: &[
            // これはフラグメントシェーダーの@location(0)がターゲットにしているものである
            Some(wgpu::RenderPassColorAttachment {
              // どのテクスチャに色を保存するか
              view: &view,
              // 解決された出力を受け取るテクスチャ
              // マルチサンプリングが有効になっていない限り、viewと同じになる
              resolve_target: None,
              // スクリーン上の色(viewによって指定される)をどう扱うか
              ops: wgpu::Operations {
                // 前のフレームから保存された色をどのように扱うか
                load: wgpu::LoadOp::Clear(wgpu::Color {
                  r: 0.1,
                  g: 0.2,
                  b: 0.3,
                  a: 1.0,
                }),
                // レンダリング結果をTextureViewの後ろにあるTexture（この場合はSurfaceTexture）に保存するかどうか
                // レンダリング結果を保存したいので、StoreOp::Storeを使用する
                store: wgpu::StoreOp::Store,
              },
            }),
          ],
          depth_stencil_attachment: None,
          occlusion_query_set: None,
          timestamp_writes: None,
        });

      render_pass.set_pipeline(&self.render_pipeline);
      // 3つの頂点と1つのインスタンスで何かを描くようにwgpuに指示する
      // これが@builtin(vertex_index)の由来である
      render_pass.draw(0..3, 0..1);
    }

    // wgpuにコマンドバッファを終了し、GPUのレンダーキューに送信するように指示する
    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
  }
}
