use glam::{DQuat, EulerRot, Mat4, Vec3, dvec3, mat3, mat4, vec3, vec4};
use glium::winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::math::perspective;

use super::Camera;

pub struct PerspectiveCamera {
    position: Vec3,
    fov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
    rotation: DQuat,
    speed: f32,
    sensitivity: f32,
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
            sensitivity: 0.1,
        }
    }
}

impl Camera for PerspectiveCamera {
    fn on_window_resize(&mut self, width: f32, height: f32) {
        self.aspect_ratio = width / height;
    }

    fn get_sensitivity(&self) -> f32 {
        self.sensitivity
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

        dbg!(self.position);
    }

    fn rotate(&mut self, pitch_delta: f64, yaw_delta: f64) {
        let (x, y, _z) = self.rotation.to_euler(EulerRot::YXZ);

        let y = y + (pitch_delta * self.sensitivity as f64);
        let mut x = x - (yaw_delta * self.sensitivity as f64);

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

        let (x, y, z) = self.rotation.to_euler(EulerRot::YXZ);

        println!("{}, {}, {}", x.to_degrees(), y.to_degrees(), z.to_degrees());
    }

    fn handle_input(
        &mut self,
        keys: &std::collections::HashMap<glium::winit::keyboard::PhysicalKey, KeyEvent>,
        delta: f32,
    ) {
        if let Some(event) = keys.get(&PhysicalKey::Code(KeyCode::KeyW)) {
            if event.state == ElementState::Pressed {
                self.translate(crate::Dir::Forward, delta);
            }
        }

        if let Some(event) = keys.get(&PhysicalKey::Code(KeyCode::KeyS)) {
            if event.state == ElementState::Pressed {
                self.translate(crate::Dir::Backward, delta);
            }
        }

        if let Some(event) = keys.get(&PhysicalKey::Code(KeyCode::KeyA)) {
            if event.state == ElementState::Pressed {
                self.translate(crate::Dir::Left, delta);
            }
        }

        if let Some(event) = keys.get(&PhysicalKey::Code(KeyCode::KeyD)) {
            if event.state == ElementState::Pressed {
                self.translate(crate::Dir::Right, delta);
            }
        }

        if let Some(event) = keys.get(&PhysicalKey::Code(KeyCode::Space)) {
            if event.state == ElementState::Pressed {
                self.translate(crate::Dir::Up, delta);
            }
        }

        if let Some(event) = keys.get(&PhysicalKey::Code(KeyCode::ControlLeft)) {
            if event.state == ElementState::Pressed {
                self.translate(crate::Dir::Down, delta);
            }
        }

        if let Some(event) = keys.get(&PhysicalKey::Code(KeyCode::ArrowLeft)) {
            if event.state == ElementState::Pressed {
                self.rotate(0., -1.);
            }
        }
        if let Some(event) = keys.get(&PhysicalKey::Code(KeyCode::ArrowRight)) {
            if event.state == ElementState::Pressed {
                self.rotate(0., 1.);
            }
        }
        if let Some(event) = keys.get(&PhysicalKey::Code(KeyCode::ArrowUp)) {
            if event.state == ElementState::Pressed {
                self.rotate(1., 0.);
            }
        }
        if let Some(event) = keys.get(&PhysicalKey::Code(KeyCode::ArrowDown)) {
            if event.state == ElementState::Pressed {
                self.rotate(-1., 0.);
            }
        }
    }
}
