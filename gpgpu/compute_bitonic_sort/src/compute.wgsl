@group(0) @binding(0) var<storage, read_write> numbers: array<i32>;

fn compare_and_swap(i: u32, j: u32, direction: bool) {
  if ((numbers[i] > numbers[j]) == direction) {
    let tmp = numbers[i];
    numbers[i] = numbers[j];
    numbers[j] = tmp;
  }
}

struct CsInput {
  @builtin(global_invocation_id) id: vec3u,
}

@compute @workgroup_size(64)
fn cs_main(in: CsInput) {
  var k: u32 = 2;
  let len = arrayLength(&numbers);
  
  while (k <= len) {
    var j: u32 = k >> 1;
    
    while (j > 0) {
      let idx = in.id.x;
      let ixj = idx ^ j;
      
      if (ixj > idx) {
        compare_and_swap(idx, ixj, (idx & k) == 0);
      }
    
      j = j >> 1;
      workgroupBarrier();
    }
    
    k = k << 1;
  }
}
