@group(0) @binding(0) var screen_sampler: sampler;
@group(0) @binding(1) var screen_texture: texture_2d<f32>;

struct VertexOutput {
  @builtin(position) position : vec4f,
  @location(0) frag_uv : vec2f,
}

const res = vec2f(800.0, 600.0);

fn alignAspect(pos: vec2f) -> vec2f {
  return pos / res * min(res.x, res.y);
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
  var pos = array<vec2f, 6>(
    alignAspect(vec2f( 1.0,  1.0)),
    alignAspect(vec2f( 1.0, -1.0)),
    alignAspect(vec2f(-1.0, -1.0)),
    alignAspect(vec2f( 1.0,  1.0)),
    alignAspect(vec2f(-1.0, -1.0)),
    alignAspect(vec2f(-1.0,  1.0)),
  );
  
  var uv = array<vec2f, 6>(
    vec2f(1.0, 0.0),
    vec2f(1.0, 1.0),
    vec2f(0.0, 1.0),
    vec2f(1.0, 0.0),
    vec2f(0.0, 1.0),
    vec2f(0.0, 0.0),
  );
  
  var output: VertexOutput;
  output.position = vec4(pos[i], 0.0, 1.0);
  output.frag_uv = uv[i];
  return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
  return textureSample(screen_texture, screen_sampler, in.frag_uv);
}
