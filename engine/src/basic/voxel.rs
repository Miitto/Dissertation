use glam::{Mat4, mat4, vec4};
use glium::{DrawParameters, uniform};
use renderer::{Renderable, State};
use shaders::Program;

pub struct Voxel {
    position: [i32; 3],
}

impl Voxel {
    pub fn new(position: [i32; 3]) -> Self {
        Self { position }
    }
}

impl basic_voxel::Vertex {
    pub fn new(x: f32, y: f32, z: f32, color: [f32; 4]) -> Self {
        Self {
            position: [x, y, z],
            color,
        }
    }
}

impl Voxel {
    pub fn get_vertices() -> Box<[basic_voxel::Vertex]> {
        // 0
        let fbl = basic_voxel::Vertex::new(0., 0., 0., [1., 0., 0., 1.]);
        // 1
        let ftl = basic_voxel::Vertex::new(0., 1., 0., [1., 0., 0., 1.]);
        // 2
        let ftr = basic_voxel::Vertex::new(1., 1., 0., [1., 0., 0., 1.]);
        // 3
        let fbr = basic_voxel::Vertex::new(1., 0., 0., [1., 0., 0., 1.]);
        // 4
        let bbl = basic_voxel::Vertex::new(0., 0., 1., [0., 1., 0., 1.]);
        // 5
        let btl = basic_voxel::Vertex::new(0., 1., 1., [0., 1., 0., 1.]);
        // 6
        let btr = basic_voxel::Vertex::new(1., 1., 1., [0., 1., 0., 1.]);
        // 7
        let bbr = basic_voxel::Vertex::new(1., 0., 1., [0., 1., 0., 1.]);

        Box::new([fbl, ftl, ftr, fbr, bbl, btl, btr, bbr])
    }

    pub fn get_indices() -> Box<[u8]> {
        Box::new([
            0, 1, 2, 2, 3, 0, // Front
            7, 6, 5, 5, 4, 7, // Back
            4, 5, 1, 1, 0, 4, // Left
            3, 2, 6, 6, 7, 3, // Right
            1, 5, 6, 6, 2, 1, // Top
            4, 0, 3, 3, 7, 4, // Bottom
        ])
    }

    pub fn get_model_matrix(&self) -> Mat4 {
        let position = vec4(
            self.position[0] as f32,
            self.position[1] as f32,
            self.position[2] as f32,
            1.0,
        );

        let forward = vec4(0., 0., -1., 0.);
        let right = vec4(1., 0., 0., 0.);
        let up = vec4(0., 1., 0., 0.);

        mat4(right, up, forward, position)
    }
}

impl Renderable for Voxel {
    fn render(&self, state: &mut State) {
        let display = if let Some(display) = &state.display {
            display
        } else {
            return;
        };

        let vertices = Voxel::get_vertices();
        let indices = Voxel::get_indices();

        let uniforms = basic_voxel::Uniforms {
            modelMatrix: self.get_model_matrix().to_cols_array_2d(),
            viewMatrix: state.camera.get_view().to_cols_array_2d(),
            projectionMatrix: state.camera.get_projection().to_cols_array_2d(),
        };

        let uniforms = uniform! {
            modelMatrix: uniforms.modelMatrix,
            viewMatrix: uniforms.viewMatrix,
            projectionMatrix: uniforms.projectionMatrix,
        };
        let v_buf =
            glium::VertexBuffer::new(display, &vertices).expect("Failed to make tri v buffer");

        let indices = glium::IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &indices,
        )
        .expect("Failed to make indices buffer");

        let program = basic_voxel::Program::get(display).expect("Failed to make shader");

        let draw_parameters = DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLessOrEqual,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise,
            ..Default::default()
        };

        _ = state.draw(&v_buf, &indices, &program, &uniforms, &draw_parameters);
    }
}

shaders::program!(basic_voxel, {
#vertex vertex
#fragment frag

uniform mat4 modelMatrix;
uniform mat4 viewMatrix;
uniform mat4 projectionMatrix;

struct vIn {
    vec3 position;
    vec4 color;
}

struct v2f {
    vec4 vertexColor;
}

v2f vertex(vIn i) {
    v2f o;
    mat4 pv = projectionMatrix * viewMatrix;

    mat4 mvp = pv * modelMatrix;

    o.vertexColor = i.color;

    gl_Position = mvp * vec4(i.position, 1.0);
    return o;
}

vec4 frag(v2f i) {
    return i.vertexColor;
}
});
