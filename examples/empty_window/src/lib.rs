use std::error::Error;

use window_app::App;
use winit::event_loop::EventLoop;

mod window_app;

pub fn run() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  let mut app = App::new("winitで作成したウィンドウ (500 x 300)")
    .with_window_size(500, 300);

  let event_loop = EventLoop::builder().build()?;
  event_loop.run_app(&mut app)?;

  Ok(())
}
