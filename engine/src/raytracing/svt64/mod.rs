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

    dbg!(nodes.nodes.len());
    dbg!(leaves.data.len());

    println!("Creating node buffer");
    let node_buffer = ShaderBuffer::single(&nodes).expect("Failed to make node buffer");
    println!("Creating leaf buffer");
    let leaf_buffer = ShaderBuffer::single(&leaves).expect("Failed to make leaf buffer");

    println!("Setup complete");
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

    #include "shaders/block.glsl"

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

    // Bit conversion operations

    float asfloat( float x ){ return x; }
    vec2  asfloat( vec2  x ){ return x; }
    vec3  asfloat( vec3  x ){ return x; }
    vec4  asfloat( vec4  x ){ return x; }

    float asfloat( int   x ){ return intBitsToFloat(x); }
    vec2  asfloat( ivec2 x ){ return intBitsToFloat(x); }
    vec3  asfloat( ivec3 x ){ return intBitsToFloat(x); }
    vec4  asfloat( ivec4 x ){ return intBitsToFloat(x); }

    float asfloat( uint  x ){ return uintBitsToFloat(x); }
    vec2  asfloat( uvec2 x ){ return uintBitsToFloat(x); }
    vec3  asfloat( uvec3 x ){ return uintBitsToFloat(x); }
    vec4  asfloat( uvec4 x ){ return uintBitsToFloat(x); }


    int   asint( int   x ){ return x; }
    ivec2 asint( ivec2 x ){ return x; }
    ivec3 asint( ivec3 x ){ return x; }
    ivec4 asint( ivec4 x ){ return x; }

    int   asint( uint  x ){ return int(x);   }
    ivec2 asint( uvec2 x ){ return ivec2(x); }
    ivec3 asint( uvec3 x ){ return ivec3(x); }
    ivec4 asint( uvec4 x ){ return ivec4(x); }

    int   asint( float x ){ return floatBitsToInt(x); }
    ivec2 asint( vec2  x ){ return floatBitsToInt(x); }
    ivec3 asint( vec3  x ){ return floatBitsToInt(x); }
    ivec4 asint( vec4  x ){ return floatBitsToInt(x); }


    uint  asuint( uint  x ){ return x; }
    uvec2 asuint( uvec2 x ){ return x; }
    uvec3 asuint( uvec3 x ){ return x; }
    uvec4 asuint( uvec4 x ){ return x; }

    uint  asuint( int  x  ){ return  uint(x); }
    uvec2 asuint( ivec2 x ){ return uvec2(x); }
    uvec3 asuint( ivec3 x ){ return uvec3(x); }
    uvec4 asuint( ivec4 x ){ return uvec4(x); }

    uint  asuint( float x ){ return floatBitsToUint(x); }
    uvec2 asuint( vec2  x ){ return floatBitsToUint(x); }
    uvec3 asuint( vec3  x ){ return floatBitsToUint(x); }
    uvec4 asuint( vec4  x ){ return floatBitsToUint(x); }

    vec3 select(bvec3 condition, vec3 a, vec3 b) {
        return mix(b, a, condition);
    }

    ivec3 select(bvec3 condition, ivec3 a, ivec3 b) {
        return mix(b, a, condition);
    }

    uvec3 select(bvec3 condition, uvec3 a, uvec3 b) {
        return mix(b, a, condition);
    }

    bvec3 vec3_less_than(vec3 a, vec3 b) {
        return bvec3(a.x < b.x, a.y < b.y, a.z < b.z);
    }

    bvec3 vec3_less_than(vec3 a, float b) {
        return bvec3(a.x < b, a.y < b, a.z < b);
    }

    bvec3 vec3_greater_than(vec3 a, vec3 b) {
        return bvec3(a.x > b.x, a.y > b.y, a.z > b.z);
    }

    bvec3 vec3_greater_than(vec3 a, float b) {
        return bvec3(a.x > b, a.y > b, a.z > b);
    }

    bvec3 vec3_greater_than_eq(vec3 a, float b) {
        return bvec3(a.x >= b, a.y >= b, a.z >= b);
    }

    bvec3 vec3_eq(vec3 a, float b) {
        return bvec3(a.x == b, a.y == b, a.z == b);
    }

    uint get_node_cell_index(vec3 position, int scale) {
        uvec3 cell_pos = (asuint(position) >> scale & 3u);
        return cell_pos.x + cell_pos.z * 4u + cell_pos.y * 16u;
    }

    vec3 floor_scale(vec3 position, int scale) {
        uint mask = (~0u << scale);
        return asfloat(asuint(position) & mask); // Erase bits lower than scale
    }

    bool is_leaf(Node node) {
        return ((node.data_offset & 1) != 0);
    }

    bool has_child_at(Node node, uint idx) {
        if (idx >= 32) {
            return ((node.mask_upper & (1u << (idx - 32))) != 0);
        } else {
            return ((node.mask_lower & (1u << idx)) != 0);
        }
    }

    uint child_count(Node node) {
        return bitCount(node.mask_lower) + bitCount(node.mask_upper);
    }

    uint child_count_below(Node node, uint idx) {
        if (idx >= 32) {
            uint mask = (1u << (idx - 32)) - 1;
            return bitCount(node.mask_upper & mask) + bitCount(node.mask_lower);

        } else {
            uint mask = (1u << idx) -1;
            return bitCount(node.mask_lower & mask);
        }
    }

    uint child_ptr(Node node) {
        return (node.data_offset >> 1);
    }

    vec3 get_mirrored_pos(Ray ray, bool range_check) {
        vec3 mirrored = asfloat(asuint(ray.origin) ^ 0x7FFFFF);

        if (range_check && any(vec3_less_than(ray.origin, 1.0) || vec3_greater_than_eq(ray.origin, 2.0))) mirrored = 3.0 - ray.origin;
        return select(vec3_less_than(ray.direction, 0), mirrored, ray.origin);
    }

    #size 1 1 1
    void c_main() {
        ivec2 screen_pos = ivec2(gl_GlobalInvocationID.xy);

        Ray ray = getRay(screen_pos);

        vec4 color = vec4(0);

        uint stack[11];
        int scale_exp = 21; // 0.25 as mantissa bit offset

        uint node_idx = 0;
        Node node = nodes.nodes[0];

        uint mirror_mask = 0;
        if (ray.direction.x > 0) mirror_mask = (mirror_mask | (3 << 0));
        if (ray.direction.y > 0) mirror_mask = (mirror_mask | (3 << 4));
        if (ray.direction.z > 0) mirror_mask = (mirror_mask | (3 << 2));

        vec3 origin = get_mirrored_pos(ray, true);

        vec3 pos = clamp(origin, 1.0, 1.99999999999);
        vec3 inv_dir = 1.0 / -abs(ray.direction);

        Ray inv_ray;
        inv_ray.origin = origin;
        inv_ray.direction = inv_dir;

        vec3 side_dist;

        for (int i = 0; i < 256; i++) {
            uint child_idx = get_node_cell_index(pos, scale_exp) ^ mirror_mask;

            while (!is_leaf(node) && has_child_at(node, child_idx) && scale_exp >= 2) {
                stack[scale_exp >> 1] = node_idx; // Save ancestor

                node_idx = child_ptr(node) + child_count_below(node, child_idx);
                node = nodes.nodes[node_idx];

                scale_exp -= 2;
                child_idx = get_node_cell_index(pos, scale_exp) ^ mirror_mask;
            }

            if (has_child_at(node, child_idx) && is_leaf(node)) break;

            int adv_scale_exp = scale_exp;
            if ((node.mask_lower >> (child_idx & 42) & 0x00330033) == 0) adv_scale_exp++;

            vec3 cell_min = floor_scale(pos, adv_scale_exp);

            side_dist = (cell_min - inv_ray.origin) * inv_ray.direction;
            float t_max = min(min(side_dist.x, side_dist.y), side_dist.z);

            ivec3 neighbor_max = asint(cell_min) + select(vec3_eq(side_dist, t_max), ivec3(-1), ivec3((1 << adv_scale_exp) - 1));
            pos = min(origin - abs(ray.direction) * t_max, asfloat(neighbor_max));

            // Common ancestor
            uvec3 diff_pos = asuint(pos) ^ asuint(cell_min);

            int diff_exp = findMSB((diff_pos.x | diff_pos.y | diff_pos.z) & 0xFFAAAAAA);

            if (diff_exp > scale_exp) {
                scale_exp = diff_exp;
                if (diff_exp > 21) break;

                node_idx = stack[scale_exp >> 1];
                node = nodes.nodes[node_idx];
            }
        }

        if (is_leaf(node) && scale_exp <= 21) {
            Ray r;
            r.origin = pos;
            r.direction = ray.direction;

            pos = get_mirrored_pos(r, false);
            uint child_idx = get_node_cell_index(pos, scale_exp);
            uint block_type = leaf_data.data[child_ptr(node) + child_count_below(node, child_idx)];

            color = get_block_color(block_type);
        }

        imageStore(img, screen_pos,color);
    }
});
