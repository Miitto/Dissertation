use renderer::Renderable;

mod voxel;

pub fn get_renderables() -> Vec<Box<dyn Renderable>> {
    use voxel::Voxel;

    let mut renderables = Vec::new();
    renderables.reserve_exact(32 * 32 * 32);

    for x in 0..32 {
        for y in 0..32 {
            for z in 0..32 {
                renderables.push(Box::new(Voxel::new([x, y, z])) as Box<dyn Renderable>);
            }
        }
    }

    renderables
}
