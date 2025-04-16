use rayon::prelude::*;

pub use chunk::Chunk;
use dashmap::DashMap;
use glam::{IVec3, Mat4, ivec3, vec3};
use rayon::iter::IntoParallelRefIterator;
use renderer::{
    DrawMode, ProgramSource, Renderable, SSBO, State,
    bounds::{BoundingHeirarchy, BoundingVolume},
    buffers::{Buffer, BufferMode, GpuBuffer, ShaderBuffer},
    camera::frustum::Frustum,
    draw::line::Line,
    indirect::DrawArraysIndirectCommand,
    mesh::{Mesh, basic::BasicMesh, ninstanced::NInstancedMesh},
};
use voxel::{
    culled_voxel,
    culled_voxel_combined::{self, buffers::ChunkData},
};

use common::{
    Args, BlockType, get_looked_at_block, seperate_global_pos,
    tests::{Test, test_scene},
};

mod chunk;
mod voxel;

pub fn chunk_data(data: &DashMap<IVec3, BlockType>, args: &Args, chunks: &DashMap<IVec3, Chunk>) {
    data.into_iter().for_each(|e| {
        let (pos, block) = e.pair();
        let (chunk_pos, in_chunk_pos) = seperate_global_pos(pos);

        let chunk = chunks.entry(chunk_pos).or_insert(Chunk::fill(
            BlockType::Air,
            !args.combine,
            args.test == Test::Greedy,
            args.frustum_cull,
        ));

        chunk.set(in_chunk_pos, *block);
    });
}

pub fn mesh_chunks(chunks: &DashMap<IVec3, Chunk>) {
    chunks.par_iter().for_each(|e| {
        let position = e.key();
        let chunk = e.value();
        let pos = vec3(position[0] as f32, position[1] as f32, position[2] as f32) * 32.0;
        let end_pos = pos + 32.0;

        chunk.update(position, chunks);
        chunk.update_bounds(BoundingHeirarchy::from_min_max(pos, end_pos));
    });
}

pub fn setup(args: &Args, _state: &State) -> ChunkManager {
    let mut manager = ChunkManager::new(args.combine, args.frustum_cull);

    let data = test_scene(args);

    chunk_data(&data, args, &manager.chunks);

    let instance_data = setup_chunks(&mut manager);

    if let Err(e) = manager.combined.mesh.set_instances(&instance_data) {
        eprintln!("Failed to set combined greedy instances: {:?}", e);
    }

    manager
}

fn setup_chunks(manager: &mut ChunkManager) -> Vec<culled_voxel_combined::Instance> {
    let mut instance_data: Vec<culled_voxel_combined::Instance> = vec![];

    manager.combined.pos_order.clear();

    mesh_chunks(&manager.chunks);

    // Need another loop as we can't flush the buffer from another thread since the OpenGL context
    // is current on the main thread
    manager.chunks.iter().for_each(|e| {
        e.value().write_mesh();
    });

    for e in manager.chunks.iter() {
        let position = e.key();
        let chunk = e.value();

        let order = &mut manager.combined.pos_order;

        let instances = chunk.instances().read().unwrap();

        instance_data.extend(
            instances
                .iter()
                .map(|i| culled_voxel_combined::Instance { data: i.data }),
        );
        order.push((*position, instances.len()));
    }

    instance_data
}

pub struct ChunkManager {
    chunks: DashMap<IVec3, chunk::Chunk>,
    combined: CombinedInstanceData,
    outline_mesh: BasicMesh<renderer::draw::line::Vertex>,
    combine: bool,
    frustum_cull: bool,
}

pub struct CombinedInstanceData {
    pub pos_order: Vec<(IVec3, usize)>,
    chunk_data_buffer: ShaderBuffer<culled_voxel_combined::buffers::ChunkData>,
    indirect_buffer: GpuBuffer,
    mesh: NInstancedMesh<culled_voxel_combined::Vertex, culled_voxel_combined::Instance>,
}

