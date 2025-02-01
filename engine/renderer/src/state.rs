use benchmark::Benchmark;
use glium::{
    Display, DrawError, DrawParameters, Frame, Program, Surface, glutin::surface::WindowSurface,
    index, uniforms, vertex,
};

use crate::{
    Input,
    camera::{Camera, PerspectiveCamera},
};

pub struct State {
    pub input: Input,
    last_frame_time: std::time::Instant,
    delta_time: f32,
    pub camera: Box<dyn Camera>,
    pub frame_times: Vec<f32>,
    pub benchmark: Box<Benchmark>,
    pub target: Option<Frame>,
}

impl State {
    fn frame_time(&self) -> f32 {
        self.last_frame_time.elapsed().as_millis_f32()
    }

    pub fn delta(&self) -> f32 {
        self.delta_time
    }

    pub fn fps(&self) -> f32 {
        1000. / self.delta_time
    }

    pub fn new_frame(&mut self, display: &mut Display<WindowSurface>) {
        self.delta_time = self.frame_time();
        self.last_frame_time = std::time::Instant::now();

        self.benchmark.restart();

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), f32::MAX);

        self.target = Some(target);
    }

    pub fn end_frame(&mut self) {
        self.input.end_frame();
        self.frame_times.push(self.frame_time());
        self.benchmark.end();

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
        self.benchmark.draw();

        self.target.as_mut().unwrap().draw(
            vertex_buffer,
            index_buffer,
            program,
            uniforms,
            draw_parameters,
        )
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            input: Input::default(),
            last_frame_time: std::time::Instant::now(),
            delta_time: 0.,
            camera: Box::new(PerspectiveCamera::default()),
            frame_times: Vec::new(),
            benchmark: Box::new(Benchmark::new("Frame")),
            target: None,
        }
    }
}
