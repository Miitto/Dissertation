use glam::{DQuat, EulerRot, Mat4, Vec3, vec3};
use glium::winit::keyboard::KeyCode;

use crate::{Input, math::perspective};

use super::Camera;

#[derive(Debug)]
pub struct PerspectiveCamera {
    position: Vec3,
    fov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
    rotation: DQuat,
    speed: f32,
    key_sensitivity: f32,
    mouse_sensitivity: f32,
    invert_mouse: bool,
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            fov: 90.0,
            aspect_ratio: 16.0 / 9.0,
            near: 0.1,
            far: 100.0,
            rotation: DQuat::IDENTITY,
            speed: 0.01,
            key_sensitivity: 0.1,
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
        let rotation = self.rotation;

        let matrix = Mat4::from_rotation_translation(rotation.as_quat(), self.position);

        matrix.inverse()
    }

    fn get_position(&self) -> Vec3 {
        self.position
    }

    fn translate(&mut self, direction: crate::Dir, delta: f32) {
        let movement = delta * self.speed;
        let rotation = self.rotation.as_quat();

        let up = vec3(0., 1., 0.);
        let look_at = rotation * vec3(0., 0., 1.);
        let forward = vec3(look_at.x, 0., look_at.z).normalize();
        let right = up.cross(forward).normalize();

        use crate::Dir::*;
        match direction {
            Forward => {
                self.position -= forward * movement;
            }
            Backward => {
                self.position += forward * movement;
            }
            Left => {
                self.position -= right * movement;
            }
            Right => {
                self.position += right * movement;
            }
            // Fix to Z axis for vertical move
            Up => {
                self.position.y += movement;
            }
            Down => {
                self.position.y -= movement;
            }
        }
    }

    fn rotate(&mut self, mut pitch_delta: f64, yaw_delta: f64, is_mouse: bool) {
        if is_mouse && !self.invert_mouse {
            pitch_delta *= -1.;
        }

        let sensitivity = if is_mouse {
            self.mouse_sensitivity
        } else {
            self.key_sensitivity
        };

        let (x, y, _z) = self.rotation.to_euler(EulerRot::YXZ);

        let y = y + (pitch_delta * sensitivity as f64);
        let mut x = x - (yaw_delta * sensitivity as f64);

        const PI: f64 = std::f64::consts::PI;

        let y = y.clamp((-PI / 2.) + 0.0001, (PI / 2.) - 0.0001);

        const MAX_VAL: f64 = PI;
        const MIN_VAL: f64 = -PI;

        if x >= MAX_VAL {
            x = (MIN_VAL + (x - MAX_VAL)) + 0.001;
        } else if y <= MIN_VAL {
            x = (MAX_VAL + (x - MIN_VAL)) - 0.001;
        }

        self.rotation = DQuat::from_euler(EulerRot::YXZ, x, y, 0.0);
    }

    fn handle_input(&mut self, input: &Input, delta: f32) {
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

        let mouse = input.mouse();
        self.rotate(mouse.y(), mouse.x(), true);
    }
}
