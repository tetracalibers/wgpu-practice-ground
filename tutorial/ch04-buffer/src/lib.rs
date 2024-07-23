use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  println!("Tutorial 04 - Buffer");

  Ok(())
}
