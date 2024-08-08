mod app;
mod atlas;
mod lookup;
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

pub fn try_etagere() -> Result<(), Box<dyn Error>> {
  use etagere::*;
  use std::fs::File;

  let mut output = File::create("export/etagere.svg")?;

  let mut atlas = AtlasAllocator::new(size2(1000, 1000));

  let a = atlas.allocate(size2(100, 100)).unwrap();
  let b = atlas.allocate(size2(900, 200)).unwrap();

  println!("Allocated {:?} and {:?}", a.rectangle, b.rectangle);
  atlas.dump_svg(&mut output)?;

  atlas.deallocate(a.id);

  let c = atlas.allocate(size2(300, 200)).unwrap();

  atlas.deallocate(c.id);
  atlas.deallocate(b.id);

  Ok(())
}

pub fn try_swash() -> Result<(), Box<dyn Error>> {
  use swash::FontRef;

  let font_path = "./font/Sankofa_Display/SankofaDisplay-Regular.ttf";
  // Read the full font file
  let font_data = std::fs::read(font_path)?;
  // Create a font reference for the first font in the file
  let font = FontRef::from_index(&font_data, 0).unwrap();
  // Print the font attributes (stretch, weight and style)
  println!("{}", font.attributes());
  // Iterate through the localized strings
  for string in font.localized_strings() {
    // Print the string identifier and the actual value
    println!("[{:?}] {}", string.id(), string.to_string());
  }

  Ok(())
}

pub fn proto() -> Result<(), Box<dyn Error>> {
  use cosmic_text::*;
  use etagere::*;
  use std::fs::File;

  let mut font_system = FontSystem::new();
  let metrics = Metrics::new(14.0, 20.0);
  let mut buffer = Buffer::new(&mut font_system, metrics);
  let mut buffer = buffer.borrow_with(&mut font_system);

  buffer.set_size(Some(80.0), Some(25.0));
  let attrs = Attrs::new();
  buffer.set_text("Hello, Rust! 🦀\n", attrs, Shaping::Advanced);
  buffer.shape_until_scroll(true);

  let mut packer = BucketedAtlasAllocator::new(size2(200, 200));
  let mut atlas_list = Vec::new();

  for run in buffer.layout_runs() {
    for glyph in run.glyphs.iter() {
      let size = size2(glyph.w as i32, glyph.w as i32);
      let a = packer.allocate(size);
      if let Some(a) = a {
        atlas_list.push(a.id);
      } else {
        println!("Failed to allocate {:?}", glyph);
      }
    }
  }

  let mut output = File::create("export/proto-bucket-alras.svg")?;
  packer.dump_svg(&mut output)?;

  for id in atlas_list {
    packer.deallocate(id);
  }

  Ok(())
}
