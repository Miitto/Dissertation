use glium::{
    Display,
    glutin::surface::WindowSurface,
    winit::{event_loop::EventLoop, window::Window},
};

pub fn make_window() -> (EventLoop<()>, Window, Display<WindowSurface>) {
    let event_loop = glium::winit::event_loop::EventLoop::builder()
        .build()
        .expect("Failed to Create event loop");

    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

    (event_loop, window, display)
}
