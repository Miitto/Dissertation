use glam::{Vec2, vec2};
use renderer::{
    DrawMode, ProgramSource, Renderable, State, buffers::UniformBuffer, mesh::basic::BasicMesh,
};

const VERTICES: [raymarching::Vertex; 4] = [
    raymarching::Vertex { pos: 0 },
    raymarching::Vertex { pos: 1 },
    raymarching::Vertex { pos: 2 },
    raymarching::Vertex { pos: 3 },
];

pub fn setup(state: &State) -> Box<dyn Renderable> {
    let mesh = BasicMesh::from_data(
        &VERTICES,
        None,
        None,
        None,
        false,
        false,
        DrawMode::TriangleStrip,
    );

    let resolution = state.display().window.inner_size();

    let resolution = vec2(resolution.width as f32, resolution.height as f32);

    let res_uniform = raymarching::uniforms::Resolution {
        res: resolution.to_array(),
    };

    let buffer = UniformBuffer::new(res_uniform).expect("Failed to make resolution buffer");

    Box::new(Screen {
        mesh,
        resolution,
        resolution_buffer: buffer,
    })
}

struct Screen {
    mesh: BasicMesh<raymarching::Vertex>,
    resolution: Vec2,
    resolution_buffer: UniformBuffer<raymarching::uniforms::Resolution>,
}

impl Renderable for Screen {
    fn render(&mut self, state: &mut renderer::State) {
        let program = raymarching::Program::get();

        state.draw(&mut self.mesh, &program, &self.resolution_buffer);
    }
}

renderer::program!(raymarching, {
    #vertex vert
    #fragment frag

    struct vIn {
        uint pos;
    }

    #snippet renderer::camera_matrices

    #bind 1
    uniform Resolution {
        vec2 res;
    } iResolution;

    void vert(vIn i) {
        vec2 pos = vec2(-1.0);

        switch (i.pos) {
            case 1: {
                pos.x = 1.0;
                break;
            }
            case 2: {
                pos.y = 1.0;
            break;
            }
            case 3: {
                pos.x = 1.0;
                pos.y = 1.0;
                break;
            }
        }

        gl_Position = vec4(pos, 0.0, 1.0);
    }

    struct Ray {
        vec3 origin;
        vec3 direction;
    }

    float sphere(vec3 sphere_pos, float radius, vec3 point_pos) {
        return length(point_pos - sphere_pos) - radius;
    }

    float map(vec3 position) {
        return max(-sphere(vec3(0, 0, -3), 1, position), sphere(vec3(2, 0, -2), 2, position));
    }

    Ray getRay(vec2 coord) {
        vec2 uv = (coord * 2 - iResolution.res.xy) / iResolution.res.y;

        vec4 far = camera.inverse_projection * vec4(uv, 1, 1);
        vec4 view = camera.view * far;

        Ray ray;
        ray.origin = camera.position;
        ray.direction = normalize(view.xyz / view.w);
        return ray;
    }


    vec4 frag() {
        const int MAX_STEPS = 80;

        Ray ray = getRay(gl_FragCoord.xy);

        vec3 color = vec3(0);

        float distance = 0.0;

        for (int i = 0; i < MAX_STEPS; ++i) {
            vec3 start_pos = ray.origin + ray.direction * distance;

            float d = map(start_pos);

            distance += d;


            if (d < 0.001 || distance > 1000.0) break;
        }

        color = vec3(distance * 0.1);

        return vec4(color, 1.0);
    }
});
