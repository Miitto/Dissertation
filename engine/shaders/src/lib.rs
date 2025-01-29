pub use shader_macros::shader;

pub trait Shader {
    fn source() -> &'static str;
}
