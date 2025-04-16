use std::collections::HashMap;

use glam::ivec3;
use renderer::{Axis, Dir};

use common::BlockType;

const CHUNK_SIZE_P: usize = 34;
pub const CHUNK_SIZE: usize = 32;

type AxisDepths = Box<[[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3]>;
type FaceDepths = Box<[[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 6]>;
type TransformedBlockDepths = [HashMap<BlockType, Box<[[u32; CHUNK_SIZE]; CHUNK_SIZE]>>; 6];
type GreedyFaces = Vec<Vec<GreedyFace>>;

#[derive(Debug, Default)]
pub struct GreedyFace {
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub width: u8,
    pub height: u8,
    pub block_type: BlockType,
}

pub fn make_culled_faces<F>(get_fn: F) -> Vec<Vec<GreedyFace>>
where
    F: Fn(isize, isize, isize) -> BlockType,
{
    let depths = build_depths(&get_fn);
    let culled = cull_depths(depths);
    let block_faces = depths_to_faces(culled, get_fn);

    culled_faces(block_faces)
}

pub fn make_greedy_faces<F>(get_fn: F) -> GreedyFaces
where
    F: Fn(isize, isize, isize) -> BlockType,
{
    let depths = build_depths(&get_fn);
    let culled = cull_depths(depths);

    let block_faces = depths_to_faces(culled, get_fn);

    greedy_faces(block_faces)
}

/// Build a depth mask for each axis.
/// Each integer is a view along the depth of that axis.
pub fn build_depths<F>(get_fn: F) -> AxisDepths
where
    F: Fn(isize, isize, isize) -> BlockType,
{
    let mut depths = Box::new([[[0; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3]);

    #[inline]
    fn add_voxel(
        solid: bool,
        x: usize,
        y: usize,
        z: usize,
        depths: &mut [[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3],
    ) {
        if solid {
            depths[usize::from(Axis::X)][y][z] |= 1 << x;
            depths[usize::from(Axis::Y)][z][x] |= 1 << y;
            depths[usize::from(Axis::Z)][y][x] |= 1 << z;
        }
    }

    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let v = get_fn(x as isize, y as isize, z as isize);
                // Add One to compensate for padding
                add_voxel(v.is_solid(), x + 1, y + 1, z + 1, &mut depths);
            }
        }
    }

    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            let min = get_fn(-1, y as isize, z as isize);
            let max = get_fn(CHUNK_SIZE as isize, y as isize, z as isize);

            add_voxel(min.is_solid(), 0, y + 1, z + 1, &mut depths);
            add_voxel(max.is_solid(), CHUNK_SIZE + 1, y + 1, z + 1, &mut depths);
        }
    }

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            let min = get_fn(x as isize, y as isize, -1);
            let max = get_fn(x as isize, y as isize, CHUNK_SIZE as isize);

            add_voxel(min.is_solid(), x + 1, y + 1, 0, &mut depths);
            add_voxel(max.is_solid(), x + 1, y + 1, CHUNK_SIZE + 1, &mut depths);
        }
    }

    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            let min = get_fn(x as isize, -1, z as isize);
            let max = get_fn(x as isize, CHUNK_SIZE as isize, z as isize);

            add_voxel(min.is_solid(), x + 1, 0, z + 1, &mut depths);
            add_voxel(max.is_solid(), x + 1, CHUNK_SIZE + 1, z + 1, &mut depths);
        }
    }

    depths
}

/// Use binary shifting and binary not to locate
/// when we move between a solid to an air block
/// Store this in binary slices for all faces.
/// Each integer is a view along the depth of that axis.
pub fn cull_depths(depths: AxisDepths) -> FaceDepths {
    let mut culled_faces = Box::new([[[0; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 6]);

    for axis in Axis::all() {
        for z in 0..CHUNK_SIZE_P {
            for x in 0..CHUNK_SIZE_P {
                let col = depths[usize::from(axis)][z][x];

                // Binary not against a left / right shift
                // only leaves a 1 where you moved from 0-1 in the
                // opposite direction of the shift
                //       1->0
                //     001100 &
                // !<< 100111 =
                //     000100
                culled_faces[2 * usize::from(axis)][z][x] = col & !(col << 1);
                culled_faces[2 * usize::from(axis) + 1][z][x] = col & !(col >> 1);
            }
        }
    }

    culled_faces
}

/// Transform depth from going along the integer, to the horizonal axis (X-Z) going along the integer.
pub fn depths_to_faces<F>(depths: FaceDepths, get_fn: F) -> TransformedBlockDepths
where
    F: Fn(isize, isize, isize) -> BlockType,
{
    let mut faces: TransformedBlockDepths = [
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    ];

    for dir in Dir::all() {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                // Cut the padding, as it's not part of the chunk
                let mut col = depths[usize::from(dir)][z + 1][x + 1];
                col >>= 1;
                col &= !(1 << CHUNK_SIZE);

                while col != 0 {
                    // Get the depth
                    let y = col.trailing_zeros() as usize;

                    // Clear least signicant set bit
                    // This marks that depth as read,
                    // so we can get the next
                    col &= col - 1;

                    let pos = match dir {
                        Dir::Up | Dir::Down => ivec3(x as i32, y as i32, z as i32),
                        Dir::Left | Dir::Right => ivec3(y as i32, z as i32, x as i32),
                        Dir::Forward | Dir::Backward => ivec3(x as i32, z as i32, y as i32),
                    };

                    let block_type = get_fn(pos.x as isize, pos.y as isize, pos.z as isize);

                    let data = faces[usize::from(dir)].entry(block_type).or_default();
                    data[y][x] |= 1 << z;
                }
            }
        }
    }

    faces
}

