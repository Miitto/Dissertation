use std::{
    ops::{Deref, DerefMut},
    sync::RwLock,
};

use dashmap::DashMap;
use glam::IVec3;
use renderer::{
    Axis, DrawMode, ProgramSource, SSBO, Uniforms,
    bounds::BoundingHeirarchy,
    buffers::{BlankVao, ShaderBuffer},
    mesh::{Mesh, ninstanced::NInstancedMesh},
};

use crate::binary::common::{
    AxisDepths, CHUNK_SIZE, ChunkRefs, VoxelArray, VoxelArrayRef, VoxelRef, build_depths,
    make_faces,
};
use common::{BasicVoxel, BlockType, InstanceData, Voxel};

use super::voxel::{
    culled_voxel,
    culled_voxel_vertex_pull::{self, uses::vertex_pull_face_data::buffers::FaceData},
};

pub struct VoxelData {
    pub voxels: RwLock<Box<VoxelArray>>,
    pub depth_mask: RwLock<Option<Box<AxisDepths>>>,
}

impl VoxelData {
    pub fn new(voxels: Box<VoxelArray>) -> Self {
        Self {
            voxels: RwLock::new(voxels),
            depth_mask: RwLock::new(None),
        }
    }

    pub fn invalidate(&self) {
        *self.depth_mask.write().unwrap() = None;
    }

    pub fn set(&self, pos: &IVec3, block_type: &BlockType) {
        // If in chunk
        if pos.max_element() < CHUNK_SIZE as i32 && pos.min_element() >= 0 {
            let voxel =
                &mut self.voxels.write().unwrap()[pos.x as usize][pos.y as usize][pos.z as usize];
            voxel.set_type(*block_type);
        }

        if let Some(mask) = self.depth_mask.write().unwrap().as_mut() {
            assert!(pos.x >= -1 && pos.x < CHUNK_SIZE as i32 + 1);

            let x = (pos.x + 1) as usize;
            let y = (pos.y + 1) as usize;
            let z = (pos.z + 1) as usize;
            if block_type.is_solid() {
                mask[usize::from(Axis::X)][y][z] |= 1 << x;
                mask[usize::from(Axis::Y)][z][x] |= 1 << y;
                mask[usize::from(Axis::Z)][y][x] |= 1 << z;
            } else {
                mask[usize::from(Axis::X)][y][z] &= !(1 << x);
                mask[usize::from(Axis::Y)][z][x] &= !(1 << y);
                mask[usize::from(Axis::Z)][y][x] &= !(1 << z);
            }
        }
    }

