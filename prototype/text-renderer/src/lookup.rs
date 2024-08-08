use std::collections::HashMap;

pub struct Lookups {
  uvs: HashMap<usize, [f32; 4]>,
  atlas_width: usize,
  atlas_height: usize,
  atlas_positions: Vec<[f32; 2]>,
  atlas_sizes: Vec<[f32; 2]>,
}

impl Lookups {
  pub fn new(atlas_width: usize, atlas_height: usize) -> Self {
    Lookups {
      uvs: HashMap::new(),
      atlas_width,
      atlas_height,
      atlas_positions: Vec::new(),
      atlas_sizes: Vec::new(),
    }
  }
}
