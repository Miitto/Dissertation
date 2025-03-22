use shaders::Program;
use std::{cell::RefCell, collections::HashMap};

use chunk::Chunk;
use glam::{ivec3, vec3};
use renderer::{
    DrawMode, Renderable, SSBO, State,
    bounds::{BoundingHeirarchy, BoundingVolume},
    buffers::{Buffer, BufferMode, GpuBuffer, ShaderBuffer},
    camera::frustum::Frustum,
    indirect::DrawArraysIndirectCommand,
    mesh::{Mesh, ninstanced::NInstancedMesh},
};
use voxel::{
    greedy_voxel,
    greedy_voxel_combined::{self, buffers::ChunkData},
};

use crate::{Args, common::BlockType, tests::test_scene};

mod chunk;
mod voxel;

pub fn setup(args: &Args, _state: &State) -> ChunkManager {
    let mut manager = ChunkManager::new(args.combine, args.frustum_cull);

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
            .or_insert_with(|| RefCell::new(Chunk::fill(BlockType::Air, !args.combine, true)));

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

        chunk.borrow_mut().set(pos, block);
    }

    let mut instance_data: Vec<greedy_voxel_combined::Instance> = vec![];

    for (position, chunk) in manager.chunks.iter() {
        let pos = vec3(position[0] as f32, position[1] as f32, position[2] as f32) * 32.0;
        let end_pos = pos + 32.0;

        chunk.borrow_mut().update(position, &manager.chunks);
        chunk
            .borrow_mut()
            .update_bounds(BoundingHeirarchy::from_min_max(pos, end_pos));

        if let Some(combined) = &mut manager.combined {
            let order = &mut combined.pos_order;

            let borrow = chunk.borrow();
            let instances = borrow.instances();

            instance_data.extend(
                instances
                    .iter()
                    .map(|i| greedy_voxel_combined::Instance { data: i.data }),
            );
            order.push((*position, instances.len()));
        }
    }

    dbg!(instance_data.len());

    if let Some(combined) = &mut manager.combined {
        println!("Setting instance data");
        if let Err(e) = combined.mesh.set_instances(&instance_data) {
            eprintln!("Failed to set combined greedy instances: {:?}", e);
        }
    }

    println!("Setup Finished");
    manager
}

#[allow(dead_code)]
pub struct ChunkManager {
    chunks: HashMap<[i32; 3], RefCell<chunk::Chunk>>,
    combined: Option<CombinedInstanceData>,
    frustum_cull: bool,
}

pub struct CombinedInstanceData {
    pub pos_order: Vec<([i32; 3], usize)>,
    chunk_data_buffer: ShaderBuffer<greedy_voxel_combined::buffers::ChunkData>,
    indirect_buffer: GpuBuffer,
    mesh: NInstancedMesh<greedy_voxel_combined::Vertex, greedy_voxel_combined::Instance>,
}

impl ChunkManager {
    pub fn new(combine: bool, frustum_cull: bool) -> Self {
        let vertices = vec![
            greedy_voxel_combined::Vertex::new([0, 0, 0]),
            greedy_voxel_combined::Vertex::new([1, 0, 0]),
            greedy_voxel_combined::Vertex::new([0, 0, 1]),
            greedy_voxel_combined::Vertex::new([1, 0, 1]),
        ];

        let combined = if combine {
            let mesh = NInstancedMesh::with_vertices(&vertices, None, DrawMode::TriangleStrip)
                .expect("Failed to create greedy ChunkManager mesh");

            Some(CombinedInstanceData {
                mesh,
                pos_order: vec![],
                chunk_data_buffer: ShaderBuffer::new(&[])
                    .expect("Failed to make shader buffer for chunk positions"),
                indirect_buffer: GpuBuffer::empty(
                    std::mem::size_of::<DrawArraysIndirectCommand>() * 100,
                    BufferMode::Persistent,
                )
                .expect("Failed to make indirect buffer"),
            })
        } else {
            None
        };

        Self {
            chunks: HashMap::new(),
            combined,
            frustum_cull,
        }
    }
}

