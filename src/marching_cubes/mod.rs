
mod data;

use mesher::{Mesher, calculate_normals};
use cgmath::{Vector3, InnerSpace};
use voxel_source::VoxelSource;
use mesh::{Mesh, Vertex};

use self::data::*;

pub struct MarchingCubes { pub size: i32, pub smooth: bool }

pub struct Builder<'a> {
    size: i32,
    source: &'a VoxelSource,
    voxels: Vec<f32>,
    voxel_normals: Vec<Vector3<f32>>,

    vertices: Vec<Vector3<f32>>,
    indices: Vec<u16>,

    // Vertex caches, inspired on:
    // http://alphanew.net/index.php?section=articles&site=marchoptim&lang=eng
    ybuf: Vec<u16>,
    xbuf: Vec<u16>,
    zbuf: Vec<u16>,
    ybuf_next: Vec<u16>,
    xbuf_top: Vec<u16>,
    zbuf_top: Vec<u16>,
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
            voxel_normals: vec![],
            vertices: vec![],
            indices: vec![],

            ybuf: vec![],
            xbuf: vec![],
            zbuf: vec![],
            ybuf_next: vec![],
            xbuf_top: vec![],
            zbuf_top: vec![],
        }
    }

    fn get(&self, x: i32, y: i32, z: i32) -> f32 {
        let s = self.size + 5;
        self.voxels[(x + y*s + z*s*s) as usize]
    }
    
    fn get_normal (&self, x: i32, y: i32, z: i32) -> Vector3<f32> {
        let s = self.size + 5;
        self.voxel_normals[(x + y*s + z*s*s) as usize]
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

    // Box blur
    fn blur (&mut self) {
        // The voxel just outside the limits also needs to be blured
        let size = self.size+1;
        let s = size + 4;

        // Blur X
        for x in 0 .. size {
            for y in 0 .. s {
                for z in 0 .. s {
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
                for z in 0 .. s {
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

    fn calculate_voxel_normals (&mut self) {
        // The voxel just outside the limits also needs it's normals
        let size = self.size+2;
        let s = size + 3;

        self.voxel_normals = vec![Vector3::new(0.0, 0.0, 0.0); (s*s*s) as usize];

        // This calculates the normal of each voxel. If you have a 3d array of data
        // the normal is the derivitive of the x, y and z axis.
        // Normally you need to flip the normal (*-1) (but maybe it is not needed in this case?).
        // If you dont call this function default vertex normals will be calculated.

        for x in 0 .. size {
            for y in 0 .. size {
                for z in 0 .. size {
                    // Because i'm bluring all voxels with 5 voxels ahead, the
                    // actual center of the voxel is 3 voxels ahead, so the
                    // neighbors are not -1 and +1, but +1 and +3
                    // TODO: Correct that
                    // TODO: What I said above doesn't work (?)
                    let dx = self.get(x+2,y,z) - self.get(x+0,y,z);
                    let dy = self.get(x,y+2,z) - self.get(x,y+0,z);
                    let dz = self.get(x,y,z+2) - self.get(x,y,z+0);

                    let vec = -Vector3::new(dx, dy, dz).normalize();

                    self.voxel_normals[(x + y*s + z*s*s) as usize] = vec;
                }
            }
        }
    }

    fn interpolate_normal(&self, pos: Vector3<f32>) -> Vector3<f32> {
        let x = pos.x as i32;
        let y = pos.y as i32;
        let z = pos.z as i32;
        
        let fx = pos.x - x as f32;
        let fy = pos.y - y as f32;
        let fz = pos.z - z as f32;
        
        let x0 = self.get_normal(x,y,z) * (1.0-fx) + self.get_normal(x+1,y,z) * fx;
        let x1 = self.get_normal(x,y,z+1) * (1.0-fx) + self.get_normal(x+1,y,z+1) * fx;
        
        let x2 = self.get_normal(x,y+1,z) * (1.0-fx) + self.get_normal(x+1,y+1,z) * fx;
        let x3 = self.get_normal(x,y+1,z+1) * (1.0-fx) + self.get_normal(x+1,y+1,z+1) * fx;
        
        let z0 = x0 * (1.0-fz) + x1 * fz;
        let z1 = x2 * (1.0-fz) + x3 * fz;
        
        (z0 * (1.0-fy) + z1 * fy).normalize()
    }

    fn create_vertex (&mut self, pos: [f32; 3], cube: [f32; 8], a: usize, b: usize) -> u16 {
        let voff = VERTEX_OFFSET[a];
        let voffb = VERTEX_OFFSET[b];

        let direction = [
            voffb[0] as f32 - voff[0] as f32,
            voffb[1] as f32 - voff[1] as f32,
            voffb[2] as f32 - voff[2] as f32
        ];

        let offset = get_offset(cube[a], cube[b]);

        let mut edge_vertex = Vector3::new(0.0, 0.0, 0.0);
        edge_vertex.x = pos[0] + voff[0] as f32 + offset * direction[0] as f32;
        edge_vertex.y = pos[1] + voff[1] as f32 + offset * direction[1] as f32;
        edge_vertex.z = pos[2] + voff[2] as f32 + offset * direction[2] as f32;

        let index = self.vertices.len();
        self.vertices.push(edge_vertex);
        index as u16
    }

    fn cube(&mut self, pos: [f32; 3], cube: [f32; 8]) {
        // Indices acording to http://paulbourke.net/geometry/polygonise/
        //           Edges                   Vertices / Voxels
        //
        //        _____4______                  ____________
        //       /|          /|                /4         5/|
        //    7 / |       5 / |               / |         / |
        //     /__|__6_____/  |              /__|_______ /  |
        //    |  8|       |   | 9           |7  |      6|   |
        //    |   |       |10 |             |   |       |   |
        // 11 |   |____0__|___|             |   |0______|__1|
        //    |  /        |  /              |  /        |  /
        //    | / 3       | / 1             | /         | /
        //    |/______2___|/                |3_________2|/

        let mut flag_index = 0;
    
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
        
        let mut edge_vertex_indices = [0; 12];

        fn bit (x: i32, n: usize) -> bool { x&(1<<n) != 0 }

        macro_rules! edge {
            ($i:expr, $value:expr) => {
                if bit(edge_flags, $i) {
                    edge_vertex_indices[$i] = $value;
                }
            }
        }

        let x = pos[0] as usize;
        let z = pos[2] as usize;
        let s = self.size as usize + 1;

        // Bottom edges. Don't exist at the bottom plane.
        if pos[1] == 0.0 {
            // TODO: Cubes should also share bottom edges
            edge!(0, self.create_vertex(pos, cube, 0, 1));
            edge!(1, self.create_vertex(pos, cube, 1, 2));
            edge!(2, self.create_vertex(pos, cube, 2, 3));
            edge!(3, self.create_vertex(pos, cube, 3, 0));
        } else {
            edge!(0, self.xbuf[x + z*s]);
            edge!(1, self.zbuf[x+1 + z*s]);
            edge!(2, self.xbuf[x + (z+1)*s]);
            edge!(3, self.zbuf[x + z*s]);
        }


        edge!(4,
            if pos[2] == 0.0 {
                let index = self.create_vertex(pos, cube, 4, 5);
                self.xbuf_top[x + z*s] = index;
                index
            } else { self.xbuf_top[x + z*s] }
        );
        edge!(5, {
            let index = self.create_vertex(pos, cube, 5, 6);
            self.zbuf_top[x+1 + z*s] = index;
            index
        });
        edge!(6, {
            let index = self.create_vertex(pos, cube, 6, 7);
            self.xbuf_top[x + (z+1)*s] = index;
            index
        });
        edge!(7, {
            if pos[0] == 0.0 {
                let index = self.create_vertex(pos, cube, 7, 4);
                self.zbuf_top[x + z*s] = index;
                index
            } else { self.zbuf_top[x + z*s] }
        });


        edge!(8,
            if z == 0 && x == 0 {
                self.create_vertex(pos, cube, 0, 4)
            } else { self.ybuf[x] }
        );
        edge!(9,
            if z == 0 {
                let index = self.create_vertex(pos, cube, 1, 5);
                self.ybuf[x+1] = index;
                index
            } else { self.ybuf[x+1] }
        );

        edge!(10, {
            let index = self.create_vertex(pos, cube, 2, 6);
            self.ybuf_next[x+1] = index;
            index
        });
        edge!(11, {
            if x == 0 {
                let index = self.create_vertex(pos, cube, 3, 7);
                self.ybuf_next[x] = index;
                index
            } else { self.ybuf_next[x] }
        });
    
        //Find the point of intersection of the surface with each edge, WAAY simpler without buffers.
        /*for i in 0 .. 12 {

            //if there is an intersection on this edge
            if bit(edge_flags, i) {
                let connection = EDGE_CONNECTION[i];
                let direction = EDGE_DIRECTION[i];

                let offset = get_offset(cube[connection[0]], cube[connection[1]]);
                let voff = VERTEX_OFFSET[connection[0]];

                let index = self.create_vertex(pos, cube, connection[0], connection[1]);
                edge_vertex_indices[i] = index;
            }
        }*/
    
        //Save the triangles that were found. There can be up to five per cube

        let connection_table = TRIANGLE_CONNECTION_TABLE[flag_index];

        for i in 0..5 {
            if connection_table[3*i] == 12 { break; }

            for j in 0..3 {
                let ix = 3*i + WINDING_ORDER[j];
                let vert = connection_table[ix];
                self.indices.push(edge_vertex_indices[vert]);
            }
        }
    }

    fn mesh (&mut self) {

        let line_size = (self.size + 1) as usize;
        let plane_size = line_size * line_size;

        self.ybuf = vec![0; line_size];
        self.xbuf = vec![0; plane_size];
        self.zbuf = vec![0; plane_size];
        self.ybuf_next = vec![0; line_size];
        self.xbuf_top = vec![0; plane_size];
        self.zbuf_top = vec![0; plane_size];

        // Go in horizontal layers. y points up.
        for y in 0 .. self.size {

            // In each plane, move the top cache to the bottom
            ::std::mem::swap(&mut self.xbuf, &mut self.xbuf_top);
            ::std::mem::swap(&mut self.zbuf, &mut self.zbuf_top);

            // z points backwards
            for z in 0 .. self.size {

                ::std::mem::swap(&mut self.ybuf, &mut self.ybuf_next);

                // x aligned lines, pointing right
                for x in 0 .. self.size {
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
    fn mesh (&mut self, source: &VoxelSource) -> Mesh {

        println!("Marching the cubes...");
        let now = ::std::time::Instant::now();

        let mut builder = Builder::new(source, self.size);
        builder.fill_voxels();
        if self.smooth {
            builder.blur();

            // Comment this line for default normal calculation
            builder.calculate_voxel_normals();
        }

        builder.mesh();

        let vertices = builder.vertices.iter().map(|pos| Vertex::from_pos(*pos)).collect();
        let indices = ::std::mem::replace(&mut builder.indices, vec![]);
        let mut mesh = Mesh { indices: indices, vertices: vertices };


        if builder.voxel_normals.len() > 0 {
            //Each verts in the mesh generated is its position in the voxel array
            //and you can use this to find what the normal at this position.
            //The verts are not at whole numbers though so you need to use trilinear interpolation
            //to find the normal for that position
            
            for i in 0 .. mesh.vertices.len() {
                let mut vertex = &mut mesh.vertices[i];
                let normal = builder.interpolate_normal(vertex.pos);
                vertex.normal = normal;
            }
        } else {
            calculate_normals(&mut mesh);
        }

        let tm = now.elapsed();
        println!("The cubes marched in {} ms",
            (tm.as_secs()*1000) + (tm.subsec_nanos()/1_000_000) as u64);

        mesh
    }
}