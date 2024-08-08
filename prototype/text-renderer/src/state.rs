use std::sync::Arc;

use winit::window::Window;

pub struct WindowState {
  pub window: Arc<Window>,
}

impl WindowState {
  pub async fn new(window: Arc<Window>) -> Self {
    WindowState { window }
  }
}
