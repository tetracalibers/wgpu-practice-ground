use std::{error::Error, num::NonZeroU32, sync::Arc};

use winit::window::Window;

pub trait Render<T> {
  fn new(window: Arc<&Window>, inputs: &T) -> Self;
  fn resize(&mut self, width: NonZeroU32, height: NonZeroU32);
  fn draw(&mut self) -> Result<(), Box<dyn Error>>;
}
