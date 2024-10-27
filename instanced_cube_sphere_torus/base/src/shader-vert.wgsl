@group(0) @binding(0) var<uniform> view_project_mat: mat4x4f;
@group(0) @binding(1) var<storage> model_mat: array<mat4x4f>;
@group(0) @binding(2) var<storage> normal_mat: array<mat4x4f>;
@group(0) @binding(3) var<storage> color_vec: array<vec4f>;

struct Input {
  @builtin(instance_index) idx: u32, 
  @location(0) position: vec3f, 
  @location(1) normal: vec3f
}

struct Output {
  @builtin(position) position: vec4f,
  @location(0) v_position: vec4f,
  @location(1) v_normal: vec4f,
  @location(2) v_color: vec4f,
};

@vertex
fn vs_main(in: Input) -> Output {
  var output: Output;

  let model_mat = model_mat[in.idx];
  let normal_mat = normal_mat[in.idx];
  let m_position:vec4<f32> = model_mat * vec4(in.position, 1.0);

  output.position = view_project_mat * m_position;
  output.v_position = m_position;
  output.v_normal = normal_mat * vec4(in.normal, 1.0);
  output.v_color = color_vec[in.idx];
  
  return output;
}