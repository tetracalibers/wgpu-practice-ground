struct GlobalMergeParams {
  stage: u32,
  substage: u32
}

@group(0) @binding(0) var<storage, read_write> numbers: array<i32>;
@group(1) @binding(0) var<uniform> params: GlobalMergeParams;

fn compare_and_swap(i: u32, j: u32, direction: bool) {
  if ((numbers[i] > numbers[j]) == direction) {
    let tmp = numbers[i];
    numbers[i] = numbers[j];
    numbers[j] = tmp;
  }
}

struct CsInput {
  @builtin(local_invocation_id) local_id: vec3u,
  @builtin(global_invocation_id) global_id: vec3u,
  @builtin(workgroup_id) group_id: vec3u,
}

// ローカルソート用のシェーダー
@compute @workgroup_size(64)
fn local_sort(in: CsInput) {
  let local_size = 64u;
  let group_offset = in.group_id.x * local_size;
  let idx = in.local_id.x;
  let len = arrayLength(&numbers);

  var k: u32 = 2u;
  while (k <= local_size) {
    var j: u32 = k >> 1u;
    while (j > 0u) {
      let ixj = idx ^ j;
      if (ixj > idx) {
        let index1 = group_offset + idx;
        let index2 = group_offset + ixj;
        if (index2 < len) {
          compare_and_swap(index1, index2, (idx & k) == 0u);
        }
      }
      workgroupBarrier();
      j = j >> 1u;
    }
    k = k << 1u;
  }
}

@compute @workgroup_size(64)
fn global_merge(in: CsInput) {
  let stage = params.stage;
  let substage = params.substage;

  let global_idx = in.global_id.x;
  let len = arrayLength(&numbers);

  let k = 1u << (stage + 1u);
  let j = 1u << substage;

  let ixj = global_idx ^ j;

  if ixj > global_idx && ixj < len {
    let direction = ((global_idx & k) == 0u);
    compare_and_swap(global_idx, ixj, direction);
  }
}