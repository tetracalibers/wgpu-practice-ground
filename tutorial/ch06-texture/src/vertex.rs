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
  tex_coords: [f32; 2],
}

// 五角形の各頂点
pub const VERTICES: &[Vertex] = &[
  Vertex {
    position: [-0.0868241, 0.49240386, 0.0],
    tex_coords: [0.4131759, 0.99240386],
  }, // A
  Vertex {
    position: [-0.49513406, 0.06958647, 0.0],
    tex_coords: [0.0048659444, 0.56958647],
  }, // B
  Vertex {
    position: [-0.21918549, -0.44939706, 0.0],
    tex_coords: [0.28081453, 0.05060294],
  }, // C
  Vertex {
    position: [0.35966998, -0.3473291, 0.0],
    tex_coords: [0.85967, 0.1526709],
  }, // D
  Vertex {
    position: [0.44147372, 0.2347359, 0.0],
    tex_coords: [0.9414737, 0.7347359],
  }, // E
];

// VERTICESの要素へのインデックス
// 頂点をどう結べば五角形になるかを示す順番
pub const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

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
          format: wgpu::VertexFormat::Float32x2,
        },
      ],
    }
  }
}
