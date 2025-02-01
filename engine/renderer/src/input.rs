use std::collections::HashMap;

use glium::winit::{event::KeyEvent, keyboard::KeyCode};

#[derive(Clone, Debug, Default)]
pub struct PositionDelta {
    x: f64,
    y: f64,
}

impl PositionDelta {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }
}

#[derive(Clone, Debug, Default)]
pub struct Input {
    keys: HashMap<KeyCode, KeyEvent>,
    mouse: PositionDelta,
    wheel: f32,
}

impl Input {
    pub fn is_pressed(&self, key: &KeyCode) -> bool {
        if let Some(key_event) = self.keys.get(key) {
            key_event.state.is_pressed()
        } else {
            false
        }
    }

    pub fn set_key(&mut self, key: KeyCode, key_event: KeyEvent) {
        self.keys.insert(key, key_event);
    }

    pub fn keys(&self) -> &HashMap<KeyCode, KeyEvent> {
        &self.keys
    }

    pub fn mouse(&self) -> &PositionDelta {
        &self.mouse
    }

    pub fn mouse_move(&mut self, x: f64, y: f64) {
        self.mouse = PositionDelta::new(x, y);
    }

    pub fn wheel(&self) -> f32 {
        self.wheel
    }

    pub fn wheel_scroll(&mut self, delta: f32) {
        self.wheel = delta;
    }

    pub fn end_frame(&mut self) {
        self.mouse = PositionDelta::default();
        self.wheel = 0.;
    }
}
