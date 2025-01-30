use std::collections::HashMap;

use crate::Dir;
use glam::{Mat4, Vec3};

mod orthographic;
mod perspective;

use glium::winit::{event::KeyEvent, keyboard::PhysicalKey};
pub use perspective::PerspectiveCamera;

pub trait Camera {
    fn on_window_resize(&mut self, width: f32, height: f32);
    fn get_projection(&self) -> Mat4;
    fn get_view(&self) -> Mat4;
    fn get_position(&self) -> Vec3;
    fn get_sensitivity(&self) -> f32;
    fn translate(&mut self, direction: Dir, delta: f32);
    fn rotate(&mut self, pitch_delta: f64, yaw_delta: f64);
    fn handle_input(&mut self, keys: &HashMap<PhysicalKey, KeyEvent>, delta: f32);
}
