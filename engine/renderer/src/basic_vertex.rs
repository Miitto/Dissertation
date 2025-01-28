use glium::implement_vertex;

#[derive(Debug, Copy, Clone)]
pub struct BasicVertex {
    pub position: [f32; 3],
}

impl BasicVertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: [x, y, z],
        }
    }
}

implement_vertex!(BasicVertex, position);
