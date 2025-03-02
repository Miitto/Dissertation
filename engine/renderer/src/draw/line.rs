use glam::Vec3;
use shaders::Program as _;

pub use line::{Program, Uniforms, Vertex};

use crate::Renderable;

pub struct Line {
    start: Vec3,
    end: Vec3,
    color: Vec3,
}

impl Line {
    pub fn new(start: Vec3, end: Vec3, color: Vec3) -> Self {
        Self { start, end, color }
    }

    pub fn to_vec(&self) -> Vec<Vec3> {
        vec![self.start, self.end]
    }

    pub fn to_vertices(&self) -> Vec<line::Vertex> {
        self.to_vec()
            .iter()
            .map(|v| line::Vertex {
                pos: v.to_array(),
                color: self.color.to_array(),
            })
            .collect()
    }
}

impl Renderable for Line {
    fn render(&self, state: &mut crate::State) {
        let uniforms = line::Uniforms {
            projectionMatrix: state.cameras.active().get_projection().to_cols_array_2d(),
            viewMatrix: state.cameras.active().get_view().to_cols_array_2d(),
        };

        let vertices = self.to_vertices();

        let vao = crate::buffers::Vao::new(
            &vertices,
            None,
            crate::DrawType::Static,
            crate::DrawMode::Lines,
        );

        let program = line::Program::get();

        crate::draw::draw(&vao, &program, &uniforms);
    }
}

crate::program!(line, {
    #vertex vert
    #fragment frag

    struct vIn {
        vec3 pos;
        vec3 color;
    }

    struct v2f {
        vec3 color;
    }

    uniform mat4 viewMatrix;
    uniform mat4 projectionMatrix;

    v2f vert(vIn i) {
        mat4 pv = projectionMatrix * viewMatrix;

        gl_Position = pv * vec4(i.pos, 1.0);

        v2f o;
        o.color = i.color;
        return o;
    }

    vec4 frag(v2f i) {
        return vec4(i.color, 1.0);
    }
}, true);
