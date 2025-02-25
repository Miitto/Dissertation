use std::collections::HashMap;

use winit::{
    event::{ElementState, KeyEvent, MouseButton},
    keyboard::KeyCode,
    window::Window,
};

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

#[derive(Debug, Clone, Default)]
pub struct MouseData {
    position: PositionDelta,
    buttons: HashMap<MouseButton, ElementState>,
    is_locked: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Input {
    keys: HashMap<KeyCode, KeyEvent>,
    mouse: MouseData,
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

    pub fn mouse_pos(&self) -> &PositionDelta {
        &self.mouse.position
    }

    pub fn mouse_move(&mut self, x: f64, y: f64) {
        self.mouse.position = PositionDelta::new(x, y);
    }

    pub fn click(&mut self, button: MouseButton, state: ElementState) {
        self.mouse.buttons.insert(button, state);
    }

    pub fn is_clicked(&self, button: MouseButton) -> bool {
        self.mouse
            .buttons
            .get(&button)
            .map(|s| s.is_pressed())
            .unwrap_or(false)
    }

    pub fn is_cursor_locked(&self) -> bool {
        self.mouse.is_locked
    }

    pub fn lock_cursor(&mut self, window: &Window) {
        self.mouse.is_locked = true;
        window.set_cursor_visible(false);
        _ = window.set_cursor_grab(winit::window::CursorGrabMode::Confined);
    }

    pub fn unlock_cursor(&mut self, window: &Window) {
        self.mouse.is_locked = false;
        window.set_cursor_visible(true);
        _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
    }

    pub fn wheel(&self) -> f32 {
        self.wheel
    }

    pub fn wheel_scroll(&mut self, delta: f32) {
        self.wheel = delta;
    }

    pub fn end_frame(&mut self) {
        self.mouse.position = PositionDelta::default();
        self.wheel = 0.;
    }
}
