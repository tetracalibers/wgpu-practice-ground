struct DirectionLight {
  direction: vec3f,
  color: vec3f,
}

@group(1) @binding(0) var<uniform> light: DirectionLight;
@group(1) @binding(1) var<uniform> ambient: f32;

struct Input {
  @location(0) v_position:vec4f,
  @location(1) v_normal:vec4f,
  @location(2) v_color: vec4f,
}

@fragment
fn fs_main(in: Input) -> @location(0) vec4f {
  let N = normalize(in.v_normal.xyz);
  let L = normalize(-light.direction.xyz);

  //
  // 拡散反射光：Lambert拡散反射モデル
  //
  let diffuse = light.color * max(dot(N, L), 0.0);
  
  //
  // 最終的な光
  //
  let lig = diffuse + ambient;

  let final_color = in.v_color.rgb * lig;

  return vec4<f32>(final_color.rgb, 1.0);
}