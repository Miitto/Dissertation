#![feature(duration_millis_float)]

mod voxel;
use std::collections::HashMap;

use glium::{
    Display, Surface,
    glutin::surface::WindowSurface,
    uniform,
    winit::{
        application::ApplicationHandler,
        event::{ElementState, KeyEvent, WindowEvent},
        event_loop::ActiveEventLoop,
        keyboard::{KeyCode, PhysicalKey},
        window::Window,
    },
};
use renderer::{
    Renderable,
    camera::{Camera, PerspectiveCamera},
    make_event_loop, make_window,
};
use shaders::Program;
use voxel::Voxel;

shaders::program!(Basic, 330, {
in vec3 position;
in vec4 color;

uniform mat4 modelMatrix;
uniform mat4 viewMatrix;
uniform mat4 projectionMatrix;

out vec4 vertexColor;

void main() {
    mat4 pv = projectionMatrix * viewMatrix;

    mat4 mvp = pv * modelMatrix;

    vertexColor = color;

    gl_Position = mvp * vec4(position, 1.0);
}
},
{
in vec4 vertexColor;

out vec4 color;

void main() {
    color = vertexColor;
}
});

fn main() {
    let event_loop = make_event_loop();

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}

struct App {
    window: Option<Window>,
    display: Option<Display<WindowSurface>>,
    camera: Box<dyn Camera>,
    last_frame_time: std::time::Instant,
    keys: HashMap<PhysicalKey, KeyEvent>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            display: None,
            camera: Box::new(PerspectiveCamera::default()),
            last_frame_time: std::time::Instant::now(),
            keys: HashMap::new(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, display) = make_window(event_loop);

        let size = window.inner_size();
        self.camera
            .on_window_resize(size.width as f32, size.height as f32);

        self.window = Some(window);
        self.display = Some(display);
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
                if let Some(display) = &mut self.display {
                    display.resize(window_size.into());
                }
                self.camera
                    .on_window_resize(window_size.width as f32, window_size.height as f32);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.keys.insert(event.physical_key, event);
            }
            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                let delta = self.last_frame_time.elapsed().as_millis_f32();
                self.last_frame_time = now;

                self.camera.handle_input(&self.keys, delta);

                if let Some(display) = &mut self.display {
                    let mut target = display.draw();

                    target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), f32::MAX);

                    let voxel = Voxel::new([0, 0, 0]);
                    let voxel_up = Voxel::new([0, 1, 0]);
                    let voxel_down = Voxel::new([0, -1, 0]);
                    let voxel_right = Voxel::new([1, 0, 0]);

                    voxel.render(display, &mut target, self.camera.as_ref());
                    voxel_up.render(display, &mut target, self.camera.as_ref());
                    voxel_right.render(display, &mut target, self.camera.as_ref());
                    voxel_down.render(display, &mut target, self.camera.as_ref());

                    target.finish().expect("Failed to finish target");

                    self.window.as_ref().unwrap().request_redraw();
                }
            }
            _ => (),
        }
    }
}
