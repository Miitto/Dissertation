use std::cell::{Ref, RefCell};

use renderer::Dir;

use crate::{
    common::{BasicVoxel, BlockType, InstanceData, Voxel},
    meshing::binary::common::make_culled_faces,
};

use super::voxel::culled_voxel;

const CHUNK_SIZE: usize = 32;

pub struct Chunk {
    voxels: Box<[[[BasicVoxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
    instances: RefCell<Option<Vec<culled_voxel::Instance>>>,
}

impl Chunk {
    pub fn set(&mut self, pos: [usize; 3], block_type: BlockType) {
        *self.instances.borrow_mut() = None;
        self.voxels[pos[0]][pos[1]][pos[2]].set_type(block_type);
    }

    pub fn fill(block_type: BlockType) -> Self {
        let voxels =
            Box::new([[[BasicVoxel::new(block_type); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);
        Self {
            voxels,
            instances: RefCell::new(None),
        }
    }

    pub fn flat(height: u8, block_type: BlockType) -> Self {
        let mut chunk = Self::fill(BlockType::Air);

        for y in 0..height {
            for x in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    chunk.voxels[x][y as usize][z] = BasicVoxel::new(block_type);
                }
            }
        }

        chunk
    }

    pub fn instance_positions(&self) -> Ref<'_, Vec<culled_voxel::Instance>> {
        if self.instances.borrow().is_some() {
            return Ref::map(self.instances.borrow(), |o| o.as_ref().unwrap());
        }

        let get_fn = |x: usize, y: usize, z: usize| self.voxels[x][y][z].get_type();

        let culled = make_culled_faces(get_fn);

        let mut instances = vec![];

        for dir in Dir::all() {
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let col = culled[usize::from(dir)][z][x];

                        let solid = (col >> y) & 1 == 1;

                        if !solid {
                            continue;
                        }

                        let pos = InstanceData::new(
                            x as u8,
                            y as u8,
                            z as u8,
                            dir,
                            1,
                            1,
                            self.voxels[x][y][z].get_type(),
                        )
                        .rotate_on_dir();
                        instances.push(culled_voxel::Instance { data: pos.into() });
                    }
                }
            }
        }

        *self.instances.borrow_mut() = Some(instances);

        self.instance_positions()
    }
}
