use chunk::RenderType;
use rayon::prelude::*;

pub use chunk::Chunk;
use dashmap::DashMap;
use glam::{IVec3, Mat4, ivec3, vec3};
use rayon::iter::IntoParallelRefIterator;
use renderer::{
    DrawMode, ProgramSource, Renderable, SSBO, State,
    bounds::{BoundingHeirarchy, BoundingVolume},
    buffers::{BlankVao, Buffer, BufferMode, GpuBuffer, ShaderBuffer},
    camera::frustum::Frustum,
    draw::line::Line,
    indirect::DrawArraysIndirectCommand,
    mesh::{Mesh, ninstanced::NInstancedMesh},
};
use voxel::{
    combined_chunk_data::buffers::ChunkData,
    culled_voxel_combined::{self},
    culled_voxel_vertex_pull_combined,
    vertex_pull_face_data::buffers::FaceData,
};

use common::{
    Args, BlockType, seperate_global_pos,
    tests::{Test, test_scene},
};

use super::common::CHUNK_SIZE;

mod chunk;
mod voxel;

pub fn chunk_data(data: &DashMap<IVec3, BlockType>, args: &Args, chunks: &DashMap<IVec3, Chunk>) {
    data.into_iter().for_each(|e| {
        let (pos, block) = e.pair();
        let (chunk_pos, in_chunk_pos) = seperate_global_pos(pos);

        let chunk = chunks.entry(chunk_pos).or_insert(Chunk::fill(
            BlockType::Air,
            if args.combine {
                RenderType::None
            } else if args.vertex_pull {
                RenderType::VertexPull
            } else {
                RenderType::Instance
            },
            args.test == Test::Greedy,
            args.frustum_cull,
        ));

        chunk.set(in_chunk_pos, *block, chunks, &chunk_pos, false);
    });
}

pub fn mesh_chunks(chunks: &DashMap<IVec3, Chunk>) {
    chunks.par_iter().for_each(|e| {
        let position = e.key();
        let chunk = e.value();
        let pos =
            vec3(position[0] as f32, position[1] as f32, position[2] as f32) * (CHUNK_SIZE as f32);
        let end_pos = pos + (CHUNK_SIZE as f32);

        chunk.update(position, chunks);
        chunk.update_bounds(BoundingHeirarchy::from_min_max(pos, end_pos));
    });
}

pub fn setup(args: &Args, _state: &State) -> ChunkManager {
    let mut manager = ChunkManager::new(args.combine, args.frustum_cull, args.vertex_pull);

    let data = test_scene(args);

    chunk_data(&data, args, &manager.chunks);

    setup_chunks(&mut manager);

    manager
}

fn setup_chunks(manager: &mut ChunkManager) {
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

    match &mut manager.combined.render_data {
        RenderData::Instance(mesh) => {
            if let Err(e) = mesh.set_instances(&instance_data) {
                eprintln!("Error setting instances: {:?}", e);
            }
        }
        RenderData::VertexPull(_, buffer) => {
            let instance_data = FaceData {
                face_data: instance_data
                    .into_iter()
                    .map(|i| i.data)
                    .collect::<Vec<_>>(),
            };

            if let Some(buffer) = buffer {
                if let Err(e) = buffer.set_single(&instance_data, 0) {
                    eprintln!("Error setting vertex pull data: {:?}", e);
                }
            } else {
                let buf = ShaderBuffer::single(&instance_data)
                    .expect("Failed to make shader buffer for vertex pull data");
                *buffer = Some(buf);
                println!("Setup vertex pull buffer");
            }
        }
    }
}

pub struct ChunkManager {
    chunks: DashMap<IVec3, chunk::Chunk>,
    combined: CombinedData,
    combine: bool,
    frustum_cull: bool,
}

enum RenderData {
    Instance(NInstancedMesh<culled_voxel_combined::Vertex, culled_voxel_combined::Instance>),
    VertexPull(
        BlankVao,
        Option<
            ShaderBuffer<
                culled_voxel_vertex_pull_combined::uses::vertex_pull_face_data::buffers::FaceData,
            >,
        >,
    ),
}

