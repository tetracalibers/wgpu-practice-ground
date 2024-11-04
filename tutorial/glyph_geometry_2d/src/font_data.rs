use std::{collections::HashMap, fs};

use meshtext::{IndexedMeshText, MeshGenerator, TextSection};

pub fn font_file_map(font_selection: u32) -> Option<String> {
  let mut d: HashMap<u32, String> = HashMap::new();
  d.insert(0, String::from("./font/Lusitana/Lusitana-Regular.ttf"));
  d.insert(
    1,
    String::from("./font/Emilys_Candy/EmilysCandy-Regular.ttf"),
  );
  d.get(&font_selection).cloned()
}

pub struct TextVertices2d {
  pub vertices: Vec<u8>,
  pub indices: Vec<u8>,
  pub indices_len: u32,
}

pub fn get_text_vertices_2d(
  font_selection: u32,
  text: &str,
  pos: [f32; 2],
  scale: f32,
  aspect: f32,
) -> TextVertices2d {
  let font_data = fs::read(font_file_map(font_selection).unwrap()).unwrap();
  let font_data_static = Box::leak(font_data.into_boxed_slice());

  let mut generator = MeshGenerator::new(font_data_static);
  let transform = [
    scale,
    0.0,
    0.0,
    0.0,
    scale * aspect,
    0.0,
    pos[0],
    pos[1],
    1.0,
  ];
  let data: IndexedMeshText = generator
    .generate_section_2d(text, Some(&transform))
    .expect("failed to generate glyph.");

  let mut vertex_data: Vec<u8> = Vec::new();
  for vert in data.vertices.iter() {
    vertex_data.extend_from_slice(vert.to_le_bytes().as_slice());
  }

  let mut index_data: Vec<u8> = Vec::new();
  for ind in data.indices.iter() {
    index_data.extend_from_slice(ind.to_le_bytes().as_slice());
  }

  TextVertices2d {
    vertices: vertex_data,
    indices: index_data,
    indices_len: data.indices.len() as u32,
  }
}
