use glam::Vec3;
use shaders::Program as _;

pub use line::{Program, Uniforms, Vertex, uniforms};

use crate::{
    Renderable,
    bounds::{AABB, BoundingHeirarchy, BoundingSphere},
};

pub struct Line {
    start: Vec3,
    end: Vec3,
    color: Vec3,
}

impl From<&Line> for BoundingHeirarchy {
    fn from(value: &Line) -> Self {
        let center = value.center();
        let radius = value.length() / 2.0;

        BoundingHeirarchy::new(
            BoundingSphere::new(center, radius),
            AABB::new(center, Vec3::splat(radius)),
        )
    }
}

impl Line {
    pub fn new(start: Vec3, end: Vec3, color: Vec3) -> Self {
        Self { start, end, color }
    }

    pub fn to_vec(&self) -> Vec<Vec3> {
        vec![self.start, self.end]
    }

    pub fn to_vertices(&self) -> Vec<line::Vertex> {
        self.to_vec()
            .iter()
            .map(|v| line::Vertex {
                pos: v.to_array(),
                color: self.color.to_array(),
            })
            .collect()
    }

    pub fn center(&self) -> Vec3 {
        (self.start + self.end) / 2.0
    }

    pub fn length(&self) -> f32 {
        (self.start - self.end).length()
    }
}

crate::program!(line, {
    #vertex vert
    #fragment frag

    struct vIn {
        vec3 pos;
        vec3 color;
    }

    struct v2f {
        vec3 color;
    }

    v2f vert(vIn i) {
        mat4 pv = camera.projection * camera.view;

        gl_Position = pv * vec4(i.pos, 1.0);

        v2f o;
        o.color = i.color;
        return o;
    }

    vec4 frag(v2f i) {
        return vec4(i.color, 1.0);
    }
}, true);
