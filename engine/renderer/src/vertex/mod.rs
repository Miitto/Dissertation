use format::AttributeType;

pub mod format;

use format::VertexFormat;

pub enum AttrType {
    I8,
    U8,
}

pub trait Vertex: Copy + Sized {
    fn bindings() -> VertexFormat;
}

// From https://github.com/glium/glium/blob/master/src/vertex/mod.rs

/// Trait for types that can be used as vertex attributes.
///
/// # Safety
///
pub unsafe trait Attribute: Sized {
    /// The type of data.
    const TYPE: AttributeType;

    #[inline]
    /// Get the type of data.
    fn get_type() -> AttributeType {
        Self::TYPE
    }
}
