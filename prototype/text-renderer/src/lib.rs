mod app;
mod atlas;
mod lookup;
mod renderer;
mod sdf;
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

pub fn try_rusttype() -> Result<(), Box<dyn Error>> {
  use image::{DynamicImage, Rgba};
  use rusttype::{point, Font, Scale};

  let font_path = "./font/Poiret_One/PoiretOne-Regular.ttf";
  let font_data = std::fs::read(font_path)?;

  let font = Font::try_from_bytes(font_data.as_slice())
    .expect("error constructing a Font from bytes");

  // The font size to use
  let scale = Scale::uniform(32.0);

  // The text to render
  let text = "This is RustType rendered into a png!";

  // Use a dark red colour
  let colour = (150, 0, 0);

  let v_metrics = font.v_metrics(scale);

  // layout the glyphs in a line with 20 pixels padding
  let glyphs: Vec<_> =
    font.layout(text, scale, point(20.0, 20.0 + v_metrics.ascent)).collect();

  // work out the layout size
  let glyphs_height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;
  let glyphs_width = {
    let min_x =
      glyphs.first().map(|g| g.pixel_bounding_box().unwrap().min.x).unwrap();
    let max_x =
      glyphs.last().map(|g| g.pixel_bounding_box().unwrap().max.x).unwrap();
    (max_x - min_x) as u32
  };

  // Create a new rgba image with some padding
  let mut image =
    DynamicImage::new_rgba8(glyphs_width + 40, glyphs_height + 40).to_rgba8();

  // Loop through the glyphs in the text, positing each one on a line
  for glyph in glyphs {
    if let Some(bounding_box) = glyph.pixel_bounding_box() {
      // Draw the glyph into the image per-pixel by using the draw closure
      glyph.draw(|x, y, v| {
        image.put_pixel(
          // Offset the position by the glyph bounding box
          x + bounding_box.min.x as u32,
          y + bounding_box.min.y as u32,
          // Turn the coverage into an alpha value
          Rgba([colour.0, colour.1, colour.2, (v * 255.0) as u8]),
        )
      });
    }
  }

  // Save the image to a png file
  image.save("./export/glyphs-rusttype.png").unwrap();

  Ok(())
}

