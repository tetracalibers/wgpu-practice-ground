use bytemuck::{cast_slice, Pod, Zeroable};

use cgmath::{Matrix, SquareMatrix};
use rand::Rng;
use wgpu::util::DeviceExt;
use wgsim::geometry::generator as ge;
use wgsim::geometry::{Cube, Sphere, Torus};
use wgsim::matrix;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
  pub position: [f32; 3],
  pub normal: [f32; 3],
}

struct Geometry {
  pub vertices: Vec<Vertex>,
  pub indices: Vec<u16>,
}

fn cube_vertices() -> Geometry {
  let Cube {
    positions,
    normals,
    indices,
    ..
  } = ge::create_cube_data(2.0);

  let mut data: Vec<Vertex> = Vec::with_capacity(positions.len());
  for i in 0..positions.len() {
    data.push(Vertex {
      position: positions[i],
      normal: normals[i],
    });
  }

  Geometry {
    vertices: data,
    indices,
  }
}

fn sphere_vertices() -> Geometry {
  let Sphere {
    positions,
    normals,
    indices,
    ..
  } = ge::create_sphere_data(2.2, 20, 30);

  let mut data: Vec<Vertex> = Vec::with_capacity(positions.len());
  for i in 0..positions.len() {
    data.push(Vertex {
      position: positions[i],
      normal: normals[i],
    });
  }

  Geometry {
    vertices: data,
    indices,
  }
}

fn torus_vertices() -> Geometry {
  let Torus {
    positions,
    normals,
    indices,
    ..
  } = ge::create_torus_data(1.8, 0.4, 60, 20);

  let mut data: Vec<Vertex> = Vec::with_capacity(positions.len());
  for i in 0..positions.len() {
    data.push(Vertex {
      position: positions[i],
      normal: normals[i],
    });
  }

  Geometry {
    vertices: data,
    indices,
  }
}

pub struct Model {
  pub vertex_buffer: wgpu::Buffer,
  pub index_buffer: wgpu::Buffer,
  pub index_count: u32,
}

pub struct Shapes {
  pub cube: Model,
  pub sphere: Model,
  pub torus: Model,
}

pub fn create_object_buffers(device: &wgpu::Device) -> Shapes {
  let cube = cube_vertices();
  let sphere = sphere_vertices();
  let torus = torus_vertices();

  let cube_vertex_buffer =
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Cube Vertex Buffer"),
      contents: cast_slice(&cube.vertices),
      usage: wgpu::BufferUsages::VERTEX,
    });

  let cube_index_buffer =
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Cube Index Buffer"),
      contents: cast_slice(&cube.indices),
      usage: wgpu::BufferUsages::INDEX,
    });

  let sphere_vertex_buffer =
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("sphere Vertex Buffer"),
      contents: cast_slice(&sphere.vertices),
      usage: wgpu::BufferUsages::VERTEX,
    });

  let sphere_index_buffer =
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Sphere Index Buffer"),
      contents: cast_slice(&sphere.indices),
      usage: wgpu::BufferUsages::INDEX,
    });

  let torus_vertex_buffer =
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Torus Vertex Buffer"),
      contents: cast_slice(&torus.vertices),
      usage: wgpu::BufferUsages::VERTEX,
    });

  let torus_index_buffer =
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Torus Index Buffer"),
      contents: cast_slice(&torus.indices),
      usage: wgpu::BufferUsages::INDEX,
    });

  Shapes {
    cube: Model {
      vertex_buffer: cube_vertex_buffer,
      index_buffer: cube_index_buffer,
      index_count: cube.indices.len() as u32,
    },
    sphere: Model {
      vertex_buffer: sphere_vertex_buffer,
      index_buffer: sphere_index_buffer,
      index_count: sphere.indices.len() as u32,
    },
    torus: Model {
      vertex_buffer: torus_vertex_buffer,
      index_buffer: torus_index_buffer,
      index_count: torus.indices.len() as u32,
    },
  }
}

pub struct Matrices {
  pub model_mat: Vec<[f32; 16]>,
  pub normal_mat: Vec<[f32; 16]>,
  pub color_vec: Vec<[f32; 4]>,
}

pub fn create_transform_mat_color(
  objects_count: u32,
  translate_default: bool,
) -> Matrices {
  let mut model_mat: Vec<[f32; 16]> = vec![];
  let mut normal_mat: Vec<[f32; 16]> = vec![];
  let mut color_vec: Vec<[f32; 4]> = vec![];

  for _i in 0..objects_count {
    let mut rng = rand::thread_rng();
    let mut translation = [
      rng.gen::<f32>() * 60.0 - 53.0,
      rng.gen::<f32>() * 50.0 - 45.0,
      -15.0 - rng.gen::<f32>() * 50.0,
    ];
    if !translate_default {
      translation = [
        rng.gen::<f32>() * 50.0 - 25.0,
        rng.gen::<f32>() * 40.0 - 18.0,
        -30.0 - rng.gen::<f32>() * 50.0,
      ];
    }
    let rotation = [rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>()];
    let scale = [1.0, 1.0, 1.0];
    let m = matrix::create_model_mat(translation, rotation, scale);
    let n = (m.invert().unwrap()).transpose();
    let color = [rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>(), 1.0];
    model_mat.push(*(m.as_ref()));
    normal_mat.push(*(n.as_ref()));
    color_vec.push(color);
  }

  Matrices {
    model_mat,
    normal_mat,
    color_vec,
  }
}
