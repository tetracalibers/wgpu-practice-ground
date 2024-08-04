use std::sync::Arc;

use winit::window::Window;

pub struct GfxState {
  window: Arc<Window>,
}

impl GfxState {
  pub fn new(window: Window) -> Self {
    let window = Arc::new(window);

    Self { window }
  }
}