pub struct CombinedData {
    pub pos_order: Vec<(IVec3, usize)>,
    chunk_data_buffer:
        ShaderBuffer<culled_voxel_combined::uses::combined_chunk_data::buffers::ChunkData>,
    indirect_buffer: GpuBuffer,
    render_data: RenderData,
}

impl CombinedData {
    pub fn bind(&self) {
        match &self.render_data {
            RenderData::Instance(mesh) => mesh.bind(),
            RenderData::VertexPull(vao, buffer) => {
                if let Some(buffer) = buffer {
                    vao.bind();
                    buffer.bind();
                }
            }
        }
    }

    pub fn is_vertex_pull(&self) -> bool {
        matches!(self.render_data, RenderData::VertexPull(_, _))
    }
}

impl ChunkManager {
    pub fn new(combine: bool, frustum_cull: bool, vertex_pull: bool) -> Self {
        let vertices = vec![
            culled_voxel_combined::Vertex::new([0, 0, 0]),
            culled_voxel_combined::Vertex::new([1, 0, 0]),
            culled_voxel_combined::Vertex::new([0, 0, 1]),
            culled_voxel_combined::Vertex::new([1, 0, 1]),
        ];

        let combined = {
            let render_data = if !vertex_pull {
                RenderData::Instance(
                    NInstancedMesh::with_vertices(&vertices, None, DrawMode::TriangleStrip)
                        .expect("Failed to create greedy ChunkManager mesh"),
                )
            } else {
                RenderData::VertexPull(BlankVao::new(), None)
            };
            let chunk_data_buffer =
                ShaderBuffer::new(&[]).expect("Failed to make shader buffer for chunk positions");
            let indirect_buffer = GpuBuffer::empty(
                std::mem::size_of::<DrawArraysIndirectCommand>() * 100,
                BufferMode::Persistent,
            )
            .expect("Failed to make indirect buffer");

            CombinedData {
                render_data,
                pos_order: vec![],
                chunk_data_buffer,
                indirect_buffer,
            }
        };

        Self {
            chunks: DashMap::new(),
            combined,
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

    fn args(&mut self, args: &Args) {
        self.frustum_cull = args.frustum_cull;
        for e in self.chunks.iter() {
            e.value().set_frustum_culling(args.frustum_cull);
            e.value().set_vertex_pull(args.vertex_pull);
        }

        self.combine = args.combine;

        match self.combined.render_data {
            RenderData::Instance(ref mut mesh) => {
                if args.vertex_pull {
                    self.combined.render_data = RenderData::VertexPull(BlankVao::new(), None);
                    setup_chunks(self);
                }
            }
            RenderData::VertexPull(_, _) => {
                if !args.vertex_pull {
                    let vertices = vec![
                        culled_voxel_combined::Vertex::new([0, 0, 0]),
                        culled_voxel_combined::Vertex::new([1, 0, 0]),
                        culled_voxel_combined::Vertex::new([0, 0, 1]),
                        culled_voxel_combined::Vertex::new([1, 0, 1]),
                    ];
                    self.combined.render_data = RenderData::Instance(
                        NInstancedMesh::with_vertices(&vertices, None, DrawMode::TriangleStrip)
                            .expect("Failed to create greedy ChunkManager mesh"),
                    );
                    setup_chunks(self);
                }
            }
        }
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
        let ipos = ivec3(pos[0], pos[1], pos[2]) * CHUNK_SIZE as i32;

        chunk.write_mesh();

        chunk.render(&ipos, state);
    }
}

fn render_combined(manager: &mut ChunkManager, state: &mut renderer::State) {
    renderer::profiler::event!("Greedy Render Combined");
    let frustum = &state.cameras.game_frustum();

    if let RenderData::VertexPull(_, None) = &mut manager.combined.render_data {
        println!("Vertex pull buffer not set up");
        return;
    }

    manager.combined.bind();
    let program = if manager.combined.is_vertex_pull() {
        culled_voxel_vertex_pull_combined::Program::get()
    } else {
        culled_voxel_combined::Program::get()
    };
    program.bind();

    if manager
        .chunks
        .par_iter()
        .any(|e| e.value().update(e.key(), &manager.chunks))
    {
        setup_chunks(manager);
    }

    fn setup_multidraw(
        chunks: &DashMap<IVec3, Chunk>,
        combined: &CombinedData,
        frustum: &Frustum,
        cull: bool,
    ) -> (ChunkData, Vec<DrawArraysIndirectCommand>) {
        renderer::profiler::event!("Greedy setup multidraw");

        let mut instance = 0;

        let mut chunk_data = ChunkData {
            chunk_positions: vec![],
        };
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

            let vec = ivec3(pos[0], pos[1], pos[2]) * CHUNK_SIZE as i32;

            chunk_data.chunk_positions.push(vec.to_array());

            let indirect = match combined.render_data {
                RenderData::Instance(_) => DrawArraysIndirectCommand {
                    vertex_count: 4,
                    instance_count: *count as u32,
                    first: 0,
                    base_instance: instance,
                },
                RenderData::VertexPull(_, _) => DrawArraysIndirectCommand {
                    vertex_count: 6 * *count as u32,
                    instance_count: 1,
                    first: 6 * instance,
                    base_instance: 0,
                },
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

    fn set_chunk_data(chunk_data: ChunkData, combined: &mut CombinedData) {
        renderer::profiler::event!("Greedy set combined chunk data");
        if let Err(e) = combined.chunk_data_buffer.set_single(&chunk_data, 0) {
            eprintln!("Error setting chunk data: {:?}", e);
        }
    }
    set_chunk_data(uniforms, &mut manager.combined);
    manager.combined.chunk_data_buffer.bind();

    fn set_indirect_commands(
        draw_params: Vec<DrawArraysIndirectCommand>,
        combined: &mut CombinedData,
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

    fn draw_combined(len: i32, vertex_pull: bool) {
        renderer::profiler::event!("Greedy multidraw");
        let draw_mode = if vertex_pull {
            DrawMode::Triangles
        } else {
            DrawMode::TriangleStrip
        };
        unsafe {
            gl::MultiDrawArraysIndirect(draw_mode.into(), std::ptr::null(), len, 0);
        }
    }

    draw_combined(len, manager.combined.is_vertex_pull());

    // if let Some(global_pos) = get_looked_at_block(state.cameras.active(), |pos: &IVec3| {
    //     manager.get_block_at(pos)
    // }) {
    //     let (chunk_pos, in_chunk_pos) = seperate_global_pos(&global_pos);
    //     let mut pos = (vec3(
    //         chunk_pos[0] as f32,
    //         chunk_pos[1] as f32,
    //         chunk_pos[2] as f32,
    //     ) * CHUNK_SIZE as f32)
    //         + vec3(
    //             in_chunk_pos[0] as f32,
    //             in_chunk_pos[1] as f32,
    //             in_chunk_pos[2] as f32,
    //         );
    //
    //     let mut model = Mat4::IDENTITY;
    //
    //     if pos.x < 0.0 {
    //         pos.x += 1.0;
    //     }
    //
    //     if pos.y < 0.0 {
    //         pos.y += 1.0;
    //     }
    //
    //     if pos.z < 0.0 {
    //         pos.z += 1.0;
    //     }
    //
    //     model.w_axis.x = pos.x;
    //     model.w_axis.y = pos.y;
    //     model.w_axis.z = pos.z;
    //
    //     let program = renderer::draw::line::Program::get();
    //     let uniforms = renderer::draw::line::Uniforms {
    //         model: Some(model.to_cols_array_2d()),
    //     };
    //
    //     state.draw(&mut manager.outline_mesh, &program, &uniforms);
    //
    //     if state.is_clicked(winit::event::MouseButton::Left) {
    //         if let Some(chunk) = manager.chunks.get_mut(&chunk_pos) {
    //             chunk.set(
    //                 in_chunk_pos,
    //                 BlockType::Air,
    //                 &manager.chunks,
    //                 &chunk_pos,
    //                 true,
    //             );
    //         }
    //     }
    // }
}
