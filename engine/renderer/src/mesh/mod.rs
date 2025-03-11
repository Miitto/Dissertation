use render_common::Program;

use crate::{
    DrawMode, Uniforms,
    bounds::BoundingHeirarchy,
    buffers::{BufferError, EmptyVertex},
    camera::frustum::Frustum,
    vertex::Vertex,
};

pub mod basic;
pub mod ninstanced;

pub trait Mesh<V, I = EmptyVertex>
where
    V: Vertex,
    I: Vertex,
{
    fn set_bounds(&mut self, bounds: BoundingHeirarchy);

    fn bind(&self);
    fn draw_mode(&self) -> DrawMode;
    fn vertex_count(&self) -> usize;
    fn instance_count(&self) -> usize;
    fn is_instanced(&self) -> bool;
    fn has_indices(&self) -> bool;

    fn render(&mut self, frustum: &Frustum);
}
