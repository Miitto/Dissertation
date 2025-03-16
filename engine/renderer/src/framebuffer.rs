use std::cell::RefCell;

use crate::texture::{Texture, Texture2D};

thread_local! {
    static GENERAL_FRAMEBUFFER: RefCell<Option<u32>> = const { RefCell::new(None)};
    static READ_FRAMEBUFFER: RefCell<Option<u32>> = const { RefCell::new(None) };
    static DRAW_FRAMEBUFFER: RefCell<Option<u32>> = const { RefCell::new(None) };
}

#[derive(Debug)]
pub struct Framebuffer {
    id: gl::types::GLuint,
    width: u32,
    height: u32,
}

impl Default for Framebuffer {
    fn default() -> Self {
        let id = unsafe {
            let mut id = 0;
            gl::CreateFramebuffers(1, &mut id);
            id
        };

        Self {
            id,
            width: 0,
            height: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TextureAttachPoint {
    Color0,
}

impl From<TextureAttachPoint> for gl::types::GLenum {
    fn from(value: TextureAttachPoint) -> Self {
        use TextureAttachPoint::*;
        match value {
            Color0 => gl::COLOR_ATTACHMENT0,
        }
    }
}

impl Framebuffer {
    pub fn bind(&self) {
        if GENERAL_FRAMEBUFFER.with_borrow(|v| v.filter(|v| *v == self.id).is_some()) {
            return;
        }
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, self.id) }
        GENERAL_FRAMEBUFFER.with_borrow_mut(|v| v.replace(self.id));
    }

    pub fn unbind() {
        if GENERAL_FRAMEBUFFER.with_borrow(|v| v.filter(|v| *v == 0).is_some()) {
            return;
        }
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0) }
        GENERAL_FRAMEBUFFER.with_borrow_mut(|v| v.replace(0));
    }
    pub fn bind_read(&self) {
        if READ_FRAMEBUFFER.with_borrow(|v| v.filter(|v| *v == self.id).is_some()) {
            return;
        }
        unsafe { gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.id) }
        READ_FRAMEBUFFER.with_borrow_mut(|v| v.replace(self.id));
    }

    pub fn unbind_read() {
        if READ_FRAMEBUFFER.with_borrow(|v| v.filter(|v| *v == 0).is_some()) {
            return;
        }
        unsafe { gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0) }
        READ_FRAMEBUFFER.with_borrow_mut(|v| v.replace(0));
    }
    pub fn bind_draw(&self) {
        if DRAW_FRAMEBUFFER.with_borrow(|v| v.filter(|v| *v == self.id).is_some()) {
            return;
        }
        unsafe { gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.id) }
        DRAW_FRAMEBUFFER.with_borrow_mut(|v| v.replace(self.id));
    }

    pub fn unbind_draw() {
        if DRAW_FRAMEBUFFER.with_borrow(|v| v.filter(|v| *v == 0).is_some()) {
            return;
        }
        unsafe { gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0) }
        DRAW_FRAMEBUFFER.with_borrow_mut(|v| v.replace(0));
    }

    pub fn set_tex_2d(&mut self, attach_point: TextureAttachPoint, texture: &Texture2D) {
        self.width = texture.width();
        self.height = texture.height();
        unsafe { gl::NamedFramebufferTexture(self.id, attach_point.into(), texture.id(), 0) }
    }

    pub fn blit_to_screen(&self, x: i32, y: i32) {
        unsafe {
            gl::BlitNamedFramebuffer(
                self.id,
                0,
                0,
                0,
                self.width as i32,
                self.height as i32,
                0,
                0,
                x,
                y,
                gl::COLOR_BUFFER_BIT,
                gl::LINEAR,
            )
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id);
        }
    }
}
