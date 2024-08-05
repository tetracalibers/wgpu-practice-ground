struct VertexInput {
  @builtin(vertex_index) v_id: u32,
}

struct VertexOutput {
  @builtin(position) pos: vec4f,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
  let x = f32((in.v_id & 1) << 2);
  let y = f32((in.v_id & 2) << 1);
  
  var out: VertexOutput;
  out.pos = vec4f(x - 1.0, y - 1.0, 0, 1);
  
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
  return vec4f(1, 0, 1, 1);
}
