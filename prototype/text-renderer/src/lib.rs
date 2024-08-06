use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  println!("Prototype: Text Renderer");

  Ok(())
}
