mod tex2d;
pub use tex2d::Texture2D;

pub trait Texture {
    fn id(&self) -> gl::types::GLuint;
    fn bind_to(&self, slot: u32);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TextureParameters {
    pub min_filter: TextureFilterMode,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextureFilterMode {
    Linear,
    #[default]
    NearestMipmapLinear,
}

#[derive(Clone, Copy, Debug)]
pub enum ColorMode {
    Rgba23f,
}

impl TextureParameters {
    pub fn setup_texture(&self, texture: &dyn Texture) {
        if self.min_filter != TextureFilterMode::default() {
            unsafe {
                gl::TextureParameteri(texture.id(), gl::TEXTURE_MIN_FILTER, self.min_filter.into())
            }
        }
    }
}

impl From<TextureFilterMode> for gl::types::GLuint {
    fn from(filter_mode: TextureFilterMode) -> gl::types::GLuint {
        use TextureFilterMode::*;
        match filter_mode {
            Linear => gl::LINEAR,
            NearestMipmapLinear => gl::NEAREST_MIPMAP_LINEAR,
        }
    }
}

impl From<TextureFilterMode> for gl::types::GLint {
    fn from(filter_mode: TextureFilterMode) -> gl::types::GLint {
        let unsinged: gl::types::GLuint = filter_mode.into();

        unsinged as gl::types::GLint
    }
}

impl From<ColorMode> for gl::types::GLuint {
    fn from(color_mode: ColorMode) -> gl::types::GLuint {
        use ColorMode::*;
        match color_mode {
            Rgba23f => gl::RGBA32F,
        }
    }
}

impl From<ColorMode> for gl::types::GLint {
    fn from(filter_mode: ColorMode) -> gl::types::GLint {
        let unsinged: gl::types::GLuint = filter_mode.into();

        unsinged as gl::types::GLint
    }
}
