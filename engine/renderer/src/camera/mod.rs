use crate::{
    Dir, DrawMode, Input, LayoutBlock, State, Transform, Uniforms,
    buffers::UniformBuffer,
    draw::line::{self, Line},
    mesh::{Mesh, basic::BasicMesh},
};
use frustum::FrustumCorners;
use glam::{Mat4, Vec3, vec3, vec4};

pub mod frustum;
mod perspective;

pub use perspective::PerspectiveCamera;
use render_common::Program;
use shaders::Program as _;
use winit::keyboard::KeyCode;

pub trait Camera: std::fmt::Debug {
    fn on_window_resize(&mut self, width: f32, height: f32);
    fn get_projection(&self) -> Mat4;
    fn get_view(&self) -> Mat4;
    fn transform(&self) -> &Transform;
    fn translate(&mut self, direction: Dir, delta: f32);
    fn rotate(&mut self, pitch_delta: f32, yaw_delta: f32, is_mouse: bool);
    fn handle_input(&mut self, keys: &Input, delta: f32);
    fn frustum(&self) -> frustum::Frustum;
    fn get_frustum_corners(&self) -> FrustumCorners;
    fn forward(&self) -> Vec3;
}

pub struct CameraManager {
    cameras: Vec<Box<dyn Camera>>,
    active_camera: usize,
    scene_camera: usize,
    game_camera: usize,
    base_camera_gizmo_mesh: BasicMesh<camera_gizmo::Vertex>,
    frustum_mesh: BasicMesh<line::Vertex>,

    camera_matrices_buffer: UniformBuffer<camera_matrices::uniforms::CameraMatrices>,
}

impl CameraManager {
    pub fn bind_camera_uniforms(&self, program: &Program) {
        self.camera_matrices_buffer.bind(program);
    }

    pub fn on_window_resize(&mut self, width: f32, height: f32) {
        for camera in &mut self.cameras {
            camera.on_window_resize(width, height);
        }
    }

    pub fn active(&self) -> &dyn Camera {
        self.cameras[self.active_camera].as_ref()
    }

    pub fn active_mut(&mut self) -> &mut dyn Camera {
        self.cameras[self.active_camera].as_mut()
    }

    pub fn scene(&self) -> &dyn Camera {
        self.cameras[self.scene_camera].as_ref()
    }

    pub fn game(&self) -> &dyn Camera {
        self.cameras[self.game_camera].as_ref()
    }

    pub fn game_frustum(&self) -> frustum::Frustum {
        self.cameras[self.game_camera].frustum()
    }

    pub fn handle_input(&mut self, keys: &Input, delta: f32) {
        if keys.is_pressed(&KeyCode::Digit1) {
            self.active_camera = self.scene_camera;
        }

        if keys.is_pressed(&KeyCode::Digit2) {
            self.active_camera = self.game_camera;
        }

        self.active_mut().handle_input(keys, delta);

        let active = self.active();
        let projection = active.get_projection();
        let view = active.get_view();

        let matrices = camera_matrices::uniforms::CameraMatrices {
            projection: projection.to_cols_array_2d(),
            inverse_projection: projection.inverse().to_cols_array_2d(),
            view: view.to_cols_array_2d(),
            inverse_view: view.inverse().to_cols_array_2d(),
            position: active.transform().position.to_array(),
        };

        if let Err(e) = self.camera_matrices_buffer.set_data(&matrices) {
            eprintln!("Error Setting CameraMatrices: {:?}", e);
        }
    }

    pub fn render_gizmos(state: &mut State) {
        if state.cameras.active_camera == state.cameras.game_camera {
            return;
        }

        CameraManager::render_other_cameras(state);
        CameraManager::render_game_frustum(state);
    }

    fn render_other_cameras(state: &mut State) {
        let program = camera_gizmo::Program::get();

        let model_mat = state.cameras.game().transform().to_mat4();

        let scaled = model_mat * Mat4::from_scale(Vec3::splat(0.25));

        let uniforms = camera_gizmo::Uniforms {
            modelMatrix: scaled.to_cols_array_2d(),
            color: vec4(0.75, 0.75, 0.75, 1.0).to_array(),
        };

        let frustum = state.cameras.game_frustum();

        program.bind();
        uniforms.bind(&program);
        state.cameras.bind_camera_uniforms(&program);

        state.cameras.base_camera_gizmo_mesh.render(&frustum);
    }

