use shaders::Program;
use std::collections::HashMap;

use chunk::Chunk;
use glam::{ivec3, vec3, vec4};
use renderer::{
    DrawMode, Renderable, State, bounds::BoundingHeirarchy, draw, mesh::ninstanced::NInstancedMesh,
};
use voxel::greedy_voxel;

use crate::{Args, common::BlockType, tests::test_scene};

mod chunk;
mod voxel;

pub fn setup(args: &Args, _state: &State) -> ChunkManager {
    let mut manager = ChunkManager::new(args.frustum_cull);

    let data = test_scene(args);

    for (pos, block) in data {
        let mut chunk_pos = [pos[0] / 32, pos[1] / 32, pos[2] / 32];

        if pos[0] < 0 {
            chunk_pos[0] -= 1;
        }

        if pos[1] < 0 {
            chunk_pos[1] -= 1;
        }

        if pos[2] < 0 {
            chunk_pos[2] -= 1;
        }

        let chunk = manager
            .chunks
            .entry(chunk_pos)
            .or_insert_with(|| Chunk::fill(BlockType::Air, true));

        let mut chunk_x = pos[0].abs() % 32;
        let mut chunk_y = pos[1].abs() % 32;
        let mut chunk_z = pos[2].abs() % 32;

        if pos[0] < 0 {
            chunk_x = 31 - chunk_x;
        }

        if pos[1] < 0 {
            chunk_y = 31 - chunk_y;
        }

        if pos[2] < 0 {
            chunk_z = 31 - chunk_z;
        }

        let pos = [(chunk_x) as usize, (chunk_y) as usize, (chunk_z) as usize];

        chunk.set(pos, block);
    }

    for (pos, chunk) in manager.chunks.iter_mut() {
        let pos = vec3(pos[0] as f32, pos[1] as f32, pos[2] as f32) * 32.0;
        let end_pos = pos + 32.0;

        chunk.update_bounds(BoundingHeirarchy::from_min_max(pos, end_pos));
    }

    println!("Setup Finished");
    manager
}

pub struct ChunkManager {
    chunks: HashMap<[i32; 3], chunk::Chunk>,
    mesh: NInstancedMesh<greedy_voxel::Vertex, greedy_voxel::Instance>,
}

impl ChunkManager {
    pub fn new(frustum_cull: bool) -> Self {
        let vertices = vec![
            greedy_voxel::Vertex::new([0, 0, 0]),
            greedy_voxel::Vertex::new([1, 0, 0]),
            greedy_voxel::Vertex::new([0, 0, 1]),
            greedy_voxel::Vertex::new([1, 0, 1]),
        ];

        let mut mesh: NInstancedMesh<greedy_voxel::Vertex, greedy_voxel::Instance> =
            NInstancedMesh::with_vertices(&vertices, None, DrawMode::TriangleStrip)
                .expect("Failed to create greedy ChunkManager mesh");

        if frustum_cull {
            mesh.enable_frustum_culling();
        }

        Self {
            chunks: HashMap::new(),
            mesh,
        }
    }
}

impl Renderable for ChunkManager {
    fn render(&mut self, state: &mut renderer::State) {
        for (pos, chunk) in &mut self.chunks {
            let ipos = ivec3(pos[0], pos[1], pos[2]) * 32;

            chunk.update();

            let mesh = chunk.mesh();

            let uniforms = greedy_voxel::Uniforms {
                chunk_position: ipos.to_array(),
                viewMatrix: state.cameras.active().get_view().to_cols_array_2d(),
                projectionMatrix: state.cameras.active().get_projection().to_cols_array_2d(),
                sky_light_color: vec4(1.0, 1.0, 1.0, 1.0).to_array(),
                sky_light_direction: vec3(-1.0, -1.0, -1.0).normalize().to_array(),
                ambient_light: 0.5,
            };

            let program = greedy_voxel::Program::get();

            draw::draw(mesh, &program, &uniforms, state)
        }
    }
}
