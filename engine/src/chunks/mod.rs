use std::{collections::HashMap, rc::Rc};

use glam::{mat4, vec4};
use glium::{DrawParameters, VertexBuffer, uniform};
use renderer::{Renderable, State};
use shaders::Program;
use voxel::{BlockType, chunk_voxel};

use crate::tests::Test;

mod chunk;
mod voxel;

pub fn setup(test: Test, state: &State) -> ChunkManager {
    let mut manager = ChunkManager::new(state);

    match test {
        Test::Single => {
            let mut chunk = chunk::Chunk::fill(BlockType::Air);
            chunk.set([0, 0, 1], BlockType::Grass);
            manager.chunks.insert([0, 0, 0], chunk);
        }
        Test::Cube => {
            let chunk = chunk::Chunk::fill(BlockType::Grass);
            manager.chunks.insert([0, 0, 0], chunk);
        }
        Test::Plane(radius, height) => {
            for x in 0..radius {
                for z in 0..radius {
                    let chunk = chunk::Chunk::flat(height, BlockType::Grass);
                    manager.chunks.insert([x as i32, 0, z as i32], chunk);
                }
            }
        }
        Test::Perlin(_radius) => {
            // TODO: Perlin noise
            todo!("Perlin noise")
        }
    }
    manager
}

pub struct ChunkManager {
    chunks: HashMap<[i32; 3], chunk::Chunk>,
    vertex_buffer: glium::VertexBuffer<chunk_voxel::Vertex>,
    index_buffer: glium::index::NoIndices,
    program: Rc<glium::Program>,
    draw_parameters: DrawParameters<'static>,
}

impl ChunkManager {
    pub fn new(state: &State) -> Self {
        let vertices = [
            chunk_voxel::Vertex::new([0.0, 0.0, 0.0]),
            chunk_voxel::Vertex::new([1.0, 0.0, 0.0]),
            chunk_voxel::Vertex::new([0.0, 0.0, 1.0]),
            chunk_voxel::Vertex::new([1.0, 0.0, 1.0]),
        ];

        let v_buf = VertexBuffer::new(state.display.as_ref().unwrap(), &vertices)
            .expect("Failed to make chunk vertex buffer");
        let i_buf = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

        let program = chunk_voxel::Program::get(state.display.as_ref().unwrap())
            .expect("Failed to make chunk shader");

        let draw_parameters = DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLessOrEqual,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise,
            ..Default::default()
        };

        Self {
            chunks: HashMap::new(),
            vertex_buffer: v_buf,
            index_buffer: i_buf,
            program,
            draw_parameters,
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

            let instances = chunk.instance_positions();
            let uniforms = chunk_voxel::Uniforms {
                modelMatrix: model_matrix.to_cols_array_2d(),
                viewMatrix: state.camera.get_view().to_cols_array_2d(),
                projectionMatrix: state.camera.get_projection().to_cols_array_2d(),
            };

            let uniforms = uniform! {
                modelMatrix: uniforms.modelMatrix,
                viewMatrix: uniforms.viewMatrix,
                projectionMatrix: uniforms.projectionMatrix,
            };

            let instances = glium::VertexBuffer::new(state.display.as_ref().unwrap(), &instances)
                .expect("Failed to make instance buffer");
            let instances = instances
                .per_instance()
                .expect("Failed to convert to per instance");

            _ = state.draw(
                (&self.vertex_buffer, instances),
                self.index_buffer,
                &self.program,
                &uniforms,
                &self.draw_parameters,
            );
        }
    }
}
