struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) color: vec3<f32>,
}

// 頂点シェーダの出力を格納するstructを宣言
struct VertexOutput {
  // builtin(position)
  // - WGPUにこれが頂点のクリップ座標として使いたい値であることを伝える
  // - like: GLSLのgl_Position変数
  @builtin(position) clip_position: vec4<f32>,
  @location(0) color: vec3<f32>,
};

// @vertex
// - この関数を頂点シェーダーの有効なエントリー・ポイントとしてマークする
@vertex
fn vs_main(
  model: VertexInput,
) -> VertexOutput {
  // varで定義された変数は変更できるが、型を指定する必要がある
  var out: VertexOutput;
  
  out.color = model.color;
  out.clip_position = vec4<f32>(model.position, 1.0);
  
  return out;
}

// location(0)
// - この関数が返すvec4値を最初のカラーターゲットに格納するようWGPUに指示する
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(in.color, 1.0);
}
