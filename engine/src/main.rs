use glium::{
    Display, Surface,
    glutin::surface::WindowSurface,
    winit::{
        application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
        window::Window,
    },
};
use renderer::{make_event_loop, make_window};

fn main() {
    let event_loop = make_event_loop();

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}

#[derive(Default)]
struct App {
    window: Option<Window>,
    display: Option<Display<WindowSurface>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, display) = make_window(event_loop);

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
            }
            WindowEvent::RedrawRequested => {
                if let Some(display) = &mut self.display {
                    let mut target = display.draw();

                    target.clear_color(0.0, 0.0, 1.0, 1.0);

                    target.finish().expect("Failed to finish target");
                }
            }
            _ => (),
        }
    }
}
