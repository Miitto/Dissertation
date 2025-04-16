use std::{fs::OpenOptions, io::Write, path::PathBuf};

use renderer::{
    Renderable, State,
    camera::{Camera, CameraManager, PerspectiveCamera},
    make_event_loop, make_window,
};
use winit::{
    application::ApplicationHandler,
    event::{self, DeviceEvent, KeyEvent, MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
};

use common::{
    Args, Parser,
    tests::{Scene, Test},
};

macro_rules! make_test {
    ($scene:ident, $test:ident, $frustum:literal, $combine:literal) => {{
        let mut args = Args::default();
        args.scene = Scene::$scene;
        args.test = Test::$test;
        args.frustum_cull = $frustum;
        args.combine = $combine;
        args.auto_test = true;
        args
    }};
    ($scene:ident, $test:ident, $frustum:literal, $combine:literal, $radius:literal) => {{
        let mut args = make_test!($scene, $test, $frustum, $combine);
        args.radius = $radius;
        args
    }};
}

const TIME_PER_TEST: f64 = 5.0;
const TESTS: [Args; 77] = [
    make_test!(Single, Basic, false, false),
    make_test!(Single, Instanced, false, false),
    make_test!(Single, Culled, false, false),
    make_test!(Single, Culled, true, false),
    make_test!(Single, Culled, false, true),
    make_test!(Single, Culled, true, true),
    make_test!(Single, Greedy, false, false),
    make_test!(Single, Greedy, true, false),
    make_test!(Single, Greedy, false, true),
    make_test!(Single, Greedy, true, true),
    make_test!(Cube, Basic, false, false),
    make_test!(Cube, Instanced, false, false),
    make_test!(Cube, Culled, false, false),
    make_test!(Cube, Culled, true, false),
    make_test!(Cube, Culled, false, true),
    make_test!(Cube, Culled, true, true),
    make_test!(Cube, Greedy, false, false),
    make_test!(Cube, Greedy, true, false),
    make_test!(Cube, Greedy, false, true),
    make_test!(Cube, Greedy, true, true),
    make_test!(Perlin, Basic, false, false, 32),
    make_test!(Perlin, Instanced, false, false, 32),
    make_test!(Perlin, Culled, false, false, 32),
    make_test!(Perlin, Culled, true, false, 32),
    make_test!(Perlin, Culled, false, true, 32),
    make_test!(Perlin, Culled, true, true, 32),
    make_test!(Perlin, Greedy, false, false, 32),
    make_test!(Perlin, Greedy, true, false, 32),
    make_test!(Perlin, Greedy, false, true, 32),
    make_test!(Perlin, Greedy, true, true, 32),
    make_test!(Perlin, Basic, false, false, 64),
    make_test!(Perlin, Instanced, false, false, 64),
    make_test!(Perlin, Culled, false, false, 64),
    make_test!(Perlin, Culled, true, false, 64),
    make_test!(Perlin, Culled, false, true, 64),
    make_test!(Perlin, Culled, true, true, 64),
    make_test!(Perlin, Greedy, false, false, 64),
    make_test!(Perlin, Greedy, true, false, 64),
    make_test!(Perlin, Greedy, false, true, 64),
    make_test!(Perlin, Greedy, true, true, 64),
    make_test!(Perlin, Basic, false, false, 128),
    make_test!(Perlin, Instanced, false, false, 128),
    make_test!(Perlin, Culled, false, false, 128),
    make_test!(Perlin, Culled, true, false, 128),
    make_test!(Perlin, Culled, false, true, 128),
    make_test!(Perlin, Culled, true, true, 128),
    make_test!(Perlin, Greedy, false, false, 128),
    make_test!(Perlin, Greedy, true, false, 128),
    make_test!(Perlin, Greedy, false, true, 128),
    make_test!(Perlin, Greedy, true, true, 128),
    make_test!(Perlin, Basic, false, false, 256),
    make_test!(Perlin, Instanced, false, false, 256),
    make_test!(Perlin, Culled, false, false, 256),
    make_test!(Perlin, Culled, true, false, 256),
    make_test!(Perlin, Culled, false, true, 256),
    make_test!(Perlin, Culled, true, true, 256),
    make_test!(Perlin, Greedy, false, false, 256),
    make_test!(Perlin, Greedy, true, false, 256),
    make_test!(Perlin, Greedy, false, true, 256),
    make_test!(Perlin, Greedy, true, true, 256),
    make_test!(Perlin, Instanced, false, false, 512),
    make_test!(Perlin, Culled, false, false, 512),
    make_test!(Perlin, Culled, true, false, 512),
    make_test!(Perlin, Culled, false, true, 512),
    make_test!(Perlin, Culled, true, true, 512),
    make_test!(Perlin, Greedy, false, false, 512),
    make_test!(Perlin, Greedy, true, false, 512),
    make_test!(Perlin, Greedy, false, true, 512),
    make_test!(Perlin, Greedy, true, true, 512),
    make_test!(Perlin, Culled, false, false, 1024),
    make_test!(Perlin, Culled, true, false, 1024),
    make_test!(Perlin, Culled, false, true, 1024),
    make_test!(Perlin, Culled, true, true, 1024),
    make_test!(Perlin, Greedy, false, false, 1024),
    make_test!(Perlin, Greedy, true, false, 1024),
    make_test!(Perlin, Greedy, false, true, 1024),
    make_test!(Perlin, Greedy, true, true, 1024),
];

fn setup_test(app: &mut App) {
    app.setup = Some(match app.args.test {
        Test::Tri => meshing::setup(),
        Test::Culled | Test::Greedy => Box::new(meshing::binary::culled::setup(
            &app.args,
            app.state.as_ref().unwrap(),
        )) as Box<dyn Renderable>,

        Test::Instanced => Box::new(meshing::basic::setup(&app.args, true)) as Box<dyn Renderable>,
        Test::Basic => Box::new(meshing::basic::setup(&app.args, false)) as Box<dyn Renderable>,
        Test::Raymarch => raytracing::setup(app.state.as_ref().unwrap()),
        Test::Flat => raytracing::flat::setup(&app.args, app.state.as_ref().unwrap()),
        Test::Svt64 => raytracing::svt64::setup(&app.args, app.state.as_ref().unwrap()),
    })
}

#[allow(dead_code)]
struct Prolfiler(String);

#[allow(dead_code)]
impl Prolfiler {
    pub fn new(args: &Args) -> Self {
        let test_name = format!("{:?}_{:?}", args.test, args.scene);
        renderer::profiler::start_capture();
        println!("Profiler started for test: {}", test_name);
        Self(test_name)
    }
}

impl Drop for Prolfiler {
    fn drop(&mut self) {
        let base = PathBuf::from("profiler");
        let path = base.join(&self.0);
        let s = path
            .to_str()
            .expect("Failed to convert profiler path to str");
        println!("Profiler path: {}", s);
        renderer::profiler::stop_capture(s);
    }
}

fn main() {
    let args = Args::parse();

    println!("Running {:?} test in scene: {:?}", args.test, args.scene);

    let event_loop = make_event_loop();

    let mut app = App::new(args);

    {
        let _profiler = if app.args.profile {
            Some(Prolfiler::new(&app.args))
        } else {
            None
        };
        let _ = event_loop.run_app(&mut app);
        println!("Average FPS: {}", app.state().avg_fps());
    }

    println!();
    println!("Compiled {} shaders", shaders::shaders_compiled());

    for (i, fps) in app.test_fps.iter().enumerate() {
        println!("Test {:?}: Average FPS: {}", TESTS[i], fps);
    }
}

struct App {
    state: Option<State>,
    setup: Option<Box<dyn Renderable>>,
    args: Args,
    test_step: usize,
    last_test_time: std::time::Instant,
    test_fps: Vec<f64>,
}

impl App {
    fn new(args: Args) -> Self {
        Self {
            state: None,
            setup: None,
            args,
            test_step: 0,
            last_test_time: std::time::Instant::now(),
            test_fps: vec![],
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
                if self.setup.is_none()
                    || (self.args.auto_test
                        && self.last_test_time.elapsed().as_secs_f64() >= TIME_PER_TEST)
                {
                    if self.args.auto_test {
                        if self.test_step != 0 {
                            let fps = self.state().avg_fps();
                            self.test_fps.push(fps);
                            println!("Average FPS for {:?}: {}", self.args, fps);

                            let mut file = OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open("test_results.txt")
                                .unwrap();

                            file.write_all(format!("{}: {}\n", self.args, fps).as_bytes())
                                .unwrap();
                        }
                        let old_scene = self.args.scene;
                        let old_test = self.args.test;
                        let old_radius = self.args.radius;
                        self.args = *TESTS.get(self.test_step).unwrap_or_else(|| {
                            println!("No more tests to run");
                            event_loop.exit();
                            &self.args
                        });

                        println!("Switching to test: {}", self.args);
                        if self.args.scene != old_scene
                            || self.args.test != old_test
                            || self.args.radius != old_radius
                            || self.setup.is_none()
                        {
                            setup_test(self);
                        } else {
                            self.setup.as_mut().unwrap().combine(self.args.combine);
                            self.setup.as_mut().unwrap().cull(self.args.frustum_cull);
                        }

                        let new_cam = PerspectiveCamera::default();
                        self.state().cameras.active_mut().transform_mut().rotation =
                            new_cam.transform().rotation;
                        self.test_step += 1;
                        self.last_test_time = std::time::Instant::now();
                        self.state().new_frame();
                        self.state().wipe_fps();
                    } else {
                        setup_test(self);
                    }
                } else {
                    self.state().new_frame();
                }

                if self.args.auto_test {
                    self.state().cameras.active_mut().rotate(0.0, 0.1, false);
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
