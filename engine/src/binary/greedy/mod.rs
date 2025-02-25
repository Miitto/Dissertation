use shaders::Program;
use std::collections::HashMap;

use chunk::Chunk;
use glam::{mat4, vec3, vec4};
use renderer::{DrawMode, DrawType, Renderable, State, buffers::Vao, draw};
use voxel::{BlockType, greedy_voxel};

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
            let mut chunk = Chunk::fill(BlockType::Air);
            chunk.set([0, 0, 1], BlockType::Grass);
            manager.chunks.insert([0, 0, 0], chunk);
        }
        Scene::Cube => {
            let mut chunk = Chunk::fill(BlockType::Grass);
            chunk.set([0, 0, 1], BlockType::Air);
            manager.chunks.insert([0, 0, 0], chunk);
        }
        Scene::Plane => {
            for x in 0..radius {
                for z in 0..radius {
                    let chunk = Chunk::flat(height, BlockType::Grass);
                    manager.chunks.insert([x as i32, 0, z as i32], chunk);
                }
            }
        }
        Scene::Perlin => {
            // TODO: Perlin noise
            todo!("Perlin noise")
        }
    }

    println!("Setup Finished");
    manager
}

pub struct ChunkManager {
    chunks: HashMap<[i32; 3], chunk::Chunk>,
    plane_vao: Vao<greedy_voxel::Vertex, greedy_voxel::Instance>,
}

impl ChunkManager {
    pub fn new(_state: &State) -> Self {
        let vertices = [
            greedy_voxel::Vertex::new([0.0, 0.0, 0.0]),
            greedy_voxel::Vertex::new([1.0, 0.0, 0.0]),
            greedy_voxel::Vertex::new([0.0, 0.0, 1.0]),
            greedy_voxel::Vertex::new([1.0, 0.0, 1.0]),
        ];

        let vao = Vao::new(
            &vertices,
            None,
            DrawType::Static,
            DrawMode::TriangleStrip,
            None as Option<&[greedy_voxel::Instance]>,
        );

        Self {
            chunks: HashMap::new(),
            plane_vao: vao,
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

            let instance_vbo = chunk.get_instances();

            let vao = self.plane_vao.with_instance(instance_vbo);

            let uniforms = greedy_voxel::Uniforms {
                modelMatrix: model_matrix.to_cols_array_2d(),
                viewMatrix: state.camera.get_view().to_cols_array_2d(),
                projectionMatrix: state.camera.get_projection().to_cols_array_2d(),
                sky_light_color: vec4(1.0, 1.0, 1.0, 1.0).to_array(),
                sky_light_direction: vec3(-1.0, -1.0, -1.0).normalize().to_array(),
                ambient_light: 0.5,
            };

            let program = greedy_voxel::Program::get();

            draw::draw(&vao, &program, &uniforms)
        }
    }
}
