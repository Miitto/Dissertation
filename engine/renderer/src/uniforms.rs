use std::ffi::CString;

use render_common::Program;

use crate::buffers::{BufferError, Mapping};

pub trait Uniforms {
    fn bind(&self, program: &Program);
}

pub fn get_uniform_location(program: &Program, name: &str) -> usize {
    let c_str = CString::new(name).expect("Failed to create CString");
    unsafe { gl::GetUniformLocation(program.id() as gl::types::GLuint, c_str.as_ptr()) as usize }
}

pub trait LayoutBlock: std::fmt::Debug {
    fn bind_point() -> u32;
    fn size() -> usize;
    fn set_buffer_data<'a>(
        &self,
        mapping: &mut Mapping<'a>,
        offset: usize,
    ) -> Result<usize, BufferError>;
}

pub trait UniformBlock: LayoutBlock {}

pub trait SSBO {
    fn bind(&self);
}

pub trait SSBOBlock: LayoutBlock {}
