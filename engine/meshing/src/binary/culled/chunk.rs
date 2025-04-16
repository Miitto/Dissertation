use std::sync::RwLock;

use dashmap::DashMap;
use glam::{IVec3, ivec3};
use renderer::{
    Dir, DrawMode,
    bounds::BoundingHeirarchy,
    mesh::{Mesh, ninstanced::NInstancedMesh},
};

use crate::binary::common::{CHUNK_SIZE, make_culled_faces, make_greedy_faces};
use common::{BasicVoxel, BlockType, InstanceData, Voxel};

use super::voxel::culled_voxel;

pub struct Chunk {
    voxels: RwLock<Box<[[[BasicVoxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>>,
    bounds: RwLock<BoundingHeirarchy>,
    instances: RwLock<Vec<culled_voxel::Instance>>,
    mesh: RwLock<Option<NInstancedMesh<culled_voxel::Vertex, culled_voxel::Instance>>>,
    greedy: RwLock<bool>,
    needs_update: RwLock<bool>,
    needs_mesh_written: RwLock<bool>,
}

impl Chunk {
    fn new(
        voxels: Box<[[[BasicVoxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
        make_mesh: bool,
        greedy: bool,
        frustum_cull: bool,
    ) -> Self {
        let vertices = vec![
            culled_voxel::Vertex::new([0, 0, 0]),
            culled_voxel::Vertex::new([1, 0, 0]),
            culled_voxel::Vertex::new([0, 0, 1]),
            culled_voxel::Vertex::new([1, 0, 1]),
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
            voxels: RwLock::new(voxels),
            mesh: RwLock::new(mesh),
            bounds: RwLock::new(BoundingHeirarchy::default()),
            instances: RwLock::new(vec![]),
            greedy: RwLock::new(greedy),
            needs_update: RwLock::new(true),
            needs_mesh_written: RwLock::new(false),
        }
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockType {
        assert!(x < CHUNK_SIZE);
        assert!(y < CHUNK_SIZE);
        assert!(z < CHUNK_SIZE);

        self.voxels.read().unwrap()[x][y][z].get_type()
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

    pub fn invalidate(&self) {
        *self.needs_update.write().unwrap() = true;
    }

    pub fn bounds(&self) -> BoundingHeirarchy {
        *self.bounds.read().unwrap()
    }

    pub fn update_bounds(&self, bounds: BoundingHeirarchy) {
        *self.bounds.write().unwrap() = bounds;
        if let Some(ref mut mesh) = self.mesh.write().unwrap().as_mut() {
            mesh.set_bounds(bounds);
        }
    }

    pub fn set_frustum_culling(&self, cull: bool) {
        if let Some(ref mut mesh) = self.mesh.write().unwrap().as_mut() {
            mesh.set_frustum_cull(cull);
        }
    }

    pub fn get_at(
        &self,
        x: isize,
        y: isize,
        z: isize,
        pos: &IVec3,
        chunks: &DashMap<IVec3, Self>,
    ) -> BlockType {
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

        let chunk_pos = pos + ivec3(chunk_x_offset, chunk_y_offset, chunk_z_offset);

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
    }

    pub fn update(&self, position: &IVec3, chunks: &DashMap<IVec3, Self>) -> bool {
        if !*self.needs_update.read().unwrap() {
            return false;
        }

        let get_fn =
            |x: isize, y: isize, z: isize| -> BlockType { self.get_at(x, y, z, position, chunks) };

        let raw_faces = if *self.greedy.read().unwrap() {
            make_greedy_faces(get_fn)
        } else {
            make_culled_faces(get_fn)
        };

        let mut instances = self.instances.write().unwrap();
        instances.clear();

        for dir in Dir::all() {
            for face in raw_faces[usize::from(dir)].iter() {
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

                instances.push(culled_voxel::Instance { data: data.into() });
            }
        }

        *self.needs_update.write().unwrap() = false;
        *self.needs_mesh_written.write().unwrap() = true;

        true
    }

    pub fn write_mesh(&self) -> bool {
        if !*self.needs_mesh_written.read().unwrap() {
            return false;
        }

        if let Some(ref mut mesh) = self.mesh.write().unwrap().as_mut() {
            if let Err(e) = mesh.set_instances(self.instances.read().unwrap().as_slice()) {
                eprintln!("Error: {:?}", e);
                return false;
            }
        }

        *self.needs_mesh_written.write().unwrap() = false;

        true
    }

    pub fn voxels(&self) -> &RwLock<Box<[[[BasicVoxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>> {
        &self.voxels
    }

    pub fn mesh(
        &self,
    ) -> &RwLock<Option<NInstancedMesh<culled_voxel::Vertex, culled_voxel::Instance>>> {
        &self.mesh
    }

    pub fn instances(&self) -> &RwLock<Vec<culled_voxel::Instance>> {
        &self.instances
    }
}
