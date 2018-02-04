
use voxel_source::VoxelSource;
use mesher::Mesher;
use base;
use base::{FactoryExt, Base};

struct Data {
    vbuf: base::VertexBuffer,
    slice: base::Slice,
}

pub struct Chunk {
  pub x: i32,
  pub y: i32,
  pub z: i32,
  data: Option<Data>
}

pub struct ChunkSource<'a> {
  orig: &'a VoxelSource,
  chunk: &'a Chunk,
}

impl<'a> VoxelSource for ChunkSource<'a> {
  fn get(&self, x: i32, y: i32, z: i32) -> bool {
    self.orig.get(
        x + self.chunk.x,
        y + self.chunk.y,
        z + self.chunk.z
    )
  }
}

// In the future, use this for infinite voxels
//use std::collections::BTreeMap;

pub struct ChunkManager {
  chunks: Vec<Chunk>,
  source: Box<VoxelSource>,
  mesher: Box<Mesher>,
  modified: bool,
}

impl ChunkManager {
    pub fn new <S: VoxelSource + 'static, M: Mesher + 'static> (s: S, m: M) -> Self {
        ChunkManager{
            chunks: vec![],
            source: Box::new(s),
            mesher: Box::new(m),
            modified: false,
        }
    }

    pub fn generate (&mut self, x: i32, y: i32, z: i32) {
        for chunk in self.chunks.iter() {
            if chunk.x == x && chunk.y == y && chunk.z == z { return; }
        }
        self.chunks.push(Chunk{ x: x, y: y, z: z, data: None });
        self.modified = true;
    }

    pub fn set_mesher <M: Mesher + 'static> (&mut self, m: M) {
        self.mesher = Box::new(m);
        self.modified = true;
    }

    pub fn update (&mut self, base: &mut Base) {
        if !self.modified { return; }

        match self { &mut ChunkManager {
            ref mut source, ref mut mesher, ref mut chunks, ..
        } => {
            for chunk in chunks.iter_mut() {
                if chunk.data.is_some() { continue; }

                let mut mesh = mesher.mesh(&ChunkSource {
                    orig: source.as_ref(), chunk: chunk
                });

                mesh.translate(::mesh::Vector3::new(
                    chunk.x as f32,
                    chunk.y as f32,
                    chunk.z as f32
                ));

                let vertices: Vec<base::Vertex> = mesh.vertices.iter().map( |vertex|
                    base::Vertex {
                        pos: *vertex.pos.as_ref(),
                        color: [1.0, 1.0, 1.0],
                        normal: *vertex.normal.as_ref()
                    }
                ).collect();
                let indices: &[u16] = mesh.indices.as_ref();

                let (vbuf, slice) = base.factory.create_vertex_buffer_with_slice(
                    &vertices, indices
                );

                chunk.data = Some(Data{vbuf: vbuf, slice: slice});
            }
        } }

        self.modified = false;
    }

    pub fn render (&self, base: &mut Base) {

        for chunk in self.chunks.iter() {
            match chunk.data {
                Some(Data{ref vbuf, ref slice}) => {
                    let data = base::terrain_pipe::Data {
                        vbuf: vbuf.clone(),
                        world: base.world_buffer.clone(),
                        out_color: base.out_color.clone(),
                        out_depth: base.out_depth.clone(),
                    };
                    match base {
                        &mut Base { ref mut encoder, ref mut terrain_pso, .. } => {
                            encoder.draw(&slice, &terrain_pso, &data);
                        }
                    }
                },
                None => {}
            }
        }

    }
}