    pub fn build_depths(&self, chunks: &DashMap<IVec3, Chunk>, position: &IVec3) {
        if self.depth_mask.read().unwrap().is_none() {
            macro_rules! get_chunk {
                ($chunk_name:ident, $block_name:ident,$pos:expr) => {
                    let $chunk_name = chunks.get($pos);
                    let $chunk_name = if let Some(ref chunk) = $chunk_name {
                        let voxels = chunk.voxels();
                        let read = voxels.voxels.read().expect("Failed to read blocks");

                        Some((chunk, voxels, read))
                    } else {
                        None
                    };
                    let $block_name = if let Some(ref read) = $chunk_name {
                        let read = &read.2;
                        read.as_ref()
                    } else {
                        &crate::binary::common::BLANK_VOXELS
                    };
                };
            }
            let chunk_lock = self.voxels.read().unwrap();
            let blocks_center = chunk_lock.as_ref();

            get_chunk!(
                chunk_x_neg,
                blocks_x_neg,
                &IVec3::new(position.x - 1, position.y, position.z)
            );
            get_chunk!(
                chunk_y_neg,
                blocks_y_neg,
                &IVec3::new(position.x, position.y - 1, position.z)
            );
            get_chunk!(
                chunk_z_neg,
                blocks_z_neg,
                &IVec3::new(position.x, position.y, position.z - 1)
            );
            get_chunk!(
                chunk_x_pos,
                blocks_x_pos,
                &IVec3::new(position.x + 1, position.y, position.z)
            );
            get_chunk!(
                chunk_y_pos,
                blocks_y_pos,
                &IVec3::new(position.x, position.y + 1, position.z)
            );
            get_chunk!(
                chunk_z_pos,
                blocks_z_pos,
                &IVec3::new(position.x, position.y, position.z + 1)
            );
            let x_pos = VoxelRef {
                voxels: blocks_x_pos,
                position: IVec3::new(position.x + 1, position.y, position.z),
            };

            let y_pos = VoxelRef {
                voxels: blocks_y_pos,
                position: IVec3::new(position.x, position.y + 1, position.z),
            };

            let z_pos = VoxelRef {
                voxels: blocks_z_pos,
                position: IVec3::new(position.x, position.y, position.z + 1),
            };

            let pos = VoxelArrayRef {
                x: x_pos,
                y: y_pos,
                z: z_pos,
            };

            let x_neg = VoxelRef {
                voxels: blocks_x_neg,
                position: IVec3::new(position.x - 1, position.y, position.z),
            };

            let y_neg = VoxelRef {
                voxels: blocks_y_neg,
                position: IVec3::new(position.x, position.y - 1, position.z),
            };

            let z_neg = VoxelRef {
                voxels: blocks_z_neg,
                position: IVec3::new(position.x, position.y, position.z - 1),
            };

            let neg = VoxelArrayRef {
                x: x_neg,
                y: y_neg,
                z: z_neg,
            };

            let chunks_center = VoxelRef {
                voxels: blocks_center,
                position: *position,
            };

            let refs = ChunkRefs {
                chunk: chunks_center,
                pos,
                neg,
            };

            let mask = build_depths(&refs);

            *self.depth_mask.write().unwrap() = Some(mask);
        }
    }
}

enum RenderData {
    None,
    Instance(NInstancedMesh<culled_voxel::Vertex, culled_voxel::Instance>),
    VertexPull(
        (
            BlankVao,
            Option<
                ShaderBuffer<
                    culled_voxel_vertex_pull::uses::vertex_pull_face_data::buffers::FaceData,
                >,
            >,
        ),
    ),
}

