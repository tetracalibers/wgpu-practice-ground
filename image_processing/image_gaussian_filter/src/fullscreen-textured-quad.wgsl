@group(0) @binding(0) var screen_sampler: sampler;
@group(0) @binding(1) var screen_texture: texture_2d<f32>;
@group(0) @binding(2) var<uniform> resolution: vec2f;

struct VertexOutput {
  @builtin(position) position : vec4f,
  @location(0) frag_uv : vec2f,
}

fn object_fit_contain(pos: vec2f, aspect_ratio: f32) -> vec2f {
  var scale: vec2<f32>;

  if (aspect_ratio < 1.0) {
    // テクスチャがスクリーンに比べて横長
    scale = vec2<f32>(aspect_ratio, 1.0);
  } else {
    // テクスチャがスクリーンに比べて縦長または同じ比率
    scale = vec2<f32>(1.0, 1.0 / aspect_ratio);
  }
  
  return pos * scale;
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
  let tex_size = textureDimensions(screen_texture, 0);
  
  let tex_aspect = f32(tex_size.x) / f32(tex_size.y);
  let screen_aspect = resolution.x / resolution.y;
  
  let aspect_ratio = tex_aspect / screen_aspect;
  
  var pos = array<vec2f, 6>(
    object_fit_contain(vec2f( 1.0,  1.0), aspect_ratio),
    object_fit_contain(vec2f( 1.0, -1.0), aspect_ratio),
    object_fit_contain(vec2f(-1.0, -1.0), aspect_ratio),
    object_fit_contain(vec2f( 1.0,  1.0), aspect_ratio),
    object_fit_contain(vec2f(-1.0, -1.0), aspect_ratio),
    object_fit_contain(vec2f(-1.0,  1.0), aspect_ratio),
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
