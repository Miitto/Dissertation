use crate::camera::frustum::Frustum;

mod aabb;
pub use aabb::AABB;

mod sphere;
use glam::Vec3;
pub use sphere::BoundingSphere;

pub trait BoundingVolume {
    fn intersects(&self, frustum: &Frustum) -> BoundingIntersection;
    fn center(&self) -> Vec3;
}

pub enum BoundingIntersection {
    None,
    Partial,
    Full,
}

impl BoundingIntersection {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_partial(&self) -> bool {
        matches!(self, Self::Partial)
    }

    pub fn is_full(&self) -> bool {
        matches!(self, Self::Full)
    }

    pub fn min(&self, rhs: Self) -> Self {
        if self.is_none() || rhs.is_none() {
            return Self::None;
        }

        if self.is_partial() || rhs.is_partial() {
            return Self::Partial;
        }

        Self::Full
    }
}

impl From<BoundingIntersection> for bool {
    fn from(intersection: BoundingIntersection) -> bool {
        match intersection {
            BoundingIntersection::None => false,
            BoundingIntersection::Partial | BoundingIntersection::Full => true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BoundingHeirarchy {
    sphere: BoundingSphere,
    aabb: AABB,
}

impl BoundingHeirarchy {
    pub fn new(sphere: BoundingSphere, aabb: AABB) -> Self {
        Self { sphere, aabb }
    }

    pub fn from_min_max(min: Vec3, max: Vec3) -> Self {
        let center = (min + max) / 2.0;
        let radius = (max - min).length() / 2.0;

        let extents = (max - min) / 2.0;

        let sphere = BoundingSphere::new(center, radius);
        let aabb = AABB::new(center, extents);

        Self { sphere, aabb }
    }
}

impl BoundingVolume for BoundingHeirarchy {
    fn intersects(&self, frustum: &Frustum) -> BoundingIntersection {
        let sphere = self.sphere.intersects(frustum);

        if sphere.is_partial() {
            return self.aabb.intersects(frustum);
        }

        sphere
    }

    fn center(&self) -> Vec3 {
        self.sphere.center()
    }
}
