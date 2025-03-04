use bracket_noise::prelude::*;
use shaders::Program;
use std::{cell::RefCell, collections::HashMap};

use chunk::Chunk;
use glam::{ivec3, mat4, vec3, vec4};
use renderer::{
    DrawMode, DrawType, Renderable, State, bounds::BoundingHeirarchy, buffers::Vao, draw,
    mesh::Mesh,
};
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
    mesh: RefCell<Mesh<greedy_voxel::Vertex, greedy_voxel::Instance>>,
    instance_needs_calculating: bool,
}

impl ChunkManager {
    pub fn new(frustum_cull: bool) -> Self {
        let vertices = vec![
            greedy_voxel::Vertex::new([0, 0, 0]),
            greedy_voxel::Vertex::new([1, 0, 0]),
            greedy_voxel::Vertex::new([0, 0, 1]),
            greedy_voxel::Vertex::new([1, 0, 1]),
        ];

        let mut mesh = Mesh::new_instance(
            vertices,
            None,
            vec![],
            BoundingHeirarchy::default(),
            DrawMode::TriangleStrip,
            DrawType::Static,
        );

        mesh.set_frustum_cull(frustum_cull);

        Self {
            chunks: HashMap::new(),
            frustum_cull,
            mesh: RefCell::new(mesh),
            instance_needs_calculating: true,
        }
    }
}

impl Renderable for ChunkManager {
    fn render(&self, state: &mut renderer::State) {
        for (pos, chunk) in &self.chunks {
            let ipos = ivec3(pos[0], pos[1], pos[2]) * 32;

            let pos = vec3(pos[0] as f32, pos[1] as f32, pos[2] as f32) * 32.0;
            let end_pos = pos + 32.0;

            self.mesh
                .borrow_mut()
                .set_bounds(BoundingHeirarchy::from_min_max(pos, end_pos));

            let instances = chunk.instance_positions();

            self.mesh.borrow_mut().set_instances_shared(instances);

            let uniforms = greedy_voxel::Uniforms {
                chunk_position: ipos.to_array(),
                viewMatrix: state.cameras.active().get_view().to_cols_array_2d(),
                projectionMatrix: state.cameras.active().get_projection().to_cols_array_2d(),
                sky_light_color: vec4(1.0, 1.0, 1.0, 1.0).to_array(),
                sky_light_direction: vec3(-1.0, -1.0, -1.0).normalize().to_array(),
                ambient_light: 0.5,
            };

            let program = greedy_voxel::Program::get();

            draw::draw(&*self.mesh.borrow(), &program, &uniforms, state)
        }
    }
}
