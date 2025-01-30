mod projection;
mod transform;

use glam::Vec3;
pub use projection::*;

pub trait Translate {
    fn translate(&self, translation: Vec3) -> Self;
}

trait Dot {
    fn dot(&self, other: Vec3) -> f32;
}

pub trait Normalizable {
    fn normalize(&self) -> Self;
}

trait Cross {
    fn cross(&self, other: Vec3) -> Vec3;
}
