// 頂点シェーダの出力を格納するstructを宣言
struct VertexOutput {
  // builtin(position)
  // - WGPUにこれが頂点のクリップ座標として使いたい値であることを伝える
  // - like: GLSLのgl_Position変数
  @builtin(position) clip_position: vec4<f32>,
};

// @vertex
// - この関数を頂点シェーダーの有効なエントリー・ポイントとしてマークする
@vertex
fn vs_main(
  // builtin(vertex_index)から値を取得するin_vertex_indexというu32を期待している
  @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
  // varで定義された変数は変更できるが、型を指定する必要がある
  var out: VertexOutput;
  
  // letで作成された変数はその型を推測することができるが、その値を変更することはできない
  let x = f32(1 - i32(in_vertex_index)) * 0.5;
  let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
  
  out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
  
  return out;
}

// location(0)
// - この関数が返すvec4値を最初のカラーターゲットに格納するようWGPUに指示する
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}
