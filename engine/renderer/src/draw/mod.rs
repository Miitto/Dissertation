use render_common::Program as P;
pub mod line;
use crate::{State, Uniforms as UniformsTrait, mesh::Mesh, vertex::Vertex as VertexTrait};

pub fn draw<M, U, V, I>(mesh: &mut M, program: &P, uniforms: &U, state: &State)
where
    V: VertexTrait,
    I: VertexTrait,
    M: Mesh<V, I>,
    U: UniformsTrait,
{
    let frustum = state.cameras.game_frustum();

    mesh.render(program, uniforms, &frustum);
}
