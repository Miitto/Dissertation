#[macro_export]
macro_rules! event {
    () => {};
    ($str: literal) => {};
}

pub use crate::event;

pub fn start_capture() {}

pub fn stop_capture(_: &str) {}
pub fn next_frame() {}
