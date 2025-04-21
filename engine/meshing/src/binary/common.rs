use std::collections::HashMap;

use dashmap::DashMap;
use glam::IVec3;
use renderer::{Axis, Dir};

use common::{BasicVoxel, BlockType, Voxel};

use super::culled::Chunk;

const CHUNK_SIZE_P: usize = 32;
pub const CHUNK_SIZE: usize = 30;

type Depth = u32;
pub type VoxelArray = [[[BasicVoxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
pub type AxisDepths = [[[Depth; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3];
type FaceDepths = Box<[[[Depth; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 6]>;
type TransformedBlockDepths = [HashMap<BlockType, Box<[[u32; CHUNK_SIZE]; CHUNK_SIZE]>>; 6];
type GreedyFaces = Vec<GreedyFace>;

#[derive(Debug)]
pub struct GreedyFace {
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub dir: Dir,
    pub width: u8,
    pub height: u8,
    pub block_type: BlockType,
}

pub static BLANK_VOXELS: [[[BasicVoxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE] =
    [[[BasicVoxel::new(BlockType::Air); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

pub struct ChunkRefs<'a> {
    pub chunk: &'a VoxelArray,
    pub x_pos: &'a VoxelArray,
    pub y_pos: &'a VoxelArray,
    pub z_pos: &'a VoxelArray,
    pub x_neg: &'a VoxelArray,
    pub y_neg: &'a VoxelArray,
    pub z_neg: &'a VoxelArray,
}

pub fn make_faces(
    chunks: &DashMap<IVec3, Chunk>,
    position: &IVec3,
    depths: &AxisDepths,
    greedy: bool,
) -> GreedyFaces {
    macro_rules! get_chunk {
        ($chunk_name:ident, $block_name:ident,$pos:expr) => {
            let $chunk_name = chunks.get($pos);
            let $chunk_name = if let Some(ref chunk) = $chunk_name {
                let voxels = chunk.voxels();
                let read = voxels.voxels.read().expect("Failed to read blocks");

                Some((chunk, voxels, read))
            } else {
                None
            };
            let $block_name = if let Some(ref read) = $chunk_name {
                let read = &read.2;
                read.as_ref()
            } else {
                &BLANK_VOXELS
            };
        };
    }
    get_chunk!(chunk_center, blocks_center, position);
    get_chunk!(
        chunk_x_neg,
        blocks_x_neg,
        &IVec3::new(position.x - 1, position.y, position.z)
    );
    get_chunk!(
        chunk_y_neg,
        blocks_y_neg,
        &IVec3::new(position.x, position.y - 1, position.z)
    );
    get_chunk!(
        chunk_z_neg,
        blocks_z_neg,
        &IVec3::new(position.x, position.y, position.z - 1)
    );
    get_chunk!(
        chunk_x_pos,
        blocks_x_pos,
        &IVec3::new(position.x + 1, position.y, position.z)
    );
    get_chunk!(
        chunk_y_pos,
        blocks_y_pos,
        &IVec3::new(position.x, position.y + 1, position.z)
    );
    get_chunk!(
        chunk_z_pos,
        blocks_z_pos,
        &IVec3::new(position.x, position.y, position.z + 1)
    );

    let refs = ChunkRefs {
        chunk: blocks_center,
        x_pos: blocks_x_pos,
        y_pos: blocks_y_pos,
        z_pos: blocks_z_pos,
        x_neg: blocks_x_neg,
        y_neg: blocks_y_neg,
        z_neg: blocks_z_neg,
    };

    if greedy {
        make_greedy_faces(&refs, depths)
    } else {
        make_culled_faces(&refs, depths)
    }
}

pub fn make_culled_faces(refs: &ChunkRefs, depths: &AxisDepths) -> GreedyFaces {
    let culled = cull_depths(depths);
    let block_faces = depths_to_faces(culled, refs);

    culled_faces(block_faces)
}

pub fn make_greedy_faces(chunks: &ChunkRefs, depths: &AxisDepths) -> GreedyFaces {
    let culled = cull_depths(depths);
    let block_faces = depths_to_faces(culled, chunks);

    greedy_faces(block_faces)
}

/// Build a depth mask for each axis.
/// Each integer is a view along the depth of that axis.
pub fn build_depths(chunks: &ChunkRefs) -> Box<AxisDepths> {
    let mut depths = Box::new([[[0; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3]);

    #[inline]
    fn add_voxel(
        solid: bool,
        x: usize,
        y: usize,
        z: usize,
        depths: &mut [[[Depth; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3],
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
                let v = chunks.chunk[x][y][z];
                // Add One to compensate for padding
                add_voxel(v.get_type().is_solid(), x + 1, y + 1, z + 1, &mut depths);
            }
        }
    }

    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            let min = chunks.x_pos[0][y][z];
            let max = chunks.x_neg[CHUNK_SIZE - 1][y][z];

            add_voxel(min.get_type().is_solid(), 0, y + 1, z + 1, &mut depths);
            add_voxel(
                max.get_type().is_solid(),
                CHUNK_SIZE + 1,
                y + 1,
                z + 1,
                &mut depths,
            );
        }
    }

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            let min = chunks.z_pos[x][y][0];
            let max = chunks.z_neg[x][y][CHUNK_SIZE - 1];

            add_voxel(min.get_type().is_solid(), x + 1, y + 1, 0, &mut depths);
            add_voxel(
                max.get_type().is_solid(),
                x + 1,
                y + 1,
                CHUNK_SIZE + 1,
                &mut depths,
            );
        }
    }

    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let min = chunks.y_pos[x][0][z];
            let max = chunks.y_neg[x][CHUNK_SIZE - 1][z];

            add_voxel(min.get_type().is_solid(), x + 1, 0, z + 1, &mut depths);
            add_voxel(
                max.get_type().is_solid(),
                x + 1,
                CHUNK_SIZE + 1,
                z + 1,
                &mut depths,
            );
        }
    }

    depths
}

/// Use binary shifting and binary not to locate
/// when we move between a solid to an air block
/// Store this in binary slices for all faces.
/// Each integer is a view along the depth of that axis.
pub fn cull_depths(depths: &AxisDepths) -> FaceDepths {
    let mut culled_faces = Box::new([[[0; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 6]);

    for axis in Axis::all() {
        for z in 0..CHUNK_SIZE_P {
            for x in 0..CHUNK_SIZE_P {
                let col = &depths[usize::from(axis)][z][x];

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
pub fn depths_to_faces(depths: FaceDepths, chunks: &ChunkRefs) -> TransformedBlockDepths {
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

                    let voxel = match dir {
                        Dir::Up | Dir::Down => chunks.chunk[x][y][z],
                        Dir::Left | Dir::Right => chunks.chunk[y][z][x],
                        Dir::Forward | Dir::Backward => chunks.chunk[x][z][y],
                    };

                    let block_type = voxel.get_type();

                    let data = faces[usize::from(dir)].entry(block_type).or_default();
                    data[y][x] |= 1 << z;
                }
            }
        }
    }

    faces
}

pub fn culled_faces(faces: TransformedBlockDepths) -> GreedyFaces {
    let mut culled = vec![];

    for dir in Dir::all() {
        for (block_type, dir_depth) in faces[usize::from(dir)].iter() {
            for depth in 0..CHUNK_SIZE {
                let faces = culled_face(&dir_depth[depth], depth as u8, dir, block_type);
                culled.extend(faces);
            }
        }
    }

    culled
}

pub fn culled_face(
    face: &[u32; CHUNK_SIZE],
    depth: u8,
    dir: Dir,
    block_type: &BlockType,
) -> Vec<GreedyFace> {
    let mut quads = vec![];

    const CS: u32 = CHUNK_SIZE as u32;

    (0..face.len()).for_each(|row| {
        let line = face[row];
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
                    dir,
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
    let mut greedy = vec![];

    for dir in Dir::all() {
        for (block_type, block_face) in depths[usize::from(dir)].iter_mut() {
            for depth in 0..CHUNK_SIZE {
                let faces = greedy_face(&mut block_face[depth], depth as u8, dir, block_type);
                greedy.extend(faces);
            }
        }
    }

    greedy
}

pub fn greedy_face(
    face: &mut [u32; CHUNK_SIZE],
    depth: u8,
    dir: Dir,
    block_type: &BlockType,
) -> Box<[GreedyFace]> {
    let mut quads = vec![];

    const CS: u32 = CHUNK_SIZE as u32;

    for row in 0..face.len() {
        let line = face[row];
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

            let h_mask = u32::checked_shl(1, h).map_or(!0, |v| v - 1);
            let mask = h_mask << y;

            let mut w = 1;

            while row + w < CHUNK_SIZE {
                let line = face[row + w];
                let next_row = (line >> y) & h_mask;

                if next_row != h_mask {
                    break;
                }

                face[row + w] &= !mask;

                w += 1;
            }

            if w != 0 && h != 0 {
                quads.push(GreedyFace {
                    x: row as u8,
                    y: y as u8,
                    z: depth,
                    dir,
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
