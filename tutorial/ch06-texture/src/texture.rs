use anyhow::*;
use image::GenericImageView;

pub struct Texture {
  pub texture: wgpu::Texture,
  pub view: wgpu::TextureView,
  pub sampler: wgpu::Sampler,
}

impl Texture {
  pub fn from_bytes(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bytes: &[u8],
    label: &str,
  ) -> Result<Self> {
    let img = image::load_from_memory(&bytes)?;
    Self::from_image(device, queue, &img, Some(label))
  }

  pub fn from_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    img: &image::DynamicImage,
    label: Option<&str>,
  ) -> Result<Self> {
    // RGBAバイトのVecに変換
    // ## as_rgba8()の代わりにto_rgba8()を使っている
    // PNGはアルファチャンネルを持っているので、as_rgba8()を使っても問題なく動作する
    // しかし、JPEGにはアルファチャンネルがないので、これから使うJPEGテクスチャ画像でas_rgba8()を呼び出そうとすると、コードがパニックになる
    // その代わりにto_rgba8()を使えば、元画像にアルファチャンネルがなくてもアルファチャンネルを持つ新しい画像バッファを生成することができる
    let rgba = img.to_rgba8();
    let dimensions = img.dimensions();

    let size = wgpu::Extent3d {
      width: dimensions.0,
      height: dimensions.1,
      depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
      label,
      // すべてのテクスチャは3Dとして保存されるので、深度を1に設定することで2Dテクスチャを表現する
      size,
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      // ほとんどの画像はsRGBで保存されているので、ここではそれを反映させる必要がある
      format: wgpu::TextureFormat::Rgba8UnormSrgb,
      // TEXTURE_BINDINGはwgpuにシェーダーでこのテクスチャーを使いたいことを伝える
      // COPY_DSTはこのテクスチャにデータをコピーすることを意味する
      usage: wgpu::TextureUsages::TEXTURE_BINDING
        | wgpu::TextureUsages::COPY_DST,
      // SurfaceConfigと同様
      // このテクスチャの TextureView を作成するためにどのテクスチャ形式を使用できるかを指定する
      // 基本となるテクスチャ形式 (この場合Rgba8UnormSrgb)は常にサポートされる
      // 異なるテクスチャ形式の使用は、WebGL2ではサポートされていないことに注意
      view_formats: &[],
    });

    // テクスチャにデータを取り込む
    // Texture構造体には、データを直接操作するメソッドはない
    // 先ほど作成したqueueのwrite_textureというメソッドを使ってテクスチャを読み込むことができる
    queue.write_texture(
      // wgpuへどこにピクセルデータをコピーすればよいか伝える
      wgpu::ImageCopyTexture {
        texture: &texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      },
      // 実際のピクセルデータ
      &rgba,
      // テクスチャのレイアウト
      wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: Some(4 * dimensions.0),
        rows_per_image: Some(dimensions.1),
      },
      size,
    );

    // TextureViewはテクスチャを表示するためのもの
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    // SamplerはTextureをどのようにサンプリングするかを制御する
    // サンプリングはGIMP/Photoshopのスポイトツールに似た働きをする
    // プログラムはテクスチャ上の座標（テクスチャ座標）を提供し、Samplerはテクスチャといくつかの内部パラメータに基づいて対応する色を返す
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      // SamplerがTextureの外側の座標を取得した場合の処理を決定する
      // - ClampToEdge：テクスチャの外側にあるテクスチャ座標は、テクスチャの端にある最も近いピクセルの色を返す
      // - Repeat：テクスチャ座標がテクスチャの寸法を超えると、テクスチャは繰り返される
      // - MirrorRepeat：Repeatと似ているが、境界を超えると画像が反転する
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      // サンプルのフットプリントが1テクセルより小さいときと大きいときの処理を記述する
      // この2つのフィールドは通常、シーン内のマッピングがカメラから遠いか近い場合に機能する
      // - Linear: 各次元で2つのテクセルを選択し、それらの値の間の線形補間を返す
      // - Nearest: テクスチャ座標に最も近いテクセル値を返す。これにより、遠くから見ると鮮明だが、近くではピクセル化された画像が作成される
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Nearest,
      // ミップマップ間のブレンド方法をサンプラーに指示する
      // (mag/min)_filterと同じように機能する
      mipmap_filter: wgpu::FilterMode::Nearest,
      ..Default::default()
    });

    Ok(Self {
      texture,
      view,
      sampler,
    })
  }
}
