use std::collections::HashMap;

use glam::{mat4, vec4};
use renderer::{Renderable, State};
use voxel::{BlockType, chunk_voxel};

use crate::{Args, tests::Scene};

mod chunk;
mod voxel;

pub fn setup(args: &Args, state: &State) -> ChunkManager {
    let mut manager = ChunkManager::new(state);

    let scene = args.scene;
    let radius = args.radius;
    let height = args.depth;

    match scene {
        Scene::Single => {
            let mut chunk = chunk::Chunk::fill(BlockType::Air);
            chunk.set([0, 0, 1], BlockType::Grass);
            manager.chunks.insert([0, 0, 0], chunk);
        }
        Scene::Cube => {
            let chunk = chunk::Chunk::fill(BlockType::Grass);
            manager.chunks.insert([0, 0, 0], chunk);
        }
        Scene::Plane => {
            for x in 0..radius {
                for z in 0..radius {
                    let chunk = chunk::Chunk::flat(height, BlockType::Grass);
                    manager.chunks.insert([x as i32, 0, z as i32], chunk);
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
            chunk_voxel::Vertex::new([0.0, 0.0, 0.0]),
            chunk_voxel::Vertex::new([1.0, 0.0, 0.0]),
            chunk_voxel::Vertex::new([0.0, 0.0, 1.0]),
            chunk_voxel::Vertex::new([1.0, 0.0, 1.0]),
        ];

        Self {
            chunks: HashMap::new(),
        }
    }
}

impl Renderable for ChunkManager {
    fn render(&self, state: &mut renderer::State) {
        let right = vec4(1., 0., 0., 0.0);
        let up = vec4(0., 1., 0., 0.0);
        let forward = vec4(0., 0., -1., 0.0);

        for (pos, chunk) in &self.chunks {
            let model_matrix = mat4(
                right,
                up,
                forward,
                vec4(pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0),
            );

            let _instances = chunk.instance_positions();
            let _uniforms = chunk_voxel::Uniforms {
                modelMatrix: model_matrix.to_cols_array_2d(),
                viewMatrix: state.cameras.active().get_view().to_cols_array_2d(),
                projectionMatrix: state.cameras.active().get_projection().to_cols_array_2d(),
            };
        }
    }
}
