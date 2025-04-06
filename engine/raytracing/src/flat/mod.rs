use glam::{IVec3, ivec3};
use renderer::ComputeProgram;
use renderer::{Renderable, SSBO, State, buffers::ShaderBuffer};

use common::{Args, BlockType, tests::test_scene};

use super::{Screen, setup_screen};

#[allow(dead_code)]
struct FlatManager {
    voxels: Vec<Vec<Vec<BlockType>>>,
    min: IVec3,
    max: IVec3,
    buffer: ShaderBuffer<flat::buffers::Voxels>,
    screen: Screen,
}

pub fn setup(args: &Args, state: &State) -> Box<dyn Renderable> {
    let data = test_scene(args);

    let min = data.keys().fold(IVec3::MAX, |acc, &pos| {
        ivec3(acc.x.min(pos.x), acc.y.min(pos.y), acc.z.min(pos.z))
    });

    let max = data.keys().fold(IVec3::MIN, |acc, &pos| {
        ivec3(acc.x.max(pos.x), acc.y.max(pos.y), acc.z.max(pos.z))
    });

    let size = max - min + ivec3(1, 1, 1);

    let mut voxels =
        vec![vec![vec![BlockType::Air; size.z as usize]; size.y as usize]; size.x as usize];

    for (pos, voxel) in data {
        let pos = pos - min;
        voxels[pos.x as usize][pos.y as usize][pos.z as usize] = voxel;
    }

    let voxel_buf = flat::buffers::Voxels {
        min: min.to_array(),
        max: max.to_array(),
        size: size.to_array(),
        blocks: voxels
            .iter()
            .flatten()
            .flatten()
            .map(|&b| b as u32)
            .collect(),
    };

    let buffer = ShaderBuffer::single(&voxel_buf).unwrap();

    let screen = setup_screen(state);

    Box::new(FlatManager {
        voxels,
        min,
        max,
        buffer,
        screen,
    })
}

impl Renderable for FlatManager {
    fn render(&mut self, state: &mut renderer::State) {
        self.screen.pre_render(state);

        let compuse = flat::flat_main::get();

        self.buffer.bind();

        compuse.dispatch(self.screen.resolution.x, self.screen.resolution.y, 1);

        self.screen.post_render();
    }
}

