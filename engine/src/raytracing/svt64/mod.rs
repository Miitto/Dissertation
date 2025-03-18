use renderer::{Renderable, State, buffers::ShaderBuffer};
use tree::generate_tree;

use crate::{Args, tests::test_scene};

use super::Screen;

mod tree;

pub struct Svt64Renderer {
    pub screen: Screen,
    pub node_buffer: ShaderBuffer<svt64::buffers::VoxelNode>,
    pub leaf_buffer: ShaderBuffer<svt64::buffers::VoxelLeaf>,
}

pub fn setup(args: &Args, state: &State) -> Box<dyn Renderable> {
    let screen = super::setup_screen(state);

    let voxels = test_scene(args);

    let (node, nodes, leaf_data) = generate_tree(&voxels);

    Box::new(screen)
}

renderer::compute!(svt64, {
    #kernel c_main
    #snippet renderer::camera_matrices
    #snippet crate::raytracing::ray

    #bind 0
    uniform image2D img;

    #bind 2
    buffer VoxelNode {
        uint data_offset;
        uint mask_upper;
        uint mask_lower;
    } nodes[];

    #bind 3
    buffer VoxelLeaf {
        uint data;
    } leaf_data[];

    float sphere(vec3 sphere_pos, float radius, vec3 point_pos) {
        return length(point_pos - sphere_pos) - radius;
    }

    float map(vec3 position) {
        return max(-sphere(vec3(0, 0, -3), 1, position), sphere(vec3(2, 0, -2), 2, position));
    }

    #size 1 1 1
    void c_main() {
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
