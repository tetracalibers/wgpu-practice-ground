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
  
  // Each result will return 1.0 (white) or 0.0 (black).
  let left = step(0.1, uv.x); // Similar to ( X greater than 0.1 )
  let top = step(0.1, uv.y); // Similar to ( Y greater than 0.1 )
  
  // The multiplication of left*bottom will be similar to the logical AND.
  let color = vec3f(left * top);
  
  //
  // Store color in texture
  //
  
  textureStore(output_texture, in.global_id.xy, vec4f(color, 1.0));
}
