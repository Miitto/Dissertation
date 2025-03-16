use renderer::{
    Dir, DrawMode,
    bounds::BoundingHeirarchy,
    mesh::{Mesh, ninstanced::NInstancedMesh},
};

use crate::{
    common::{BasicVoxel, BlockType, InstanceData, Voxel},
    meshing::binary::common::make_greedy_faces,
};

use super::voxel::greedy_voxel;

const CHUNK_SIZE: usize = 32;

pub struct Chunk {
    voxels: Box<[[[BasicVoxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
    mesh: NInstancedMesh<greedy_voxel::Vertex, greedy_voxel::Instance>,
    needs_update: bool,
}

impl Chunk {
    fn new(
        voxels: Box<[[[BasicVoxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
        frustum_cull: bool,
    ) -> Self {
        let vertices = vec![
            greedy_voxel::Vertex::new([0, 0, 0]),
            greedy_voxel::Vertex::new([1, 0, 0]),
            greedy_voxel::Vertex::new([0, 0, 1]),
            greedy_voxel::Vertex::new([1, 0, 1]),
        ];

        let mut mesh = NInstancedMesh::with_vertices(&vertices, None, DrawMode::TriangleStrip)
            .expect("Failed to make greedy chunk NInstancedMesh");

        if frustum_cull {
            mesh.enable_frustum_culling();
        }

        Self {
            voxels,
            mesh,
            needs_update: true,
        }
    }

    pub fn set(&mut self, pos: [usize; 3], block_type: BlockType) {
        if pos[0] >= CHUNK_SIZE || pos[1] >= CHUNK_SIZE || pos[2] >= CHUNK_SIZE {
            eprintln!("Coord: {:?} is outside of chunk", pos);
            return;
        }

        self.voxels[pos[0]][pos[1]][pos[2]].set_type(block_type);

        self.invalidate()
    }

    pub fn fill(block_type: BlockType, frustum_cull: bool) -> Self {
        let voxels =
            Box::new([[[BasicVoxel::new(block_type); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);
        Self::new(voxels, frustum_cull)
    }

    fn invalidate(&mut self) {
        self.needs_update = true;
    }

    pub fn update_bounds(&mut self, bounds: BoundingHeirarchy) {
        self.mesh.set_bounds(bounds);
    }

    pub fn update(&mut self) {
        if !self.needs_update {
            return;
        }

        let get_fn = |x: usize, y: usize, z: usize| self.voxels[x][y][z].get_type();

        let greedy = make_greedy_faces(get_fn);

        let mut instances: Vec<greedy_voxel::Instance> = vec![];

        for dir in Dir::all() {
            for face in greedy[usize::from(dir)].iter() {
                let data = InstanceData::new(
                    face.x,
                    face.y,
                    face.z,
                    dir,
                    face.width,
                    face.height,
                    face.block_type,
                )
                .rotate_on_dir();

                instances.push(greedy_voxel::Instance { data: data.into() });
            }
        }

        if let Err(e) = self.mesh.set_instances(instances.as_slice()) {
            eprintln!("Error: {:?}", e);
        }

        self.needs_update = false;
    }

    pub fn mesh(&mut self) -> &mut NInstancedMesh<greedy_voxel::Vertex, greedy_voxel::Instance> {
        &mut self.mesh
    }
}