impl Renderable for ChunkManager {
    fn render(&mut self, state: &mut renderer::State) {
        renderer::profiler::event!("Greedy Render");
        if self.combined.is_some() {
            render_combined(self, state);
        } else {
            render_seperate(self, state);
        }
    }
}

fn render_seperate(manager: &mut ChunkManager, state: &mut renderer::State) {
    for (pos, chunk) in manager.chunks.iter() {
        let ipos = ivec3(pos[0], pos[1], pos[2]) * 32;

        let mut borrow = chunk.borrow_mut();
        borrow.update(pos, &manager.chunks);

        let mesh = borrow.mesh().expect("Chunk should of had a mesh");

        let uniforms = greedy_voxel::Uniforms {
            chunk_position: ipos.to_array(),
        };

        let program = greedy_voxel::Program::get();

        state.draw(mesh, &program, &uniforms)
    }
}

fn render_combined(manager: &mut ChunkManager, state: &mut renderer::State) {
    renderer::profiler::event!("Greedy Render Combined");
    let frustum = &state.cameras.game_frustum();
    let combined = manager
        .combined
        .as_mut()
        .expect("Manager should of had combined");

    combined.mesh.bind();
    let program = greedy_voxel_combined::Program::get();
    program.bind();

    fn setup_multidraw(
        chunks: &HashMap<[i32; 3], RefCell<Chunk>>,
        combined: &CombinedInstanceData,
        frustum: &Frustum,
    ) -> (Vec<ChunkData>, Vec<DrawArraysIndirectCommand>) {
        renderer::profiler::event!("Greedy setup multidraw");

        let mut instance = 0;

        let mut uniforms = vec![];
        let mut draw_params = vec![];

        for (pos, count) in combined.pos_order.iter() {
            let chunk = if let Some(chunk) = chunks.get(pos) {
                chunk
            } else {
                instance += *count as u32;
                continue;
            };

            if chunk.borrow().bounds().intersects(frustum).is_none() {
                instance += *count as u32;
                continue;
            }

            let vec = ivec3(pos[0], pos[1], pos[2]) * 32;

            let uniform = greedy_voxel_combined::buffers::ChunkData {
                pos: vec.to_array(),
            };

            uniforms.push(uniform);

            let indirect = DrawArraysIndirectCommand {
                vertex_count: 4,
                instance_count: *count as u32,
                first: 0,
                base_instance: instance,
            };

            draw_params.push(indirect);

            instance += *count as u32;
        }

        (uniforms, draw_params)
    }

    let (uniforms, draw_params) = setup_multidraw(&manager.chunks, combined, frustum);

    fn set_chunk_data(chunk_data: Vec<ChunkData>, combined: &mut CombinedInstanceData) {
        renderer::profiler::event!("Greedy set combined chunk data");
        if let Err(e) = combined.chunk_data_buffer.set_data(&chunk_data, 0) {
            eprintln!("Error setting chunk data: {:?}", e);
        }
    }
    set_chunk_data(uniforms, combined);
    combined.chunk_data_buffer.bind();

    fn set_indirect_commands(
        draw_params: Vec<DrawArraysIndirectCommand>,
        combined: &mut CombinedInstanceData,
    ) {
        renderer::profiler::event!("Greedy set combined indirect commands");
        if let Err(e) = combined.indirect_buffer.set_data(&draw_params) {
            eprintln!("Error setting indirect commands: {:?}", e);
        }
    }

    let len = draw_params.len() as i32;
    set_indirect_commands(draw_params, combined);

    unsafe {
        gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, combined.indirect_buffer.id());
    }

    fn draw_combined(len: i32) {
        renderer::profiler::event!("Greedy multidraw");
        unsafe {
            gl::MultiDrawArraysIndirect(DrawMode::TriangleStrip.into(), std::ptr::null(), len, 0);
        }
    }

    draw_combined(len);
}
