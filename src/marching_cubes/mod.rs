
mod data;

use mesher::{Mesher, calculate_normals};
use voxel_source::VoxelSource;
use cgmath::{Vector3};

use self::data::*;

pub struct MarchingCubes { pub size: i32 }

pub struct Builder<'a> {
    size: i32,
    source: &'a VoxelSource,
    voxels: Vec<f32>,
    vertices: Vec<Vector3<f32>>,
    indices: Vec<u16>,
}

fn get_offset (v1: f32, v2: f32) -> f32 {
    let delta = v2 - v1;
    if delta == 0.0 {0.5} else {(TARGET - v1) / delta}
}

impl<'a> Builder<'a> {
    fn new (source: &'a VoxelSource, size: i32) -> Self {
        Builder {
            size: size,
            source: source,
            voxels: vec![],
            vertices: vec![],
            indices: vec![],
        }
    }

    fn get(&self, x: i32, y: i32, z: i32) -> f32 {
        let s = self.size + 5;
        self.voxels[(x + y*s + z*s*s) as usize]
    }

    fn fill_voxels(&mut self) {
        let s = self.size + 5;
        self.voxels = vec![0.0; (s*s*s) as usize];

        for x in 0 .. s {
            for y in 0 .. s {
                for z in 0 .. s {
                    let vox = self.source.get(x, y, z);
                    let v = if vox {1.0} else {-1.0};
                    self.voxels[(x + y*s + z*s*s) as usize] = v;
                }
            }
        }
    }

    fn blur (&mut self) {
        // Blur just one more beyond the limits
        let size = self.size+1;
        let s = size + 4;

        // Blur X
        for x in 0 .. size {
            for y in 0 .. size {
                for z in 0 .. size {
                    let mut sum = 0.0;
                    for i in 0..5 {
                        sum += self.get(x+i, y, z);
                    }
                    self.voxels[(x + y*s + z*s*s) as usize] = sum;
                }
            }
        }

        // Blur Y
        for x in 0 .. size {
            for y in 0 .. size {
                for z in 0 .. size {
                    let mut sum = 0.0;
                    for i in 0..5 {
                        sum += self.get(x, y+i, z);
                    }
                    self.voxels[(x + y*s + z*s*s) as usize] = sum;
                }
            }
        }

        // Blur Z
        for x in 0 .. size {
            for y in 0 .. size {
                for z in 0 .. size {
                    let mut sum = 0.0;
                    for i in 0..5 {
                        sum += self.get(x, y, z+i);
                    }
                    self.voxels[(x + y*s + z*s*s) as usize] = sum;
                }
            }
        }
    }

    fn cube(&mut self, pos: [f32; 3], cube: [f32; 8]) {
        let mut flag_index = 0;
        
        let mut edge_vertex = [Vector3::new(0.0, 0.0, 0.0); 12];
    
        //Find which vertices are inside of the surface and which are outside
        for i in 0 .. 8 {
            if cube[i] <= TARGET {
                flag_index |= 1 << i;
            }
        }
    
        //Find which edges are intersected by the surface
        let edge_flags = CUBE_EDGE_FLAGS[flag_index];
    
        //If the cube is entirely inside or outside of the surface, then there will be no intersections
        if edge_flags == 0 { return; }
    
        //Find the point of intersection of the surface with each edge
        for i in 0 .. 12 {
            //if there is an intersection on this edge
            if edge_flags & (1<<i) != 0 {
                let connection = EDGE_CONNECTION[i];
                let direction = EDGE_DIRECTION[i];

                let offset = get_offset(cube[connection[0]], cube[connection[1]]);
    
                edge_vertex[i].x = pos[0] + VERTEX_OFFSET[connection[0]][0] as f32 + offset * direction[0] as f32;
                edge_vertex[i].y = pos[1] + VERTEX_OFFSET[connection[0]][1] as f32 + offset * direction[1] as f32;
                edge_vertex[i].z = pos[2] + VERTEX_OFFSET[connection[0]][2] as f32 + offset * direction[2] as f32;
            }
        }
    
        //Save the triangles that were found. There can be up to five per cube

        let connection_table = TRIANGLE_CONNECTION_TABLE[flag_index];

        for i in 0..5 {
            if connection_table[3*i] < 0 { break; }
            
            let idx = self.vertices.len();

            for j in 0..3 {
                let vert = connection_table[3*i+j] as usize;
                self.indices.push((idx + WINDING_ORDER[j]) as u16);
                self.vertices.push(edge_vertex[vert]);
            }
        }
    }

    fn mesh (&mut self) {
        for x in 0 .. self.size {
            for y in 0 .. self.size {
                for z in 0 .. self.size {
                    let mut cube = [0.0; 8];
                    for i in 0 .. 8 {
                        cube[i] = self.get(
                            x + VERTEX_OFFSET[i][0],
                            y + VERTEX_OFFSET[i][1],
                            z + VERTEX_OFFSET[i][2]
                        );
                    }
                    let pos = [x as f32, y as f32, z as f32];
                    self.cube(pos, cube);
                }
            }
        }
    }
}

impl Mesher for MarchingCubes {
    fn mesh (&mut self, source: &VoxelSource) ->
        (Vec<Vector3<f32>>, Vec<Vector3<f32>>, Vec<u16>) {

        let mut builder = Builder::new(source, self.size);
        builder.fill_voxels();
        builder.blur();
        builder.mesh();

        let normals = calculate_normals(&builder.vertices, &builder.indices);

        (builder.vertices, normals, builder.indices)
    }
}