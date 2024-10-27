pub mod cube;
pub mod cylinder;
pub mod sphere;
pub mod torus;

pub mod prelude {
  pub use super::cube::*;
  pub use super::cylinder::*;
  pub use super::sphere::*;
  pub use super::torus::*;
}
