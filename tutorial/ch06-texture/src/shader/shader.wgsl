// --- Vertex shader

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) tex_coords: vec2<f32>,
}

// 頂点シェーダの出力を格納するstructを宣言
struct VertexOutput {
  // builtin(position)
  // - WGPUにこれが頂点のクリップ座標として使いたい値であることを伝える
  // - like: GLSLのgl_Position変数
  @builtin(position) clip_position: vec4<f32>,
  @location(0) tex_coords: vec2<f32>,
};

// @vertex
// - この関数を頂点シェーダーの有効なエントリー・ポイントとしてマークする
@vertex
fn vs_main(
  model: VertexInput,
) -> VertexOutput {
  // varで定義された変数は変更できるが、型を指定する必要がある
  var out: VertexOutput;
  
  // wgpuのワールド座標はY軸が上を向いているのに対し、テクスチャ座標はY軸が下を向いている
  // 画像を上下反転させないため、各テクスチャ座標のy座標を1-yに置き換える
  out.tex_coords = vec2<f32>(model.tex_coords.x, 1.0 - model.tex_coords.y);
  out.clip_position = vec4<f32>(model.position, 1.0);
  
  return out;
}

// --- Fragment shader

// uniforms
// - group()はset_bind_group()の第1パラメータに対応する
// - binding()はBindGroupLayoutとBindGroupを作成したときに指定したバインドに対応する
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

// location(0)
// - この関数が返すvec4値を最初のカラーターゲットに格納するようWGPUに指示する
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  // Samplerを使用してTexutureから色を取得する
  return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
