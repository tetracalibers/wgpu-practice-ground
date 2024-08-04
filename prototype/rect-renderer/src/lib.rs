mod color;
mod geometry_value;
mod renderer;

use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  println!("Rect Renderer");

  Ok(())
}
