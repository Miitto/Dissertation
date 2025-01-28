use glium::{
    Display, Surface,
    glutin::surface::WindowSurface,
    winit::{
        application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
        window::Window,
    },
};
use renderer::{make_event_loop, make_window};
use shader_macros::shader;

shader!(VERT, Basic, 330, {
in vec3 position;

void main() {
    gl_Position = vec4(position, 1.0);
}
});

shader!(FRAG, Basic, 330, {
out vec4 color;

void main() {
    color = vec4(1.0, 0.0, 0.0, 1.0);
}
});

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

                    let v1 = BasicVertex {
                        position: [-0.5, -0.5, 0.],
                    };
                    let v2 = BasicVertex {
                        position: [0., 0.5, 0.],
                    };
                    let v3 = BasicVertex {
                        position: [0.5, -0.5, 0.],
                    };

                    let tri = vec![v1, v2, v3];

                    let v_buf = glium::VertexBuffer::new(display, &tri)
                        .expect("Failed to make tri v buffer");
                    let indices =
                        glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

                    let program = glium::Program::from_source(
                        display,
                        BasicVertex::source(),
                        BasicFragment::source(),
                        None,
                    )
                    .expect("Failed to make shader");

                    target
                        .draw(
                            &v_buf,
                            indices,
                            &program,
                            &glium::uniforms::EmptyUniforms,
                            &Default::default(),
                        )
                        .expect("Failed to draw tri");

                    target.finish().expect("Failed to finish target");
                }
            }
            _ => (),
        }
    }
}
