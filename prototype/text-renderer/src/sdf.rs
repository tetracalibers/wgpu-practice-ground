const INF: f64 = 1e20;

pub fn to_sdf(
  image_data: &Vec<u8>,
  width: usize,
  height: usize,
  radius: f64,
) -> Vec<u8> {
  let mut grid_outer = vec![0.0; width * height];
  let mut grid_inner = vec![0.0; width * height];
  let s = std::cmp::max(width, height);
  let mut f = vec![0.0; s];
  let mut z = vec![0.0; s + 1];
  let mut v = vec![0; s * 2];

  for i in 0..width * height {
    let a = image_data[i] as f64 / 255.0; // Alpha value.
    grid_outer[i] = if a == 1.0 {
      0.0
    } else if a == 0.0 {
      INF
    } else {
      (0.5 - a).max(0.0).powi(2)
    };
    grid_inner[i] = if a == 1.0 {
      INF
    } else if a == 0.0 {
      0.0
    } else {
      (a - 0.5).max(0.0).powi(2)
    };
  }

  edt(&mut grid_outer, width, height, &mut f, &mut v, &mut z);
  edt(&mut grid_inner, width, height, &mut f, &mut v, &mut z);

  let mut alpha_channel = vec![0u8; width * height];
  for i in 0..width * height {
    let d = grid_outer[i].sqrt() - grid_inner[i].sqrt();
    let buffer = 0.5;
    let value = buffer - d / radius;

    alpha_channel[i] = (value * 255.0).clamp(0.0, 255.0) as u8;
  }

  let mut data = vec![0u8; width * height];
  for i in 0..width * height {
    data[i] = alpha_channel[i];
  }

  data
}

fn edt(
  data: &mut [f64],
  width: usize,
  height: usize,
  f: &mut [f64],
  v: &mut [usize],
  z: &mut [f64],
) {
  for x in 0..width {
    edt1d(data, x, width, height, f, v, z);
  }
  for y in 0..height {
    edt1d(data, y * width, 1, width, f, v, z);
  }
}

fn edt1d(
  grid: &mut [f64],
  offset: usize,
  stride: usize,
  length: usize,
  f: &mut [f64],
  v: &mut [usize],
  z: &mut [f64],
) {
  v[0] = 0;
  z[0] = -INF;
  z[1] = INF;

  for q in 0..length {
    f[q] = grid[offset + q * stride];
  }

  let mut k = 0;
  for q in 1..length {
    loop {
      let r = v[k];
      let s = (f[q] - f[r] + (q as f64).powi(2) - (r as f64).powi(2))
        / (2. * (q as f64 - r as f64));
      if s <= z[k] {
        if k == 0 {
          break; // kが0の場合はこれ以上減らせないのでループを抜ける
        }
        k -= 1;
      } else {
        k += 1;
        v[k] = q;
        z[k] = s;
        z[k + 1] = INF;
        break;
      }
    }
  }

  k = 0;
  for q in 0..length {
    while z[k + 1] < q as f64 {
      k += 1;
    }
    let r = v[k];
    grid[offset + q * stride] = f[r] + (q as f64 - r as f64).powi(2);
  }
}
