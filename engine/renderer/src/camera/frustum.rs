use glam::Vec3;

pub struct Plane {
    normal: Vec3,
    distance: f32,
}

impl Plane {
    pub fn new(closest_point: Vec3, normal: Vec3) -> Self {
        let normal = normal.normalize();
        Self {
            normal,
            distance: normal.dot(closest_point),
        }
    }

    pub fn distance(&self, point: Vec3) -> f32 {
        self.normal.dot(point) - self.distance
    }

    pub fn normal(&self) -> &Vec3 {
        &self.normal
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
