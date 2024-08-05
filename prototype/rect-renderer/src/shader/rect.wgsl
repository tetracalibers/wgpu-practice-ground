struct Rectangle {
  color: vec4f,
  origin: vec2f,
  sigma: f32,
  corners: vec4f,
  size: vec2f,
  window: vec2f,
};

struct UniformStorage {
  rectangles: array<Rectangle>,
};

@group(0) @binding(0) var<storage> data: UniformStorage;

fn rect_sdf(abs_pixel: vec2f, origin: vec2f, size: vec2f) -> f32 {
  let half_size = size * 0.5;
  let center = origin + half_size;
  
  // abs_pixelを矩形の中心からの座標に変換
  let rel_pixel = abs(abs_pixel - center);
  // 中心から矩形の角までの長さ
  let corner_from_center = half_size;
  
  // rel_pixelから矩形の角までの距離ベクトル（負の成分を除外）
  let pixel_to_corner = max(vec2f(0.0), rel_pixel - corner_from_center);
  let distance = length(pixel_to_corner);
  
  return distance;
}

struct VertexInput {
  @builtin(vertex_index) v_id: u32,
  @builtin(instance_index) instance: u32
}

struct VertexOutput {
  @builtin(position) pos: vec4f,
  @location(1) instance: u32, // TODO: @interpolate(flat)
  @location(2) coord: vec2f, // TODO: @interpolate(linear))
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
  let x = f32((in.v_id & 1) << 2);
  let y = f32((in.v_id & 2) << 1);
  let pos = vec4f(x - 1.0, y - 1.0, 0, 1);
  
  let rect = data.rectangles[in.instance];
  
  let vertex = mix(
    rect.origin.xy,
    rect.origin.xy + rect.size,
    pos.xy
  );
  
  var out: VertexOutput;
  out.pos = vec4f(vertex / rect.window, 0, 1);
  out.instance = in.instance;
  out.coord = vertex;
  
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
  let rect = data.rectangles[in.instance];
  let distance = rect_sdf(in.coord, rect.origin, rect.size);
  
  return vec4f(step(0.0, distance) * rect.color.rgb, 1.0);
}