pub fn culled_faces(faces: TransformedBlockDepths) -> Vec<Vec<GreedyFace>> {
    let mut culled = vec![vec![], vec![], vec![], vec![], vec![], vec![]];

    for dir in Dir::all() {
        let face = &mut culled[usize::from(dir)];
        for (block_type, dir_depth) in faces[usize::from(dir)].iter() {
            for depth in 0..CHUNK_SIZE {
                let faces = culled_face(&dir_depth[depth], depth as u8, block_type);
                face.extend(faces);
            }
        }
    }

    culled
}

pub fn culled_face(face: &[u32; 32], depth: u8, block_type: &BlockType) -> Vec<GreedyFace> {
    let mut quads = vec![];

    const CS: u32 = CHUNK_SIZE as u32;

    (0..face.len()).for_each(|row| {
        let line = face[row] as u64;
        if line == 0 {
            return;
        }

        let mut y = 0u32;

        while y < CS {
            y += (face[row] >> y).trailing_zeros();

            if y > CS {
                continue;
            }

            let h = (line >> y).trailing_ones();

            for h in 0..h {
                quads.push(GreedyFace {
                    x: row as u8,
                    y: (y + h) as u8,
                    z: depth,
                    width: 0,
                    height: 0,
                    block_type: *block_type,
                });
            }

            y += h;
        }
    });

    quads
}

pub fn greedy_faces(mut depths: TransformedBlockDepths) -> GreedyFaces {
    let mut greedy = vec![vec![], vec![], vec![], vec![], vec![], vec![]];

    for dir in Dir::all() {
        let face = &mut greedy[usize::from(dir)];
        for (block_type, block_face) in depths[usize::from(dir)].iter_mut() {
            for depth in 0..CHUNK_SIZE {
                let faces = greedy_face(&mut block_face[depth], depth as u8, block_type);
                face.extend(faces);
            }
        }
    }

    greedy
}

pub fn greedy_face(
    face: &mut [u32; CHUNK_SIZE],
    depth: u8,
    block_type: &BlockType,
) -> Box<[GreedyFace]> {
    let mut quads = vec![];

    const CS: u32 = CHUNK_SIZE as u32;

    for row in 0..face.len() {
        let line = face[row] as u64;
        if line == 0 {
            continue;
        }
        let mut y = 0u32;

        while y < CS {
            y += (face[row] >> y).trailing_zeros();

            if y > CS {
                continue;
            }

            let h = (line >> y).trailing_ones();

            let h_mask = u64::checked_shl(1, h).map_or(!0, |v| v - 1);
            let mask = h_mask << y;

            let mut w = 1;

            while row + w < CHUNK_SIZE {
                let line = face[row + w] as u64;
                let next_row = (line >> y) & h_mask;

                if next_row != h_mask {
                    break;
                }

                face[row + w] &= (!mask) as u32;

                w += 1;
            }

            if w != 0 && h != 0 {
                quads.push(GreedyFace {
                    x: row as u8,
                    y: y as u8,
                    z: depth,
                    width: w as u8 - 1,
                    height: h as u8 - 1,
                    block_type: *block_type,
                });
            }

            y += h;
        }
    }

    quads.into_boxed_slice()
}

renderer::snippet!(get_pos, {
    #include "shaders/lighting.glsl"
    #include "shaders/block.glsl"

    struct PlaneData {
        vec3 position;
        vec4 color;
    }

    PlaneData unpack_data(ivec3 v_pos, uint instance_data, ivec3 chunk_position) {
        int v_x = v_pos.x;
        int v_y = v_pos.y;
        int v_z = v_pos.z;

        int in_x = int((instance_data >> 10) & 31);
        int in_y = int((instance_data >> 5) & 31);
        int in_z = int(instance_data & 31);

        uint direction = (instance_data >> 15) & 7;

        uint width = (instance_data >> 18) & 31;
        uint height = (instance_data >> 23) & 31;

        uint block_type = (instance_data >> 28);

        int w = int(width) + 1;
        int h = int(height) + 1;

        int x;
        int y;
        int z;

        ivec3 normal = ivec3(0, 0, 0);

        // left right up down forward back
        switch (direction) {
            // Left
            case 0: {
                x = 0;
                y = (1-v_x) * h;
                z = v_z * w;

                normal.x = -1;
                break;
            }
            // Right
            case 1: {
                x = 1;
                y = v_x * h;
                z = v_z * w;

                normal.x = 1;
                break;
            }
            // Up
            case 2: {
                x = v_x * w;
                y = 0;
                z = v_z * h;

                normal.y = 1;
                break;
            }
            // Down
            case 3: {
                x = (1-v_x) * w;
                y = 1;
                z = v_z * h;

                normal.y = -1;
                break;
            }
            // Forward
            case 4: {
                z = 0;
                x = (1-v_z) * w;
                y = (1-v_x) * h;

                normal.z = -1;
                break;
            }
            // Backward
            case 5: {
                z = 1;
                x = (1-v_x) * w;
                y = (1-v_z) * h;

                normal.z = 1;
                break;
            }
        }

        vec4 color = get_block_color(block_type);

        int c_x = chunk_position.x;
        int c_y = chunk_position.y;
        int c_z = chunk_position.z;

        if (chunk_position.x < 0) {
            c_x += 1;
        }
        if (chunk_position.y < 0) {
            c_y += 1;
        }
        if (chunk_position.z < 0) {
            c_z += 1;
        }

        int o_x = x + in_x + c_x;
        int o_y = y + in_y + c_y;
        int o_z = z + in_z + c_z;

        vec3 position = vec3(float(o_x), float(o_y), float(o_z));

        vec4 lit = apply_sky_lighting(color, normal, position);

        PlaneData data;
        data.position = position;
        data.color = lit;

        return data;
    }
});
