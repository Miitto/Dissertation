use glam::Vec3;

use crate::camera::frustum::Plane;

use super::{BoundingIntersection, BoundingVolume};

#[derive(Clone, Copy, Debug, Default)]
pub struct BoundingSphere {
    pub center: Vec3,
    pub radius: f32,
}

impl BoundingSphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }
}

impl BoundingVolume for BoundingSphere {
    fn intersects(&self, frustum: &super::Frustum) -> BoundingIntersection {
        let is_on_plane = |plane: &Plane| {
            let dist = plane.distance(self.center);

            if dist >= 0.0 {
                BoundingIntersection::Full
            } else if dist > -self.radius {
                BoundingIntersection::Partial
            } else {
                BoundingIntersection::None
            }
        };

        frustum
            .iter()
            .map(is_on_plane)
            .fold(BoundingIntersection::Full, |acc, el| acc.min(el))
    }

    fn center(&self) -> Vec3 {
        self.center
    }
}
