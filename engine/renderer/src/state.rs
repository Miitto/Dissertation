use std::collections::HashMap;

use glium::{
    Display, DrawError, DrawParameters, Frame, Program, Surface,
    glutin::surface::WindowSurface,
    index, uniforms, vertex,
    winit::{
        event::{ElementState, KeyEvent, MouseButton},
        keyboard::KeyCode,
        window::Window,
    },
};

use crate::{
    Input, PositionDelta,
    camera::{Camera, PerspectiveCamera},
};

pub struct State {
    pub window: Option<Window>,
    pub display: Option<Display<WindowSurface>>,
    input: Input,
    last_frame_time: std::time::Instant,
    delta_time: f32,
    pub camera: Box<dyn Camera>,
    pub target: Option<Frame>,
}

impl State {
    pub fn new_window(&mut self, window: Window, display: Display<WindowSurface>) {
        self.window = Some(window);
        self.display = Some(display);
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
        if let Some(display) = &self.display {
            optick::next_frame();
            self.delta_time = self.frame_time();
            self.last_frame_time = std::time::Instant::now();

            let mut target = display.draw();
            target.clear_color_and_depth((0.1, 0.1, 0.1, 1.0), f32::MAX);

            self.target = Some(target);
        }
    }

    pub fn end_frame(&mut self) {
        optick::event!("State End Frame");
        self.input.end_frame();

        if let Some(target) = self.target.take() {
            _ = target.finish();
        }
    }

    pub fn draw<'a, 'b, V, I, U>(
        &mut self,
        vertex_buffer: V,
        index_buffer: I,
        program: &Program,
        uniforms: &U,
        draw_parameters: &DrawParameters<'_>,
    ) -> Result<(), DrawError>
    where
        I: Into<index::IndicesSource<'a>>,
        U: uniforms::Uniforms,
        V: vertex::MultiVerticesSource<'b>,
    {
        optick::event!("Draw Call");
        self.target.as_mut().unwrap().draw(
            vertex_buffer,
            index_buffer,
            program,
            uniforms,
            draw_parameters,
        )
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

    pub fn mouse_move(&mut self, x: f64, y: f64) {
        self.input.mouse_move(x, y);
    }

    pub fn click(&mut self, button: MouseButton, state: ElementState) {
        if button == MouseButton::Right {
            if state.is_pressed() {
                if let Some(window) = &self.window {
                    self.input.lock_cursor(window);
                }
            } else if let Some(window) = &self.window {
                self.input.unlock_cursor(window);
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
        self.camera.handle_input(&self.input, delta);
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            window: None,
            display: None,
            input: Input::default(),
            last_frame_time: std::time::Instant::now(),
            delta_time: 0.,
            camera: Box::new(PerspectiveCamera::default()),
            target: None,
        }
    }
}
