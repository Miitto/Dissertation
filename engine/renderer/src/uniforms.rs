use std::ffi::CString;

use render_common::Program;

pub trait Uniforms {
    fn bind(&self, program: &Program);
}

pub fn get_uniform_location(program: &Program, name: &str) -> usize {
    let c_str = CString::new(name).expect("Failed to create CString");
    unsafe { gl::GetUniformLocation(program.id() as gl::types::GLuint, c_str.as_ptr()) as usize }
}
