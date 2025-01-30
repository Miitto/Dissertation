use glam::{Vec3, vec3};

use super::{Cross, Dot};

impl Cross for Vec3 {
    fn cross(&self, other: Vec3) -> Vec3 {
        vec3(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
}

impl Dot for Vec3 {
    fn dot(&self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}
