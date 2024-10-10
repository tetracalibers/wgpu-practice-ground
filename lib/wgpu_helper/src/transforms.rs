use cgmath::*;
use std::f32::consts::PI;

// cgmath is built for OpenGL's coordinate system
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
  1.0, 0.0, 0.0, 0.0,
  0.0, 1.0, 0.0, 0.0,
  0.0, 0.0, 0.5, 0.0,
  0.0, 0.0, 0.5, 1.0,
);

pub fn create_model_mat(
  [translation_x, translation_y, translation_z]: [f32; 3],
  [rotation_x, rotation_y, rotation_z]: [f32; 3],
  [scaling_x, scaling_y, scaling_z]: [f32; 3],
) -> Matrix4<f32> {
  let trans_mat = Matrix4::from_translation(Vector3::new(
    translation_x,
    translation_y,
    translation_z,
  ));

  let rotate_mat_x = Matrix4::from_angle_x(Rad(rotation_x));
  let rotate_mat_y = Matrix4::from_angle_y(Rad(rotation_y));
  let rotate_mat_z = Matrix4::from_angle_z(Rad(rotation_z));

  let scale_mat =
    Matrix4::from_nonuniform_scale(scaling_x, scaling_y, scaling_z);

  let model_mat =
    trans_mat * rotate_mat_z * rotate_mat_y * rotate_mat_x * scale_mat;

  model_mat
}

pub fn create_initial_model_mat() -> Matrix4<f32> {
  create_model_mat([0., 0., 0.], [0., 0., 0.], [1., 1., 1.])
}

pub fn create_model_mat_with_rotation(rotation: [f32; 3]) -> Matrix4<f32> {
  create_model_mat([0., 0., 0.], rotation, [1., 1., 1.])
}

pub fn create_view_mat(
  camera_position: Point3<f32>,
  look_direction: Point3<f32>,
  up_direction: Vector3<f32>,
) -> Matrix4<f32> {
  Matrix4::look_at_rh(camera_position, look_direction, up_direction)
}

pub fn create_projection_mat(
  aspect: f32,
  is_perspective: bool,
) -> Matrix4<f32> {
  let project_mat = if is_perspective {
    create_perspective_mat(Rad(2. * PI / 5.), aspect, 0.1, 1000.)
  } else {
    create_ortho_mat(-4., 4., -3., 3., -1., 6.)
  };

  project_mat
}

pub fn create_perspective_mat(
  fovy: Rad<f32>,
  aspect: f32,
  near: f32,
  far: f32,
) -> Matrix4<f32> {
  OPENGL_TO_WGPU_MATRIX * perspective(fovy, aspect, near, far)
}

pub fn create_ortho_mat(
  left: f32,
  right: f32,
  bottom: f32,
  top: f32,
  near: f32,
  far: f32,
) -> Matrix4<f32> {
  OPENGL_TO_WGPU_MATRIX * ortho(left, right, bottom, top, near, far)
}

pub fn create_vp_ortho_mat(
  left: f32,
  right: f32,
  bottom: f32,
  top: f32,
  near: f32,
  far: f32,
  camera_position: Point3<f32>,
  look_direction: Point3<f32>,
  up_direction: Vector3<f32>,
) -> (Matrix4<f32>, Matrix4<f32>, Matrix4<f32>) {
  let view_mat = create_view_mat(camera_position, look_direction, up_direction);
  let project_mat = create_ortho_mat(left, right, bottom, top, near, far);

  // view-projection matrix
  let vp_mat = project_mat * view_mat;

  (view_mat, project_mat, vp_mat)
}

pub fn create_vp_mat(
  camera_position: Point3<f32>,
  look_direction: Point3<f32>,
  up_direction: Vector3<f32>,
  aspect: f32,
) -> (Matrix4<f32>, Matrix4<f32>, Matrix4<f32>) {
  let view_mat = create_view_mat(camera_position, look_direction, up_direction);
  let project_mat = create_projection_mat(aspect, true);

  let vp_mat = project_mat * view_mat;

  (view_mat, project_mat, vp_mat)
}
