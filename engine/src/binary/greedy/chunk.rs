use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use renderer::{Dir, DrawType, buffers::Vbo, camera::frustum::Frustum};

use crate::{
    binary::common::{greedy_faces, make_culled_faces},
    common::InstanceData,
};

use super::voxel::{self, BlockType, greedy_voxel};

const CHUNK_SIZE: usize = 32;

pub struct Chunk {
    voxels: Box<[[[voxel::Voxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
    instances: RefCell<Option<Vec<greedy_voxel::Instance>>>,
    cached: RefCell<Option<Rc<Vbo<greedy_voxel::Instance>>>>,
}

impl Chunk {
    pub fn set(&mut self, pos: [usize; 3], block_type: BlockType) {
        if pos[0] >= CHUNK_SIZE || pos[1] >= CHUNK_SIZE || pos[2] >= CHUNK_SIZE {
            eprintln!("Coord: {:?} is outside of chunk", pos);
            return;
        }

        *self.instances.borrow_mut() = None;
        self.voxels[pos[0]][pos[1]][pos[2]].set_type(block_type);

        self.invalidate()
    }

    pub fn fill(block_type: BlockType) -> Self {
        let voxels =
            Box::new([[[voxel::Voxel::new(block_type); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);
        Self {
            voxels,
            instances: RefCell::new(None),
            cached: RefCell::new(None),
        }
    }

    pub fn flat(height: u8, block_type: BlockType) -> Self {
        let mut chunk = Self::fill(BlockType::Air);

        for y in 0..height {
            for x in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    chunk.voxels[x][y as usize][z] = voxel::Voxel::new(block_type);
                }
            }
        }

        chunk
    }

    fn invalidate(&self) {
        *self.instances.borrow_mut() = None;
        *self.cached.borrow_mut() = None;
    }

    pub fn instance_positions(&self) -> Ref<'_, Vec<greedy_voxel::Instance>> {
        if self.instances.borrow().is_some() {
            return Ref::map(self.instances.borrow(), |o| o.as_ref().unwrap());
        }

        let get_fn = |x: usize, y: usize, z: usize| self.voxels[x][y][z].is_solid();

        let culled = make_culled_faces(get_fn);

        let greedy = greedy_faces(culled);

        let mut instances = vec![];

        for dir in Dir::all() {
            for face in greedy[usize::from(dir)].iter() {
                let data = InstanceData::new(face.x, face.y, face.z, dir, face.width, face.height)
                    .rotate_on_dir();

                instances.push(greedy_voxel::Instance { data: data.into() });
            }
        }

        *self.instances.borrow_mut() = Some(instances);

        self.instance_positions()
    }

    pub fn get_instances(&self) -> Rc<Vbo<greedy_voxel::Instance>> {
        if self.cached.borrow().is_some() {
            self.cached.borrow().as_ref().unwrap().clone()
        } else {
            let vbo = {
                let instances = self.instance_positions();

                Vbo::new(instances.as_slice(), DrawType::Static, true)
            };

            *self.cached.borrow_mut() = Some(Rc::new(vbo));

            self.cached.borrow().as_ref().unwrap().clone()
        }
    }
}
