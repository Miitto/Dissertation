use glam::{EulerRot, Mat4, Quat, Vec3};
use winit::keyboard::KeyCode;

use crate::{Input, Transform, math::perspective};

use super::{
    Camera,
    frustum::{FrustumCorners, Plane},
};

#[derive(Debug)]
pub struct PerspectiveCamera {
    transform: Transform,
    fov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
    speed: f32,
    key_sensitivity: f32,
    mouse_sensitivity: f32,
    invert_mouse: bool,
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            fov: 90.0,
            aspect_ratio: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
            speed: 0.01,
            key_sensitivity: 0.025,
            mouse_sensitivity: 0.01,
            invert_mouse: false,
        }
    }
}

impl Camera for PerspectiveCamera {
    fn on_window_resize(&mut self, width: f32, height: f32) {
        self.aspect_ratio = width / height;
    }

    fn get_projection(&self) -> Mat4 {
        perspective(self.fov, self.aspect_ratio, self.near, self.far)
    }

    fn get_view(&self) -> Mat4 {
        self.transform.into()
    }

    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn translate(&mut self, direction: crate::Dir, delta: f32) {
        let movement = delta * self.speed;

        let forward = self.transform().flat_forward();
        let right = self.transform().right();

        use crate::Dir::*;
        match direction {
            Forward => {
                self.transform.position += forward * movement;
            }
            Backward => {
                self.transform.position -= forward * movement;
            }
            Left => {
                self.transform.position -= right * movement;
            }
            Right => {
                self.transform.position += right * movement;
            }
            // Fix to Z axis for vertical move
            Up => {
                self.transform.position.y += movement;
            }
            Down => {
                self.transform.position.y -= movement;
            }
        }
    }

    fn rotate(&mut self, mut pitch_delta: f32, yaw_delta: f32, is_mouse: bool) {
        if is_mouse && !self.invert_mouse {
            pitch_delta *= -1.;
        }

        let sensitivity = if is_mouse {
            self.mouse_sensitivity
        } else {
            self.key_sensitivity
        };

        let (x, y, _z) = self.transform.rotation.to_euler(EulerRot::YXZ);

        let y = y + (pitch_delta * sensitivity);
        let mut x = x - (yaw_delta * sensitivity);

        const PI: f32 = std::f32::consts::PI;

        let y = y.clamp((-PI / 2.) + 0.0001, (PI / 2.) - 0.0001);

        const MAX_VAL: f32 = PI;
        const MIN_VAL: f32 = -PI;

        if x >= MAX_VAL {
            x = (MIN_VAL + (x - MAX_VAL)) + 0.001;
        } else if y <= MIN_VAL {
            x = (MAX_VAL + (x - MIN_VAL)) - 0.001;
        }

        self.transform.rotation = Quat::from_euler(EulerRot::YXZ, x, y, 0.0);
    }

    fn handle_input(&mut self, input: &Input, delta: f32) {
        if input.is_pressed_no_repeat(&KeyCode::F5) {
            println!("Pos: {}", self.transform.position);
        }

        let scroll = input.wheel();

        self.mouse_sensitivity += 0.001 * scroll;

        if input.is_pressed(&KeyCode::KeyW) {
            self.translate(crate::Dir::Forward, delta);
        }

        if input.is_pressed(&KeyCode::KeyS) {
            self.translate(crate::Dir::Backward, delta);
        }

        if input.is_pressed(&KeyCode::KeyA) {
            self.translate(crate::Dir::Left, delta);
        }

        if input.is_pressed(&KeyCode::KeyD) {
            self.translate(crate::Dir::Right, delta);
        }

        if input.is_pressed(&KeyCode::Space) {
            self.translate(crate::Dir::Up, delta);
        }

        if input.is_pressed(&KeyCode::ControlLeft) {
            self.translate(crate::Dir::Down, delta);
        }

        if input.is_pressed(&KeyCode::ArrowLeft) {
            self.rotate(0., -1., false);
        }
        if input.is_pressed(&KeyCode::ArrowRight) {
            self.rotate(0., 1., false);
        }
        if input.is_pressed(&KeyCode::ArrowUp) {
            self.rotate(1., 0., false);
        }
        if input.is_pressed(&KeyCode::ArrowDown) {
            self.rotate(-1., 0., false);
        }

        if input.is_cursor_locked() {
            let mouse = input.mouse_pos();
            self.rotate(mouse.y(), mouse.x(), true);
        }
    }

    fn frustum(&self) -> crate::camera::frustum::Frustum {
        // https://learnopengl.com/Guest-Articles/2021/Scene/Frustum-Culling

        let half_v = self.far * (self.fov.to_radians() / 2.0).tan();
        let half_h = half_v * self.aspect_ratio;

        let cam_front = self.transform().forward();
        let cam_right = self.transform().right();
        let cam_up = self.transform().up();

        let front_mul_far = self.far * cam_front;

        let pos = self.transform.position;

        let near = Plane::new(pos + self.near * cam_front, cam_front);
        let far = Plane::new(pos + front_mul_far, -cam_front);

        let right = Plane::new(pos, (front_mul_far - cam_right * half_h).cross(cam_up));
        let left = Plane::new(pos, cam_up.cross(front_mul_far + cam_right * half_h));

        let top = Plane::new(pos, cam_right.cross(front_mul_far - cam_up * half_v));
        let bottom = Plane::new(pos, (front_mul_far + cam_up * half_v).cross(cam_right));

        crate::camera::frustum::Frustum::new(top, bottom, left, right, near, far)
    }

    fn get_frustum_corners(&self) -> FrustumCorners {
        let front = self.transform().forward();
        let up = self.transform().up();
        let right = self.transform().right();

        let fov = self.fov.to_radians();

        let near_height = (fov / 2.0).tan() * self.near;
        let near_width = near_height * self.aspect_ratio;

        let far_height = (fov / 2.0).tan() * self.far;
        let far_width = far_height * self.aspect_ratio;

        let near_center = self.transform().position + front * self.near;
        let far_center = self.transform().position + front * self.far;

        let near_tb = up * near_height;
        let near_lr = right * near_width;
        let near_top = near_center + near_tb;
        let near_bottom = near_center - near_tb;

        let near_top_left = near_top - near_lr;
        let near_top_right = near_top + near_lr;
        let near_bottom_left = near_bottom - near_lr;
        let near_bottom_right = near_bottom + near_lr;

        let far_tb = up * far_height;
        let far_lr = right * far_width;
        let far_top = far_center + far_tb;
        let far_bottom = far_center - far_tb;

        let far_top_left = far_top - far_lr;
        let far_top_right = far_top + far_lr;
        let far_bottom_left = far_bottom - far_lr;
        let far_bottom_right = far_bottom + far_lr;

        FrustumCorners {
            near_top_left,
            near_top_right,
            near_bottom_left,
            near_bottom_right,
            far_top_left,
            far_top_right,
            far_bottom_left,
            far_bottom_right,
        }
    }

    fn forward(&self) -> Vec3 {
        self.transform().forward()
    }
}
