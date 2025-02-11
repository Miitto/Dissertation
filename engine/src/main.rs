use glium::winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::PhysicalKey,
};
use renderer::{Renderable, State, make_event_loop, make_window};
use tests::Test;

mod basic;
mod binary;
mod chunks;
mod common;
mod tests;

const TEST: Test = Test::Cube;

fn main() {
    let name = if cfg!(feature = "binary") {
        if cfg!(feature = "greedy") {
            println!("Running Binary greedy test");
            "binary_greedy"
        } else {
            println!("Running Binary culled test");
            "binary_culled"
        }
    } else if cfg!(feature = "culled") {
        println!("Running culled test");
        "culled"
    } else if cfg!(feature = "chunks") {
        println!("Running chunks test");
        "chunks"
    } else {
        println!("Running basic test");
        "basic"
    };

    let mut app = App::new();

    let event_loop = make_event_loop();
    optick::start_capture();
    let _ = event_loop.run_app(&mut app);
    optick::stop_capture(name);

    println!();
    println!("Compiled {} shaders", shaders::shaders_compiled());
}

struct App {
    state: State,
    setup: Option<Box<dyn Renderable>>,
}

impl App {
    fn new() -> Self {
        let state = State::default();

        Self { state, setup: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, display) = make_window(event_loop);

        let size = window.inner_size();
        self.state
            .camera
            .on_window_resize(size.width as f32, size.height as f32);

        self.state.new_window(window, display);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _id: glium::winit::event::DeviceId,
        event: glium::winit::event::DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                if delta.0 == 0.0 && delta.1 == 0.0 {
                    return;
                }
                self.state.mouse_move(delta.0, delta.1);
            }
            DeviceEvent::MouseWheel { delta } => match delta {
                MouseScrollDelta::LineDelta(_, y) => {
                    self.state.wheel_scroll(y);
                }
                MouseScrollDelta::PixelDelta(y) => {
                    self.state.wheel_scroll(y.y as f32);
                }
            },
            _ => (),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: glium::winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(window_size) => {
                if let Some(display) = &mut self.state.display {
                    display.resize(window_size.into());
                }
                self.state
                    .camera
                    .on_window_resize(window_size.width as f32, window_size.height as f32);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.state.click(button, state);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    self.state.set_key(key, event);
                }
            }
            WindowEvent::RedrawRequested => {
                if self.state.window.is_some() && self.state.display.is_some() {
                    self.state.new_frame();

                    if self.setup.is_none() {
                        self.setup = Some(if cfg!(feature = "binary") {
                            if cfg!(feature = "greedy") {
                                todo!("Setup Binary greedy mesher")
                            } else {
                                Box::new(binary::culled::setup(TEST, &self.state))
                                    as Box<dyn Renderable>
                            }
                        } else if cfg!(feature = "chunks") {
                            Box::new(chunks::setup(TEST, &self.state)) as Box<dyn Renderable>
                        } else {
                            Box::new(basic::setup(TEST)) as Box<dyn Renderable>
                        });
                    }

                    self.state.handle_input();

                    self.setup.as_ref().unwrap().render(&mut self.state);

                    self.state.end_frame();

                    self.state.window.as_ref().unwrap().request_redraw();
                }
            }
            _ => (),
        }
    }
}
