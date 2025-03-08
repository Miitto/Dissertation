use renderer::{Renderable, mesh::basic::BasicMesh};
use shaders::Program;

const VERTICES: [[f32; 2]; 3] = [[-0.5, -0.5], [0.0, 0.5], [0.5, -0.5]];

pub struct Triangle {
    mesh: BasicMesh<triangle::Vertex>,
}

impl Renderable for Triangle {
    fn render(&mut self, state: &mut renderer::State) {
        let uniforms = triangle::Uniforms {
            projectionMatrix: state.cameras.active().get_projection().to_cols_array_2d(),
            viewMatrix: state.cameras.active().get_view().to_cols_array_2d(),
        };
        let program = triangle::Program::get();

        renderer::draw::draw(&mut self.mesh, &program, &uniforms, state);
    }
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

    uniform mat4 projectionMatrix;
    uniform mat4 viewMatrix;

    struct vIn {
        vec2 pos;
    }

    struct v2f {
        vec4 color;
    }

    v2f vert(vIn i) {
        mat4 vp = projectionMatrix * viewMatrix;
        gl_Position = vp * vec4(i.pos, 1.0, 1.0);

        v2f o;
        o.color = vec4(1.0, 0.0, 0.0, 1.0);
        return o;
    }

    vec4 frag(v2f i) {
        return i.color;
    }
});
