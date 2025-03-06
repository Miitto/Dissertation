use glam::vec3;
use renderer::{Renderable, bounds::BoundingHeirarchy, mesh::Mesh};
use shaders::Program;
use voxel::{Voxel, instanced_voxel};

use crate::{Args, tests::Scene};
use bracket_noise::prelude::*;

mod voxel;

pub fn setup(args: &Args, instance: bool) -> VoxelManager {
    use voxel::Voxel;

    let scene = args.scene;
    let radius = args.radius;
    let height = args.depth;

    let mut renderables = Vec::new();
    match scene {
        Scene::Single => {
            renderables.push(Voxel::new(vec3(0.0, 0., 0.)));
        }
        Scene::Cube => {
            renderables.reserve_exact(32 * 32 * 32);

            for x in 0..32 {
                for y in 0..32 {
                    for z in 0..32 {
                        renderables.push(Voxel::new(vec3(x as f32, y as f32, z as f32)));
                    }
                }
            }
        }
        Scene::Plane => {
            renderables.reserve_exact(radius as usize * radius as usize * height as usize);

            for x in -radius..radius {
                for z in -radius..radius {
                    for y in 0..height {
                        renderables.push(Voxel::new(vec3(x as f32, y as f32, z as f32)));
                    }
                }
            }
        }
        Scene::Perlin => {
            let mut noise = FastNoise::seeded(1234);
            noise.set_noise_type(NoiseType::Perlin);
            noise.set_frequency(0.1);

            let radius = (radius as u32) * 32;

            renderables.reserve_exact((radius * radius) as usize);

            for x in 0..radius as i32 {
                for z in 0..radius as i32 {
                    let height = (noise.get_noise(x as f32, z as f32) * (height as f32)) as i32;
                    for y in 0..height {
                        renderables.push(Voxel::new(vec3(x as f32, y as f32, z as f32)));
                    }
                }
            }
        }
    }

    VoxelManager::new(renderables, instance)
}

pub struct VoxelManager {
    voxels: Vec<Voxel>,
    instance: bool,
}

impl VoxelManager {
    pub fn new(voxels: Vec<Voxel>, instance: bool) -> Self {
        Self { voxels, instance }
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

        let vertices = Voxel::get_vertices();
        let indices = Voxel::get_indices();

        let instances = self.voxels.iter().map(|v| instanced_voxel::Instance {
            pos: v.get_position().to_array(),
        });

        let min_x = self
            .voxels
            .iter()
            .map(|v| v.get_position().x)
            .fold(f32::INFINITY, f32::min);
        let max_x = self
            .voxels
            .iter()
            .map(|v| v.get_position().x)
            .fold(f32::NEG_INFINITY, f32::max);
        let min_y = self
            .voxels
            .iter()
            .map(|v| v.get_position().y)
            .fold(f32::INFINITY, f32::min);
        let max_y = self
            .voxels
            .iter()
            .map(|v| v.get_position().y)
            .fold(f32::NEG_INFINITY, f32::max);
        let min_z = self
            .voxels
            .iter()
            .map(|v| v.get_position().z)
            .fold(f32::INFINITY, f32::min);
        let max_z = self
            .voxels
            .iter()
            .map(|v| v.get_position().z)
            .fold(f32::NEG_INFINITY, f32::max);

        let min_coord = vec3(min_x, min_y, min_z);
        let max_coord = vec3(max_x, max_y, max_z);

        let bounds = BoundingHeirarchy::from_min_max(min_coord, max_coord);

        let mesh = Mesh::new_instance(
            vertices,
            Some(indices),
            instances.collect::<Vec<_>>(),
            bounds,
            renderer::DrawMode::Triangles,
            renderer::DrawType::Static,
        );

        let program = instanced_voxel::Program::get();

        let uniforms = instanced_voxel::Uniforms {
            viewMatrix: state.cameras.active().get_view().to_cols_array_2d(),
            projectionMatrix: state.cameras.active().get_projection().to_cols_array_2d(),
        };

        renderer::draw::draw(&mesh, &program, &uniforms, state);
    }
}
