struct BlurParams {
  kernel_size: u32,
}

// スレッド数
const workgroup_size = 32u;

// 各スレッドは、1つのタイル（複数のピクセル）を処理する
const tile_size = 4u;

// 1つのワークグループに必要なすべてのピクセルを保持する
const cache_size = tile_size * workgroup_size; // 128

// テクスチャルックアップ用のキャッシュ
// 各スレッドは、ピクセルのタイルをワークグループの共有メモリに追加する
var<workgroup> cache: array<array<vec3f, 128>, 4>;

@group(0) @binding(0) var samp: sampler;
@group(0) @binding(1) var<uniform> params: BlurParams;

@group(1) @binding(0) var input_tex: texture_2d<f32>;
@group(1) @binding(1) var output_tex: texture_storage_2d<rgba8unorm, write>;
@group(1) @binding(2) var<uniform> flip_blur_dir: u32; // 0 or 1

struct CsInput {
  @builtin(workgroup_id) workgroup_id: vec3u,
  @builtin(local_invocation_id) local_id: vec3u,
  @builtin(global_invocation_id) global_id: vec3u
}

@compute @workgroup_size(32, 1, 1)
fn cs_main(in: CsInput) {
  let workgroup_id = in.workgroup_id.xy;
  let local_id = in.local_id.xy;
  
  // テクスチャの寸法
  let dims = vec2u(textureDimensions(input_tex, 0));
  
  let kernel_size = params.kernel_size;
  
  // キャッシュには、ディスパッチエリア内でカーネルを正しく評価するために必要な境界ピクセルも含める必要がある
  let dispatch_size = vec2u(cache_size - (kernel_size - 1), 4u);
  
  // カーネルオフセット（カーネルの中心に隣接するピクセルの数）は、
  // ピクセルキャッシュに含めるべき、ディスパッチエリア（=作業エリア）の
  // 隣接する境界エリアを定義する
  let kernel_offset = (kernel_size - 1) / 2;
  
  // このスレッドのタイルのローカルピクセルオフセット（ワークグループ内のタイル）
  let tile_offset = local_id * vec2u(tile_size, 1u);
  
  // ワークグループのグローバルピクセルオフセット
  let dispatch_offset = workgroup_id * dispatch_size;
  
  // カーネルの畳み込みに必要な境界ピクセルを含めるために、
  // カーネルオフセットを引く（ディスパッチエリア内での処理のため）
  let base_index = dispatch_offset + tile_offset - vec2u(kernel_offset, 0u);
  
  // このスレッドのタイルのピクセルをキャッシュに追加
  for (var r = 0u; r < tile_size; r++) {
    for (var c = 0u; c < tile_size; c++) {
      var load_index = base_index + vec2u(c, r);
      
      if (flip_blur_dir != 0u) {
        load_index = load_index.yx;
      }
      
      let x = r;
      let y = tile_size * local_id.x + c;
      
      // convert to uv space
      let sample_coord: vec2f = vec2f(load_index) + vec2f(0.25);
      let sample_uv: vec2f = sample_coord / vec2f(dims);
      
      let value = textureSampleLevel(input_tex, samp, sample_uv, 0.0).rgb;
      cache[x][y] = value;
    }
  }

  workgroupBarrier();
  
  for (var r = 0u; r < tile_size; r++) {
    for (var c = 0u; c < tile_size; c++) {
      var write_index = base_index + vec2u(c, r);
      
      if (flip_blur_dir != 0u) {
        write_index = write_index.yx;
      }
    
      let center = (tile_size * local_id.x) + c;
    
      if (center >= kernel_offset && center < cache_size - kernel_offset && all(write_index < dims)) {
        // convolution with kernel
        var acc = vec3(0.0);
        for (var f = 0u; f < kernel_size; f++) {
          let i = center + f - kernel_offset;
          acc += (1.0 / f32(kernel_size)) * cache[r][i];
        }
        
        textureStore(output_tex, write_index, vec4(acc, 1.0));
      }
    }
  }
}
