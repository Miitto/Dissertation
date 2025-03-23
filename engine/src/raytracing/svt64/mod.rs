/// Code in this module is based on this article:
/// https://dubiousconst282.github.io/2024/10/03/voxel-ray-tracing/
use renderer::{Renderable, SSBO, State, buffers::ShaderBuffer};
use shaders::ComputeProgram;
use svt64::buffers::{VoxelLeaves, VoxelNodes};
use tree::generate_tree;

use crate::{Args, tests::test_scene};

use super::Screen;

mod tree;

pub struct Svt64Renderer {
    pub screen: Screen,
    pub nodes: VoxelNodes,
    pub leaves: VoxelLeaves,
    pub node_buffer: ShaderBuffer<svt64::buffers::VoxelNodes>,
    pub leaf_buffer: ShaderBuffer<svt64::buffers::VoxelLeaves>,
}

impl Renderable for Svt64Renderer {
    fn render(&mut self, state: &mut State) {
        self.screen.pre_render(state);

        let compute = svt64::c_main::get();

        self.node_buffer.bind();
        self.leaf_buffer.bind();

        compute.dispatch(self.screen.resolution.x, self.screen.resolution.y, 1);

        self.screen.post_render();
    }
}

pub fn setup(args: &Args, state: &State) -> Box<dyn Renderable> {
    let screen = super::setup_screen(state);

    let voxels = test_scene(args);

    let (nodes, leaf_data) = generate_tree(&voxels);

    let nodes = VoxelNodes { nodes };
    let leaves = VoxelLeaves { data: leaf_data };

    let node_buffer = ShaderBuffer::single(&nodes).expect("Failed to make node buffer");
    let leaf_buffer = ShaderBuffer::single(&leaves).expect("Failed to make leaf buffer");

    Box::new(Svt64Renderer {
        screen,
        nodes,
        leaves,
        node_buffer,
        leaf_buffer,
    })
}

renderer::compute!(svt64, {
    #kernel c_main
    #snippet renderer::camera_matrices
    #snippet crate::raytracing::ray

    #bind 0
    uniform image2D img;

    struct Node {
        uint data_offset;
        uint mask_upper;
        uint mask_lower;
    }

    #bind 2
    buffer VoxelNodes {
        Node nodes[];
    } nodes;

    #bind 3
    buffer VoxelLeaves {
        uint data[];
    } leaf_data;

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
