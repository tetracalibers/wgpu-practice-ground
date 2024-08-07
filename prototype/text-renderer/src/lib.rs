mod app;
mod renderer;
mod state;

use std::error::Error;

use app::Application;
use winit::event_loop::EventLoop;

pub fn run() -> Result<(), Box<dyn Error>> {
  env_logger::init();

  let event_loop = EventLoop::builder().build()?;
  let mut app = Application::default();

  event_loop.run_app(&mut app)?;

  Ok(())
}

pub fn try_cosmic_text() -> Result<(), Box<dyn Error>> {
  use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache};

  // A FontSystem provides access to detected system fonts, create one per application
  let mut font_system = FontSystem::new();

  // A SwashCache stores rasterized glyphs, create one per application
  // let mut swash_cache = SwashCache::new();

  // Text metrics indicate the font size and line height of a buffer
  let metrics = Metrics::new(14.0, 20.0);

  // A Buffer provides shaping and layout for a UTF-8 string, create one per text widget
  let mut buffer = Buffer::new(&mut font_system, metrics);

  // Borrow buffer together with the font system for more convenient method calls
  let mut buffer = buffer.borrow_with(&mut font_system);

  // Set a size for the text buffer, in pixels
  buffer.set_size(Some(80.0), Some(25.0));

  // Attributes indicate what font to choose
  let attrs = Attrs::new();

  // Add some text!
  buffer.set_text("Hello, Rust! 🦀\n", attrs, Shaping::Advanced);

  // Perform shaping as desired
  buffer.shape_until_scroll(true);

  // Inspect the output runs
  for run in buffer.layout_runs() {
    for glyph in run.glyphs.iter() {
      println!("{:#?}", glyph);
    }
  }

  Ok(())
}