pub fn proto() -> Result<(), Box<dyn Error>> {
  use etagere::size2;
  use std::collections::HashMap;
  use ttf_parser as ttf;

  env_logger::init();

  let version = 12;

  const ATLAS_FONT_SIZE: u16 = 48;
  const ATLAS_GAP: u16 = 2;
  const ATLAS_RADIUS: u16 = ATLAS_FONT_SIZE / 6; // sometimes called `spread`

  let text = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
  let chars = text.chars();

  //let font_path = "./font/Sankofa_Display/SankofaDisplay-Regular.ttf";
  //let font_path = "./font/Poiret_One/PoiretOne-Regular.ttf";
  //let font_path = "./font/Crimson_Text/CrimsonText-Regular.ttf";
  let font_path = "./font/Lusitana/Lusitana-Regular.ttf";
  let font_data = std::fs::read(font_path)?;

  // use ttf-parser
  let font_face = ttf::Face::parse(&font_data, 0)?;

  let glyph_ids = text
    .chars()
    .map(|c| {
      font_face
        .glyph_index(c)
        .expect(std::format!("unknown character: {}", c).as_str())
    })
    .collect::<Vec<_>>();

  // --- calculateGlyphQuads ---

  struct Glyph {
    id: ttf::GlyphId,
    x: i16,
    y: i16,
    width: i16,
    height: i16,
    lsb: i16,
    rsb: i16,
  }

  let tables = font_face.tables();

  let glyf = tables.glyf.unwrap();
  let hmtx = tables.hmtx.unwrap();
  let head = tables.head;

  let num_glyphs = glyph_ids.len();

  let glyphs = glyph_ids
    .iter()
    .map(|glyph_id| {
      let ttf::Rect {
        x_min,
        x_max,
        y_min,
        y_max,
      } = glyf.bbox(*glyph_id).unwrap_or(ttf::Rect {
        x_min: 0,
        x_max: 0,
        y_min: 0,
        y_max: 0,
      });

      let x = x_min;
      let y = y_min;
      let width = x_max - x_min;
      let height = y_max - y_min;

      let advance_width = hmtx.advance(*glyph_id).unwrap_or(0) as i16;
      let lsb = hmtx.side_bearing(*glyph_id).unwrap_or(0);
      let rsb = advance_width - lsb - width;

      let glyph = Glyph {
        id: *glyph_id,
        x: x.into(),
        y: y.into(),
        width: width.into(),
        height: height.into(),
        lsb,
        rsb,
      };

      glyph
    })
    .collect::<Vec<_>>();

  let glyph_map =
    glyphs.iter().map(|gly| (gly.id, gly)).collect::<HashMap<_, _>>();

  // --- prepareLookups ---

  let ppem = font_face.units_per_em();
  let scale_factor = ATLAS_FONT_SIZE as f32 / ppem as f32;

  let transform = |x: f32| -> f32 { (x * scale_factor).ceil() };
  let sizes = glyphs
    .iter()
    .map(|gly| {
      let x = transform(gly.width as f32) as u16;
      let y = transform(gly.height as f32) as u16;
      (x + ATLAS_GAP * 2, y + ATLAS_GAP * 2)
    })
    .collect::<Vec<_>>();

  let glyph_size = ATLAS_FONT_SIZE as f32;
  println!("glyph_size: {}", glyph_size);
  // glyph_size * glyph_size が 1グリフの面積となる
  let atlas_size =
    (glyph_size.powi(2) * num_glyphs as f32).sqrt().ceil() as i32;
  println!("atlas_size: {}", atlas_size);

  let mut atlas = etagere::AtlasAllocator::with_options(
    size2(atlas_size, atlas_size),
    &etagere::AllocatorOptions {
      alignment: size2(2, 1),
      ..Default::default()
    },
  );

  // TODO: キャッシュの仕組みを用意し、allocateがNoneの場合に対応する
  let allocations = sizes
    .iter()
    .map(|(w, h)| atlas.allocate(size2(*w as i32, *h as i32)).unwrap())
    .collect::<Vec<_>>();

  let mut atlas_svg =
    std::fs::File::create(std::format!("export/font-atlas-v{}.svg", version))?;
  atlas.dump_svg(&mut atlas_svg)?;

  let atlas_positions = allocations
    .iter()
    .map(|alloc| {
      let rect = alloc.rectangle.to_rect();
      (rect.origin.x, rect.origin.y)
    })
    .collect::<Vec<_>>();

  //println!("atlas_positions: {:?}", atlas_positions);

  for alloc in allocations {
    //if let Some(alloc) = alloc {
    atlas.deallocate(alloc.id);
    //}
  }

  let uv_map = glyph_map
    .keys()
    .enumerate()
    .map(|(i, g_id)| {
      let (w, h) = sizes[i];
      let (x, y) = atlas_positions[i];
      let (w, h) = (w as f32, h as f32);
      let (x, y) = (x as f32, y as f32);
      let (w, h) = (w / atlas_size as f32, h / atlas_size as f32);
      let (x, y) = (x / atlas_size as f32, y / atlas_size as f32);
      let vec4 = (x, y, w, h);
      (g_id, vec4)
    })
    .collect::<HashMap<_, _>>();

  // --- renderFontAtlas ---

  // use rusttype
  let font = rusttype::Font::try_from_bytes(font_data.as_slice())
    .expect("error constructing a Font from bytes");

  // The font size to use
  let scale = rusttype::Scale::uniform(ATLAS_FONT_SIZE as f32);

  let positioned_glyphs = font
    .glyphs_for(chars)
    .scan(None, |last, gl| {
      let gl = gl.scaled(scale);
      let gl = gl.positioned(rusttype::point(0., 0.));
      let next = gl;
      *last = Some(next.id());
      Some(next)
    })
    .collect::<Vec<_>>();

  let mut bitmap = vec![0u8; (atlas_size * atlas_size) as usize];

  // println!("atlas_size: {}, bitmap.len(): {}", atlas_size, bitmap.len());

  for (i, glyph) in positioned_glyphs.iter().enumerate() {
    glyph.draw(|x, y, v| {
      let (at_x, at_y) = atlas_positions[i];
      let x = at_x as f32 + x as f32;
      let y = at_y as f32 + y as f32;
      let pos = (x as usize) + (y as usize) * atlas_size as usize;
      bitmap[pos] = (v * 255.0) as u8;
    });
  }

  let atlas_bitmap_file =
    std::fs::File::create(std::format!("./export/all-glyph-v{}.png", version))
      .unwrap();
  let ref mut w = std::io::BufWriter::new(atlas_bitmap_file);

  let mut encoder = png::Encoder::new(w, atlas_size as u32, atlas_size as u32);
  encoder.set_color(png::ColorType::Grayscale);

  let mut writer = encoder.write_header().unwrap();
  writer.write_image_data(&bitmap).unwrap();

  // --- toSDF ---

  let sdf = crate::sdf::to_sdf(
    &bitmap,
    atlas_size as usize,
    atlas_size as usize,
    ATLAS_RADIUS as f64,
  );

  let atlas_sdf_file =
    std::fs::File::create(std::format!("./export/atlas-svg-v{}.png", version))
      .unwrap();
  let ref mut w = std::io::BufWriter::new(atlas_sdf_file);

  let mut encoder = png::Encoder::new(w, atlas_size as u32, atlas_size as u32);
  encoder.set_color(png::ColorType::Grayscale);

  let mut writer = encoder.write_header().unwrap();
  writer.write_image_data(&sdf).unwrap();

  // --- getTextShape ---

  let font_size = 16;

  let cap_height = font_face.capital_height().unwrap_or(0);
  let padding = (ATLAS_GAP * font_size) / ATLAS_FONT_SIZE;

  let mut cursor_x = 0.;
  let char_rects = glyph_ids
    .iter()
    .map(|glyph_id| {
      let glyph = glyph_map
        .get(&glyph_id)
        .expect(std::format!("unknown glyph: {:?}", glyph_id).as_str());
      let Glyph {
        y,
        width,
        height,
        lsb,
        rsb,
        ..
      } = glyph;

      let pos_x = cursor_x as f32 + *lsb as f32 * scale_factor - padding as f32;
      let pos_y = (cap_height as f32 - *y as f32 - *height as f32)
        * scale_factor
        - padding as f32;
      let size_x = *width as f32 * scale_factor + padding as f32 * 2.;
      let size_y = *height as f32 * scale_factor + padding as f32 * 2.;

      cursor_x += (lsb + width + rsb) as f32 * scale_factor;

      (pos_x, pos_y, size_x, size_y)
    })
    .collect::<Vec<_>>();

  //println!("char_rects: {:?}", char_rects);

  let text_width = char_rects.last().map(|(x, _, w, _)| x + w).unwrap_or(0.);
  let text_height = (cap_height as f32 * font_size as f32) / ppem as f32;

  let text_width = text_width.ceil() as u16;
  let text_height = text_height.ceil() as u16;

  println!("text_width: {}, text_height: {}", text_width, text_height);

  // --- rendering ---

  let uvs = glyph_ids
    .iter()
    .map(|g_id| {
      let vec4 = uv_map
        .get(&g_id)
        .expect(std::format!("invalid g_id: {:?}", g_id).as_str());
      [vec4.0, vec4.1, vec4.2, vec4.3]
    })
    .collect::<Vec<_>>();

  let event_loop = EventLoop::builder().build()?;
  let mut app = Application::new(
    (atlas_size as u32, atlas_size as u32),
    sdf,
    char_rects,
    [0., 0.],
    [1., 0.5, 1., 1.],
    uvs,
  );

  event_loop.run_app(&mut app)?;

  Ok(())
}
