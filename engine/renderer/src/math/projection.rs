use glam::{Mat4, mat4, vec4};

pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let f = 1.0 / (fov.to_radians() / 2.0).tan();
    let range = near - far;

    mat4(
        vec4(f / aspect, 0.0, 0.0, 0.0),
        vec4(0.0, f, 0.0, 0.0),
        vec4(0.0, 0.0, (far + near) / range, -1.0),
        vec4(0.0, 0.0, 2.0 * far * near / range, 0.0),
    )
}