    fn render_game_frustum(state: &mut State) {
        let game = state.cameras.game();
        let corners = game.get_frustum_corners();

        let color = vec3(1., 1., 1.);

        let lines: Vec<crate::draw::line::Vertex> = [
            Line::new(corners.near_top_left, corners.near_top_right, color),
            Line::new(corners.near_top_right, corners.near_bottom_right, color),
            Line::new(corners.near_bottom_right, corners.near_bottom_left, color),
            Line::new(corners.near_bottom_left, corners.near_top_left, color),
            Line::new(corners.far_top_left, corners.far_top_right, color),
            Line::new(corners.far_top_right, corners.far_bottom_right, color),
            Line::new(corners.far_bottom_right, corners.far_bottom_left, color),
            Line::new(corners.far_bottom_left, corners.far_top_left, color),
            Line::new(corners.near_top_left, corners.far_top_left, color),
            Line::new(corners.near_top_right, corners.far_top_right, color),
            Line::new(corners.near_bottom_right, corners.far_bottom_right, color),
            Line::new(corners.near_bottom_left, corners.far_bottom_left, color),
        ]
        .iter()
        .flat_map(|l| l.to_vertices())
        .collect();

        let program = crate::draw::line::Program::get();

        let uniforms = crate::draw::line::Uniforms {};

        if let Err(e) = state.cameras.frustum_mesh.set_vertices(&lines) {
            eprintln!("Error setting vertices: {:?}", e);
        }

        let frustum = state.cameras.game_frustum();

        program.bind();
        uniforms.bind(&program);
        state.cameras.bind_camera_uniforms(&program);

        state.cameras.frustum_mesh.render(&frustum);
    }
}

impl Default for CameraManager {
    fn default() -> Self {
        // 0
        let fbl = camera_gizmo::Vertex {
            pos: [-0.25, -0.25, 0.],
        };
        // 1
        let ftl = camera_gizmo::Vertex {
            pos: [-0.25, 0.25, 0.],
        };
        // 2
        let ftr = camera_gizmo::Vertex {
            pos: [0.25, 0.25, 0.],
        };
        // 3
        let fbr = camera_gizmo::Vertex {
            pos: [0.25, -0.25, 0.],
        };
        // 4
        let bbl = camera_gizmo::Vertex {
            pos: [-0.5, -0.5, 2.],
        };
        // 5
        let btl = camera_gizmo::Vertex {
            pos: [-0.5, 0.5, 2.],
        };
        // 6
        let btr = camera_gizmo::Vertex {
            pos: [0.5, 0.5, 2.],
        };
        // 7
        let bbr = camera_gizmo::Vertex {
            pos: [0.5, -0.5, 2.],
        };

        let vertices = vec![fbl, ftl, ftr, fbr, bbl, btl, btr, bbr];

        let indices = vec![
            0, 1, 2, 2, 3, 0, // Front
            7, 6, 5, 5, 4, 7, // Back
            4, 5, 1, 1, 0, 4, // Left
            3, 2, 6, 6, 7, 3, // Right
            1, 5, 6, 6, 2, 1, // Top
            4, 0, 3, 3, 7, 4, // Bottom
        ];

        println!("Creating Camera Gizmo Mesh");
        let gizmo_mesh = BasicMesh::from_data(
            &vertices,
            Some(&indices),
            None,
            None,
            false,
            false,
            DrawMode::Triangles,
        );

        println!("Creating Frustum Mesh");
        let size = std::mem::size_of::<[line::line::Vertex; 12]>();
        let frustum_mesh = BasicMesh::empty(size, true, DrawMode::Lines);

        let default_camera = Box::new(PerspectiveCamera::default());

        let projection = default_camera.get_projection();
        let view = default_camera.get_view();

        let matrices = camera_matrices::uniforms::CameraMatrices {
            projection: projection.to_cols_array_2d(),
            inverse_projection: projection.inverse().to_cols_array_2d(),
            view: view.to_cols_array_2d(),
            inverse_view: view.inverse().to_cols_array_2d(),
            position: default_camera.transform().position.to_array(),
        };

        let cam_buf =
            UniformBuffer::new(matrices).expect("Failed to create camera matrices buffer");

        unsafe {
            gl::BindBufferBase(
                gl::UNIFORM_BUFFER,
                camera_matrices::uniforms::CameraMatrices::bind_point(),
                cam_buf.id(),
            )
        }

        Self {
            cameras: vec![default_camera, Box::new(PerspectiveCamera::default())],
            active_camera: 0,
            scene_camera: 0,
            game_camera: 1,
            base_camera_gizmo_mesh: gizmo_mesh,
            frustum_mesh,
            camera_matrices_buffer: cam_buf,
        }
    }
}

crate::program!(camera_gizmo, {
    #vertex vert
    #fragment frag

    struct vIn {
        vec3 pos;
    }

    struct v2f {
        vec4 color;
    }

    #snippet crate::camera_matrices

    uniform mat4 modelMatrix;
    uniform vec4 color;

    v2f vert(vIn i) {
        mat4 pv = camera.projection * camera.inverse_view;
        mat4 mvp = pv * modelMatrix;

        gl_Position = mvp * vec4(i.pos, 1.0);

        v2f o;
        o.color = color;
        return o;
    }

    vec4 frag(v2f i) {
        return i.color;
    }
}, true);

crate::snippet!(camera_matrices, {
    #bind 0
    uniform CameraMatrices {
        mat4 projection;
        mat4 inverse_projection;
        mat4 view;
        mat4 inverse_view;
        vec3 position;
    } camera;
}, true);
