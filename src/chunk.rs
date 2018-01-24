
use cgmath::{Vector3, InnerSpace};

pub struct Chunk {
    pub data: Vec<bool>,
    positions: Vec<(i32, i32, i32)>,
    pub previous: Vec<Vector3<f32>>,
    pub vertices: Vec<Vector3<f32>>,
    pub normals: Vec<Vector3<f32>>,
    pub indices: Vec<u16>,
    indexmap: Vec<i32>,
}


const SZ: usize = 64;
const ISZ: i32 = 64;

impl Chunk {
    pub fn new () -> Self {
        let mut chunk = Chunk {
            data: vec![false; SZ*SZ*SZ],
            positions: vec![],
            previous: vec![],
            vertices: vec![],
            normals: vec![],
            indices: vec![],
            indexmap: vec![-1; SZ*SZ*SZ],
        };
        for ix in 0..SZ {
            for iy in 0..SZ {
                for iz in 0..SZ {
                    let sz = SZ as f32;
                    let x = ix as f32/sz-0.5;
                    let y = iy as f32/sz;
                    let z = iz as f32/sz-0.5;
                    let d = x*x + y*y + z*z;
                    chunk.set(ix, iy, iz, d < 0.5);
                }
            }
        }
        chunk
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, b: bool) {
        self.data[x + y*SZ + z*SZ*SZ] = b;
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> bool {
        if x >= SZ || y >= SZ || z >= SZ {
            false
        } else {
            self.data[x + y*SZ + z*SZ*SZ]
        }
    }

    fn index_at(&self, x: i32, y: i32, z: i32) -> i32 {
        if x >= 0 && y >= 0 && z >= 0 && x < ISZ && y < ISZ && z < ISZ {
            let i = x + y*ISZ + z*ISZ*ISZ;
            self.indexmap[i as usize]
        } else { -1 }
    }

    fn index_at_off(&self, x: i32, y: i32, z: i32, off: [i32;3]) -> i32 {
        self.index_at(x+off[0], y+off[1], z+off[2])
    }

    fn create_vertex (&mut self, x: usize, y: usize, z: usize) {
        // How many voxels adjacent to this vertex are not empty
        let mut count = 0;
        if self.get(x, y, z) { count+=1; }
        if self.get(x, y, z+1) { count+=1; }
        if self.get(x, y+1, z) { count+=1; }
        if self.get(x, y+1, z+1) { count+=1; }
        if self.get(x+1, y, z) { count+=1; }
        if self.get(x+1, y, z+1) { count+=1; }
        if self.get(x+1, y+1, z) { count+=1; }
        if self.get(x+1, y+1, z+1) { count+=1; }

        // If not voxels are empty nor full,
        // then there's a vertex here.
        if count > 0 && count < 8 {
            let index = self.positions.len();
            self.indexmap[x + y*SZ + z*SZ*SZ] = index as i32;
            self.positions.push( (x as i32, y as i32, z as i32) );
            self.vertices.push( Vector3::new(x as f32, y as f32, z as f32) );
        }
    }

    fn connect_faces (
            &mut self,
            x: i32, y: i32, z: i32,
            offs: [[i32; 3]; 4] ) {

        fn get_ix(x: i32, y: i32, z: i32, off: [i32;3]) -> i32 {
            x+off[0] + (y+off[1])*ISZ + (z+off[2])*ISZ*ISZ
        }

        let mut vertices = 0;
        for off in offs.iter() {
            if self.index_at_off(x,y,z,*off) > -1 {
                vertices += 1;
            }
        }

        if vertices == 4 {
            let pos = self.data[get_ix(x,y,z,[1,1,1]) as usize];
            let neg = self.data[get_ix(x,y,z,offs[3]) as usize];

            if pos != neg {
                let o = if neg {[0,1,2, 2,1,3]} else {[2,1,0, 3,1,2]};

                let mut ix = [0; 6];
                for i in 0..6 {
                    let ii = o[i];
                    ix[i] = self.index_at_off(x,y,z,offs[ii]) as u16;
                }

                self.indices.push(ix[0]);
                self.indices.push(ix[1]);
                self.indices.push(ix[2]);

                self.indices.push(ix[3]);
                self.indices.push(ix[4]);
                self.indices.push(ix[5]);
            }
        }
    }

    fn relax (&mut self, x: i32, y: i32, z: i32) {

        let ix = self.index_at(x, y, z) as usize;
        let mut p = self.previous[ix];

        let offs = [
            [-1, 0, 0], [1, 0, 0],
            [0, -1, 0], [0, 1, 0],
            [0, 0, -1], [0, 0, 1]
        ];

        let mut sum: u8 = 1;

        for off in offs.iter() {
            let i = self.index_at_off(x, y, z, *off);
            if i > -1 {
                p += self.previous[i as usize];
                sum += 1;
            }
        }

        self.vertices[ix] = p/sum as f32;
    }

    fn calculate_normals (&mut self) {
        let len = self.positions.len();
        self.normals = vec![Vector3::new(0.0, 0.0, 0.0); len];

        let high = self.indices.len()/3-1;

        for ii in 0 .. high {
            let i = ii*3;

            let aa = self.indices[i  ] as usize;
            let bb = self.indices[i+1] as usize;
            let cc = self.indices[i+2] as usize;

            let a = self.vertices[aa];
            let b = self.vertices[bb];
            let c = self.vertices[cc];

            let n = (b-a).cross(c-a);

            self.normals[aa] += n;
            self.normals[bb] += n;
            self.normals[cc] += n;
        }

        for i in 0 .. len-1 {
            self.normals[i] = self.normals[i].normalize();
        }
    }

    pub fn create_mesh (&mut self) {
        for x in 0..SZ {
            for y in 0..SZ {
                for z in 0..SZ {
                    self.create_vertex(x, y, z)
                }
            }
        }

        let len = self.positions.len();
        self.previous = vec![Vector3::new(0.0, 0.0, 0.0); len];

        let relax_level = 10;

        for _ in 0..relax_level {
            match self {
                &mut Chunk{ref mut previous, ref mut vertices, ..} =>
                    ::std::mem::swap(previous, vertices),
                _ => {},
            }

            for i in 0..len {
                let (x, y, z) = self.positions[i];
                self.relax(x, y, z);
            }
        }

        for x in 0 .. ISZ-1 {
            for y in 0 .. ISZ-1 {
                for z in 0 .. ISZ-1 {
                    self.connect_faces(x, y, z, [
                        [0, 0, 0],
                        [0, 1, 0],
                        [0, 0, 1],
                        [0, 1, 1],
                    ]);
                    self.connect_faces(x, y, z, [
                        [0, 0, 0],
                        [0, 0, 1],
                        [1, 0, 0],
                        [1, 0, 1],
                    ]);
                    self.connect_faces(x, y, z, [
                        [0, 0, 0],
                        [1, 0, 0],
                        [0, 1, 0],
                        [1, 1, 0],
                    ]);
                }
            }
        }

        self.calculate_normals();
    }
}