impl ChunkManager {
    pub fn new(combine: bool, frustum_cull: bool) -> Self {
        let vertices = vec![
            culled_voxel_combined::Vertex::new([0, 0, 0]),
            culled_voxel_combined::Vertex::new([1, 0, 0]),
            culled_voxel_combined::Vertex::new([0, 0, 1]),
            culled_voxel_combined::Vertex::new([1, 0, 1]),
        ];

        let combined = {
            println!("Creating combined mesh");
            let mesh = NInstancedMesh::with_vertices(&vertices, None, DrawMode::TriangleStrip)
                .expect("Failed to create greedy ChunkManager mesh");
            println!("Creating chunk data buffer");
            let chunk_data_buffer =
                ShaderBuffer::new(&[]).expect("Failed to make shader buffer for chunk positions");
            println!("Creating indirect buffer");
            let indirect_buffer = GpuBuffer::empty(
                std::mem::size_of::<DrawArraysIndirectCommand>() * 100,
                BufferMode::Persistent,
            )
            .expect("Failed to make indirect buffer");

            CombinedInstanceData {
                mesh,
                pos_order: vec![],
                chunk_data_buffer,
                indirect_buffer,
            }
        };

        let outline_color = vec3(0.0, 0.0, 0.0);

        let outlines = [
            Line::new(vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), outline_color),
            Line::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), outline_color),
            Line::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), outline_color),
            Line::new(vec3(1.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0), outline_color),
            Line::new(vec3(1.0, 1.0, 1.0), vec3(1.0, 0.0, 1.0), outline_color),
            Line::new(vec3(1.0, 1.0, 1.0), vec3(1.0, 1.0, 0.0), outline_color),
            Line::new(vec3(0.0, 0.0, 1.0), vec3(0.0, 1.0, 1.0), outline_color),
            Line::new(vec3(1.0, 0.0, 0.0), vec3(1.0, 1.0, 0.0), outline_color),
            Line::new(vec3(0.0, 1.0, 0.0), vec3(1.0, 1.0, 0.0), outline_color),
            Line::new(vec3(0.0, 1.0, 0.0), vec3(0.0, 1.0, 1.0), outline_color),
            Line::new(vec3(0.0, 0.0, 1.0), vec3(1.0, 0.0, 1.0), outline_color),
            Line::new(vec3(1.0, 0.0, 1.0), vec3(1.0, 0.0, 0.0), outline_color),
        ]
        .iter()
        .flat_map(|l| l.to_vertices())
        .collect::<Vec<_>>();

        let outline_mesh =
            BasicMesh::from_data(&outlines, None, None, None, false, false, DrawMode::Lines);

        Self {
            chunks: DashMap::new(),
            combined,
            outline_mesh,
            frustum_cull,
            combine,
        }
    }
    pub fn get_block_at(&self, pos: &IVec3) -> BlockType {
        let (chunk_pos, in_chunk_pos) = seperate_global_pos(pos);

        let chunk = if let Some(chunk) = self.chunks.get(&chunk_pos) {
            chunk
        } else {
            return BlockType::Air;
        };

        chunk.get(
            in_chunk_pos.x as usize,
            in_chunk_pos.y as usize,
            in_chunk_pos.z as usize,
        )
    }
}

impl Renderable for ChunkManager {
    fn render(&mut self, state: &mut renderer::State) {
        renderer::profiler::event!("Greedy Render");
        if self.combine {
            render_combined(self, state);
        } else {
            render_seperate(self, state);
        }
    }

    fn cull(&mut self, cull: bool) {
        self.frustum_cull = cull;
        for e in self.chunks.iter() {
            e.value().set_frustum_culling(cull);
        }
    }

    fn combine(&mut self, combine: bool) {
        self.combine = combine;
    }
}

fn render_seperate(manager: &mut ChunkManager, state: &mut renderer::State) {
    manager.chunks.par_iter().for_each(|e| {
        let chunk = e.value();
        chunk.update(e.key(), &manager.chunks);
    });
    for e in manager.chunks.iter() {
        let pos = e.key();
        let chunk = e.value();
        let ipos = ivec3(pos[0], pos[1], pos[2]) * 32;

        chunk.write_mesh();

        let mut mesh = chunk.mesh().write().unwrap();
        let mesh = mesh.as_mut().expect("Chunk should of had a mesh");

        let uniforms = culled_voxel::Uniforms {
            chunk_position: ipos.to_array(),
        };

        let program = culled_voxel::Program::get();

        state.draw(mesh, &program, &uniforms)
    }
}

