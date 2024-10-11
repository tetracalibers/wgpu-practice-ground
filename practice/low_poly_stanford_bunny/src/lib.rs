use std::error::Error;

use wgpu_helper::model::load_model_json;

pub fn run(window_title: &str) -> Result<(), Box<dyn Error>> {
  env_logger::init();

  let model = load_model_json("./assets/model/Bunny-LowPoly.json")?;

  println!("{:?}", model);

  Ok(())
}
