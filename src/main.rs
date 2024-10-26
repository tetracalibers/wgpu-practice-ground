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
    "tutorial/compute_single_thread" => {
      Ok(pollster::block_on(compute_single_thread::run())?)
    }
    "tutorial/compute_atomic_add" => {
      Ok(pollster::block_on(compute_atomic_add::run())?)
    }
    "prototype/rect-renderer" => rect_renderer::run(),
    "prototype/text-renderer" => text_renderer::proto(),
    "prototype/with_gif" => Ok(with_gif::run("with_gif")?),
    "export/with_gif" => Ok(pollster::block_on(with_gif::export_gif())?),
    "with_gif/life_game" => Ok(with_gif_life_game::run()?),
    "export:gif/life_game" => {
      Ok(pollster::block_on(with_gif_life_game::export_gif())?)
    }
    "practice/cube_blinn_phong" => {
      Ok(cube_blinn_phong::run("cube_blinn_phong")?)
    }
    "practice/rotate_cube_basic" => {
      Ok(rotate_cube_basic::run("rotate_cube_basic")?)
    }
    _ => {
      eprintln!("Not found: {}", target);
      Ok(())
    }
  }
}
