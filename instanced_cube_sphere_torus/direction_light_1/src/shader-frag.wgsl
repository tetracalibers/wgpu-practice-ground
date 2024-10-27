struct Input {
  @location(0) v_position:vec4f,
  @location(1) v_normal:vec4f,
  @location(2) v_color: vec4f,
}

@fragment
fn fs_main(in: Input) -> @location(0) vec4f {
  let final_color = in.v_color;

  return vec4<f32>(final_color.rgb, 1.0);
}