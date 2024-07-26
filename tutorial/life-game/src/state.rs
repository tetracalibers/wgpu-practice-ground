use std::sync::Arc;

use winit::window::Window;

pub struct State {
  window: Arc<Window>,
}

impl State {
  pub async fn new(window: Window) -> Self {
    let window = Arc::new(window);

    Self { window }
  }

  pub fn window(&self) -> &Window {
    &self.window
  }
}
