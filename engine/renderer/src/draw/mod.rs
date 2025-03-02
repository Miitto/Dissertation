use render_common::Program as P;
pub mod line;
use crate::{Uniforms as UniformsTrait, buffers::Vao, vertex::Vertex as VertexTrait};

pub fn draw<T, I, U>(vao: &Vao<T, I>, program: &P, uniforms: &U)
where
    T: VertexTrait,
    I: VertexTrait,
    U: UniformsTrait,
{
    program.bind();
    vao.bind();
    uniforms.bind(program);

    let size = vao.len() as i32;
    let mode = vao.mode.into();

    if vao.instanced() {
        let count = vao.instance_count();
        if vao.has_indices() {
            unsafe {
                gl::DrawElementsInstanced(mode, size, gl::UNSIGNED_INT, std::ptr::null(), count);
            }
        } else {
            unsafe {
                gl::DrawArraysInstanced(mode, 0, size, count);
            }
        }
    } else if vao.has_indices() {
        unsafe {
            gl::DrawElements(mode, size, gl::UNSIGNED_INT, std::ptr::null());
        }
    } else {
        unsafe {
            gl::DrawArrays(mode, 0, size);
        }
    }
}
