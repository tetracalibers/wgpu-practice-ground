use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
  let args: Vec<String> = env::args().collect();

  if args.len() < 2 {
    eprintln!("Usage: {} <workspace_member>", args[0]);
    return Ok(());
  }

  let target = &args[1];

  match target.as_str() {
    "tutorial/ch01-window" => ch01_window::run(),
    "tutorial/ch02-surface" => ch02_surface::run(),
    "tutorial/ch03-pipeline" => ch03_pipeline::run(),
    "tutorial/ch04-buffer" => ch04_buffer::run(),
    "tutorial/ch05-indice" => ch05_indice::run(),
    "tutorial/ch06-texture" => ch06_texture::run(),
    "tutorial/life-game" => life_game::run(),
    "export-gif:tutorial/life-game" => {
      life_game::export_gif();
      Ok(())
    }
    "tutorial/hello-glyphon" => hello_glyphon::run(),
    "prototype/rect-renderer" => rect_renderer::run(),
    "prototype/text-renderer" => text_renderer::proto(),
    "practice/cube_blinn_phong" => {
      Ok(cube_blinn_phong::run("cube_blinn_phong")?)
    }
    _ => {
      eprintln!("Not found: {}", target);
      Ok(())
    }
  }
}
