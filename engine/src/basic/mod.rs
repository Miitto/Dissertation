use std::{cell::RefCell, rc::Rc};

use glam::{ivec3, vec3};
use renderer::{
    DrawMode, Renderable,
    bounds::BoundingHeirarchy,
    mesh::{Mesh, basic::BasicMesh, ninstanced::NInstancedMesh},
};
use shaders::Program;
use voxel::{Voxel, instanced_voxel};

use crate::{Args, tests::test_scene};

mod voxel;

pub fn setup(args: &Args, instance: bool) -> VoxelManager {
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

    VoxelManager::new(renderables, instance)
}

pub struct VoxelManager {
    voxels: Vec<Voxel>,
    instance: bool,
    mesh: Option<NInstancedMesh<instanced_voxel::Vertex, instanced_voxel::Instance>>,
}

impl VoxelManager {
    pub fn new(voxels: Vec<Voxel>, instance: bool) -> Self {
        let mesh = if instance {
            let vertices = Voxel::get_vertices()
                .into_iter()
                .map(|v| instanced_voxel::Vertex {
                    position: v.position,
                })
                .collect::<Vec<_>>();
            let indices = Voxel::get_indices();

            let mut mesh =
                NInstancedMesh::with_vertices(&vertices, Some(&indices), DrawMode::Triangles)
                    .expect("Failed to create NInstancedMehs for instanced basic voxel");

            let instances = voxels
                .iter()
                .map(|v| {
                    println!("Pos: {} | Block: {:?}", v.get_position(), v.block_type);
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

            Some(mesh)
        } else {
            None
        };

        Self {
            voxels,
            instance,
            mesh,
        }
    }
}

impl Renderable for VoxelManager {
    fn render(&mut self, state: &mut renderer::State) {
        if !self.instance {
            for voxel in &mut self.voxels {
                voxel.render(state);
            }
            return;
        }

        let mesh = self.mesh.as_mut().unwrap();

        let program = instanced_voxel::Program::get();

        let uniforms = instanced_voxel::Uniforms {
            viewMatrix: state.cameras.active().get_view().to_cols_array_2d(),
            projectionMatrix: state.cameras.active().get_projection().to_cols_array_2d(),
        };

        state.draw(mesh, &program, &uniforms);
    }
}
