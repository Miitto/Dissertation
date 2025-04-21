use std::sync::RwLock;

use dashmap::DashMap;
use glam::IVec3;
use renderer::{
    Axis, DrawMode,
    bounds::BoundingHeirarchy,
    mesh::{Mesh, ninstanced::NInstancedMesh},
};

use crate::binary::common::{
    AxisDepths, CHUNK_SIZE, ChunkRefs, VoxelArray, build_depths, make_faces,
};
use common::{BasicVoxel, BlockType, InstanceData, Voxel};

use super::voxel::culled_voxel;

pub struct VoxelData {
    pub voxels: RwLock<Box<VoxelArray>>,
    pub depth_mask: RwLock<Option<Box<AxisDepths>>>,
}

impl VoxelData {
    pub fn new(voxels: Box<VoxelArray>) -> Self {
        Self {
            voxels: RwLock::new(voxels),
            depth_mask: RwLock::new(None),
        }
    }

    pub fn set(&self, pos: &IVec3, block_type: &BlockType) {
        // If in chunk
        if pos.max_element() < CHUNK_SIZE as i32 && pos.min_element() >= 0 {
            let mut voxel =
                self.voxels.write().unwrap()[pos.x as usize][pos.y as usize][pos.z as usize];
            voxel.set_type(*block_type);
        }

        if let Some(mask) = self.depth_mask.write().unwrap().as_mut() {
            assert!(pos.x >= -1 && pos.x < CHUNK_SIZE as i32 + 1);

            let x = (pos.x + 1) as usize;
            let y = (pos.y + 1) as usize;
            let z = (pos.z + 1) as usize;
            if block_type.is_solid() {
                mask[usize::from(Axis::X)][y][z] |= 1 << x;
                mask[usize::from(Axis::Y)][z][x] |= 1 << y;
                mask[usize::from(Axis::Z)][y][x] |= 1 << z;
            } else {
                mask[usize::from(Axis::X)][y][z] &= !(1 << x);
                mask[usize::from(Axis::Y)][z][x] &= !(1 << y);
                mask[usize::from(Axis::Z)][y][x] &= !(1 << z);
            }
        }
    }

    pub fn build_depths(&self, chunks: &DashMap<IVec3, Chunk>, position: &IVec3) {
        if self.depth_mask.read().unwrap().is_none() {
            macro_rules! get_chunk {
                ($chunk_name:ident, $block_name:ident,$pos:expr) => {
                    let $chunk_name = chunks.get($pos);
                    let $chunk_name = if let Some(ref chunk) = $chunk_name {
                        let voxels = chunk.voxels();
                        let read = voxels.voxels.read().expect("Failed to read blocks");

                        Some((chunk, voxels, read))
                    } else {
                        None
                    };
                    let $block_name = if let Some(ref read) = $chunk_name {
                        let read = &read.2;
                        read.as_ref()
                    } else {
                        &crate::binary::common::BLANK_VOXELS
                    };
                };
            }
            get_chunk!(chunk_center, blocks_center, position);
            get_chunk!(
                chunk_x_neg,
                blocks_x_neg,
                &IVec3::new(position.x - 1, position.y, position.z)
            );
            get_chunk!(
                chunk_y_neg,
                blocks_y_neg,
                &IVec3::new(position.x, position.y - 1, position.z)
            );
            get_chunk!(
                chunk_z_neg,
                blocks_z_neg,
                &IVec3::new(position.x, position.y, position.z - 1)
            );
            get_chunk!(
                chunk_x_pos,
                blocks_x_pos,
                &IVec3::new(position.x + 1, position.y, position.z)
            );
            get_chunk!(
                chunk_y_pos,
                blocks_y_pos,
                &IVec3::new(position.x, position.y + 1, position.z)
            );
            get_chunk!(
                chunk_z_pos,
                blocks_z_pos,
                &IVec3::new(position.x, position.y, position.z + 1)
            );
            let refs = ChunkRefs {
                chunk: blocks_center,
                x_pos: blocks_x_pos,
                y_pos: blocks_y_pos,
                z_pos: blocks_z_pos,
                x_neg: blocks_x_neg,
                y_neg: blocks_y_neg,
                z_neg: blocks_z_neg,
            };

            let mask = build_depths(&refs);
            *self.depth_mask.write().unwrap() = Some(mask);
        }
    }
}

pub struct Chunk {
    voxels: VoxelData,
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
            voxels: VoxelData::new(voxels),
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

        self.voxels.voxels.read().unwrap()[x][y][z].get_type()
    }

    pub fn set(&self, pos: IVec3, block_type: BlockType) {
        if pos.max_element() >= CHUNK_SIZE as i32 || pos.min_element() < 0 {
            eprintln!("Coord: {:?} is outside of chunk", pos);
            return;
        }

        self.voxels.set(&pos, &block_type);

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

    pub fn update(&self, position: &IVec3, chunks: &DashMap<IVec3, Self>) -> bool {
        if !*self.needs_update.read().unwrap() {
            return false;
        }

        self.voxels.build_depths(chunks, position);

        let raw_faces = make_faces(
            chunks,
            position,
            self.voxels.depth_mask.read().unwrap().as_ref().unwrap(),
            *self.greedy.read().expect("Failed to read greedy"),
        );

        let mut instances = self.instances.write().unwrap();
        instances.clear();

        for face in raw_faces.iter() {
            let data = InstanceData::new(
                face.x,
                face.y,
                face.z,
                face.dir,
                face.width,
                face.height,
                face.block_type,
            )
            .rotate_on_dir();

            instances.push(culled_voxel::Instance { data: data.into() });
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

    pub fn voxels(&self) -> &VoxelData {
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
