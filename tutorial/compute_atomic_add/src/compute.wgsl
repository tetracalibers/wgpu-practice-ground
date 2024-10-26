@group(0) @binding(0) var<storage, read> source: array<i32, 3>;
@group(0) @binding(1) var<storage, read_write> non_atomic_result: array<i32, 3>;
@group(0) @binding(2) var<storage, read_write> atomic_result: array<atomic<i32>, 3>;

struct Input {
  @builtin(global_invocation_id) global_id: vec3u,
}

// ワークグループのサイズが `8` -> 8つのスレッドが作成される（数を`2`のべき乗に保っている）
// ただし、使用されるスレッドは `3` だけ（配列のサイズが `3`）
@compute @workgroup_size(8, 1)
fn cs_main(in: Input) {
  // 配列には3つのメモリ領域があるため、global_idを使用してそれぞれの領域 `(0, 1, 2)` に対して操作を行う
  // 開始時点でglobal_idが `3` より大きい場合は、何もせずにこのスレッドを終了する
  if (in.global_id.x < 0 || in.global_id.x > 3) { return; }
  
  // 非アトミックな加算
  non_atomic_result[0] += source[in.global_id.x];
  
  // アトミックな加算
  atomicAdd(&atomic_result[0], source[in.global_id.x]);
}
