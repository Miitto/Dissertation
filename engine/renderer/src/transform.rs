use glam::{Mat4, Quat, Vec3, vec3};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
}

impl Transform {
    pub fn to_mat4(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.rotation, self.position)
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation * vec3(0.0, 0.0, -1.0)
    }

    pub fn flat_forward(&self) -> Vec3 {
        let mut forward = self.forward();
        forward.y = 0.0;
        forward.normalize()
    }

    pub fn up(&self) -> Vec3 {
        self.rotation * vec3(0.0, 1.0, 0.0)
    }

    pub fn right(&self) -> Vec3 {
        self.rotation * vec3(1.0, 0.0, 0.0)
    }
}

impl Transform {
    pub fn new(position: Vec3, rotation: Quat) -> Self {
        Self { position, rotation }
    }
}

impl From<Transform> for Mat4 {
    fn from(transform: Transform) -> Self {
        Mat4::from_rotation_translation(transform.rotation, transform.position)
    }
}
