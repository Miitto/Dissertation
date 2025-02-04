#![feature(duration_millis_float)]
pub mod camera;
mod input;
pub mod math;
mod state;
pub use input::{Input, PositionDelta};
pub use state::State;

use glium::{
    Display,
    glutin::surface::WindowSurface,
    winit::{
        event_loop::{ActiveEventLoop, EventLoop},
        window::Window,
    },
};

pub fn make_event_loop() -> EventLoop<()> {
    let event_loop =
        glium::winit::event_loop::EventLoop::new().expect("Failed to create event loop");

    event_loop.set_control_flow(glium::winit::event_loop::ControlFlow::Poll);

    event_loop
}

pub fn make_window(event_loop: &ActiveEventLoop) -> (Window, Display<WindowSurface>) {
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(event_loop);

    (window, display)
}

pub enum Dir {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
}

pub trait Renderable {
    fn render(&self, state: &mut State);
}
