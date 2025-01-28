use glium::{
    Display, Surface,
    glutin::surface::WindowSurface,
    winit::{
        event::{Event, WindowEvent},
        event_loop::ActiveEventLoop,
        window,
    },
};

fn main() {
    let (event_loop, window, mut display) = renderer::make_window();

    let _ = event_loop.run(move |event, window_target| match event {
        Event::WindowEvent { event, .. } => {
            on_window_event(event, window_target, &mut display);
        }
        Event::AboutToWait => {
            window.request_redraw();
        }
        _ => (),
    });
}

fn on_window_event(
    event: WindowEvent,
    window_target: &ActiveEventLoop,
    display: &mut Display<WindowSurface>,
) {
    match event {
        WindowEvent::CloseRequested => window_target.exit(),
        WindowEvent::Resized(window_size) => {
            display.resize(window_size.into());
        }
        WindowEvent::RedrawRequested => {}
        _ => (),
    }
}