pub struct Chunk {
    voxels: VoxelData,
    bounds: RwLock<BoundingHeirarchy>,
    instances: RwLock<Vec<culled_voxel::Instance>>,
    render_data: RwLock<RenderData>,
    greedy: RwLock<bool>,
    needs_update: RwLock<bool>,
    needs_mesh_written: RwLock<bool>,
    frustum_cull: RwLock<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderType {
    Instance,
    VertexPull,
    None,
}

impl Chunk {
    fn new(
        voxels: Box<[[[BasicVoxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
        render_type: RenderType,
        greedy: bool,
        frustum_cull: bool,
    ) -> Self {
        let vertices = vec![
            culled_voxel::Vertex::new([0, 0, 0]),
            culled_voxel::Vertex::new([1, 0, 0]),
            culled_voxel::Vertex::new([0, 0, 1]),
            culled_voxel::Vertex::new([1, 0, 1]),
        ];

        let render_data = match render_type {
            RenderType::None => RenderData::None,
            RenderType::Instance => {
                let mut mesh =
                    NInstancedMesh::with_vertices(&vertices, None, DrawMode::TriangleStrip)
                        .expect("Failed to make chunk NInstancedMesh");
                mesh.set_bounds(BoundingHeirarchy::default());
                if frustum_cull {
                    mesh.enable_frustum_culling();
                }
                RenderData::Instance(mesh)
            }
            RenderType::VertexPull => {
                let vao = BlankVao::new();
                RenderData::VertexPull((vao, None))
            }
        };

        Self {
            voxels: VoxelData::new(voxels),
            render_data: RwLock::new(render_data),
            bounds: RwLock::new(BoundingHeirarchy::default()),
            instances: RwLock::new(vec![]),
            greedy: RwLock::new(greedy),
            needs_update: RwLock::new(true),
            needs_mesh_written: RwLock::new(false),
            frustum_cull: RwLock::new(frustum_cull),
        }
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockType {
        assert!(x < CHUNK_SIZE);
        assert!(y < CHUNK_SIZE);
        assert!(z < CHUNK_SIZE);

        self.voxels.voxels.read().unwrap()[x][y][z].get_type()
    }

    pub fn set(
        &self,
        pos: IVec3,
        block_type: BlockType,
        chunks: &DashMap<IVec3, Self>,
        chunk_pos: &IVec3,
        set_neighbours: bool,
    ) {
        if pos.max_element() >= CHUNK_SIZE as i32 || pos.min_element() < 0 {
            eprintln!("Coord: {:?} is outside of chunk", pos);
            return;
        }

        self.voxels.set(&pos, &block_type);

        if set_neighbours {
            if pos.x == 0 {
                if let Some(chunk) =
                    chunks.get(&IVec3::new(chunk_pos.x - 1, chunk_pos.y, chunk_pos.z))
                {
                    chunk
                        .voxels
                        .set(&IVec3::new(CHUNK_SIZE as i32, pos.y, pos.z), &block_type);
                }
            } else if pos.x == CHUNK_SIZE as i32 - 1 {
                if let Some(chunk) =
                    chunks.get(&IVec3::new(chunk_pos.x + 1, chunk_pos.y, chunk_pos.z))
                {
                    chunk.voxels.set(&IVec3::new(-1, pos.y, pos.z), &block_type);
                }
            }

            if pos.y == 0 {
                if let Some(chunk) =
                    chunks.get(&IVec3::new(chunk_pos.x, chunk_pos.y - 1, chunk_pos.z))
                {
                    chunk
                        .voxels
                        .set(&IVec3::new(pos.x, CHUNK_SIZE as i32, pos.z), &block_type);
                }
            } else if pos.y == CHUNK_SIZE as i32 - 1 {
                if let Some(chunk) =
                    chunks.get(&IVec3::new(chunk_pos.x, chunk_pos.y + 1, chunk_pos.z))
                {
                    chunk.voxels.set(&IVec3::new(pos.x, -1, pos.z), &block_type);
                }
            }

            if pos.z == 0 {
                if let Some(chunk) =
                    chunks.get(&IVec3::new(chunk_pos.x, chunk_pos.y, chunk_pos.z - 1))
                {
                    chunk
                        .voxels
                        .set(&IVec3::new(pos.x, pos.y, CHUNK_SIZE as i32), &block_type);
                }
            } else if pos.z == CHUNK_SIZE as i32 - 1 {
                if let Some(chunk) =
                    chunks.get(&IVec3::new(chunk_pos.x, chunk_pos.y, chunk_pos.z + 1))
                {
                    chunk.voxels.set(&IVec3::new(pos.x, pos.y, -1), &block_type);
                }
            }
        }

        self.invalidate();
    }

    pub fn fill(
        block_type: BlockType,
        render_type: RenderType,
        greedy: bool,
        frustum_cull: bool,
    ) -> Self {
        let voxels =
            Box::new([[[BasicVoxel::new(block_type); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]);
        Self::new(voxels, render_type, greedy, frustum_cull)
    }

    pub fn invalidate(&self) {
        *self.needs_update.write().unwrap() = true;
    }

    pub fn bounds(&self) -> BoundingHeirarchy {
        *self.bounds.read().unwrap()
    }

    pub fn update_bounds(&self, bounds: BoundingHeirarchy) {
        *self.bounds.write().unwrap() = bounds;
        if let RenderData::Instance(mesh) = self.render_data.write().unwrap().deref_mut() {
            mesh.set_bounds(bounds);
        }
    }

    pub fn set_frustum_culling(&self, cull: bool) {
        if let RenderData::Instance(mesh) = self.render_data.write().unwrap().deref_mut() {
            mesh.set_frustum_cull(cull);
        }
    }

    pub fn set_vertex_pull(&self, pull: bool) {
        let mut data = self.render_data.write().unwrap();
        let data = data.deref_mut();
        match data {
            RenderData::None => {}
            RenderData::Instance(_) => {
                if pull {
                    let vao = BlankVao::new();
                    *data = RenderData::VertexPull((vao, None));
                    self.invalidate();
                }
            }
            RenderData::VertexPull(_) => {
                if !pull {
                    let vertices = vec![
                        culled_voxel::Vertex::new([0, 0, 0]),
                        culled_voxel::Vertex::new([1, 0, 0]),
                        culled_voxel::Vertex::new([0, 0, 1]),
                        culled_voxel::Vertex::new([1, 0, 1]),
                    ];

                    let mut mesh =
                        NInstancedMesh::with_vertices(&vertices, None, DrawMode::TriangleStrip)
                            .expect("Failed to make chunk NInstancedMesh");
                    mesh.set_bounds(BoundingHeirarchy::default());
                    if *self.frustum_cull.read().unwrap() {
                        mesh.enable_frustum_culling();
                    }
                    *data = RenderData::Instance(mesh);
                }
            }
        }
    }

    pub fn update(&self, position: &IVec3, chunks: &DashMap<IVec3, Self>) -> bool {
        if !*self.needs_update.read().unwrap() {
            return false;
        }

        self.voxels.build_depths(chunks, position);

        let raw_faces = make_faces(
            chunks,
            position,
            self.voxels.depth_mask.read().unwrap().as_ref().unwrap(),
            *self.greedy.read().expect("Failed to read greedy"),
        );

        let mut instances = self.instances.write().unwrap();
        instances.clear();

        for face in raw_faces.iter() {
            let data = InstanceData::new(
                face.x,
                face.y,
                face.z,
                face.dir,
                face.width,
                face.height,
                face.block_type,
            )
            .rotate_on_dir();

            instances.push(culled_voxel::Instance { data: data.into() });
        }

        *self.needs_update.write().unwrap() = false;
        *self.needs_mesh_written.write().unwrap() = true;

        true
    }

    pub fn write_mesh(&self) -> bool {
        if !*self.needs_mesh_written.read().unwrap() {
            return false;
        }

        if self.instances.read().unwrap().is_empty() {
            return false;
        }

        match self.render_data.write().unwrap().deref_mut() {
            RenderData::None => {}
            RenderData::Instance(mesh) => {
                if let Err(e) = mesh.set_instances(self.instances.read().unwrap().as_slice()) {
                    eprintln!("Error: {:?}", e);
                    return false;
                }
            }
            RenderData::VertexPull((_, buffer)) => {
                let instances = self.instances.read().unwrap();
                let faces = instances.iter().map(|i| i.data).collect::<Vec<_>>();

                let face_data = FaceData { face_data: faces };

                if let Some(buf) = buffer {
                    if let Err(e) = buf.set_single(&face_data, 0) {
                        eprintln!("Error: {:?}", e);
                        return false;
                    }
                } else {
                    let mut buf = ShaderBuffer::single(&face_data)
                        .expect("Failed to create face data buffer");

                    buf.set_label("Chunk face data buffer");
                    *buffer = Some(buf);
                }
            }
        }

        *self.needs_mesh_written.write().unwrap() = false;

        true
    }

    pub fn voxels(&self) -> &VoxelData {
        &self.voxels
    }

    pub fn instances(&self) -> &RwLock<Vec<culled_voxel::Instance>> {
        &self.instances
    }

    pub fn render(&self, ipos: &IVec3, state: &mut renderer::State) {
        match self.render_data.write().as_mut().unwrap().deref_mut() {
            RenderData::None => {
                panic!("No render data");
            }
            RenderData::Instance(mesh) => {
                let uniforms = culled_voxel::Uniforms {
                    chunk_position: ipos.to_array(),
                };

                let program = culled_voxel::Program::get();

                state.draw(mesh, &program, &uniforms)
            }
            RenderData::VertexPull((vao, buffer)) => {
                if buffer.is_none() {
                    return;
                }

                let buffer = buffer.as_ref().unwrap();
                buffer.bind();

                let program = culled_voxel_vertex_pull::Program::get();
                program.bind();

                state.cameras.bind_camera_uniforms();

                vao.bind();

                let uniforms = culled_voxel::Uniforms {
                    chunk_position: ipos.to_array(),
                };
                uniforms.bind(&program);

                unsafe {
                    gl::DrawArrays(
                        DrawMode::Triangles.into(),
                        0,
                        (self.instances.read().unwrap().len() * 6) as i32,
                    );
                }
            }
        }
    }
}
