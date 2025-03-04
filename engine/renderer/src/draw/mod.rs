use render_common::Program as P;
pub mod line;
use crate::{
    State, Uniforms as UniformsTrait, buffers::Vao, mesh::Mesh, vertex::Vertex as VertexTrait,
};

pub fn draw<T, I, U>(mesh: &Mesh<T, I>, program: &P, uniforms: &U, state: &State)
where
    T: VertexTrait,
    I: VertexTrait,
    U: UniformsTrait,
{
    if mesh.frustum_cull() && !mesh.is_on_frustum(&state.cameras.game_frustum()) {
        return;
    }

    program.bind();
    mesh.bind();
    uniforms.bind(program);

    let size = mesh.len() as i32;
    let mode = mesh.draw_mode.into();

    if mesh.instanced() {
        let count = mesh.instance_count();
        if mesh.has_indices() {
            unsafe {
                gl::DrawElementsInstanced(mode, size, gl::UNSIGNED_INT, std::ptr::null(), count);
            }
        } else {
            unsafe {
                gl::DrawArraysInstanced(mode, 0, size, count);
            }
        }
    } else if mesh.has_indices() {
        unsafe {
            gl::DrawElements(mode, size, gl::UNSIGNED_INT, std::ptr::null());
        }
    } else {
        unsafe {
            gl::DrawArrays(mode, 0, size);
        }
    }
}
