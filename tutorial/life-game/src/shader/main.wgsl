//
// GPUでは、
// 1. 頂点シェーダーと呼ばれる小さなプログラムを利用して、頂点をクリップ空間に変換するために必要な数学的操作や、頂点の描画に必要なその他の計算を実行する
// 2. シェーダーを適用した後、変換された頂点で構成された三角形を画面に描画するために必要なピクセルを決定する
// 3. その後、もうひとつの小さなプログラムであるフラグメントシェーダーが実行され、各ピクセルの色が計算される
// これらのピクセルの色の結果がテクスチャに蓄積され、画面上に表示される
//

// gridという名前のユニフォームを定義
@group(0) @binding(0) var<uniform> grid: vec2f;
// ストレージバッファの内容をcell_stateという名前で参照
@group(0) @binding(1) var<storage> cell_state: array<u32>;

//
// 頂点シェーダーは関数として定義され、GPUではVertexBuffer内の頂点ごとに1回この関数が呼び出される
// 頂点シェーダー関数が呼び出されるたびに、VertexBufferから異なる位置が引数として関数に渡され、クリップ空間内の対応する位置が返される
//
// これらは順番に呼び出されるわけではなく、並列的に実行されるため、性能によっては数百から数千もの頂点を同時に処理できる
// 並列性に伴う制約として、頂点シェーダー同士でのやり取りはできない
// 各シェーダーの呼び出しでは、一度に1つの頂点のデータのみを参照でき、1つの頂点の値のみを出力できる
//

struct VertexInput {
  // 作成したバッファのデータを利用するためには、
  // 1. @location()属性を使用して引数を宣言する（shader_locationと対応）
  // 2. VertexBufferLayoutで記述したものに一致する型を指定する（formatと対応）
  @location(0) pos: vec2f,
  // instance_index
  // - WGSLの組み込み値（WebGPU によって自動的に計算される値）
  // - この値は、同じインスタンスとして処理されるすべての頂点で同じとなる
  // - 頂点バッファの各位置について、instance_indexの値が0に設定された状態で頂点シェーダーが6回、instance_indexの値が1に設定された状態で6回、…というように呼び出される
  @builtin(instance_index) instance: u32,
};

struct VertexOutput {
  // 頂点シェーダーでは、少なくともクリップ空間で処理される頂点の最終的な位置を返す必要がある
  // 返される値が必須の位置であることを示すには、@builtin(position)属性でマークする
  @builtin(position) pos: vec4f,
  // 頂点とフラグメントのステージ間でデータを受け渡すには、
  // 1. 任意の@locationを使用して@vertex関数の出力に含める
  // 2. @fragment関数で、同じ@locationを使用して引数を追加し、値を受け取る
  @location(0) cell: vec2f,
}

// 頂点シェーダー関数には任意の名前を付けることができる
// どのステージのシェーダーなのかを示すため、先頭に@vertex属性を指定する必要がある
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
  //
  // すべての正方形が同じ場所に描画されてしまわないように、インスタンスごとにジオメトリの位置を再配置する
  //
  
  // instance_indexをfloat値にキャスト
  // この値は各正方形について 0, 1, 2, ... 15 となる
  let i = f32(in.instance);
  
  // [0, 15]の値を取りうるiをそのまま使うと、最初の4つしかキャンバス上に収まらない
  // x は iをgrid.xで割った余りとすることで、[0, grid.x]範囲内の値を繰り返すようにする
  // x が [0, grid.x] を一周するたびに、yをインクリメントしたい
  // y は [0, 15] のうち、今何周目かを求めたものとする
  // example:
  // i = 0 の時、(0, 0)のセルに表示される
  // i = 3 の時、(3, 0)のセルに表示される
  // i = 4 の時、(0, 1)のセルに表示される
  // i = 7 の時、(3, 1)のセルに表示される
  // ...
  let cell_x = i % grid.x;
  let cell_y = floor(i / grid.y);
  
  // 正方形を表示するセル
  // この値を変えることで、正方形を表示するセルを変更できる
  // instance_indexを使っているので、自動的に各正方形が異なるセルに表示される
  let cell = vec2f(cell_x, cell_y);
  
  // セルのアクティブ状態を問い合わせる
  // 状態はストレージバッファに1次元配列として保存されているので、instance_indexを使用して現在のセルの値を検索できる
  let state = f32(cell_state[instance]);
  
  // グリッドの1単位（キャンバスのgrid分の1）
  // - キャンバスの座標は-1から1の2単位にわたっている
  // - キャンバスのgrid分の1だけ頂点を移動する場合、0.5単位の移動が必要になるので、2倍する
  let cell_offset = cell / grid * 2;
  
  // 1. 状態が非アクティブだった場合にセルを非表示にするため、stateを乗算
  // - 1倍にスケーリングするとジオメトリはそのまま
  // - 0倍にスケーリングするとジオメトリは点となり、GPUによって破棄される
  // 2. すべての頂点を（クリップ空間のサイズの半分にあたる）1ずつ上と右に移動してから、グリッドサイズで除算する
  // - pos / grid だと、正方形の中心が原点に来るため、正方形がグリッドセル内に収まらない
  // 3. 左下を(0, 0)としたいので、ジオメトリの位置を(-1, -1)だけ平行移動
  // - キャンバスの座標系では中央が(0, 0)で、左下が(-1, -1)となっている
  // 4. 各セルに対して、グリッドの1単位（cell_offset）だけ正方形を動かす
  let grid_pos = (in.pos * state + 1) / grid - 1 + cell_offset;
  
  // varは可変、letは不変
  var output: VertexOutput;
  output.pos = vec4f(grid_pos, 0, 1);
  output.cell = cell;
  
  return output;
}

//
// フラグメントシェーダーは、各頂点に対して呼び出されるのではなく、描画される各ピクセルに対して呼び出される
// フラグメントシェーダーは、常に頂点シェーダーの後に呼び出される
//
// GPUは、
// 1. 頂点シェーダーの出力から三角形を作成する（3つの点をもとに三角形を作る）
// 2. その後、出力されるカラーアタッチメントのどのピクセルがその三角形に含まれるかを計算する
// 3. それらの各三角形をラスタライズして、各ピクセルにつき1回ずつフラグメントシェーダーを呼び出す
//
// フラグメントシェーダーは色を返す
// 色は通常、頂点シェーダーやテクスチャなどのアセットから送られる値に基づいて計算される
// この色がGPUによってカラーアタッチメントに書き込まれる
//

//
// ## 返り値
// - 返された色がbegin_render_pass呼び出しのどのColorAttachmentに書き込まれるかを示すため、戻り値には@location属性を指定する必要がある
//
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
  // セルの値は、それぞれの軸で 0 ~ GRID_SIZE の範囲
  // 最初の行および列で赤または緑のカラーチャネルの上限である1に達してしまい、それ以降のすべてのセルは同じ値に切り詰められてしまう
  // これを回避するため、gridで除算することで、0 ~ 1 の範囲に収める
  let rg = in.cell / grid;
  
  // 左下隅ではグリッドが黒くなり、暗く見えてしまうのを回避するため、青チャネルを調整して明るくする
  // 他の色が最も暗くなる場所で青色を最も明るくし、他の色の強度が高くなるにつれて青色が暗くなるように
  // 青色のチャネルを、最大値1からセルの他のいずれかのカラーチャネルの値を減算した値とする
  return vec4f(rg, 1 - rg.r, 1); // (Red, Green, Blue, Alpha)
}
