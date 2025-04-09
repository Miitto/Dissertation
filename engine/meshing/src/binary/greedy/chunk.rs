use std::{cell::RefCell, collections::HashMap, sync::RwLock};

use dashmap::DashMap;
use glam::{IVec3, ivec3};
use renderer::{
    Dir, DrawMode,
    bounds::BoundingHeirarchy,
    mesh::{Mesh, ninstanced::NInstancedMesh},
};

use crate::binary::common::{make_culled_faces, make_greedy_faces};
use common::{BasicVoxel, BlockType, InstanceData, Voxel};

use super::voxel::greedy_voxel;

const CHUNK_SIZE: usize = 32;

pub struct Chunk {
    voxels: RwLock<Box<[[[BasicVoxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>>,
    bounds: RwLock<BoundingHeirarchy>,
    instances: RwLock<Vec<greedy_voxel::Instance>>,
    mesh: RwLock<Option<NInstancedMesh<greedy_voxel::Vertex, greedy_voxel::Instance>>>,
    greedy: RwLock<bool>,
    needs_update: RwLock<bool>,
}

impl Chunk {
    fn new(
        voxels: Box<[[[BasicVoxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
        make_mesh: bool,
        greedy: bool,
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
            if let Some(ref mut mesh) = mesh {
                mesh.set_frustum_cull(frustum_cull);
            }
        }

        Self {
            voxels: RwLock::new(voxels),
            mesh: RwLock::new(mesh),
            bounds: RwLock::new(BoundingHeirarchy::default()),
            instances: RwLock::new(vec![]),
            greedy: RwLock::new(greedy),
            needs_update: RwLock::new(true),
        }
    }

    pub fn set_frustum_culling(&self, frustum_cull: bool) {
        if let Some(ref mut mesh) = *self.mesh.write().unwrap() {
            mesh.set_frustum_cull(frustum_cull);
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

        self.voxels.read().unwrap()[pos.x as usize][pos.y as usize][pos.z as usize].get_type()
    }

    pub fn set(&self, pos: IVec3, block_type: BlockType) {
        if pos.max_element() >= CHUNK_SIZE as i32 || pos.min_element() < 0 {
            eprintln!("Coord: {:?} is outside of chunk", pos);
            return;
        }

        self.voxels.write().unwrap()[pos.x as usize][pos.y as usize][pos.z as usize]
            .set_type(block_type);

        self.invalidate()
    }

    pub fn fill(block_type: BlockType, make_mesh: bool, greedy: bool, frustum_cull: bool) -> Self {
        let voxels =
            Box::new([[[BasicVoxel::new(block_type); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);
        Self::new(voxels, make_mesh, greedy, frustum_cull)
    }

    fn invalidate(&self) {
        *self.needs_update.write().unwrap() = true;
    }

    pub fn bounds(&self) -> BoundingHeirarchy {
        *self.bounds.read().unwrap()
    }

    pub fn update_bounds(&self, bounds: BoundingHeirarchy) {
        *self.bounds.write().unwrap() = bounds;
        if let Some(ref mut mesh) = *self.mesh.write().unwrap() {
            mesh.set_bounds(bounds);
        }
    }

    pub fn update(&self, position: &IVec3, chunks: &DashMap<IVec3, Self>) -> bool {
        if !*self.needs_update.read().unwrap() {
            return false;
        }
        dbg!("Updating chunk at {:?}", position);

        let get_fn = |x: isize, y: isize, z: isize| {
            if (0..CHUNK_SIZE as isize).contains(&x)
                && (0..CHUNK_SIZE as isize).contains(&y)
                && (0..CHUNK_SIZE as isize).contains(&z)
            {
                return self.voxels.read().unwrap()[x as usize][y as usize][z as usize].get_type();
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

            chunk.voxels.read().unwrap()[x_pos][y_pos][z_pos].get_type()
        };

        let faces = if *self.greedy.read().unwrap() {
            make_greedy_faces(get_fn)
        } else {
            make_culled_faces(get_fn)
        };

        let mut instances = self.instances.write().unwrap();

        instances.clear();

        for dir in Dir::all() {
            for face in faces[usize::from(dir)].iter() {
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

        if let Some(ref mut mesh) = *self.mesh.write().unwrap() {
            if let Err(e) = mesh.set_instances(instances.as_slice()) {
                eprintln!("Error: {:?}", e);
            }
        }

        println!("Updated {} instances", instances.len());

        *self.needs_update.write().unwrap() = false;

        true
    }

    pub fn mesh(
        &self,
    ) -> &RwLock<Option<NInstancedMesh<greedy_voxel::Vertex, greedy_voxel::Instance>>> {
        &self.mesh
    }

    pub fn instances(&self) -> &RwLock<Vec<greedy_voxel::Instance>> {
        &self.instances
    }
}
