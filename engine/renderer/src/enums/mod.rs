pub use common::directions::*;

#[derive(Clone, Copy, Debug)]
pub enum DrawMode {
    Lines,
    Triangles,
    TriangleStrip,
    TriangleFan,
}

impl From<DrawMode> for gl::types::GLenum {
    fn from(draw_mode: DrawMode) -> gl::types::GLenum {
        match draw_mode {
            DrawMode::Lines => gl::LINES,
            DrawMode::Triangles => gl::TRIANGLES,
            DrawMode::TriangleStrip => gl::TRIANGLE_STRIP,
            DrawMode::TriangleFan => gl::TRIANGLE_FAN,
        }
    }
}
