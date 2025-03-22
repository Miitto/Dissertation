pub struct App {
    display: Option<renderer::Display>,
    bench: fn(c: &mut criterion::Criterion),
}

impl App {
    pub fn new(bench: fn(c: &mut criterion::Criterion)) -> Self {
        Self {
            display: None,
            bench,
        }
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let display = renderer::make_window(event_loop);

        self.display = Some(display);

        let mut criterion = criterion::Criterion::default().configure_from_args();

        (self.bench)(&mut criterion);

        criterion::Criterion::default()
            .configure_from_args()
            .final_summary();

        event_loop.exit();
    }

    fn window_event(
        &mut self,
        _: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        _: winit::event::WindowEvent,
    ) {
    }
}

pub fn run(bench: fn(c: &mut criterion::Criterion)) {
    let event_loop = renderer::make_event_loop();

    let mut app = App::new(bench);

    _ = event_loop.run_app(&mut app);
}
