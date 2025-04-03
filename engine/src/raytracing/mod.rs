use glam::{UVec2, uvec2};
use renderer::{
    Renderable, State,
    buffers::ShaderBuffer,
    framebuffer::{Framebuffer, TextureAttachPoint},
    texture::{ColorMode, Texture, Texture2D, TextureParameters},
};
use shaders::ComputeProgram;

pub mod esvo;
pub mod flat;
pub mod svt64;

pub fn setup_screen(state: &State) -> Screen {
    let resolution = state.display().window.inner_size();

    let resolution = uvec2(resolution.width, resolution.height);

    let res_uniform = raymarching::uses::ray::uniforms::iResolution {
        resolution: resolution.to_array(),
    };

    let buffer = ShaderBuffer::new(&[res_uniform]).expect("Failed to make resolution buffer");

    let texture = Texture2D::new(
        resolution.x,
        resolution.y,
        ColorMode::Rgba23f,
        TextureParameters {
            min_filter: renderer::texture::TextureFilterMode::Linear,
        },
    );

    texture.bind_to(0);

    let mut framebuffer = Framebuffer::default();
    framebuffer.set_tex_2d(TextureAttachPoint::Color0, &texture);

    framebuffer.bind();

    Screen {
        resolution,
        framebuffer,
        texture,
        resolution_buffer: buffer,
    }
}

impl Screen {
    pub fn check_resolution(&mut self, state: &State) {
        let resolution = state.display().window.inner_size();

        let resolution = uvec2(resolution.width, resolution.height);

        if resolution != self.resolution {
            self.resolution = resolution;

            let res_uniform = raymarching::uses::ray::uniforms::iResolution {
                resolution: resolution.to_array(),
            };

            if let Err(e) = self.resolution_buffer.set_data(&[res_uniform], 0) {
                eprintln!("Error setting resolution buffer: {:?}", e);
            }

            let tex = Texture2D::new(
                resolution.x,
                resolution.y,
                ColorMode::Rgba23f,
                TextureParameters {
                    min_filter: renderer::texture::TextureFilterMode::Linear,
                },
            );

            self.texture = tex;

            self.framebuffer
                .set_tex_2d(TextureAttachPoint::Color0, &self.texture);
        }
    }
}

pub fn setup(state: &State) -> Box<dyn Renderable> {
    let screen = setup_screen(state);

    Box::new(screen)
}

pub struct Screen {
    resolution: UVec2,
    framebuffer: Framebuffer,
    texture: Texture2D,
    resolution_buffer: ShaderBuffer<raymarching::uses::ray::uniforms::iResolution>,
}

impl Screen {
    pub fn pre_render(&mut self, state: &State) {
        self.check_resolution(state);
        self.resolution_buffer.bind();
    }

    pub fn post_render(&mut self) {
        self.framebuffer
            .blit_to_screen(self.resolution.x as i32, self.resolution.y as i32);
    }
}

impl Renderable for Screen {
    fn render(&mut self, state: &mut renderer::State) {
        self.pre_render(state);

        let compute = raymarching::cMain::get();

        compute.dispatch(self.resolution.x, self.resolution.y, 1);

        self.post_render();
    }
}

renderer::snippet!(ray, {
    #bind 1
    uniform iResolution {
        uvec2 resolution;
    };

    struct Ray {
        vec3 origin;
        vec3 direction;
    }

    Ray getRay(ivec2 screen_pos) {
        vec2 coord = vec2(screen_pos);

        vec2 uv = (coord * 2 - resolution.xy) / resolution.y;

        vec4 far = camera.inverse_projection * vec4(uv, 1, 1);
        vec4 view = camera.view * far;

        Ray ray;
        ray.origin = camera.position;
        ray.direction = normalize(view.xyz / view.w);
        return ray;
    }
});

renderer::compute!(raymarching, {
    #kernel cMain

    #snippet renderer::camera_matrices
    #snippet crate::raytracing::ray

    #bind 0
    uniform image2D img;

    float sphere(vec3 sphere_pos, float radius, vec3 point_pos) {
        return length(point_pos - sphere_pos) - radius;
    }

    float map(vec3 position) {
        return max(-sphere(vec3(0, 0, -3), 1, position), sphere(vec3(2, 0, -2), 2, position));
    }

    #size 1 1 1
    void cMain() {
        ivec2 screen_pos = ivec2(gl_GlobalInvocationID.xy);
        const int MAX_STEPS = 80;

        Ray ray = getRay(screen_pos);

        vec3 color = vec3(0);

        float distance = 0.0;

        for (int i = 0; i < MAX_STEPS; ++i) {
            vec3 start_pos = ray.origin + ray.direction * distance;

            float d = map(start_pos);

            distance += d;


            if (d < 0.001 || distance > 1000.0) break;
        }

        color = vec3(distance * 0.1);

        imageStore(img, screen_pos, vec4(color, 1.0));
    }
});
