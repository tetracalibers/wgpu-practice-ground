@group(0) @binding(0) var output_texture: texture_storage_2d<rgba8unorm, write>;

struct Input {
  @builtin(global_invocation_id) global_id: vec3u,
  @builtin(local_invocation_id) local_id: vec3u,
  @builtin(workgroup_id) workgroup_id: vec3u,
  @builtin(num_workgroups) workgroup_size: vec3u,
}

@compute @workgroup_size(8, 8)
fn cs_main(in: Input) {
  let position = vec2i(in.global_id.xy);
  let color = vec4f(vec3f(in.workgroup_id) / vec3f(in.workgroup_size), 1.0);

  textureStore(output_texture, position, color);
}
