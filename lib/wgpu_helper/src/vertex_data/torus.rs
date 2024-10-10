use cgmath::*;

pub struct Torus {
  pub positions: Vec<[f32; 3]>,
  pub normals: Vec<[f32; 3]>,
  pub indices: Vec<u16>,
  pub indices_wireframe: Vec<u16>,
}

fn torus_position(
  r_torus: f32,
  r_tube: f32,
  u: Deg<f32>,
  v: Deg<f32>,
) -> [f32; 3] {
  let x = (r_torus + r_tube * v.cos()) * u.cos();
  let y = r_tube * v.sin();
  let z = -(r_torus + r_tube * v.cos()) * u.sin();
  [x, y, z]
}

pub fn create_torus_data(
  r_torus: f32,
  r_tube: f32,
  n_torus: u16,
  n_tube: u16,
) -> Torus {
  let mut positions: Vec<[f32; 3]> = vec![];
  let mut normals: Vec<[f32; 3]> = vec![];

  let eps = 0.01 * 360. / n_tube as f32;

  for i in 0..=n_torus {
    let du = i as f32 * 360. / n_torus as f32;
    for j in 0..=n_tube {
      let dv = j as f32 * 360. / n_tube as f32;

      let pos = torus_position(r_torus, r_tube, Deg(du), Deg(dv));
      positions.push(pos);

      let d0 = torus_position(r_torus, r_tube, Deg(du + eps), Deg(dv));
      let d1 = torus_position(r_torus, r_tube, Deg(du - eps), Deg(dv));
      let d2 = torus_position(r_torus, r_tube, Deg(du), Deg(dv + eps));
      let d3 = torus_position(r_torus, r_tube, Deg(du), Deg(dv - eps));

      let nu = Vector3::from(d0) - Vector3::from(d1);
      let nv = Vector3::from(d2) - Vector3::from(d3);

      let normal = nu.cross(nv).normalize();
      normals.push(normal.into());
    }
  }

  let mut indices: Vec<u16> = vec![];
  let mut indices_wireframe: Vec<u16> = vec![];

  let vertices_per_row = n_tube + 1;

  for i in 0..n_torus {
    for j in 0..n_tube {
      let idx0 = j + i * vertices_per_row;
      let idx1 = j + 1 + i * vertices_per_row;
      let idx2 = j + 1 + (i + 1) * vertices_per_row;
      let idx3 = j + (i + 1) * vertices_per_row;

      let shape = vec![idx0, idx1, idx2, idx2, idx3, idx0];
      let wireframe = vec![idx0, idx1, idx0, idx3];

      indices.extend(shape);
      indices_wireframe.extend(wireframe);
    }
  }

  Torus {
    positions,
    normals,
    indices,
    indices_wireframe,
  }
}
