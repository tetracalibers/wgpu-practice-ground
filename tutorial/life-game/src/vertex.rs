// メモリ上のalignmentを保証する
#[repr(C)]
// 1. Bufferを作成するためにVertexをCopyにする必要がある
// 2. bytemuckを動作させるために2つのtraitを実装する必要がある
// - Podは頂点が "Plain Old Data "であることを示し、&[u8]として解釈できる
// - Zeroableは、std::mem::zeroed()が使えることを示している
// - どちらもコンパイル時にキャストの安全性を保証するために必要なもの
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
  position: [f32; 2],
}

// 正方形を2つに分割して、対角線を辺として共有する2つの三角形を作成する
// 頂点のうち2つはこれらの三角形で重複することになる
/*
  A---B
  |  /|
  | / |
  |/  |
  C---D
*/
pub const VERTICES: &[Vertex] = &[
  // Triangle 1
  Vertex {
    position: [-0.8, -0.8], // C
  },
  Vertex {
    position: [0.8, -0.8], // D
  },
  Vertex {
    position: [0.8, 0.8], // B
  },
  // Triangle 2
  Vertex {
    position: [-0.8, -0.8], // C
  },
  Vertex {
    position: [0.8, 0.8], // B
  },
  Vertex {
    position: [-0.8, 0.8], // A
  },
];

//
// 多くの場合、GPUにはレンダリングに高度に最適化された専用のメモリが用意されている
// 描画時にGPUが使用するデータはそのメモリに配置する必要がある
//
// GPU側のメモリはBufferオブジェクトを介して管理される
// バッファは、GPUが容易にアクセスできるメモリのブロック
// - GPUのメモリ上に一定のメモリレイアウトで連続的に配置されたデータ
// - 特定の目的に応じたフラグが設定されている
// - 連続であることが保証されており、すべてのデータがメモリに順次格納される
//

impl Vertex {
  // VertexBufferLayoutは、バッファがメモリ上でどのように表現されるかを定義する
  // これがないと、render_pipelineはシェーダ内でバッファをどのようにマッピングすればよいかがわからない
  // バッファは単なるバイト列なので、バッファの中にどんなデータがどのように詰めこまれているかをWebGPU側に教える必要がある
  pub fn layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
      // GPUが次の頂点のデータを取得する際にバッファ内でスキップする必要があるバイト数
      // シェーダが次の頂点を読みに行くとき、array_strideのバイト数を飛び越える
      array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
      // このバッファ内の配列の各要素が頂点毎のデータを表すのか、インスタンス毎のデータを表すのかをパイプラインに伝える
      // 新しいインスタンスの描画を開始する時だけ頂点を変更したい場合はwgpu::VertexStepMode::Instanceを指定する
      step_mode: wgpu::VertexStepMode::Vertex,
      // 各頂点にエンコードされる個々の情報
      // 位置だけでなく、頂点の色やジオメトリのサーフェスが向いている方向など、頂点に複数の属性が含まれることもある
      attributes: &[wgpu::VertexAttribute {
        // 頂点データの型
        // Float32x2はシェーダコードではvec2<f32>に対応する
        // 属性に格納できる最大値はFloat32x4（Uint32x4やSint32x4も同様）
        format: wgpu::VertexFormat::Float32x2,
        // 属性の開始バイト位置
        // - 最初の属性では、オフセットは通常ゼロ
        // - それ以降の属性では、オフセットは前の属性のデータのsize_ofの和
        offset: 0,
        // シェーダにこの属性を格納する場所を指示する
        // これにより、この属性が頂点シェーダーの特定の入力にリンクされる
        shader_location: 0,
      }],
    }
  }
}
