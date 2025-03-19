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
    bounds: BoundingHeirarchy,
    instances: Vec<greedy_voxel::Instance>,
    mesh: Option<NInstancedMesh<greedy_voxel::Vertex, greedy_voxel::Instance>>,
    needs_update: bool,
}

impl Chunk {
    fn new(
        voxels: Box<[[[BasicVoxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
        make_mesh: bool,
        frustum_cull: bool,
    ) -> Self {
        let vertices = vec![
            greedy_voxel::Vertex::new([0, 0, 0]),
            greedy_voxel::Vertex::new([1, 0, 0]),
            greedy_voxel::Vertex::new([0, 0, 1]),
            greedy_voxel::Vertex::new([1, 0, 1]),
        ];

        let mut mesh = if make_mesh {
            Some(
                NInstancedMesh::with_vertices(&vertices, None, DrawMode::TriangleStrip)
                    .expect("Failed to make greedy chunk NInstancedMesh"),
            )
        } else {
            None
        };

        if frustum_cull {
            if let Some(mesh) = &mut mesh {
                mesh.enable_frustum_culling();
            }
        }

        Self {
            voxels,
            mesh,
            bounds: BoundingHeirarchy::default(),
            instances: vec![],
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

    pub fn fill(block_type: BlockType, make_mesh: bool, frustum_cull: bool) -> Self {
        let voxels =
            Box::new([[[BasicVoxel::new(block_type); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);
        Self::new(voxels, make_mesh, frustum_cull)
    }

    fn invalidate(&mut self) {
        self.needs_update = true;
    }

    pub fn bounds(&self) -> &BoundingHeirarchy {
        &self.bounds
    }

    pub fn update_bounds(&mut self, bounds: BoundingHeirarchy) {
        self.bounds = bounds;
        if let Some(mesh) = &mut self.mesh {
            mesh.set_bounds(bounds);
        }
    }

    pub fn update(&mut self) {
        if !self.needs_update {
            return;
        }

        let get_fn = |x: usize, y: usize, z: usize| self.voxels[x][y][z].get_type();

        let greedy = make_greedy_faces(get_fn);

        self.instances.clear();

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

                self.instances
                    .push(greedy_voxel::Instance { data: data.into() });
            }
        }

        if let Some(mesh) = &mut self.mesh {
            if let Err(e) = mesh.set_instances(self.instances.as_slice()) {
                eprintln!("Error: {:?}", e);
            }
        }

        self.needs_update = false;
    }

    pub fn mesh(
        &mut self,
    ) -> Option<&mut NInstancedMesh<greedy_voxel::Vertex, greedy_voxel::Instance>> {
        self.mesh.as_mut()
    }

    pub fn instances(&self) -> &[greedy_voxel::Instance] {
        &self.instances
    }
}
