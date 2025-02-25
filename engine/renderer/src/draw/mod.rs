use render_common::Program;

use crate::{Uniforms, buffers::Vao, vertex::Vertex};

pub fn draw<T, I, U>(vao: &Vao<T, I>, program: &Program, uniforms: &U)
where
    T: Vertex,
    I: Vertex,
    U: Uniforms,
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
