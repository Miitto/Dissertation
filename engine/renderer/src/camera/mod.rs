use crate::{Dir, Input};
use glam::{Mat4, Vec3};

mod orthographic;
mod perspective;

pub use perspective::PerspectiveCamera;

pub trait Camera: std::fmt::Debug {
    fn on_window_resize(&mut self, width: f32, height: f32);
    fn get_projection(&self) -> Mat4;
    fn get_view(&self) -> Mat4;
    fn get_position(&self) -> Vec3;
    fn translate(&mut self, direction: Dir, delta: f32);
    fn rotate(&mut self, pitch_delta: f64, yaw_delta: f64, is_mouse: bool);
    fn handle_input(&mut self, keys: &Input, delta: f32);
}
