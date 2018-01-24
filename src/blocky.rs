
use cgmath::{Vector3};
use mesher::{Mesher};
use voxel_source::VoxelSource;

pub struct Blocky { pub size: i32 }

pub struct Builder<'a> {
    size: i32,
    source: &'a VoxelSource,
    vertices: Vec<Vector3<f32>>,
    normals: Vec<Vector3<f32>>,
    indices: Vec<u16>,
}

impl<'a> Builder<'a> {
    fn new (source: &'a VoxelSource, size: i32) -> Self {
        Builder {
            size: size,
            source: source,
            vertices: vec![],
            normals: vec![],
            indices: vec![],
        }
    }

    fn face (&mut self, x: i32, y: i32, z: i32, axis: u8, reverse: bool) {
        let index = self.vertices.len() as u16;
        let offs = match axis {
            0 => [[0, 0, 0],
                  [0, 0, 1],
                  [0, 1, 0],
                  [0, 1, 1]],
            1 => [[0, 0, 0],
                  [0, 0, 1],
                  [1, 0, 0],
                  [1, 0, 1]],
            2 => [[0, 0, 0],
                  [1, 0, 0],
                  [0, 1, 0],
                  [1, 1, 0]],
            _ => unreachable!()
        };

        let axoff = match (axis, reverse) {
            (0, false) => [1, 0, 0],
            (0, true) => [-1, 0, 0],
            (1, false) => [0, 1, 0],
            (1, true) => [0, -1, 0],
            (2, false) => [0, 0, 1],
            (2, true) => [0, 0, -1],
            _ => unreachable!()
        };

        for i in 0..4 {
            let off = offs[i];
            self.vertices.push(Vector3::new(
                (x + off[0]) as f32,
                (y + off[1]) as f32,
                (z + off[2]) as f32
            ));
            self.normals.push(Vector3::new(
                axoff[0] as f32,
                axoff[1] as f32,
                axoff[2] as f32
            ));
        }

        let order = if reverse {[2,1,0, 3,1,2]} else {[0,1,2, 2,1,3]};
        for i in 0..6 { self.indices.push(index + order[i]); }
    }

    fn cube (&mut self, x: i32, y: i32, z: i32) {
        // Cancel empty voxels
        if !self.source.get(x, y, z) { return; }

        if !self.source.get(x+1, y, z) { self.face(x+1, y, z, 0, true); }
        if !self.source.get(x-1, y, z) { self.face(x  , y, z, 0, false); }

        if !self.source.get(x, y+1, z) { self.face(x, y+1, z, 1, false); }
        if !self.source.get(x, y-1, z) { self.face(x, y  , z, 1, true); }

        if !self.source.get(x, y, z+1) { self.face(x, y, z+1, 2, false); }
        if !self.source.get(x, y, z-1) { self.face(x, y, z  , 2, true); }
    }

    fn mesh (&mut self) {
        for x in 0 .. self.size {
            for y in 0 .. self.size {
                for z in 0 .. self.size {
                    self.cube(x, y, z);
                }
            }
        }
    }
}

impl Mesher for Blocky {
    fn mesh (&mut self, source: &VoxelSource) ->
        (Vec<Vector3<f32>>, Vec<Vector3<f32>>, Vec<u16>) {

        let mut builder = Builder::new(source, self.size);
        builder.mesh();

        (builder.vertices, builder.normals, builder.indices)
    }
}
