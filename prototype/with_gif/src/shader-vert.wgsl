struct Uniforms {
  view_project_mat: mat4x4f,
  model_mat: mat4x4f,
  normal_mat: mat4x4f,
};

@binding(0) @group(0) var<uniform> unif: Uniforms;

struct Input {
  @location(0) pos: vec3f,
  @location(1) normal: vec3f,
}

struct Output {
  @builtin(position) position: vec4f,
  @location(0) v_position: vec4f,
  @location(1) v_normal: vec4f,
}

@vertex
fn vs_main(in: Input) -> Output {
  var output: Output;
    
  let m_position = unif.model_mat * vec4(in.pos, 1.0);
  output.v_position = m_position;
  output.v_normal = unif.normal_mat * vec4(in.normal, 1.0);
  output.position = unif.view_project_mat * m_position;
  return output;
}