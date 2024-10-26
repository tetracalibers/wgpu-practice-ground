//
// Dataという構造体を使用することで、メモリ上のデータのレイアウトを定義する
//
struct Data {
  value: f32
}

//
// groupとbindingは、後で作成・リンクする正しいバッファ（メモリ）に接続するために使用される
//
@group(0) @binding(0) var<storage, read_write> data: Data;

struct Input {
  @builtin(global_invocation_id) global_id: vec3u,
}

@compute @workgroup_size(1)
fn cs_main(in: Input) {
  data.value += 1.0;
}
