use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
};

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
    needs_update: bool,
}

impl Chunk {
    pub fn set(&mut self, pos: [usize; 3], block_type: BlockType) {
        *self.instances.borrow_mut() = None;
        self.voxels[pos[0]][pos[1]][pos[2]].set_type(block_type);
        self.needs_update = true;
    }

    pub fn fill(block_type: BlockType) -> Self {
        let voxels =
            Box::new([[[BasicVoxel::new(block_type); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);
        Self {
            voxels,
            instances: RefCell::new(None),
            needs_update: true,
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

    pub fn update(&mut self, position: &[i32; 3], chunks: &HashMap<[i32; 3], RefCell<Self>>) {
        if !self.needs_update {
            return;
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

            let chunk_pos = [
                position[0] + chunk_x_offset,
                position[1] + chunk_y_offset,
                position[2] + chunk_z_offset,
            ];

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

        self.needs_update = false;
    }
}
