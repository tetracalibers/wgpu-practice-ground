use cgmath::*;

pub struct Sphere {
  pub positions: Vec<[f32; 3]>,
  pub normals: Vec<[f32; 3]>,
  pub uvs: Vec<[f32; 2]>,
  pub indices: Vec<u16>,
  pub indices_wireframe: Vec<u16>,
}

fn sphere_position(r: f32, theta: Deg<f32>, phi: Deg<f32>) -> [f32; 3] {
  let x = r * theta.sin() * phi.cos();
  let y = r * theta.cos();
  let z = -r * theta.sin() * phi.sin();
  [x, y, z]
}

/// `u` segments and `v` rings
pub fn create_sphere_data(r: f32, u: u16, v: u16) -> Sphere {
  let mut positions: Vec<[f32; 3]> = vec![];
  let mut normals: Vec<[f32; 3]> = vec![];
  let mut uvs: Vec<[f32; 2]> = vec![];

  for i in 0..=u {
    for j in 0..=v {
      let theta = i as f32 * 180. / u as f32;
      let phi = j as f32 * 360. / v as f32;
      let pos = sphere_position(r, Deg(theta), Deg(phi));

      positions.push(pos);
      normals.push(pos.map(|x| x / r));
      uvs.push([i as f32 / u as f32, j as f32 / v as f32]);
    }
  }

  let mut indices: Vec<u16> = vec![];
  let mut indices_wireframe: Vec<u16> = vec![];

  for i in 0..u {
    for j in 0..v {
      let idx0 = j + i * (v + 1);
      let idx1 = j + 1 + i * (v + 1);
      let idx2 = j + 1 + (i + 1) * (v + 1);
      let idx3 = j + (i + 1) * (v + 1);

      let shape = vec![idx0, idx1, idx2, idx2, idx3, idx0];
      let wireframe = vec![idx0, idx1, idx0, idx3];

      indices.extend(shape);
      indices_wireframe.extend(wireframe);
    }
  }

  Sphere {
    positions,
    normals,
    uvs,
    indices,
    indices_wireframe,
  }
}