renderer::compute!(flat, {
#kernel flat_main

#snippet renderer::camera_matrices
#snippet crate::ray
#include "shaders/block.glsl"

#bind 0
uniform image2D img;

struct DDA {
    vec3 pos;
    vec3 end;
    float hyp;
    float hyp_half;
    vec3 t_max;
    vec3 t_delta;
    vec3 step;
}

DDA make_cast(Ray ray) {
    const float RAY_LEN = 1000.0;

    vec3 current = ray.origin;
    vec3 end = ray.origin + (ray.direction * RAY_LEN);
    vec3 forward = ray.direction;

    vec3 delta = abs(end - current);
    vec3 step = sign(forward);

    float hypotenuse = length(delta);
    float hypotenuse_half = hypotenuse / 2.0;

    vec3 t_max = hypotenuse_half / delta;
    vec3 t_delta = hypotenuse / delta;

    DDA dda;
    dda.pos = current;
    dda.end = end;
    dda.hyp = hypotenuse;
    dda.hyp_half = hypotenuse_half;
    dda.t_max = t_max;
    dda.t_delta = t_delta;
    dda.step = step;

    return dda;
}

bool should_cast(DDA dda) {
    bool x = dda.step.x < 0.0 ? ( dda.pos.x >= dda.end.x ) : ( dda.pos.x <= dda.end.x );
    bool y = dda.step.y < 0.0 ? ( dda.pos.y >= dda.end.y ) : ( dda.pos.y <= dda.end.y );
    bool z = dda.step.z < 0.0 ? ( dda.pos.z >= dda.end.z ) : ( dda.pos.z <= dda.end.z );

    return ( x && y && z );
}

DDA next_cast(inout DDA dda) {
    if ( dda.t_max.x < dda.t_max.y ) {
        if ( dda.t_max.x < dda.t_max.z ) {
            dda.t_max.x = dda.t_max.x + dda.t_delta.x;
            dda.pos.x = dda.pos.x + dda.step.x;
        } else if ( dda.t_max.x > dda.t_max.z ) {
            dda.t_max.z = dda.t_max.z + dda.t_delta.z;
            dda.pos.z = dda.pos.z + dda.step.z;
        } else {
            dda.t_max.x = dda.t_max.x + dda.t_delta.x;
            dda.pos.x = dda.pos.x + dda.step.x;
            dda.t_max.z = dda.t_max.z + dda.t_delta.z;
            dda.pos.z = dda.pos.z + dda.step.z;
        }
    } else if ( dda.t_max.x > dda.t_max.y ) {
        if ( dda.t_max.y < dda.t_max.z ) {
            dda.t_max.y = dda.t_max.y + dda.t_delta.y;
            dda.pos.y = dda.pos.y + dda.step.y;
        } else if ( dda.t_max.y > dda.t_max.z ) {
            dda.t_max.z = dda.t_max.z + dda.t_delta.z;
            dda.pos.z = dda.pos.z + dda.step.z;
        } else {
            dda.t_max.y = dda.t_max.y + dda.t_delta.y;
            dda.pos.y = dda.pos.y + dda.step.y;
            dda.t_max.z = dda.t_max.z + dda.t_delta.z;
            dda.pos.z = dda.pos.z + dda.step.z;
        }
    } else if ( dda.t_max.y < dda.t_max.z ) {
        dda.t_max.x = dda.t_max.x + dda.t_delta.x;
        dda.pos.x = dda.pos.x + dda.step.x;
        dda.t_max.y = dda.t_max.y + dda.t_delta.y;
        dda.pos.y = dda.pos.y + dda.step.y;
    } else if ( dda.t_max.y > dda.t_max.z ) {
        dda.t_max.z = dda.t_max.z + dda.t_delta.z;
        dda.pos.z = dda.pos.z + dda.step.z;
    } else {
        dda.t_max.x = dda.t_max.x + dda.t_delta.x;
        dda.pos.x = dda.pos.x + dda.step.x;
        dda.t_max.y = dda.t_max.y + dda.t_delta.y;
        dda.pos.y = dda.pos.y + dda.step.y;
        dda.t_max.z = dda.t_max.z + dda.t_delta.z;
        dda.pos.z = dda.pos.z + dda.step.z;
    }

    return dda;
}

bool get_voxel(vec3 pos, DDA dda, out uint block) {
    ivec3 voxel_pos = ivec3(pos);
    ivec3 min = data.min;
    ivec3 max = data.max;

    bool x = dda.step.x < 0.0 ? ( voxel_pos.x < min.x ) : ( voxel_pos.x > max.x );
    bool y = dda.step.y < 0.0 ? ( voxel_pos.y < min.y ) : ( voxel_pos.y > max.y );
    bool z = dda.step.z < 0.0 ? ( voxel_pos.z < min.z ) : ( voxel_pos.z > max.z );

    if (x || y || z) {
        return true;
    }

    // Bounds check
    if (voxel_pos.x < min.x || voxel_pos.x >= max.x ||
        voxel_pos.y < min.y || voxel_pos.y >= max.y ||
        voxel_pos.z < min.z || voxel_pos.z >= max.z) {
        block = 255;
        return false;
    }

    ivec3 local_pos = voxel_pos - min;

    block = data.blocks[local_pos.x + local_pos.y * data.size.x + local_pos.z * data.size.x * data.size.y];
    return false;
}

#bind 2
buffer Voxels {
    ivec3 min;
    ivec3 max;
    ivec3 size;
    uint blocks[];
} data;

#size 1 1 1
void flat_main() {
    ivec2 screen_pos = ivec2(gl_GlobalInvocationID.xy);
    Ray ray = getRay(screen_pos);

    DDA dda = make_cast(ray);

    while (should_cast(dda)) {
        uint block = 0;
        bool out_of_bounds = get_voxel(dda.pos, dda, block);

        if (out_of_bounds) {
            break;
        }

        if (block != 0) {
            vec4 color = get_block_color(block);

            imageStore(img, screen_pos, color);
            if (block != 255)
            return;
        }

        dda = next_cast(dda);
    }
    imageStore(img, screen_pos, vec4(0.0));
}
});
