use glam::{Vec3, vec3};

pub struct Plane {
    normal: Vec3,
    distance: f32,
}

impl Plane {
    pub fn new(closest_point: Vec3, normal: Vec3) -> Self {
        Self {
            normal: normal.normalize(),
            distance: normal.dot(closest_point),
        }
    }

    pub fn distance(&self, point: Vec3) -> f32 {
        self.normal.dot(point) - self.distance
    }
}

pub struct Frustum {
    top: Plane,
    bottom: Plane,

    left: Plane,
    right: Plane,

    far: Plane,
    near: Plane,
}

impl Frustum {
    pub fn new(
        top: Plane,
        bottom: Plane,
        left: Plane,
        right: Plane,
        near: Plane,
        far: Plane,
    ) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
            near,
            far,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Plane> {
        // Ordered for most likely to fail
        [
            &self.left,
            &self.right,
            &self.far,
            &self.near,
            &self.top,
            &self.bottom,
        ]
        .into_iter()
    }

    pub fn test_sphere(&self, center: Vec3, radius: f32) -> bool {
        for plane in self.iter() {
            if plane.distance(center) < -radius {
                return false;
            }
        }

        true
    }

    pub fn test_aabb(&self, bb: AABB) -> bool {
        for plane in self.iter() {
            let r = bb.extents.x * plane.normal.x.abs()
                + bb.extents.y * plane.normal.y.abs()
                + bb.extents.z * plane.normal.z.abs();

            if -r > plane.distance(bb.center) {
                return false;
            }
        }

        true
    }
}

pub struct FrustumCorners {
    pub near_top_left: Vec3,
    pub near_top_right: Vec3,
    pub near_bottom_left: Vec3,
    pub near_bottom_right: Vec3,
    pub far_top_left: Vec3,
    pub far_top_right: Vec3,
    pub far_bottom_left: Vec3,
    pub far_bottom_right: Vec3,
}

pub struct AABB {
    pub center: Vec3,
    pub extents: Vec3,
}

impl AABB {
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

        let extends = vec3(
            (max_x - min_x) / 2.0,
            (max_y - min_y) / 2.0,
            (max_z - min_z) / 2.0,
        );

        AABB {
            center,
            extents: extends,
        }
    }
}
