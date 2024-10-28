mod cube;
mod cylinder;
mod sphere;
mod torus;

pub use cube::Cube;
pub use cylinder::Cylinder;
pub use sphere::Sphere;
pub use torus::Torus;

pub mod generator {
  pub use super::cube::create_cube_data;
  pub use super::cylinder::create_cylinder_data;
  pub use super::sphere::create_sphere_data;
  pub use super::torus::create_torus_data;
}
