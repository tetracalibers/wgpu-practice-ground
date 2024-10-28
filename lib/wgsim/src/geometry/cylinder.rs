use cgmath::*;

pub struct Cylinder {
  pub positions: Vec<[f32; 3]>,
  pub indices: Vec<u16>,
  pub indices_wireframe: Vec<u16>,
}

fn cylinder_position(r: f32, theta: Deg<f32>, y: f32) -> [f32; 3] {
  let x = r * theta.cos();
  let z = -r * theta.sin();
  [x, y, z]
}

pub fn create_cylinder_data(
  mut rin: f32,
  rout: f32,
  h: f32,
  n: u16,
) -> Cylinder {
  rin = rin.min(0.999 * rout);

  let mut positions: Vec<[f32; 3]> = vec![];

  for i in 0..=n {
    let theta = i as f32 * 360. / n as f32;
    let p0 = cylinder_position(rout, Deg(theta), h / 2.);
    let p1 = cylinder_position(rout, Deg(theta), -h / 2.);
    let p2 = cylinder_position(rin, Deg(theta), -h / 2.);
    let p3 = cylinder_position(rin, Deg(theta), h / 2.);
    positions.extend([p0, p1, p2, p3]);
  }

  let mut indices: Vec<u16> = vec![];
  let mut indices_wireframe: Vec<u16> = vec![];

  for i in 0..n {
    let idx = (0..=7).map(|j| i * 4 + j).collect::<Vec<_>>();

    let triangle = vec![
      idx[0], idx[4], idx[7], idx[7], idx[3], idx[0], // top
      idx[1], idx[2], idx[6], idx[6], idx[5], idx[1], // bottom
      idx[0], idx[1], idx[5], idx[5], idx[4], idx[0], // outer
      idx[2], idx[3], idx[7], idx[7], idx[6], idx[2], // inner
    ];
    let wireframe = vec![
      idx[0], idx[3], idx[3], idx[7], idx[4], idx[0], // top
      idx[1], idx[2], idx[2], idx[6], idx[5], idx[1], // bottom
      idx[0], idx[1], idx[3], idx[2], // side
    ];

    indices.extend(triangle);
    indices_wireframe.extend(wireframe);
  }

  Cylinder {
    positions,
    indices,
    indices_wireframe,
  }
}
