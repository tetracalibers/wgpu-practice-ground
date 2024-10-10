struct LightUniforms {
  light_position: vec4f,
  eye_position: vec4f,
  color: vec4f,
  specular_color: vec4f,
}

@binding(0) @group(1) var<uniform> light: LightUniforms;

struct MaterialUniforms {
  ambient: f32,
  diffuse: f32,
  specular: f32,
  shininess: f32,
}

@binding(1) @group(1) var<uniform> material: MaterialUniforms;

fn blinn_phong(N: vec3f, L: vec3f, V: vec3f) -> vec2f {
  let H = normalize(L + V);
  var diffuse = material.diffuse * max(dot(N, L), 0.0);
  diffuse += material.diffuse * max(dot(-N, L), 0.0);
  var specular = material.specular * pow(max(dot(N, H), 0.0), material.shininess);
  specular += material.specular * pow(max(dot(-N, H), 0.0), material.shininess);
  return vec2(diffuse, specular);
}

struct Varyings {
  @location(0) v_position: vec4f,
  @location(1) v_normal: vec4f,
}

@fragment
fn fs_main(in: Varyings) -> @location(0) vec4f {
  var N = normalize(in.v_normal.xyz);
  let L = normalize(light.light_position.xyz - in.v_position.xyz);
  let V = normalize(light.eye_position.xyz - in.v_position.xyz);
  
  let bp = blinn_phong(N, L, V);
  let diffuse = bp[0];
  let specular = bp[1];
  
  let final_color = light.color * (material.ambient + diffuse) + light.specular_color * specular;
  return vec4(final_color.rgb, 1.0);
}