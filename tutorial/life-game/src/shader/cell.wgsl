//
// GPUでは、
// 1. 頂点シェーダーと呼ばれる小さなプログラムを利用して、頂点をクリップ空間に変換するために必要な数学的操作や、頂点の描画に必要なその他の計算を実行する
// 2. シェーダーを適用した後、変換された頂点で構成された三角形を画面に描画するために必要なピクセルを決定する
// 3. その後、もうひとつの小さなプログラムであるフラグメントシェーダーが実行され、各ピクセルの色が計算される
// これらのピクセルの色の結果がテクスチャに蓄積され、画面上に表示される
//

//
// 頂点シェーダーは関数として定義され、GPUではVertexBuffer内の頂点ごとに1回この関数が呼び出される
// 頂点シェーダー関数が呼び出されるたびに、VertexBufferから異なる位置が引数として関数に渡され、クリップ空間内の対応する位置が返される
//
// これらは順番に呼び出されるわけではなく、並列的に実行されるため、性能によっては数百から数千もの頂点を同時に処理できる
// 並列性に伴う制約として、頂点シェーダー同士でのやり取りはできない
// 各シェーダーの呼び出しでは、一度に1つの頂点のデータのみを参照でき、1つの頂点の値のみを出力できる
//

//
// ## 宣言
// - 頂点シェーダー関数には任意の名前を付けることができる
// - どのステージのシェーダーなのかを示すため、先頭に@vertex属性を指定する必要がある
//
// ## 返り値
// - 頂点シェーダーでは、少なくともクリップ空間で処理される頂点の最終的な位置を返す必要がある
// - 返される値が必須の位置であることを示すには、@builtin(position)属性でマークする
//
// ## 引数
// 作成したバッファのデータを利用するためには、
// 1. 関数で@location()属性を使用して引数を宣言する（shader_locationと対応）
// 2. VertexBufferLayoutで記述したものに一致する型を指定する（formatと対応）
//
@vertex
fn vs_main(@location(0) pos: vec2f) -> @builtin(position) vec4f {
  return vec4f(pos, 0, 1);
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
fn fs_main() -> @location(0) vec4f {
  return vec4f(1, 0, 0, 1); // (Red, Green, Blue, Alpha)
}
