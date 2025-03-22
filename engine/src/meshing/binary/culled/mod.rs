use std::collections::HashMap;

use chunk::Chunk;
use glam::{mat4, vec4};
use renderer::{Renderable, State};
use voxel::culled_voxel;

use crate::{Args, common::BlockType, tests::Scene};

mod chunk;
mod voxel;

pub fn setup(args: &Args, state: &State) -> ChunkManager {
    let mut manager = ChunkManager::new(state);

    let scene = args.scene;
    let radius = args.radius;
    let height = args.depth;

    match scene {
        Scene::Single => {
            let mut chunk = Chunk::fill(BlockType::Air);
            chunk.set([0, 0, 1], BlockType::Grass);
            manager.chunks.insert([0, 0, 0], chunk);
        }
        Scene::Cube => {
            let chunk = Chunk::fill(BlockType::Grass);
            manager.chunks.insert([0, 0, 0], chunk);
        }
        Scene::Plane => {
            for x in -radius..radius {
                for z in -radius..radius {
                    let chunk = Chunk::flat(height as u8, BlockType::Grass);
                    manager.chunks.insert([x, 0, z], chunk);
                }
            }
        }
        Scene::Perlin => {
            // TODO: Perlin noise
            todo!("Perlin noise")
        }
    }
    manager
}

pub struct ChunkManager {
    chunks: HashMap<[i32; 3], chunk::Chunk>,
}

impl ChunkManager {
    pub fn new(_state: &State) -> Self {
        let _vertices = [
            culled_voxel::Vertex::new([0.0, 0.0, 0.0]),
            culled_voxel::Vertex::new([1.0, 0.0, 0.0]),
            culled_voxel::Vertex::new([0.0, 0.0, 1.0]),
            culled_voxel::Vertex::new([1.0, 0.0, 1.0]),
        ];

        Self {
            chunks: HashMap::new(),
        }
    }
}

impl Renderable for ChunkManager {
    fn render(&mut self, state: &mut renderer::State) {
        todo!("Culled")
    }
}
