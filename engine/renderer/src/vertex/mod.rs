
pub use render_common::format;

pub use format::Attribute;
use format::VertexFormat;

pub enum AttrType {
    I8,
    U8,
}

pub trait Vertex: Copy + Sized {
    fn bindings() -> VertexFormat;
}
