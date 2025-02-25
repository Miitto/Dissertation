mod directions;
pub use directions::*;

#[derive(Clone, Copy, Debug)]
pub enum DrawType {
    Static,
    Stream,
    Dynamic,
}

impl From<DrawType> for gl::types::GLenum {
    fn from(draw_type: DrawType) -> gl::types::GLenum {
        match draw_type {
            DrawType::Static => gl::STATIC_DRAW,
            DrawType::Stream => gl::STREAM_DRAW,
            DrawType::Dynamic => gl::DYNAMIC_DRAW,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DrawMode {
    Triangles,
    TriangleStrip,
    TriangleFan,
}

impl From<DrawMode> for gl::types::GLenum {
    fn from(draw_mode: DrawMode) -> gl::types::GLenum {
        match draw_mode {
            DrawMode::Triangles => gl::TRIANGLES,
            DrawMode::TriangleStrip => gl::TRIANGLE_STRIP,
            DrawMode::TriangleFan => gl::TRIANGLE_FAN,
        }
    }
}
