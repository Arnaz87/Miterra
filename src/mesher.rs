
use voxel_source::VoxelSource;
use mesh::Mesh;

pub trait Mesher {
  fn mesh (&mut self, source: &VoxelSource) -> Mesh;
}

pub fn calculate_normals (mesh: &mut Mesh) {
  use mesh::{Vector3, InnerSpace};

  let len = mesh.vertices.len();

  let high = mesh.indices.len()/3-1;

  // Reset all normals
  for vertex in mesh.vertices.iter_mut() {
    vertex.normal = Vector3::new(0.0, 0.0, 0.0);
  }

  for ii in 0 .. high {
    let i = ii*3;

    let aa = mesh.indices[i  ] as usize;
    let bb = mesh.indices[i+1] as usize;
    let cc = mesh.indices[i+2] as usize;

    let a = mesh.vertices[aa].pos;
    let b = mesh.vertices[bb].pos;
    let c = mesh.vertices[cc].pos;

    // To get a weighted sum, do not normalize this
    let n = (b-a).cross(c-a);

    mesh.vertices[aa].normal += n;
    mesh.vertices[bb].normal += n;
    mesh.vertices[cc].normal += n;
  }

  // Normalize all normals
  for vertex in mesh.vertices.iter_mut() {
    vertex.normal = vertex.normal.normalize();
  }
}
