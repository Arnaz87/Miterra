
pub type Vector3 = ::cgmath::Vector3<f32>;

// This trait has the normalize method
pub use cgmath::InnerSpace;

pub struct Vertex {
  pub pos: Vector3,
  pub normal: Vector3,
}

impl Vertex {
  pub fn new () -> Self {
    Vertex {
      pos: Vector3::new(0.0, 0.0, 0.0),
      normal: Vector3::new(0.0, 0.0, 0.0)
    }
  }

  pub fn from_pos (pos: Vector3) -> Self {
    Vertex { pos: pos, normal: Vector3::new(0.0, 0.0, 0.0) }
  }
}

pub struct Mesh {
  pub vertices: Vec<Vertex>,
  pub indices: Vec<u16>,
}

impl Mesh {
  pub fn new() -> Self { Mesh { vertices: vec![], indices: vec![] } }

  pub fn translate(&mut self, p: Vector3) {
    for vertex in self.vertices.iter_mut() {
      let old = vertex.pos;
      vertex.pos = old + p;
    }
  }
}
