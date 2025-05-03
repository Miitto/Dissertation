use std::{cell::RefCell, rc::Rc};

use glam::{ivec3, vec3};
use renderer::{
    DrawMode, ProgramSource, Renderable, SSBO,
    bounds::BoundingHeirarchy,
    buffers::{BlankVao, ShaderBuffer},
    mesh::{Mesh, basic::BasicMesh, ninstanced::NInstancedMesh},
};
use voxel::{Voxel, instanced_voxel, vertex_pull::buffers::BlockPosition};

use common::{Args, tests::test_scene};

mod voxel;

pub fn setup(args: &Args, render_type: BasicRenderType) -> VoxelManager {
    let data = test_scene(args);

    let basic_mesh = BasicMesh::from_data(
        Voxel::get_vertices().as_slice(),
        Some(&Voxel::get_indices()),
        None,
        None,
        false,
        false,
        DrawMode::Triangles,
    );

    let mesh = Rc::new(RefCell::new(basic_mesh));

    let renderables = data
        .into_iter()
        .map(|(pos, block)| Voxel::new(ivec3(pos[0], pos[1], pos[2]), block, mesh.clone()))
        .collect::<Vec<_>>();

    VoxelManager::new(renderables, render_type)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BasicRenderType {
    Basic,
    Instanced,
    VertexPull,
}

pub enum RenderData {
    None,
    Instance(NInstancedMesh<instanced_voxel::Vertex, instanced_voxel::Instance>),
    VertexPull(
        (
            BlankVao,
            ShaderBuffer<voxel::vertex_pull::buffers::BlockPosition>,
        ),
    ),
}

pub struct VoxelManager {
    voxels: Vec<Voxel>,
    data: RenderData,
}

impl VoxelManager {
    pub fn new(voxels: Vec<Voxel>, render_type: BasicRenderType) -> Self {
        let data = match render_type {
            BasicRenderType::Basic => RenderData::None,
            BasicRenderType::Instanced => setup_instanced(&voxels),
            BasicRenderType::VertexPull => setup_pull(&voxels),
        };

        Self { voxels, data }
    }
}

impl Renderable for VoxelManager {
    fn render(&mut self, state: &mut renderer::State) {
        match &mut self.data {
            RenderData::None => {
                for voxel in &mut self.voxels {
                    voxel.render(state);
                }
            }
            RenderData::Instance(mesh) => {
                let program = instanced_voxel::Program::get();

                state.draw(mesh, &program, &());
            }
            RenderData::VertexPull((vao, buffer)) => {
                let program = voxel::vertex_pull::Program::get();
                program.bind();
                buffer.bind();
                vao.bind();

                state.cameras.bind_camera_uniforms();

                unsafe {
                    gl::DrawArrays(
                        DrawMode::Triangles.into(),
                        0,
                        (self.voxels.len() * Voxel::get_indices().len()) as i32,
                    );
                }
            }
        }
    }

    fn args(&mut self, args: &Args) {
        if args.vertex_pull {
            if !matches!(self.data, RenderData::VertexPull(_)) {
                self.data = setup_pull(&self.voxels);
            }
        } else if args.combine {
            if !matches!(self.data, RenderData::Instance(_)) {
                self.data = setup_instanced(&self.voxels);
            }
        } else {
            self.data = RenderData::None;
        }
    }
}

fn setup_instanced(voxels: &[Voxel]) -> RenderData {
    let vertices = Voxel::get_vertices()
        .into_iter()
        .map(|v| instanced_voxel::Vertex {
            position: v.position,
        })
        .collect::<Vec<_>>();
    let indices = Voxel::get_indices();

    let mut mesh = NInstancedMesh::with_vertices(&vertices, Some(&indices), DrawMode::Triangles)
        .expect("Failed to create NInstancedMehs for instanced basic voxel");

    let instances = voxels
        .iter()
        .map(|v| {
            let pos = v.get_position();
            instanced_voxel::Instance {
                pos: [pos.x, pos.y, pos.z],
                block_type: v.block_type.into(),
            }
        })
        .collect::<Vec<_>>();

    let min_x = voxels
        .iter()
        .map(|v| v.get_position().x)
        .fold(i32::MAX, i32::min);
    let max_x = voxels
        .iter()
        .map(|v| v.get_position().x)
        .fold(i32::MIN, i32::max);
    let min_y = voxels
        .iter()
        .map(|v| v.get_position().y)
        .fold(i32::MAX, i32::min);
    let max_y = voxels
        .iter()
        .map(|v| v.get_position().y)
        .fold(i32::MIN, i32::max);
    let min_z = voxels
        .iter()
        .map(|v| v.get_position().z)
        .fold(i32::MAX, i32::min);
    let max_z = voxels
        .iter()
        .map(|v| v.get_position().z)
        .fold(i32::MIN, i32::max);

    let min_coord = vec3(min_x as f32, min_y as f32, min_z as f32);
    let max_coord = vec3(max_x as f32, max_y as f32, max_z as f32);

    let bounds = BoundingHeirarchy::from_min_max(min_coord, max_coord);

    mesh.set_bounds(bounds);
    mesh.set_instances(&instances)
        .expect("Failed to set instance data");

    RenderData::Instance(mesh)
}

fn setup_pull(voxels: &[Voxel]) -> RenderData {
    let positions: Vec<[i32; 4]> = voxels
        .iter()
        .map(|v| {
            let pos = v.get_position();
            let block_type = v.block_type;

            [pos.x, pos.y, pos.z, Into::<u32>::into(block_type) as i32]
        })
        .collect();

    let data = BlockPosition {
        block_data: positions,
    };

    let buffer = ShaderBuffer::single(&data).expect("Failed to create buffer");

    RenderData::VertexPull((BlankVao::default(), buffer))
}
