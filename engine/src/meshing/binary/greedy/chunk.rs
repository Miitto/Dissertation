use std::{cell::RefCell, collections::HashMap};

use glam::{IVec3, ivec3};
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

    pub fn get(&self, pos: IVec3) -> BlockType {
        if pos.x as usize >= CHUNK_SIZE
            || pos.y as usize >= CHUNK_SIZE
            || pos.z as usize >= CHUNK_SIZE
        {
            eprintln!("Coord: {:?} is outside of chunk", pos);
            return BlockType::Air;
        }

        self.voxels[pos.x as usize][pos.y as usize][pos.z as usize].get_type()
    }

    pub fn set(&mut self, pos: IVec3, block_type: BlockType) {
        if pos.max_element() >= CHUNK_SIZE as i32 || pos.min_element() < 0 {
            eprintln!("Coord: {:?} is outside of chunk", pos);
            return;
        }

        self.voxels[pos.x as usize][pos.y as usize][pos.z as usize].set_type(block_type);

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

    pub fn update(&mut self, position: &IVec3, chunks: &HashMap<IVec3, RefCell<Self>>) -> bool {
        if !self.needs_update {
            return false;
        }

        let get_fn = |x: isize, y: isize, z: isize| {
            if (0..CHUNK_SIZE as isize).contains(&x)
                && (0..CHUNK_SIZE as isize).contains(&y)
                && (0..CHUNK_SIZE as isize).contains(&z)
            {
                return self.voxels[x as usize][y as usize][z as usize].get_type();
            }

            let chunk_x_offset = if x < 0 {
                -1
            } else if x >= CHUNK_SIZE as isize {
                1
            } else {
                0
            };
            let chunk_y_offset = if y < 0 {
                -1
            } else if y >= CHUNK_SIZE as isize {
                1
            } else {
                0
            };
            let chunk_z_offset = if z < 0 {
                -1
            } else if z >= CHUNK_SIZE as isize {
                1
            } else {
                0
            };

            let chunk_pos = position + ivec3(chunk_x_offset, chunk_y_offset, chunk_z_offset);

            let chunk = if let Some(chunk) = chunks.get(&chunk_pos) {
                chunk
            } else {
                return BlockType::Air;
            };

            let x_pos = if x < 0 {
                CHUNK_SIZE - 1
            } else if x >= CHUNK_SIZE as isize {
                0
            } else {
                x as usize
            };
            let y_pos = if y < 0 {
                CHUNK_SIZE - 1
            } else if y >= CHUNK_SIZE as isize {
                0
            } else {
                y as usize
            };
            let z_pos = if z < 0 {
                CHUNK_SIZE - 1
            } else if z >= CHUNK_SIZE as isize {
                0
            } else {
                z as usize
            };

            chunk.borrow().voxels[x_pos][y_pos][z_pos].get_type()
        };

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

        true
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
