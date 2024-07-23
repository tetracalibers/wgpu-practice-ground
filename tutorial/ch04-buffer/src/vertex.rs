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
