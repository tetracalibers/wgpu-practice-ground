use wgpu::util::DeviceExt;

use crate::vertex::{Vertex, VERTICES};

// グリッドの縦方向と横方向にそれぞれいくつのセルが存在するか
// 整数値で十分だが、シェーダー側でのキャストが面倒なので最初から浮動小数点値で定義
const GRID_SIZE: f32 = 32.0;

pub struct Renderer {
  render_pipeline: wgpu::RenderPipeline,
  vertex_buffer: wgpu::Buffer,
  num_vertices: u32,
  bind_groups: Vec<wgpu::BindGroup>,
  num_instances: u32,
  step: usize,
}

impl Renderer {
  pub async fn new<'w>(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
  ) -> Self {
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

    let grid_size = GRID_SIZE as u32;
    let num_instances = grid_size * grid_size;

    //
    // GPUに保存されたなんらかの状態に基づいて、グリッド上のどのセルをレンダリングするかを制御する必要がある
    // ライフゲームのシミュレーションは、GPUのコンピューティングシェーダーで行うため、ストレージバッファを使用する
    //
    // ユニフォームバッファは、
    // - サイズに制限がある
    // - 動的サイズの配列をサポートしていない（シェーダーで配列サイズを指定する必要がある）
    // - コンピューティング シェーダーでは書き込むことができない
    //
    // ストレージバッファは、一般的なメモリのように利用できる汎用バッファ
    // - コンピューティング シェーダーで読み書きできる
    // - 頂点シェーダーで読み取ることができる
    // - 非常に大きなサイズにすることができる
    // - シェーダーで特定のサイズを宣言する必要がない
    //
    // 補足）パフォーマンスについて
    // ユニフォームバッファは、多くの場合、ストレージバッファよりも高速に更新や読み取りができるよう、GPUで特別に扱われる
    // そのため、頻繁に更新が発生する可能性のある小さなサイズのデータであれば（3D アプリケーションのモデル、ビュー、射影行列など）、一般的にユニフォームバッファの方が高いパフォーマンスを実現できる
    //

    // セルの状態を2パターン用意
    let cell_state_1: Vec<u32> = (0..grid_size * grid_size)
      .map(|i| if i % 3 == 0 { 1 } else { 0 })
      .collect();
    let cell_state_2: Vec<u32> =
      (0..grid_size * grid_size).map(|i| i as u32 % 2).collect();

    // ストレージバッファを使用してセルの状態を保存する
    let cell_state_storage_1 =
      device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Cell State 1"),
        contents: bytemuck::cast_slice(cell_state_1.as_slice()),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      });
    let cell_state_storage_2 =
      device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Cell State 2"),
        contents: bytemuck::cast_slice(cell_state_2.as_slice()),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      });

    //
    // シェーダーでユニフォームやストレージバッファを宣言しても、それだけでは作成したバッファとは接続されない
    // 接続するためには、バインドグループを作成して、設定する必要がある
    //
    // バインドグループとは、シェーダーにも同時にアクセスできるようにするリソースのコレクション
    // ユニフォームバッファなど、いくつかの種類のバッファのほか、テクスチャやサンプラーなどのその他のリソースを含めることができる
    //

    // BindGroupとBindGroupLayoutが分かれているのは、同じBindGroupLayoutを共有していれば、その場でBindGroupを入れ替えられるから
    let bind_group_layout =
      device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Cell renderer bind group layout"),
        // シェーダーのコード上で同じ@groupに属しているものは、同じバインドグループに追加する
        entries: &[
          // ユニフォームバッファ
          wgpu::BindGroupLayoutEntry {
            // シェーダーで入力した@binding()の値に対応する
            binding: 0,
            // 頂点シェーダとフラグメントシェーダから見えるようにする
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
          },
          // ストレージバッファ
          wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
              ty: wgpu::BufferBindingType::Storage { read_only: true },
              has_dynamic_offset: false,
              min_binding_size: None,
            },
            count: None,
          },
        ],
      });
    let bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Cell renderer bind group 1"),
      layout: &bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          // 指定したバインディングインデックスの変数に公開する実際のリソース
          // - バインドグループがポイントするリソースを作成後に変更することはできないが、これらのリソースの内容は変更できる
          // - たとえば、ユニフォームバッファを変更して新しいグリッドサイズを格納すると、変更後は、このバインドグループを使用する描画呼び出しで、その変更内容が反映される
          resource: uniform_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: cell_state_storage_1.as_entire_binding(),
        },
      ],
    });
    let bind_group_2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("Cell renderer bind group 2"),
      layout: &bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: uniform_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: cell_state_storage_2.as_entire_binding(),
        },
      ],
    });

    // シェーダーをコンパイルする
    // ※）ShaderModuleDescriptorの代わりに、wgpu::include_wgsl!("shader.wgsl")を使用することもできる
    // ※）必要に応じて、頂点シェーダーとフラグメントシェーダーで別々のシェーダーモジュールを作成することもできる
    // - たとえば頂点シェーダーは同じで、複数の異なるフラグメント シェーダーを使用したい場合など
    let render_shader =
      device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Cell shader"),
        source: wgpu::ShaderSource::Wgsl(
          include_str!("shader/render.wgsl").into(),
        ),
      });
    let simulation_shader =
      device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Game of Life simulation shader"),
        source: wgpu::ShaderSource::Wgsl(
          include_str!("shader/simulation.wgsl").into(),
        ),
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
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
      });
    let render_pipeline =
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Cell pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
          module: &render_shader,
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
          module: &render_shader,
          // @fragmentでマークした関数はシェーダー内に複数記述できるが、その中のどれを呼び出すか
          entry_point: "fs_main",
          // パイプラインで出力するカラーアタッチメントの詳細
          // このパイプラインで使用するレンダリングパスのColorAttachmentsで指定するテクスチャと一致している必要がある
          // 配列として複数指定できるが、今回はSurface用に1つだけ必要
          targets: &[Some(wgpu::ColorTargetState {
            format: target_format,
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
      render_pipeline,
      vertex_buffer,
      num_vertices,
      bind_groups: vec![bind_group_1, bind_group_2],
      num_instances,
      step: 0,
    }
  }

  pub fn update(&mut self) {
    self.step += 1;
  }

  pub fn draw(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
  ) {
    //
    // GPUに送信するコマンドはレンダリングに関連したものなので、encoderを使用して、レンダリングパスを開始する
    // WebGPUにおけるすべての描画操作は、レンダリングパスを通して実行される
    //

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
          view,
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
    render_pass.set_bind_group(
      0,
      // 2つの状態のコピーを交互に使用する
      // 1. まず一方の状態のコピーから読み込み、他方のコピーに書き出す
      // 2. 次のステップでは、逆に書き込んだ方のコピーから状態を読み取る
      // 各ステップで最新バージョンの状態が2つのコピーの間で行ったり来たりする
      // このような方式は一般的にPing-pongパターンと呼ばれる
      &self.bind_groups.get(self.step % 2).unwrap(),
      &[],
    );
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
}
