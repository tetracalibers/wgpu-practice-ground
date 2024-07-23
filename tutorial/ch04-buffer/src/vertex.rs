// Buffer
// - GPU上のデータの塊
// - 連続であることが保証されており、すべてのデータがメモリに順次格納される
// - つまり、GPUのメモリ上に一定のメモリレイアウトで連続的に配置されたデータ
// VertexBuffer
// - 通常、描画したい頂点データを格納するためにBufferを使う
// - シェーダー内で頂点を定義すると、描画する形状が変わるたびにシェーダーの再コンパイルが必要になるため…

// メモリ上のalignmentを保証する
#[repr(C)]
// Bufferを作成するためにVertexをCopyにする必要がある
// bytemuckを動作させるために2つのtraitを実装する必要がある
// - Podは頂点が "Plain Old Data "であることを示し、&[u8]として解釈できる
// - Zeroableは、std::mem::zeroed()が使えることを示している
// - どちらもコンパイル時にキャストの安全性を保証するために必要なもの
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
  position: [f32; 3],
  color: [f32; 3],
}

// 頂点を反時計回りに並べる：上、左下、右下
// このようにするのは、伝統的な理由もあるが、三角形の正面がwgpu::FrontFace::Ccwであることをrender_pipelineのプリミティブで指定したため
// これは、私たちの方を向いている三角形は、その頂点が反時計回りの順番であることを意味する
pub const VERTICES: &[Vertex] = &[
  Vertex {
    position: [0.0, 0.5, 0.0],
    color: [1.0, 0.0, 0.0],
  },
  Vertex {
    position: [-0.5, -0.5, 0.0],
    color: [0.0, 1.0, 0.0],
  },
  Vertex {
    position: [0.5, -0.5, 0.0],
    color: [0.0, 0.0, 1.0],
  },
];

impl Vertex {
  // VertexBufferLayoutは、バッファがメモリ上でどのように表現されるかを定義する
  // これがないと、render_pipelineはシェーダ内でバッファをどのようにマッピングすればよいかがわからない
  // バッファは単なるバイト列なので、バッファの中にどんなデータがどのように詰めこまれているかをWebGPU側に教える必要がある
  pub fn desc() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
      // 頂点の幅を定義する
      // シェーダが次の頂点を読みに行くとき、array_strideのバイト数を飛び越える
      array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
      // このバッファ内の配列の各要素が頂点毎のデータを表すのか、インスタンス毎のデータを表すのかをパイプラインに伝える
      // 新しいインスタンスの描画を開始する時だけ頂点を変更したい場合はwgpu::VertexStepMode::Instanceを指定する
      step_mode: wgpu::VertexStepMode::Vertex,
      // 頂点の個々のパーツを記述する
      // 一般的に、これは構造体のフィールドと1:1の対応になる
      attributes: &[
        wgpu::VertexAttribute {
          // 属性が始まるまでのオフセットをバイト単位で定義する
          // - 最初の属性では、オフセットは通常ゼロ
          // - それ以降の属性では、オフセットは前の属性のデータのsize_ofの和
          offset: 0,
          // シェーダにこのアトリビュートを格納する場所を指示する
          shader_location: 0,
          // シェーダに属性の形状を伝える
          // Float32x3はシェーダコードではvec3<f32>に対応する
          // アトリビュートに格納できる最大値はFloat32x4（Uint32x4やSint32x4も同様）
          format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
          offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
          shader_location: 1,
          format: wgpu::VertexFormat::Float32x3,
        },
      ],
    }
  }
}
