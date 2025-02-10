#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Test {
    Single,
    Cube,
    Plane(u8, u8),
    Perlin(u8),
}
