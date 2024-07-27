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
