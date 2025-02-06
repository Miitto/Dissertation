use glium::winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::PhysicalKey,
};
use renderer::{Renderable, State, make_event_loop, make_window};

mod basic;
mod chunks;

fn main() {
    let mut app = App::default();

    let event_loop = make_event_loop();
    optick::start_capture();
    let _ = event_loop.run_app(&mut app);
    optick::stop_capture("basic");

    println!();
    println!("Compiled {} shaders", shaders::shaders_compiled());
}

#[derive(Default)]
struct App {
    state: State,
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

                    self.state.handle_input();

                    draw(&mut self.state);

                    self.state.end_frame();

                    self.state.window.as_ref().unwrap().request_redraw();
                }
            }
            _ => (),
        }
    }
}

fn draw(state: &mut State) {
    optick::event!("Draw Logic");

    let renderables = basic::get_renderables();

    fn render(state: &mut State, renderables: &[Box<dyn Renderable>]) {
        optick::event!("Render");
        for renderable in renderables {
            renderable.render(state);
        }
    }

    render(state, &renderables);
}
