@group(0) @binding(0) var output_texture: texture_storage_2d<rgba8unorm, write>;

struct CsInput {
  @builtin(global_invocation_id) global_id: vec3u,
  @builtin(local_invocation_id) local_id: vec3u,
  @builtin(workgroup_id) workgroup_id: vec3u,
  @builtin(num_workgroups) workgroup_size: vec3u,
}

@compute @workgroup_size(8, 8)
fn cs_main(in: CsInput) {
  let tex_size = 512;
  
  //
  // Map the global_id to the UV coordinates
  //
  
  let uv = vec2f(f32(in.global_id.x), f32(in.global_id.y)) / f32(tex_size);
  
  //
  // Shader Art
  //
  
  let left = step(0.1, uv.x);
  let top = step(0.1, uv.y);
  
  let right = step(0.1, 1.0 - uv.x);
  let bottom = step(0.1, 1.0 - uv.y);
  
  let color = vec3f(left * top * right * bottom);
  
  //
  // Store color in texture
  //
  
  textureStore(output_texture, in.global_id.xy, vec4f(color, 1.0));
}
