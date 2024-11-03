use std::{error::Error, path::Path};

use wgsim::compute::pixel::ComputePixel;

pub async fn run() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  const IMG_SIZE: u32 = 128;
  const EXPORT_PATH: &str = "export/compute_visualize_workgroup_local.png";

  let pixel = ComputePixel::new(
    wgpu::include_wgsl!("./compute.wgsl"),
    "cs_main",
    wgpu::TextureFormat::Rgba8Unorm,
    IMG_SIZE,
  )
  .await?;

  let buf = pixel.compute(8, 8).await?;

  pixel.export_png(Path::new(EXPORT_PATH), &buf)?;
  pixel.clean_up(buf);

  Ok(())
}
