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
    "tutorial/glyph_geometry_2d" => Ok(glyph_geometry_2d::run()?),
    "tutorial/compute_single_thread" => {
      Ok(pollster::block_on(compute_single_thread::run())?)
    }
    "tutorial/compute_atomic_add" => {
      Ok(pollster::block_on(compute_atomic_add::run())?)
    }
    "tutorial/compute_visualize_workgroup_global" => Ok(pollster::block_on(
      compute_visualize_workgroup_global::run(),
    )?),
    "tutorial/compute_visualize_workgroup_local" => {
      Ok(pollster::block_on(compute_visualize_workgroup_local::run())?)
    }
    "tutorial/compute_mandelbrot_set" => {
      Ok(pollster::block_on(compute_mandelbrot_set::run())?)
    }
    "shader_art/shader_step_rect" => {
      Ok(pollster::block_on(shader_step_rect::run())?)
    }
    "image_processing/image_blur" => Ok(image_blur::run()?),
    "image_processing/image_average_filter" => Ok(image_average_filter::run()?),
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
    "instanced_cube_sphere_torus/base" => {
      Ok(instanced_cube_sphere_torus_base::run()?)
    }
    "instanced_cube_sphere_torus/direction_light_1" => {
      Ok(instanced_cube_sphere_torus_direction_light_1::run()?)
    }
    "export-gif:instanced_cube_sphere_torus/direction_light_1" => {
      Ok(pollster::block_on(
        instanced_cube_sphere_torus_direction_light_1::export_gif(),
      )?)
    }
    "instanced_cube_sphere_torus/direction_light_2" => {
      Ok(instanced_cube_sphere_torus_direction_light_2::run()?)
    }
    "export-gif:instanced_cube_sphere_torus/direction_light_2" => {
      Ok(pollster::block_on(
        instanced_cube_sphere_torus_direction_light_2::export_gif(),
      )?)
    }
    "examples/empty_window" => Ok(empty_window::run()?),
    _ => {
      eprintln!("Not found: {}", target);
      Ok(())
    }
  }
}
