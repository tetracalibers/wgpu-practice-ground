use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  println!("Tutorial 03 - Pipeline");

  Ok(())
}
