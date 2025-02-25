#![feature(duration_millis_float)]
pub mod buffers;
pub mod camera;
mod input;
pub mod math;
mod state;
pub mod vertex;
use std::{ffi::CString, num::NonZeroU32};

use glutin_winit::DisplayBuilder;
pub use input::{Input, PositionDelta};
pub use render_common::*;
pub use state::State;
mod enums;
pub use enums::*;
pub mod draw;
mod uniforms;
pub use memoffset::offset_of;
pub use uniforms::*;

use glutin::{
    config::ConfigTemplateBuilder,
    display::GetGlDisplay,
    surface::{GlSurface, Surface, WindowSurface},
};
use winit::{
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::Window,
};

pub fn make_event_loop() -> EventLoop<()> {
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop
}

pub fn resize(display: &Display, width: u32, height: u32) {
    let surface = &display.surface;
    let context = &display.context;
    surface.resize(
        &context,
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
    );

    unsafe {
        gl::Viewport(
            0,
            0,
            width as gl::types::GLsizei,
            height as gl::types::GLsizei,
        );
    };
}

pub fn make_window(event_loop: &ActiveEventLoop) -> Display {
    use glutin::prelude::*;
    use raw_window_handle::HasWindowHandle;

    let attrs = Window::default_attributes().with_title("Dissertation");

    let display_builder = DisplayBuilder::new().with_window_attributes(Some(attrs));

    let config_template_builder = ConfigTemplateBuilder::new();

    let (window, gl_config) = display_builder
        .build(event_loop, config_template_builder, |mut configs| {
            configs.next().unwrap()
        })
        .unwrap();

    let window = window.expect("Failed to create window");

    let (width, height) = window.inner_size().into();

    let attrs = glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
        .build(
            window
                .window_handle()
                .expect("Failed to get raw window handle")
                .into(),
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );

    let surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &attrs)
            .unwrap()
    };

    let context_attributes = glutin::context::ContextAttributesBuilder::new().build(Some(
        window
            .window_handle()
            .expect("Failed to get window handle")
            .into(),
    ));

    let gl_context = unsafe {
        gl_config
            .display()
            .create_context(&gl_config, &context_attributes)
            .unwrap()
            .treat_as_possibly_current()
    };

    gl_context.make_current(&surface).unwrap();

    let display = gl_config.display();

    gl::load_with(|symbol| {
        let symbol = CString::new(symbol).unwrap();
        display.get_proc_address(symbol.as_c_str()).cast()
    });

    let swap_interval = glutin::surface::SwapInterval::DontWait;

    surface
        .set_swap_interval(&gl_context, swap_interval)
        .unwrap();

    if cfg!(debug_assertions) {
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(Some(gl_error_callback), std::ptr::null());
        }
    }

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        // gl::Enable(gl::CULL_FACE);

        gl::CullFace(gl::BACK);
        gl::FrontFace(gl::CW);

        gl::DepthFunc(gl::LESS);
    }

    Display {
        window,
        context: gl_context,
        surface,
    }
}

pub trait Renderable {
    fn render(&self, state: &mut State);
}

extern "system" fn gl_error_callback(
    source: gl::types::GLenum,
    ty: gl::types::GLenum,
    id: gl::types::GLuint,
    severity: gl::types::GLenum,
    length: gl::types::GLsizei,
    message: *const i8,
    _user_param: *mut std::ffi::c_void,
) {
    let v = unsafe { std::slice::from_raw_parts(message as *const u8, length as usize) };
    let message = String::from_utf8_lossy(v);

    if ty != gl::DEBUG_TYPE_ERROR {
        return;
    }

    let source = match source {
        gl::DEBUG_SOURCE_API => "API",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "Window System",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "Shader Compiler",
        gl::DEBUG_SOURCE_THIRD_PARTY => "Third Party",
        gl::DEBUG_SOURCE_APPLICATION => "Application",
        gl::DEBUG_SOURCE_OTHER => "Other",
        _ => "unknown",
    };

    let ty = match ty {
        gl::DEBUG_TYPE_ERROR => "Error",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecated Behavior",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Undefined Behavior",
        gl::DEBUG_TYPE_PORTABILITY => "Portability",
        gl::DEBUG_TYPE_PERFORMANCE => "Performance",
        gl::DEBUG_TYPE_MARKER => "Marker",
        gl::DEBUG_TYPE_OTHER => "Other",
        _ => "unknown",
    };

    let severity = match severity {
        gl::DEBUG_SEVERITY_HIGH => "high",
        gl::DEBUG_SEVERITY_MEDIUM => "medium",
        gl::DEBUG_SEVERITY_LOW => "low",
        gl::DEBUG_SEVERITY_NOTIFICATION => "notification",
        _ => "unknown",
    };

    eprintln!(
        "OpenGL Error: {} | {} | {} | {} | {}",
        source, ty, id, severity, message
    );
}
