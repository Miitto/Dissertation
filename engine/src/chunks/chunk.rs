use super::voxel;

const CHUNK_SIZE: usize = 32;

pub struct Chunk {
    pub voxels: Box<[[[voxel::BlockType; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
}

impl Chunk {
    pub fn new() -> Self {
        let voxels = Box::new([[[voxel::BlockType::Air; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);
        Self { voxels }
    }
}
