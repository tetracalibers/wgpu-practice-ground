@group(0) @binding(0) var output_texture: texture_storage_2d<rgba8unorm, write>;

struct Input {
  @builtin(global_invocation_id) global_id: vec3u,
  @builtin(local_invocation_id) local_id: vec3u,
  @builtin(workgroup_id) workgroup_id: vec3u,
  @builtin(num_workgroups) workgroup_size: vec3u,
}

@compute @workgroup_size(8, 8)
fn cs_main(in: Input) {
  let tex_size = 512;
  let iterations = 100;
  
  //
  // Map the global_id to the UV coordinates
  //
  
  var uv = vec2f(f32(in.global_id.x), f32(in.global_id.y)) / f32(tex_size);
  uv = uv * 2.0 - 1.0; // [0, 1] to [-1, 1]
  
  //
  // Mandelbrot set iteration
  //
  
  var z = vec2f(0.0);
  let c = uv + vec2f(-0.5, 0.0); // shift x-axis (in middle of the screen)
  
  var color = vec4f(0.0, 0.0, 0.0, 1.0);
  
  for (var i = 0; i < iterations; i = i + 1) {
    // Mandelbrot formula: z = z^2 + c
    z = vec2f(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
    
    // If magnitude of z exceeds 2, the point is not in the Mandelbrot set
    if (length(z) > 2.0) {
      // Outside the Mandelbrot set, color based on iteration count
      let t = f32(i) / f32(iterations);
      color = vec4f(t, t, t, 1.0);
      
      break;
    }
  }
  
  //
  // Store color in texture
  //
  
  textureStore(output_texture, in.global_id.xy, color);
}
