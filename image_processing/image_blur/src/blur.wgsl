struct BlurParams {
  filter_dim: i32,
  block_dim: u32,
}

struct Flip {
  value: u32,
}

@group(0) @binding(0) var samp: sampler;
@group(0) @binding(1) var<uniform> params: BlurParams;

@group(1) @binding(1) var input_tex: texture_2d<f32>;
@group(1) @binding(2) var output_tex: texture_storage_2d<rgba8unorm, write>;
@group(1) @binding(3) var<uniform> flip: Flip;

var<workgroup> tile: array<array<vec3f, 128>, 4>;

struct CsInput {
  @builtin(workgroup_id) workgroup_id: vec3u,
  @builtin(local_invocation_id) local_id: vec3u
}

@compute @workgroup_size(32, 1, 1)
fn cs_main(in: CsInput) {
  let filter_offset = (params.filter_dim - 1) / 2;
  let dims = vec2i(textureDimensions(input_tex, 0));
  let base_index = vec2i(in.workgroup_id.xy * vec2(params.block_dim, 4) + in.local_id.xy * vec2(4, 1)) - vec2(filter_offset, 0);

  for (var r = 0; r < 4; r++) {
    for (var c = 0; c < 4; c++) {
      var load_index = base_index + vec2(c, r);
      if (flip.value != 0u) {
        load_index = load_index.yx;
      }

      tile[r][4 * in.local_id.x + u32(c)] = textureSampleLevel(input_tex, samp, (vec2f(load_index) + vec2f(0.25, 0.25)) / vec2f(dims), 0.0).rgb;
    }
  }

  workgroupBarrier();

  for (var r = 0; r < 4; r++) {
    for (var c = 0; c < 4; c++) {
      var write_index = base_index + vec2(c, r);
      if (flip.value != 0) {
        write_index = write_index.yx;
      }

      let center = i32(4 * in.local_id.x) + c;
      if (center >= filter_offset && center < 128 - filter_offset && all(write_index < dims)) {
        var acc = vec3(0.0, 0.0, 0.0);
        for (var f = 0; f < params.filter_dim; f++) {
          var i = center + f - filter_offset;
          acc = acc + (1.0 / f32(params.filter_dim)) * tile[r][i];
        }
        textureStore(output_tex, write_index, vec4(acc, 1.0));
      }
    }
  }
}
