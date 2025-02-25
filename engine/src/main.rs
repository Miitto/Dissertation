use clap::Parser;
use renderer::{Renderable, State, make_event_loop, make_window};
use tests::{Scene, Test};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::PhysicalKey,
};

mod basic;
mod binary;
mod chunks;
mod common;
mod tests;

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Scene to use
    #[arg(short, long, default_value = "single")]
    scene: Scene,

    /// Test type
    #[arg(short, long, default_value = "basic")]
    test: Test,

    /// Radius
    #[arg(short, long, default_value = "8")]
    radius: u8,

    /// Height
    #[arg(short, long, default_value = "8")]
    depth: u8,
}

fn main() {
    let args = Args::parse();

    println!("Running {:?} test in scene: {:?}", args.test, args.scene);

    let event_loop = make_event_loop();

    let mut app = App::new(args);
    // optick::start_capture();
    let _ = event_loop.run_app(&mut app);
    // optick::stop_capture(name);

    println!();
    println!("Compiled {} shaders", shaders::shaders_compiled());
}

struct App {
    state: State,
    setup: Option<Box<dyn Renderable>>,
    args: Args,
}

impl App {
    fn new(args: Args) -> Self {
        let state = State::default();

        Self {
            state,
            setup: None,
            args,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let display = make_window(event_loop);

        let size = display.get_window().inner_size();
        self.state
            .camera
            .on_window_resize(size.width as f32, size.height as f32);

        self.state.new_window(display);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
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
        _id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(window_size) => {
                // TODO: GL Surface resize

                self.state
                    .camera
                    .on_window_resize(window_size.width as f32, window_size.height as f32);

                let display = self.state.display();

                renderer::resize(&display, window_size.width, window_size.height);
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
                self.state.new_frame();

                if self.setup.is_none() {
                    self.setup = Some(match self.args.test {
                        Test::Greedy => Box::new(binary::greedy::setup(&self.args, &self.state))
                            as Box<dyn Renderable>,
                        Test::Culled => Box::new(binary::culled::setup(&self.args, &self.state))
                            as Box<dyn Renderable>,
                        Test::Chunk => {
                            Box::new(chunks::setup(&self.args, &self.state)) as Box<dyn Renderable>
                        }
                        Test::BasicInstanced => {
                            Box::new(basic::setup(&self.args, true)) as Box<dyn Renderable>
                        }
                        Test::Basic => {
                            Box::new(basic::setup(&self.args, false)) as Box<dyn Renderable>
                        }
                    })
                }

                self.state.handle_input();

                self.setup.as_ref().unwrap().render(&mut self.state);

                self.state.end_frame();

                self.state.display().window.request_redraw();
            }
            _ => (),
        }
    }
}