fn render_combined(manager: &mut ChunkManager, state: &mut renderer::State) {
    renderer::profiler::event!("Greedy Render Combined");
    let frustum = &state.cameras.game_frustum();

    manager.combined.mesh.bind();
    let program = culled_voxel_combined::Program::get();
    program.bind();

    if manager
        .chunks
        .par_iter()
        .any(|e| e.value().update(e.key(), &manager.chunks))
    {
        let instance_data = setup_chunks(manager);

        if let Err(e) = manager
            .combined
            .mesh
            .set_instances(instance_data.as_slice())
        {
            eprintln!("Failed to set combined greedy instances: {:?}", e);
        }
    }

    fn setup_multidraw(
        chunks: &DashMap<IVec3, Chunk>,
        combined: &CombinedInstanceData,
        frustum: &Frustum,
        cull: bool,
    ) -> (ChunkData, Vec<DrawArraysIndirectCommand>) {
        renderer::profiler::event!("Greedy setup multidraw");

        let mut instance = 0;

        let mut chunk_data = ChunkData { pos: vec![] };
        let mut draw_params = vec![];

        for (pos, count) in combined.pos_order.iter() {
            let chunk = if let Some(chunk) = chunks.get(pos) {
                chunk
            } else {
                instance += *count as u32;
                continue;
            };

            if cull && chunk.bounds().intersects(frustum).is_none() {
                instance += *count as u32;
                continue;
            }

            let vec = ivec3(pos[0], pos[1], pos[2]) * 32;

            chunk_data.pos.push(vec.to_array());

            let indirect = DrawArraysIndirectCommand {
                vertex_count: 4,
                instance_count: *count as u32,
                first: 0,
                base_instance: instance,
            };

            draw_params.push(indirect);

            instance += *count as u32;
        }

        (chunk_data, draw_params)
    }

    let (uniforms, draw_params) = setup_multidraw(
        &manager.chunks,
        &manager.combined,
        frustum,
        manager.frustum_cull,
    );

    fn set_chunk_data(chunk_data: ChunkData, combined: &mut CombinedInstanceData) {
        renderer::profiler::event!("Greedy set combined chunk data");
        if let Err(e) = combined.chunk_data_buffer.set_single(&chunk_data, 0) {
            eprintln!("Error setting chunk data: {:?}", e);
        }
    }
    set_chunk_data(uniforms, &mut manager.combined);
    manager.combined.chunk_data_buffer.bind();

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
    set_indirect_commands(draw_params, &mut manager.combined);

    unsafe {
        gl::BindBuffer(
            gl::DRAW_INDIRECT_BUFFER,
            manager.combined.indirect_buffer.id(),
        );
    }

    fn draw_combined(len: i32) {
        renderer::profiler::event!("Greedy multidraw");
        unsafe {
            gl::MultiDrawArraysIndirect(DrawMode::TriangleStrip.into(), std::ptr::null(), len, 0);
        }
    }

    draw_combined(len);

    if let Some(global_pos) = get_looked_at_block(state.cameras.active(), |pos: &IVec3| {
        manager.get_block_at(pos)
    }) {
        let (chunk_pos, in_chunk_pos) = seperate_global_pos(&global_pos);
        let mut pos = (vec3(
            chunk_pos[0] as f32,
            chunk_pos[1] as f32,
            chunk_pos[2] as f32,
        ) * 32.0)
            + vec3(
                in_chunk_pos[0] as f32,
                in_chunk_pos[1] as f32,
                in_chunk_pos[2] as f32,
            );

        let mut model = Mat4::IDENTITY;

        if pos.x < 0.0 {
            pos.x += 1.0;
        }

        if pos.y < 0.0 {
            pos.y += 1.0;
        }

        if pos.z < 0.0 {
            pos.z += 1.0;
        }

        model.w_axis.x = pos.x;
        model.w_axis.y = pos.y;
        model.w_axis.z = pos.z;

        let program = renderer::draw::line::Program::get();
        let uniforms = renderer::draw::line::Uniforms {
            model: Some(model.to_cols_array_2d()),
        };

        state.draw(&mut manager.outline_mesh, &program, &uniforms);

        if state.is_clicked(winit::event::MouseButton::Left) {
            if let Some(chunk) = manager.chunks.get_mut(&chunk_pos) {
                chunk.set(in_chunk_pos, BlockType::Air);
            }
        }
    }
}
