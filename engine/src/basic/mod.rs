use renderer::{Renderable, State, buffers::Vao};
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
            renderables.push(Voxel::new([0, 0, 0]));
        }
        Scene::Cube => {
            renderables.reserve_exact(32 * 32 * 32);

            for x in 0..32 {
                for y in 0..32 {
                    for z in 0..32 {
                        renderables.push(Voxel::new([x, y, z]));
                    }
                }
            }
        }
        Scene::Plane => {
            renderables.reserve_exact(radius as usize * radius as usize * height as usize);

            for x in 0..radius as i32 {
                for z in 0..radius as i32 {
                    for y in 0..height as i32 {
                        renderables.push(Voxel::new([x, y, z]));
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
                        renderables.push(Voxel::new([x, y, z]));
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
    fn render(&self, state: &mut renderer::State) {
        if !self.instance {
            for voxel in &self.voxels {
                voxel.render(state);
            }
            return;
        }

        let vertices = Voxel::get_vertices();
        let indices = Voxel::get_indices();

        let instances = self.voxels.iter().map(|v| instanced_voxel::Instance {
            pos: v.get_position(),
        });

        let vao = Vao::new_instanced(
            &vertices,
            Some(&indices),
            renderer::DrawType::Static,
            renderer::DrawMode::Triangles,
            &instances.collect::<Vec<_>>(),
        );

        let program = instanced_voxel::Program::get();

        let uniforms = instanced_voxel::Uniforms {
            viewMatrix: state.cameras.active().get_view().to_cols_array_2d(),
            projectionMatrix: state.cameras.active().get_projection().to_cols_array_2d(),
        };

        renderer::draw::draw(&vao, &program, &uniforms);
    }
}
