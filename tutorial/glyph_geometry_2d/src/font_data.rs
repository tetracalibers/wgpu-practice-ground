use std::{collections::HashMap, fs};

use bytemuck::{Pod, Zeroable};
use cgmath::{InnerSpace, Vector3};
use meshtext::{IndexedMeshText, MeshGenerator, MeshText, TextSection};

pub fn font_file_map(font_selection: u32) -> Option<String> {
  let mut d: HashMap<u32, String> = HashMap::new();
  d.insert(0, String::from("./font/Lusitana/Lusitana-Regular.ttf"));
  d.insert(
    1,
    String::from("./font/Emilys_Candy/EmilysCandy-Regular.ttf"),
  );
  d.insert(
    2,
    String::from("./font/Rubik_Puddles/RubikPuddles-Regular.ttf"),
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
  let mut scale1 = 0.22 * scale;
  let font_data = fs::read(font_file_map(font_selection).unwrap()).unwrap();
  if font_selection == 1 {
    scale1 *= 1.5;
  } else if font_selection == 2 {
    scale1 *= 2.0;
  }
  let font_data_static = Box::leak(font_data.into_boxed_slice());

  let mut generator = MeshGenerator::new(font_data_static);
  let transform = [
    scale1,
    0.0,
    0.0,
    0.0,
    scale1 * aspect,
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

fn get_text_positions(font_selection: u32, text: &str) -> Vec<[f32; 3]> {
  let mut scale = 1.0;
  let font_data = fs::read(font_file_map(font_selection).unwrap()).unwrap();
  if font_selection == 1 {
    scale *= 1.5;
  } else if font_selection == 2 {
    scale *= 2.0;
  }
  let font_data_static = Box::leak(font_data.into_boxed_slice());

  let mut generator = MeshGenerator::new(font_data_static);
  let transform = [
    scale,
    0.0,
    0.0,
    0.0,
    0.0,
    scale,
    0.0,
    0.0,
    0.0,
    0.0,
    0.1 * scale,
    0.0,
    0.0,
    0.0,
    0.0,
    1.0,
  ];
  let data: MeshText = generator
    .generate_section(text, false, Some(&transform))
    .expect("failed to generate glyph.");
  let vertices = data.vertices;
  let positions: Vec<[f32; 3]> = vertices
    .chunks(3)
    .map(|c| [c[0] - 0.5 * data.bbox.size().x, c[1], c[2]])
    .collect();

  positions
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
  pub position: [f32; 3],
  pub normal: [f32; 3],
}

impl Vertex {
  pub fn new(position: [f32; 3], normal: [f32; 3]) -> Self {
    Self { position, normal }
  }
}

pub fn create_text_vertices(font_selection: u32, text: &str) -> Vec<Vertex> {
  let pos = get_text_positions(font_selection, text);
  let mut vertices: Vec<Vertex> = vec![];

  for chunk in pos.chunks_exact(3) {
    let p1 = Vector3::from(chunk[0]);
    let p2 = Vector3::from(chunk[1]);
    let p3 = Vector3::from(chunk[2]);
    let normal = (p2 - p1).cross(p3 - p1).normalize();

    vertices.push(Vertex::new(p1.into(), normal.into()));
    vertices.push(Vertex::new(p2.into(), normal.into()));
    vertices.push(Vertex::new(p3.into(), normal.into()));
  }
  vertices
}
