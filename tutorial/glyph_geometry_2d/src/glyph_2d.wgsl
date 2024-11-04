struct VsInput {
  @location(0) position: vec2f
}

@vertex
fn vs_main(in: VsInput) -> @builtin(position) vec4f {
  return vec4(in.position.x - 1.0, in.position.y, 0.0, 1.0);
}

@group(0) @binding(0) var<uniform> color: vec4f;

@fragment
fn fs_main() -> @location(0) vec4f {
  return color;
}
