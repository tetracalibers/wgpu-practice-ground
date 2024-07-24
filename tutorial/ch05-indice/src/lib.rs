use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  println!("Tutorial 05 - Indice");

  Ok(())
}
