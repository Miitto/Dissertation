use std::cell::{Ref, RefCell};

use renderer::Dir;

use common::{BasicVoxel, BlockType, InstanceData, Voxel};

use super::voxel::chunk_voxel;

const CHUNK_SIZE: usize = 32;

pub struct Chunk {
    voxels: Box<[[[BasicVoxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
    axes: RefCell<Option<Vec<chunk_voxel::Instance>>>,
}

impl Chunk {
    pub fn set(&mut self, pos: [usize; 3], block_type: BlockType) {
        *self.axes.borrow_mut() = None;
        self.voxels[pos[0]][pos[1]][pos[2]].set_type(block_type);
    }

    pub fn fill(block_type: BlockType) -> Self {
        let voxels =
            Box::new([[[BasicVoxel::new(block_type); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);
        Self {
            voxels,
            axes: RefCell::new(None),
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

    pub fn instance_positions(&self) -> Ref<'_, Vec<chunk_voxel::Instance>> {
        if self.axes.borrow().is_some() {
            return Ref::map(self.axes.borrow(), |o| o.as_ref().unwrap());
        }

        let mut data = vec![];

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let voxel = &self.voxels[x][y][z];
                    if !voxel.get_type().is_solid() {
                        continue;
                    }

                    for dir in Dir::all() {
                        let pos = InstanceData::new(
                            x as u8,
                            y as u8,
                            z as u8,
                            dir,
                            1,
                            1,
                            voxel.get_type(),
                        );
                        data.push(chunk_voxel::Instance { data: pos.into() });
                    }
                }
            }
        }

        *self.axes.borrow_mut() = Some(data);

        self.instance_positions()
    }
}
