use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

use crate::vertex::{Vertex, VERTICES};

// グリッドの縦方向と横方向にそれぞれいくつのセルが存在するか
// 整数値で十分だが、シェーダー側でのキャストが面倒なので最初から浮動小数点値で定義
const GRID_SIZE: f32 = 32.0;

pub struct State<'w> {
  window: Arc<Window>,
  size: PhysicalSize<u32>,
  surface: wgpu::Surface<'w>,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  render_pipeline: wgpu::RenderPipeline,
  vertex_buffer: wgpu::Buffer,
  num_vertices: u32,
  uniform_bind_group: wgpu::BindGroup,
  num_instances: u32,
}

impl<'w> State<'w> {
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

    // 頂点データを保持するためのバッファ
    let vertex_buffer =
      device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        // 作成するすべてのWebGPUオブジェクトには、オプションでラベルを指定することができる
        // 問題が発生した場合は、WebGPUが生成するエラーメッセージでこれらのラベルが使用される
        label: Some("Cell vertices"),
        // bytemuckを使ってVERTICESを&[u8]としてキャスト
        contents: bytemuck::cast_slice(VERTICES),
        // バッファの使用方法を指定する
        // ここでは、バッファを頂点データとして使用するとともに、データのコピー先としても使用する
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      });
    let num_vertices = VERTICES.len() as u32;

    //
    // シェーダーは、グリッドのサイズに応じて表示内容を変更するため、まずは選択したグリッドのサイズをシェーダーに伝える必要がある
    // シェーダーにサイズをハードコードすることもできるが…
    // この場合、グリッドのサイズを変更するたびにシェーダーおよびレンダリングパイプラインの再作成が必要となり、コストがかかる
    // ハードコードよりもスマートな方法として、グリッドのサイズをユニフォームとしてシェーダーに提供する方法がある
    //
    // 頂点シェーダーが呼び出されるたびに、頂点バッファから異なる値が渡されるが、
    // ユニフォームを使用すると、すべての呼び出しでユニフォームバッファから同じ値を渡すことができる
    //
    // ユニフォームは、
    // - ジオメトリで共通する値（位置など）
    // - アニメーションのフレーム全体で共通する値（現在の時刻など）
    // - アプリの存続期間全体で共通する値（ユーザーの設定など）
    // といった、共通する値を伝えるのに便利
    //

    // ユニフォームバッファを作成する
    let uniform_buffer =
      device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Grid uniforms"),
        contents: bytemuck::cast_slice(&[GRID_SIZE, GRID_SIZE]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      });

    //
    // シェーダーでユニフォームを宣言しても、それだけでは作成したバッファとは接続されない
    // 接続するためには、バインドグループを作成して、設定する必要がある
    //
    // バインドグループとは、シェーダーにも同時にアクセスできるようにするリソースのコレクション
    // ユニフォームバッファなど、いくつかの種類のバッファのほか、テクスチャやサンプラーなどのその他のリソースを含めることができる
    //

    // BindGroupとBindGroupLayoutが分かれているのは、同じBindGroupLayoutを共有していれば、その場でBindGroupを入れ替えられるから
    let uniform_bind_group_layout =
      device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Cell renderer bind group layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
          // シェーダーで入力した@binding()の値に対応する
          binding: 0,
          // 頂点シェーダからのみ見える
          visibility: wgpu::ShaderStages::VERTEX,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            // バッファ内のデータの位置が変わる可能性があることを意味する
            // これは、サイズが異なる複数のデータセットを1つのバッファに格納する場合に当てはまる
            // これをtrueに設定すると、後でオフセットを指定する必要がある
            has_dynamic_offset: false,
            // バッファの最小サイズを指定する
            min_binding_size: None,
          },
          count: None,
        }],
      });
    let uniform_bind_group =
      device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Cell renderer bind group"),
        layout: &uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
          binding: 0,
          // 指定したバインディングインデックスの変数に公開する実際のリソース
          // - バインドグループがポイントするリソースを作成後に変更することはできないが、これらのリソースの内容は変更できる
          // - たとえば、ユニフォームバッファを変更して新しいグリッドサイズを格納すると、変更後は、このバインドグループを使用する描画呼び出しで、その変更内容が反映される
          resource: uniform_buffer.as_entire_binding(),
        }],
      });

    let grid_size = GRID_SIZE as u32;
    let num_instances = grid_size * grid_size;

    // シェーダーをコンパイルする
    // ※）ShaderModuleDescriptorの代わりに、wgpu::include_wgsl!("shader.wgsl")を使用することもできる
    // ※）必要に応じて、頂点シェーダーとフラグメントシェーダーで別々のシェーダーモジュールを作成することもできる
    // - たとえば頂点シェーダーは同じで、複数の異なるフラグメント シェーダーを使用したい場合など
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("Cell shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("shader/main.wgsl").into()),
    });

    //
    // シェーダーモジュールは、単独でレンダリングに使用することはできない
    // RenderPipelineの一部として使用することで、初めてレンダリングを行うことができる
    //
    // レンダリングパイプラインでは、
    // - 使用するシェーダー
    // - 頂点バッファ内のデータの解釈方法
    // - レンダリングするジオメトリの種類（線分、点、三角形）
    // など、ジオメトリをどのように描画するかを制御する
    //
    // すべてのオプションを1か所（レンダリングパイプライン）にまとめることで、
    // 1. 各オプションがパイプラインが使用できるものであるかを作成時に簡単に判断できる
    // - あらゆるオプションの組み合わせが有効というわけではないので…
    // - まとめておくことで、後でさまざまなオプションについてチェックする必要がなくなるため、パイプラインでの描画が高速になる
    // - これは、描画呼び出しのたびに数多くの設定を検証する必要があるWebGLから大きく進化した点
    // 2. 描画時に1回呼び出すだけでレンダリングパスに対して大量の情報を渡すことができる
    // - これにより、全体としての呼び出し回数を減らすことができるため、レンダリングをさらに効率化できる
    //

    // 頂点バッファ以外にどのような種類の入力がパイプラインで必要かを示す
    let render_pipeline_layout =
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Cell pipeline layout"),
        // パイプラインが使用できるBindGroupLayoutのリスト
        bind_group_layouts: &[&uniform_bind_group_layout],
        push_constant_ranges: &[],
      });
    let render_pipeline =
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Cell pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
          module: &shader,
          // すべての頂点に対して呼び出される頂点シェーダーのコード内の関数名
          // @vertexでマークした関数はシェーダー内に複数記述できるが、その中のどれを呼び出すか
          entry_point: "vs_main",
          // 頂点シェーダに渡したい頂点の種類（バッファの読み取り方法）を伝える
          buffers: &[Vertex::layout()],
          compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        // fragmentは技術的にはオプションなので、Some()でラップする必要がある
        // fragmentは色データをSurfaceに保存したい場合に必要になる
        fragment: Some(wgpu::FragmentState {
          module: &shader,
          // @fragmentでマークした関数はシェーダー内に複数記述できるが、その中のどれを呼び出すか
          entry_point: "fs_main",
          // パイプラインで出力するカラーアタッチメントの詳細
          // このパイプラインで使用するレンダリングパスのColorAttachmentsで指定するテクスチャと一致している必要がある
          // 配列として複数指定できるが、今回はSurface用に1つだけ必要
          targets: &[Some(wgpu::ColorTargetState {
            // Surfaceと同じフォーマットを使用する
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
      window,
      size,
      surface,
      device,
      queue,
      config,
      render_pipeline,
      vertex_buffer,
      num_vertices,
      uniform_bind_group,
      num_instances,
    }
  }

  pub fn window(&self) -> &Window {
    &self.window
  }

  pub fn size(&self) -> PhysicalSize<u32> {
    self.size
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      // windowのサイズが変わるたびにsurfaceを再設定する必要がある
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
    }
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

    //
    // GPUに送信するコマンドはレンダリング（ここではウィンドウのクリア）に関連したものなので、encoderを使用して、レンダリングパスを開始する
    // WebGPUにおけるすべての描画操作は、レンダリングパスを通して実行される
    //

    // begin_render_pass()はencoderをミュータブルに借用する
    // このミュータブルな借用を解放するまで、encoder.finish()を呼び出すことはできない
    // このブロックは、コードがそのスコープから出たときに、その中の変数をドロップするようにRustに指示する
    // ※）{}の代わりに、drop(render_pass)を使っても同じ効果が得られる
    {
      // 各レンダリングパスは、beginRenderPass()の呼び出しで始まる
      // beginRenderPass()では、実行されたすべての描画コマンドの出力を受け取るテクスチャを定義する
      let mut render_pass =
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
          label: Some("Render Pass"),
          // アタッチメントは、複数のテクスチャを使用できる仕組み
          // レンダリングされるジオメトリの奥行きを保存したり、アンチエイリアスを提供したりもできる
          color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            // どのテクスチャに色を保存するか
            // フラグメントシェーダーの結果の書き込み先として使用するTextureViewを指定する
            view: &view,
            // 解決された出力を受け取るテクスチャ
            // マルチサンプリングが有効になっていない限り、viewと同じになる
            resolve_target: None,
            // 開始時および終了時にレンダリングパスでテクスチャに対して行う処理を指定する
            ops: wgpu::Operations {
              // 前のフレームから保存された色をどのように扱うか
              // Clearは、レンダリングパス開始時にテクスチャをクリアすることを示す
              load: wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.2,
                a: 1.0,
              }),
              // レンダリング結果をviewで指定したTexture（この場合はSurfaceTexture）に保存するかどうか
              // Storeは、レンダリングパス終了時にレンダリングパスで行われたすべての描画処理の結果をテクスチャに保存することを示す
              store: wgpu::StoreOp::Store,
            },
          })],
          depth_stencil_attachment: None,
          timestamp_writes: None,
          occlusion_query_set: None,
        });

      // 描画に使用するパイプラインを指定する
      // パイプラインには、使用するシェーダー、頂点データのレイアウト、その他関連する状態データが含まれる
      render_pass.set_pipeline(&self.render_pipeline);
      // バインドグループを使用するようWebGPUに伝える
      // - 1つ目の引数として渡される0は、シェーダーのコードの@group(0)に対応する
      // - ここでは、@group(0)に属する各@bindingで、このバインドグループのリソースを使用すると指定している
      render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
      // 実際に頂点バッファを設定する
      // - このバッファは現在のパイプラインのvertex.buffers定義の0番目の要素に相当するため、0を指定して呼び出す
      // - sliceによってバッファのどの部分を使うかを指定できる（ここでは、バッファ全体を指定）
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      // VERTICESで指定された頂点数の頂点をnum_instances回描くようにwgpuに指示する
      // - インスタンス化を使用すると、drawを1回呼び出すだけで、同じジオメトリの複数のコピーを描画するようにGPUに対して指示できる
      // - すべてのコピーに対して毎回drawを呼び出すよりもはるかに高速
      // - ジオメトリの各コピーをインスタンスと呼ぶ
      render_pass.draw(0..self.num_vertices, 0..self.num_instances);
    }

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

  fn create_gpu_instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor {
      // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
      // allとすることですべてのバックエンドから必要なものを選択してくれる
      backends: wgpu::Backends::all(),
      ..Default::default()
    })
  }

  async fn create_adapter(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'_>,
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
