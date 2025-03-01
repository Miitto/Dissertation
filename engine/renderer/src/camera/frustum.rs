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
            for point in bb.iter() {
                if plane.distance(point) > 0.0 {
                    return false;
                }
            }
        }

        true
    }
}

pub struct AABB {
    fbl: Vec3,
    btr: Vec3,
}

impl AABB {
    pub fn from_points(fbl: Vec3, btr: Vec3) -> Self {
        Self { fbl, btr }
    }

    pub fn min_x(&self) -> f32 {
        self.fbl.x.min(self.btr.x)
    }

    pub fn max_x(&self) -> f32 {
        self.fbl.x.max(self.btr.x)
    }

    pub fn min_y(&self) -> f32 {
        self.fbl.y.min(self.btr.y)
    }

    pub fn max_y(&self) -> f32 {
        self.fbl.y.max(self.btr.y)
    }

    pub fn min_z(&self) -> f32 {
        self.fbl.z.min(self.btr.z)
    }

    pub fn max_z(&self) -> f32 {
        self.fbl.z.max(self.btr.z)
    }

    pub fn iter(&self) -> impl Iterator<Item = Vec3> {
        [
            vec3(self.min_x(), self.min_y(), self.min_z()),
            vec3(self.max_x(), self.min_y(), self.min_z()),
            vec3(self.min_x(), self.max_y(), self.min_z()),
            vec3(self.max_x(), self.max_y(), self.min_z()),
            vec3(self.min_x(), self.min_y(), self.max_z()),
            vec3(self.max_x(), self.min_y(), self.max_z()),
            vec3(self.min_x(), self.max_y(), self.max_z()),
            vec3(self.max_x(), self.max_y(), self.max_z()),
        ]
        .into_iter()
    }
}
