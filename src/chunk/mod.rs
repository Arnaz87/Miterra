
use voxel_source::VoxelSource;
use mesher::Mesher;
use base;
use base::{FactoryExt, Base, Texture};

struct Data {
    vbuf: base::VertexBuffer,
    slice: base::Slice,
}

pub struct Chunk {
  pub x: i32,
  pub y: i32,
  pub z: i32,
  pub r: i32, // resolution
  data: Option<Data>
}

pub struct ChunkSource<'a> {
  orig: &'a VoxelSource,
  chunk: &'a Chunk,
}

impl<'a> VoxelSource for ChunkSource<'a> {
  fn get(&self, x: i32, y: i32, z: i32) -> bool {
    let c = &self.chunk;
    // Voxels are half a meter big
    self.orig.get(
        x * c.r + c.x*2,
        y * c.r + c.y*2,
        z * c.r + c.z*2
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
  grass_texture: Texture,
  soilsand_texture: Texture,
  sampler: base::Sampler,
}

impl ChunkManager {
    pub fn new <S: VoxelSource + 'static, M: Mesher + 'static> (s: S, m: M, base: &mut Base) -> Self {
        use ::gfx::Factory;
        let sampler = base.factory.create_sampler(
            ::gfx::texture::SamplerInfo::new(
                ::gfx::texture::FilterMethod::Trilinear, // Trilinear
                ::gfx::texture::WrapMode::Tile
            )
        );
        ChunkManager{
            chunks: vec![],
            source: Box::new(s),
            mesher: Box::new(m),
            modified: false,
            grass_texture: base.load_texture("assets/grass.jpg"),
            soilsand_texture: base.load_texture("assets/soilsand.jpg"),
            sampler: sampler,
        }
    }

    pub fn generate (&mut self, x: i32, y: i32, z: i32, r: i32) {
        for chunk in self.chunks.iter() {
            if chunk.x == x && chunk.y == y && chunk.z == z { return; }
        }
        self.chunks.push(Chunk{ x: x, y: y, z: z, r: r, data: None });
        self.modified = true;
    }

    pub fn set_mesher <M: Mesher + 'static> (&mut self, m: M) {
        self.mesher = Box::new(m);
        for chunk in self.chunks.iter_mut() {
            chunk.data = None;
        }
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

                // Voxels are half a meter big
                mesh.scale(chunk.r as f32 * 0.5);

                mesh.translate(::mesh::Vector3::new(
                    chunk.x as f32,
                    chunk.y as f32,
                    chunk.z as f32
                ));

                let vertices: Vec<base::Vertex> = mesh.vertices.iter().map( |vertex| {
                    use cgmath::{Vector2, InnerSpace};
                    let v2 = Vector2::new(vertex.pos.x, vertex.pos.z);
                    let i = v2.magnitude() as i32 / 4;
                    let mat = i%2;
                    base::Vertex {
                        pos: *vertex.pos.as_ref(),
                        normal: *vertex.normal.as_ref(),
                        material: mat,
                    }
                }).collect();
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
        let &ChunkManager {
            ref chunks, ref grass_texture, ref soilsand_texture, ref sampler, ..
        } = self;

        for chunk in self.chunks.iter() {
            match chunk.data {
                Some(Data{ref vbuf, ref slice}) => {
                    let &mut Base {
                        ref mut encoder, ref mut terrain_pso, ..
                    } = base;

                    let data = base::terrain_pipe::Data {
                        vbuf: vbuf.clone(),
                        world: base.world_buffer.clone(),
                        out_color: base.out_color.clone(),
                        out_depth: base.out_depth.clone(),
                        grass: (grass_texture.clone(), sampler.clone()),
                        soilsand: (soilsand_texture.clone(), sampler.clone()),
                    };

                    encoder.draw(&slice, &terrain_pso, &data);
                },
                None => {}
            }
        }

    }
}

