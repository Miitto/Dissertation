#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[allow(dead_code)]
pub enum Scene {
    Single,
    Cube,
    Plane,
    Perlin,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Test {
    Basic,
    BasicInstanced,
    Chunk,
    Culled,
    Greedy,
}
