use glam::{Vec3, vec3};

use crate::camera::frustum::Plane;

use super::{BoundingIntersection, BoundingVolume};

#[derive(Clone, Copy, Debug, Default)]
pub struct AABB {
    pub center: Vec3,
    pub extents: Vec3,
}

impl AABB {
    pub fn new(center: Vec3, extents: Vec3) -> Self {
        Self { center, extents }
    }

    pub fn from_points(fbl: Vec3, btr: Vec3) -> Self {
        let max_x = btr.x.max(fbl.x);
        let min_x = btr.x.min(fbl.x);

        let max_y = btr.y.max(fbl.y);
        let min_y = btr.y.min(fbl.y);

        let max_z = btr.z.max(fbl.z);
        let min_z = btr.z.min(fbl.z);

        let center = vec3(
            (max_x + min_x) / 2.0,
            (max_y + min_y) / 2.0,
            (max_z + min_z) / 2.0,
        );

        let extents = vec3(
            (max_x - min_x) / 2.0,
            (max_y - min_y) / 2.0,
            (max_z - min_z) / 2.0,
        );

        AABB { center, extents }
    }
}

impl BoundingVolume for AABB {
    fn intersects(&self, frustum: &crate::camera::frustum::Frustum) -> BoundingIntersection {
        // https://learnopengl.com/Guest-Articles/2021/Scene/Frustum-Culling

        let is_on_plane = |plane: &Plane| {
            let r = self.extents.x * plane.normal().x.abs()
                + self.extents.y * plane.normal().y.abs()
                + self.extents.z * plane.normal().z.abs();

            let s = plane.distance(self.center);

            if 0.0 <= s {
                BoundingIntersection::Full
            } else if -r <= s {
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
