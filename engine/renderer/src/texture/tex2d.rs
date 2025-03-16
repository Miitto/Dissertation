use super::{ColorMode, Texture, TextureParameters};

#[derive(Debug)]
pub struct Texture2D {
    id: gl::types::GLuint,
    color_mode: ColorMode,
    parameters: TextureParameters,
    width: u32,
    height: u32,
}

impl Texture2D {
    pub fn new(
        width: u32,
        height: u32,
        color_mode: ColorMode,
        parameters: TextureParameters,
    ) -> Self {
        let id = unsafe {
            let mut id = 0;
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut id);
            id
        };

        let tex = Self {
            id,
            parameters,
            color_mode,
            width,
            height,
        };

        tex.parameters.setup_texture(&tex);

        unsafe {
            gl::TextureStorage2D(
                tex.id,
                1,
                color_mode.into(),
                tex.width as i32,
                tex.height as i32,
            )
        }

        tex
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl Texture for Texture2D {
    fn id(&self) -> gl::types::GLuint {
        self.id
    }

    fn bind_to(&self, slot: u32) {
        unsafe {
            gl::BindTextureUnit(slot, self.id);
            gl::BindImageTexture(
                0,
                self.id,
                0,
                gl::FALSE,
                0,
                gl::WRITE_ONLY,
                self.color_mode.into(),
            );
        }
    }
}

impl Drop for Texture2D {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.id) }
    }
}
