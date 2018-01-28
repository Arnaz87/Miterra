
use mesher::{Mesher, calculate_normals};
use voxel_source::VoxelSource;
use cgmath::Vector3;

pub struct SurfNet {
    pub size: u16,
    pub smooth: u16,
}

pub struct Builder<'a> {
    size: i32,
    smooth: u16,
    source: &'a VoxelSource,
    positions: Vec<(i32, i32, i32)>,
    previous: Vec<Vector3<f32>>,
    vertices: Vec<Vector3<f32>>,
    normals: Vec<Vector3<f32>>,
    indices: Vec<u16>,
    indexmap: Vec<i32>,
}

impl<'a> Builder<'a> {
    pub fn new (params: &SurfNet, source: &'a VoxelSource) -> Self {
        let sz = params.size as usize;
        Builder {
            size: params.size as i32,
            smooth: params.smooth,
            source: source,
            positions: vec![],
            previous: vec![],
            vertices: vec![],
            normals: vec![],
            indices: vec![],
            indexmap: vec![-1; sz*sz*sz],
        }
    }

    fn index_at(&self, x: i32, y: i32, z: i32) -> i32 {
        let sz = self.size;
        if x >= 0 && y >= 0 && z >= 0 && x < sz && y < sz && z < sz {
            let i = x + y*sz + z*sz*sz;
            self.indexmap[i as usize]
        } else { -1 }
    }

    fn index_at_off(&self, x: i32, y: i32, z: i32, off: [i32;3]) -> i32 {
        self.index_at(x+off[0], y+off[1], z+off[2])
    }

    fn create_vertex (&mut self, x: i32, y: i32, z: i32) {
        // How many voxels adjacent to this vertex are not empty
        let mut count = 0;
        if self.source.get(x, y, z) { count+=1; }
        if self.source.get(x, y, z+1) { count+=1; }
        if self.source.get(x, y+1, z) { count+=1; }
        if self.source.get(x, y+1, z+1) { count+=1; }
        if self.source.get(x+1, y, z) { count+=1; }
        if self.source.get(x+1, y, z+1) { count+=1; }
        if self.source.get(x+1, y+1, z) { count+=1; }
        if self.source.get(x+1, y+1, z+1) { count+=1; }

        // If not voxels are empty nor full,
        // then there's a vertex here.
        if count > 0 && count < 8 {
            let ix = {
                let sz = self.size;
                (x + y*sz + z*sz*sz) as usize
            };
            let index = self.positions.len();
            self.indexmap[ix] = index as i32;
            self.positions.push( (x as i32, y as i32, z as i32) );
            self.vertices.push( Vector3::new(x as f32, y as f32, z as f32) );
        }
    }

    fn connect_faces (
            &mut self,
            x: i32, y: i32, z: i32,
            offs: [[i32; 3]; 4] ) {

        let mut vertices = 0;
        for off in offs.iter() {
            if self.index_at_off(x,y,z,*off) > -1 {
                vertices += 1;
            }
        }

        if vertices == 4 {
            let pos = self.source.get(x+1, y+1, z+1);
            let p = offs[3];
            let neg = self.source.get(x+p[0], y+p[1], z+p[2]);

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

    fn mesh (&mut self) {
        for x in 0 .. self.size {
            for y in 0 .. self.size {
                for z in 0 .. self.size {
                    self.create_vertex(x, y, z)
                }
            }
        }

        let len = self.positions.len();
        self.previous = vec![Vector3::new(0.0, 0.0, 0.0); len];

        for _ in 0 .. self.smooth {
            // This is done so that previous has the current data and vertices
            // trash, on which to write new vertices data
            match self {
                &mut Builder{ref mut previous, ref mut vertices, ..} =>
                    ::std::mem::swap(previous, vertices),
            }

            for i in 0..len {
                let (x, y, z) = self.positions[i];
                self.relax(x, y, z);
            }
        }

        for x in 0 .. self.size-1 {
            for y in 0 .. self.size-1 {
                for z in 0 .. self.size-1 {
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

        self.normals = calculate_normals(&self.vertices, &self.indices);
    }
}

impl Mesher for SurfNet {
    fn mesh (&mut self, source: &VoxelSource) ->
        (Vec<Vector3<f32>>, Vec<Vector3<f32>>, Vec<u16>) {

        println!("Netting the surfs...");
        let now = ::std::time::Instant::now();

        let mut builder = Builder::new(self, source);
        builder.mesh();

        let tm = now.elapsed();
        println!("The surf netted in {} ms",
            (tm.as_secs()*1000) + (tm.subsec_nanos()/1_000_000) as u64);

        match builder {
            Builder{vertices, normals, indices, .. } =>
                (vertices, normals, indices)
        }
    }
}
