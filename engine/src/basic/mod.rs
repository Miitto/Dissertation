use renderer::Renderable;

use crate::tests::Test;

mod voxel;

pub fn setup(test: Test) -> VoxelManager {
    use voxel::Voxel;

    let mut renderables = Vec::new();
    match test {
        Test::Single => {
            renderables.push(Box::new(Voxel::new([0, 0, 0])) as Box<dyn Renderable>);
        }
        Test::Cube => {
            renderables.reserve_exact(32 * 32 * 32);

            for x in 0..32 {
                for y in 0..32 {
                    for z in 0..32 {
                        renderables.push(Box::new(Voxel::new([x, y, z])) as Box<dyn Renderable>);
                    }
                }
            }
        }
        Test::Plane(radius, height) => {
            renderables.reserve_exact(radius as usize * radius as usize * height as usize);

            for x in 0..radius as i32 {
                for z in 0..radius as i32 {
                    for y in 0..height as i32 {
                        renderables.push(Box::new(Voxel::new([x, y, z])) as Box<dyn Renderable>);
                    }
                }
            }
        }
        Test::Perlin(_radius) => {
            // TODO: Perlin noise
        }
    }

    VoxelManager::new(renderables)
}

pub struct VoxelManager {
    voxels: Vec<Box<dyn Renderable>>,
}

impl VoxelManager {
    pub fn new(voxels: Vec<Box<dyn Renderable>>) -> Self {
        Self { voxels }
    }
}

impl Renderable for VoxelManager {
    fn render(&self, state: &mut renderer::State) {
        optick::event!("Basic Voxel Render");
        for voxel in &self.voxels {
            voxel.render(state);
        }
    }
}
