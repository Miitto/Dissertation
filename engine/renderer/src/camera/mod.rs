use crate::{Dir, DrawMode, DrawType, Input, State, buffers::Vao, draw::line::Line};
use frustum::FrustumCorners;
use glam::{Mat4, Vec3, vec3, vec4};

pub mod frustum;
mod perspective;

pub use perspective::PerspectiveCamera;
use shaders::Program;
use winit::keyboard::KeyCode;

pub trait Camera: std::fmt::Debug {
    fn on_window_resize(&mut self, width: f32, height: f32);
    fn get_projection(&self) -> Mat4;
    fn get_view(&self) -> Mat4;
    fn get_position(&self) -> Vec3;
    fn get_rotation(&self) -> glam::Quat;
    fn translate(&mut self, direction: Dir, delta: f32);
    fn rotate(&mut self, pitch_delta: f64, yaw_delta: f64, is_mouse: bool);
    fn handle_input(&mut self, keys: &Input, delta: f32);
    fn frustum(&self) -> frustum::Frustum;
    fn get_frustum_corners(&self) -> FrustumCorners;
}

pub struct CameraManager {
    cameras: Vec<Box<dyn Camera>>,
    active_camera: usize,
    scene_camera: usize,
    game_camera: usize,
}

impl CameraManager {
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
    }

    pub fn inactive(&self) -> impl Iterator<Item = &Box<dyn Camera>> {
        self.cameras
            .iter()
            .enumerate()
            .filter(|(idx, _)| {
                *idx != self.active_camera && *idx != self.scene_camera && *idx != self.game_camera
            })
            .map(|(_, camera)| camera)
    }

    pub fn render_gizmos(&self, state: &State) {
        if self.active_camera == self.game_camera {
            return;
        }

        self.render_other_cameras();
        self.render_game_frustum();
    }

    fn render_other_cameras(&self) {
        let projection = self.active().get_projection().to_cols_array_2d();
        let view = self.active().get_view().to_cols_array_2d();

        let program = camera_gizmo::Program::get();

        // 0
        let fbl = camera_gizmo::Vertex {
            pos: [0.25, 0.25, 0.],
        };
        // 1
        let ftl = camera_gizmo::Vertex {
            pos: [0.25, 0.75, 0.],
        };
        // 2
        let ftr = camera_gizmo::Vertex {
            pos: [0.75, 0.75, 0.],
        };
        // 3
        let fbr = camera_gizmo::Vertex {
            pos: [0.75, 0.25, 0.],
        };
        // 4
        let bbl = camera_gizmo::Vertex { pos: [0., 0., 2.] };
        // 5
        let btl = camera_gizmo::Vertex { pos: [0., 1., 2.] };
        // 6
        let btr = camera_gizmo::Vertex { pos: [1., 1., 2.] };
        // 7
        let bbr = camera_gizmo::Vertex { pos: [1., 0., 2.] };

        let vertices = [fbl, ftl, ftr, fbr, bbl, btl, btr, bbr];

        let indices = [
            0, 1, 2, 2, 3, 0, // Front
            7, 6, 5, 5, 4, 7, // Back
            4, 5, 1, 1, 0, 4, // Left
            3, 2, 6, 6, 7, 3, // Right
            1, 5, 6, 6, 2, 1, // Top
            4, 0, 3, 3, 7, 4, // Bottom
        ];

        let vao = Vao::new(
            &vertices,
            Some(&indices),
            DrawType::Static,
            DrawMode::Triangles,
        );

        let pos = self.game().get_position();
        let model_mat = Mat4::from_rotation_translation(self.game().get_rotation(), pos);

        let scaled = model_mat * Mat4::from_scale(Vec3::splat(0.1));

        let model_matrix = scaled.to_cols_array_2d();
        let uniforms = camera_gizmo::Uniforms {
            viewMatrix: view,
            projectionMatrix: projection,
            modelMatrix: model_matrix,
            color: vec4(1., 1., 1., 1.0).to_array(),
        };

        crate::draw::draw(&vao, &program, &uniforms);

        for inactive in self.inactive() {
            let pos = inactive.get_position();
            let model_mat = Mat4::from_rotation_translation(inactive.get_rotation(), pos);

            let scaled = model_mat * Mat4::from_scale(Vec3::splat(0.1));

            let model_matrix = scaled.to_cols_array_2d();
            let uniforms = camera_gizmo::Uniforms {
                viewMatrix: view,
                projectionMatrix: projection,
                modelMatrix: model_matrix,
                color: vec4(0.75, 0.75, 0.75, 1.0).to_array(),
            };

            crate::draw::draw(&vao, &program, &uniforms);
        }
    }

    fn render_game_frustum(&self) {
        let game = self.game();
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

        let vao = crate::buffers::Vao::new(&lines, None, DrawType::Static, DrawMode::Lines);
        let program = crate::draw::line::Program::get();

        let uniforms = crate::draw::line::Uniforms {
            projectionMatrix: self.active().get_projection().to_cols_array_2d(),
            viewMatrix: self.active().get_view().to_cols_array_2d(),
        };

        crate::draw::draw(&vao, &program, &uniforms);
    }
}

impl Default for CameraManager {
    fn default() -> Self {
        Self {
            cameras: vec![
                Box::new(PerspectiveCamera::default()),
                Box::new(PerspectiveCamera::default()),
            ],
            active_camera: 0,
            scene_camera: 0,
            game_camera: 1,
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

    uniform mat4 projectionMatrix;
    uniform mat4 viewMatrix;
    uniform mat4 modelMatrix;
    uniform vec4 color;

    v2f vert(vIn i) {
        mat4 pv = projectionMatrix * viewMatrix;
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
