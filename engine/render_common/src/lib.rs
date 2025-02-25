use std::{cell::RefCell, ffi::CString, mem::MaybeUninit};

use glutin::{
    context::PossiblyCurrentContext,
    surface::{Surface, WindowSurface},
};
use winit::window::Window;
pub mod shaders;
use shaders::ShaderType;

mod uniform_value;
pub use uniform_value::UniformValue;

thread_local! {
    static ACTIVE_PROGRAM: RefCell<Option<gl::types::GLuint>> = const { RefCell::new(None) };
}

pub struct Display {
    pub window: Window,
    pub context: PossiblyCurrentContext,
    pub surface: Surface<WindowSurface>,
}

impl Display {
    pub fn get_window(&self) -> &Window {
        &self.window
    }

    pub fn get_context(&self) -> &PossiblyCurrentContext {
        &self.context
    }
}

#[derive(Debug)]
pub struct Shader {
    id: gl::types::GLuint,
}

impl From<Shader> for gl::types::GLuint {
    fn from(shader: Shader) -> gl::types::GLuint {
        shader.id
    }
}

impl From<&Shader> for gl::types::GLuint {
    fn from(shader: &Shader) -> gl::types::GLuint {
        shader.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

fn compile_shader(source: &str, ty: ShaderType) -> Shader {
    let shader;
    let source = CString::new(source.as_bytes()).unwrap();
    unsafe {
        shader = gl::CreateShader(ty.into());
        gl::ShaderSource(shader, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);

        let mut status = gl::FALSE as gl::types::GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status); // Fail on error
        if status == 0 {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf: Vec<MaybeUninit<u8>> = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(
                shader,
                len,
                std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut gl::types::GLchar,
            );
            let buf: Vec<u8> = buf.iter().map(|b| b.assume_init()).collect();
            panic!(
                "{}",
                core::str::from_utf8(&buf).expect("ShaderInfoLog not valid utf8")
            );
        }
    };
    Shader { id: shader }
}

fn link_program(vertex: Shader, frag: Shader) -> Program {
    let program = unsafe {
        let program = gl::CreateProgram();

        gl::AttachShader(program, frag.id);
        gl::AttachShader(program, vertex.id);
        gl::LinkProgram(program);

        let mut status = gl::FALSE as gl::types::GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        if status == 0 {
            let mut v: Vec<u8> = Vec::with_capacity(1024);
            let mut log_len = 0_i32;
            gl::GetProgramInfoLog(
                program,
                1024,
                &mut log_len,
                v.as_mut_ptr() as *mut gl::types::GLchar,
            );
            v.set_len(log_len.try_into().unwrap());
            panic!("Program Compile Error: {}", String::from_utf8_lossy(&v));
        }
        program
    };

    Program { id: program }
}
pub fn make_program(vertex: &str, fragment: &str) -> Program {
    let v = compile_shader(vertex, ShaderType::Vertex);
    let f = compile_shader(fragment, ShaderType::Fragment);

    link_program(v, f)
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Program {
    id: gl::types::GLuint,
}

impl Program {
    /// Bind the program. Will not rebind if already bound.
    pub fn bind(&self) {
        if ACTIVE_PROGRAM.with(|active| active.borrow().is_some_and(|a| a == self.id)) {
            return;
        }
        unsafe {
            gl::UseProgram(self.id);
            ACTIVE_PROGRAM.with(|active| *active.borrow_mut() = Some(self.id));
        }
    }

    pub fn id(&self) -> usize {
        self.id as usize
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

impl From<Program> for gl::types::GLuint {
    fn from(program: Program) -> gl::types::GLuint {
        program.id
    }
}

impl From<&Program> for gl::types::GLuint {
    fn from(program: &Program) -> gl::types::GLuint {
        program.id
    }
}
