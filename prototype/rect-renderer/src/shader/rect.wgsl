const pi = 3.14;

fn gaussian(x: f32, sigma: f32) -> f32 {
  return exp(-(x * x) / (2 * sigma * sigma)) / (sqrt(2 * pi) * sigma);
}

fn erf(x: vec2f) -> vec2f {
  let s = sign(x);
  let a = abs(x);
  var result = 1 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
  result = result * result;
  return s - s / (result * result);
}

fn select_corner(x: f32, y: f32, c: vec4f) -> f32 {
  return mix(mix(c.x, c.y, step(0.0, x)), mix(c.w, c.z, step(0.0, x)), step(0.0, y));
}

fn rounded_box_shadow_X(x: f32, y: f32, s: f32, corner: f32, half_size: vec2f) -> f32 {
  let d = min(half_size.y - corner - abs(y), 0.0);
  let c = half_size.x - corner + sqrt(max(0.0, corner * corner - d * d));
  let integral = 0.5 + 0.5 * erf((x + vec2f(-c, c)) * (sqrt(0.5) / s));
  return integral.y - integral.x;
}

fn rounded_box_shadow(
  lower: vec2f,
  upper: vec2f,
  point: vec2f,
  sigma: f32,
  corners: vec4f
) -> f32 {
  // Center everything to make the math easier.
  let center = (lower + upper) * 0.5;
  let half_size = (upper - lower) * 0.5;
  let p = point - center;

  // The signal is only non-zero in a limited range, so don't waste samples.
  let low = p.y - half_size.y;
  let high = p.y + half_size.y;
  let start = clamp(-3 * sigma, low, high);
  let end = clamp(3 * sigma, low, high);

  // Accumulate samples (we can get away with surprisingly few samples).
  let step = (end - start) / 4.0;
  var y = start + step * 0.5;
  var value: f32 = 0;

  for (var i = 0; i < 4; i++) {
    let corner = select_corner(p.x, p.y, corners);
    value
      += rounded_box_shadow_X(p.x, p.y - y, sigma, corner, half_size)
      * gaussian(y, sigma) * step;
    y += step;
  }

  return value;
}

struct Rectangle {
  color: vec4f,
  origin: vec2f,
  _origin_z: f32,
  sigma: f32,
  corners: vec4f,
  size: vec2f,
  window: vec2f,
};

struct UniformStorage {
  rectangles: array<Rectangle>,
};

@group(0) @binding(0) var<storage> data: UniformStorage;

struct VertexInput {
  @location(0) position: vec2f,
  @builtin(instance_index) instance: u32
}

struct VertexOutput {
  @builtin(position) position: vec4f,
  @location(1) instance: u32, // TODO: @interpolate(flat)
  @location(2) v_coord: vec2f, // TODO: @interpolate(linear))
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
  let rect = data.rectangles[in.instance];
  let padding = 3 * rect.sigma;
  
  let vertex = mix(
    rect.origin.xy - padding,
    rect.origin.xy + rect.size + padding,
    in.position
  );
  
  var out: VertexOutput;
  out.position = vec4f(vertex / rect.window * 2 - 1, 0, 1);
  out.position.y *= -1.0;
  out.instance = in.instance;
  out.v_coord = vertex;
  
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
  let rect = data.rectangles[in.instance];
  let mask = rounded_box_shadow(
    rect.origin.xy,
    rect.origin.xy + rect.size,
    in.v_coord,
    rect.sigma,
    rect.corners
  );
  
  return vec4f(rect.color.rgb, rect.color.a * mask);
}
