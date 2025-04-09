use glam::Mat4;
use renderer::{ProgramSource, Renderable, buffers::ShaderBuffer, mesh::basic::BasicMesh};

pub mod basic;
pub mod binary;

const VERTICES: [[f32; 2]; 3] = [[-0.5, -0.5], [0.0, 0.5], [0.5, -0.5]];

pub struct Triangle {
    mesh: BasicMesh<triangle::Vertex>,
}

impl Renderable for Triangle {
    fn render(&mut self, state: &mut renderer::State) {
        let uniforms = triangle::uniforms::Test {
            val: Mat4::IDENTITY.to_cols_array_2d(),
        };

        let uniforms = ShaderBuffer::new(&[uniforms]).unwrap();
        uniforms.bind();

        let program = triangle::Program::get();

        let empty_uniforms = triangle::Uniforms {};

        state.draw(&mut self.mesh, &program, &empty_uniforms);
    }

    fn cull(&mut self, _cull: bool) {}

    fn combine(&mut self, _combine: bool) {}
}

pub fn setup() -> Box<dyn Renderable> {
    let vertices = VERTICES
        .iter()
        .map(|v| triangle::Vertex { pos: *v })
        .collect::<Vec<_>>();

    let mesh = BasicMesh::from_data(
        &vertices,
        None,
        None,
        None,
        false,
        false,
        renderer::DrawMode::Triangles,
    );

    Box::new(Triangle { mesh })
}

renderer::program!(triangle, {
    #vertex vert
    #fragment frag

    #bind 1
    uniform Test {
        mat4 val;
    } test;

    struct vIn {
        vec2 pos;
    }

    struct v2f {
        vec4 color;
    }

    v2f vert(vIn i) {
        mat4 vp = camera.projection * camera.view;
        gl_Position = vp * vec4(i.pos, -1.0, 1.0);

        v2f o;
        o.color = vec4(test.val[1].xyz, 1.0);
        return o;
    }

    vec4 frag(v2f i) {
        return i.color;
    }
});
