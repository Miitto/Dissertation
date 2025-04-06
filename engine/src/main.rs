use renderer::{Renderable, State, camera::CameraManager, make_event_loop, make_window};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::PhysicalKey,
};

use common::{Args, Parser, tests::Test};

#[allow(dead_code)]
struct Prolfiler(String);

#[allow(dead_code)]
impl Prolfiler {
    pub fn new(args: &Args) -> Self {
        let test_name = format!("{:?}_{:?}", args.test, args.scene);
        renderer::profiler::start_capture();
        Self(test_name)
    }
}

impl Drop for Prolfiler {
    fn drop(&mut self) {
        renderer::profiler::stop_capture(self.0.as_str());
    }
}

fn main() {
    let args = Args::parse();

    println!("Running {:?} test in scene: {:?}", args.test, args.scene);

    let event_loop = make_event_loop();

    let mut app = App::new(args);

    {
        // let _profiler = Prolfiler::new(&app.args);
        let _ = event_loop.run_app(&mut app);
    }

    println!();
    println!("Compiled {} shaders", shaders::shaders_compiled());
}

struct App {
    state: Option<State>,
    setup: Option<Box<dyn Renderable>>,
    args: Args,
}

impl App {
    fn new(args: Args) -> Self {
        Self {
            state: None,
            setup: None,
            args,
        }
    }

    fn state(&mut self) -> &mut State {
        self.state.as_mut().unwrap()
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let display = make_window(event_loop);

        if self.state.is_none() {
            self.state = Some(State::default());
        }

        let size = display.get_window().inner_size();
        self.state()
            .cameras
            .on_window_resize(size.width as f32, size.height as f32);

        self.state().new_window(display);
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
                self.state().mouse_move(delta.0 as f32, delta.1 as f32);
            }
            DeviceEvent::MouseWheel { delta } => match delta {
                MouseScrollDelta::LineDelta(_, y) => {
                    self.state().wheel_scroll(y);
                }
                MouseScrollDelta::PixelDelta(y) => {
                    self.state().wheel_scroll(y.y as f32);
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
        renderer::profiler::event!("Window Event");
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(window_size) => {
                self.state()
                    .cameras
                    .on_window_resize(window_size.width as f32, window_size.height as f32);

                let display = self.state().display();

                renderer::resize(&display, window_size.width, window_size.height);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.state().click(button, state);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    self.state().set_key(key, event);
                }
            }
            WindowEvent::RedrawRequested => {
                self.state().new_frame();

                if self.setup.is_none() {
                    self.setup = Some(match self.args.test {
                        Test::Tri => meshing::setup(),
                        Test::Greedy => Box::new(meshing::binary::greedy::setup(
                            &self.args,
                            self.state.as_ref().unwrap(),
                        )) as Box<dyn Renderable>,
                        Test::Culled => Box::new(meshing::binary::culled::setup(
                            &self.args,
                            self.state.as_ref().unwrap(),
                        )) as Box<dyn Renderable>,
                        Test::Chunk => Box::new(meshing::chunks::setup(
                            &self.args,
                            self.state.as_ref().unwrap(),
                        )) as Box<dyn Renderable>,
                        Test::BasicInstanced => {
                            Box::new(meshing::basic::setup(&self.args, true)) as Box<dyn Renderable>
                        }
                        Test::Basic => Box::new(meshing::basic::setup(&self.args, false))
                            as Box<dyn Renderable>,
                        Test::Raymarch => raytracing::setup(self.state.as_ref().unwrap()),
                        Test::Flat => {
                            raytracing::flat::setup(&self.args, self.state.as_ref().unwrap())
                        }
                        Test::Svt64 => {
                            raytracing::svt64::setup(&self.args, self.state.as_ref().unwrap())
                        }
                    })
                }

                self.state().handle_input();

                self.setup
                    .as_mut()
                    .unwrap()
                    .render(self.state.as_mut().unwrap());

                CameraManager::render_gizmos(self.state());

                self.state().end_frame();

                self.state().display().window.request_redraw();
            }
            _ => (),
        }
    }
}
