use std::{collections::HashMap, rc::Rc};

use glutin::surface::GlSurface;
use render_common::{Display, Program};
use winit::{
    event::{ElementState, KeyEvent, MouseButton},
    keyboard::KeyCode,
};

use crate::{Input, PositionDelta, Uniforms, camera::CameraManager, mesh::Mesh, vertex::Vertex};

pub struct State {
    display: Option<Rc<Display>>,
    input: Input,
    last_frame_time: std::time::Instant,
    delta_time: f32,
    pub cameras: CameraManager,
}

impl State {
    pub fn draw<M, U, V, I>(&self, mesh: &mut M, program: &Program, uniforms: &U)
    where
        V: Vertex,
        I: Vertex,
        M: Mesh<V, I>,
        U: Uniforms,
    {
        program.bind();
        uniforms.bind(program);
        self.cameras.bind_camera_uniforms();

        mesh.render(&self.cameras.game_frustum());
    }

    pub fn display(&self) -> Rc<Display> {
        self.display
            .as_ref()
            .expect("Display is not initialized")
            .clone()
    }

    pub fn new_window(&mut self, display: Display) {
        self.display = Some(Rc::new(display));
    }

    fn frame_time(&self) -> f32 {
        self.last_frame_time.elapsed().as_millis_f32()
    }

    pub fn delta(&self) -> f32 {
        self.delta_time
    }

    pub fn fps(&self) -> f32 {
        1000. / self.delta_time
    }

    pub fn new_frame(&mut self) {
        self.delta_time = self.frame_time();
        self.last_frame_time = std::time::Instant::now();

        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::ClearDepth(1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        };
    }

    pub fn end_frame(&mut self) {
        self.input.end_frame();

        let display = self.display();

        unsafe {
            gl::Finish();
            _ = display.surface.swap_buffers(&display.context);
        }
    }

    pub fn is_pressed(&self, key: &KeyCode) -> bool {
        self.input.is_pressed(key)
    }

    pub fn set_key(&mut self, key: KeyCode, key_event: KeyEvent) {
        self.input.set_key(key, key_event);
    }

    pub fn keys(&self) -> &HashMap<KeyCode, KeyEvent> {
        self.input.keys()
    }

    pub fn mouse_pos(&self) -> &PositionDelta {
        self.input.mouse_pos()
    }

    pub fn mouse_move(&mut self, x: f32, y: f32) {
        self.input.mouse_move(x, y);
    }

    pub fn click(&mut self, button: MouseButton, state: ElementState) {
        if button == MouseButton::Right {
            let display = self.display();
            if state.is_pressed() {
                self.input.lock_cursor(display.get_window());
            } else {
                self.input.unlock_cursor(display.get_window());
            }
        }

        self.input.click(button, state);
    }

    pub fn is_clicked(&self, button: MouseButton) -> bool {
        self.input.is_clicked(button)
    }

    pub fn wheel(&self) -> f32 {
        self.input.wheel()
    }

    pub fn wheel_scroll(&mut self, delta: f32) {
        self.input.wheel_scroll(delta);
    }

    pub fn handle_input(&mut self) {
        let delta = self.delta();
        self.cameras.handle_input(&self.input, delta);
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            display: None,
            input: Input::default(),
            last_frame_time: std::time::Instant::now(),
            delta_time: 0.,
            cameras: CameraManager::default(),
        }
    }
}
