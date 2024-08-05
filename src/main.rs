use std::env;

fn main() {
  let args: Vec<String> = env::args().collect();

  if args.len() < 2 {
    eprintln!("Usage: {} <workspace_member>", args[0]);
    return;
  }

  let target = &args[1];

  match target.as_str() {
    "tutorial/ch01-window" => {
      ch01_window::run().unwrap();
    }
    "tutorial/ch02-surface" => {
      ch02_surface::run().unwrap();
    }
    "tutorial/ch03-pipeline" => {
      ch03_pipeline::run().unwrap();
    }
    "tutorial/ch04-buffer" => {
      ch04_buffer::run().unwrap();
    }
    "tutorial/ch05-indice" => {
      ch05_indice::run().unwrap();
    }
    "tutorial/ch06-texture" => {
      ch06_texture::run().unwrap();
    }
    "tutorial/life-game" => {
      life_game::run().unwrap();
    }
    "export-gif:tutorial/life-game" => {
      life_game::export_gif();
    }
    "tutorial/hello-glyphon" => {
      hello_glyphon::run().unwrap();
    }
    "prototype/rect-renderer" => {
      rect_renderer::run().unwrap();
    }
    _ => {
      eprintln!("Not found: {}", target);
    }
  }
}
