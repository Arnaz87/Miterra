
use cgmath::{Vector3, InnerSpace};
use voxel_source::VoxelSource;

pub trait Mesher {
  fn mesh (&mut self, source: &VoxelSource) ->
    (Vec<Vector3<f32>>, Vec<Vector3<f32>>, Vec<u16>);
}

pub fn calculate_normals (verts: &Vec<Vector3<f32>>, indices: &Vec<u16>) -> Vec<Vector3<f32>> {
  let len = verts.len();
  let mut normals = vec![Vector3::new(0.0, 0.0, 0.0); len];

  let high = indices.len()/3-1;

  for ii in 0 .. high {
    let i = ii*3;

    let aa = indices[i  ] as usize;
    let bb = indices[i+1] as usize;
    let cc = indices[i+2] as usize;

    let a = verts[aa];
    let b = verts[bb];
    let c = verts[cc];

    let n = (b-a).cross(c-a);

    normals[aa] += n;
    normals[bb] += n;
    normals[cc] += n;
  }

  for i in 0 .. len {
    normals[i] = normals[i].normalize();
  }
  normals
}
