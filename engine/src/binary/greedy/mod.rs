use bracket_noise::prelude::*;
use shaders::Program;
use std::collections::HashMap;

use chunk::Chunk;
use glam::{mat4, vec3, vec4};
use renderer::{DrawMode, DrawType, Renderable, State, buffers::Vao, camera::frustum::AABB, draw};
use voxel::{BlockType, greedy_voxel};

use crate::{Args, tests::Scene};

mod chunk;
mod voxel;

pub fn setup(args: &Args, _state: &State) -> ChunkManager {
    let mut manager = ChunkManager::new(args.frustum_cull);

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
            let mut noise = FastNoise::seeded(1234);
            noise.set_noise_type(NoiseType::Perlin);
            noise.set_frequency(0.1);

            for chunk_x in 0..radius as usize {
                for chunk_z in 0..radius as usize {
                    for x in 0..32 {
                        for z in 0..32 {
                            let absolute_x = (chunk_x * 32) as i32 + x as i32;
                            let absolute_z = (chunk_z * 32) as i32 + z as i32;

                            let height = (noise.get_noise(absolute_x as f32, absolute_z as f32)
                                * (height as f32)) as i32;

                            for chunk_y in 0..=height / 32 {
                                let chunk = manager
                                    .chunks
                                    .entry([chunk_x as i32, chunk_y, chunk_z as i32])
                                    .or_insert_with(|| Chunk::flat(1, BlockType::Grass));

                                for y in 0..32 {
                                    let absolute_y = (chunk_y * 32) + y as i32;

                                    if absolute_y < height {
                                        chunk.set([x, y, z], BlockType::Grass);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!("Setup Finished");
    manager
}

pub struct ChunkManager {
    chunks: HashMap<[i32; 3], chunk::Chunk>,
    frustum_cull: bool,
    plane_vao: Vao<greedy_voxel::Vertex>,
}

impl ChunkManager {
    pub fn new(frustum_cull: bool) -> Self {
        let vertices = [
            greedy_voxel::Vertex::new([0.0, 0.0, 0.0]),
            greedy_voxel::Vertex::new([1.0, 0.0, 0.0]),
            greedy_voxel::Vertex::new([0.0, 0.0, 1.0]),
            greedy_voxel::Vertex::new([1.0, 0.0, 1.0]),
        ];

        let vao = Vao::new(&vertices, None, DrawType::Static, DrawMode::TriangleStrip);

        Self {
            chunks: HashMap::new(),
            frustum_cull,
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
            if self.frustum_cull {
                let frustum = state.cameras.game_frustum();

                let pos = vec3(pos[0] as f32, pos[1] as f32, pos[2] as f32);
                let end_pos = pos + 32.0;
                if !frustum.test_aabb(AABB::from_points(pos, end_pos)) {
                    continue;
                }
            }

            let model_matrix = mat4(
                right,
                up,
                forward,
                vec4(
                    (pos[0] * 32) as f32,
                    (pos[1] * 32) as f32,
                    (pos[2] * 32) as f32,
                    1.0,
                ),
            );

            let instance_vbo = chunk.get_instances();

            let vao = self.plane_vao.with_instance(instance_vbo);

            let uniforms = greedy_voxel::Uniforms {
                modelMatrix: model_matrix.to_cols_array_2d(),
                viewMatrix: state.cameras.active().get_view().to_cols_array_2d(),
                projectionMatrix: state.cameras.active().get_projection().to_cols_array_2d(),
                sky_light_color: vec4(1.0, 1.0, 1.0, 1.0).to_array(),
                sky_light_direction: vec3(-1.0, -1.0, -1.0).normalize().to_array(),
                ambient_light: 0.5,
            };

            let program = greedy_voxel::Program::get();

            draw::draw(&vao, &program, &uniforms)
        }
    }
}
