use renderer::{Axis, Dir};

const CHUNK_SIZE_P: usize = 34;
const CHUNK_SIZE: usize = 32;

type AxisDepths = Box<[[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3]>;
type FaceDepths = Box<[[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 6]>;
type TransformedDepths = Box<[[[u32; CHUNK_SIZE]; CHUNK_SIZE]; 6]>;
type GreedyFaces = Box<[Box<[GreedyFace]>; 6]>;

#[derive(Debug, Default)]
pub struct GreedyFace {
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub width: u8,
    pub height: u8,
}

pub fn make_culled_faces<F>(get_fn: F) -> TransformedDepths
where
    F: Fn(usize, usize, usize) -> bool,
{
    let depths = build_depths(get_fn);
    let culled = cull_depths(depths);
    depths_to_faces(culled)
}

/// Build a depth mask for each axis.
/// Each integer is a view along the depth of that axis.
pub fn build_depths<F>(get_fn: F) -> AxisDepths
where
    F: Fn(usize, usize, usize) -> bool,
{
    let mut depths = Box::new([[[0; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3]);

    #[inline]
    fn add_voxel(
        solid: bool,
        x: usize,
        y: usize,
        z: usize,
        depths: &mut Box<[[[u64; CHUNK_SIZE_P]; CHUNK_SIZE_P]; 3]>,
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
                let v = get_fn(x, y, z);
                // Add One to compensate for padding
                add_voxel(v, x + 1, y + 1, z + 1, &mut depths);
            }
        }
    }

    // TODO: Grab blocks from neighboring chunks

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
pub fn depths_to_faces(depths: FaceDepths) -> TransformedDepths {
    let mut faces = Box::new([[[0; CHUNK_SIZE]; CHUNK_SIZE]; 6]);

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

                    faces[usize::from(dir)][y][x] |= 1 << z;
                }
            }
        }
    }

    faces
}

pub fn greedy_faces(mut depths: TransformedDepths) -> GreedyFaces {
    let mut greedy = vec![vec![], vec![], vec![], vec![], vec![], vec![]];

    for dir in Dir::all() {
        let face = &mut greedy[usize::from(dir)];
        for depth in 0..CHUNK_SIZE {
            let faces = greedy_face(&mut depths[usize::from(dir)][depth], depth as u8);
            face.extend(faces);
        }
    }

    if greedy.len() != 6 {
        panic!("Greedy faces must have 6 faces, has {}", greedy.len());
    }

    let raw = Box::into_raw(
        greedy
            .into_iter()
            .map(|v| v.into_boxed_slice())
            .collect::<Vec<Box<[GreedyFace]>>>()
            .into_boxed_slice(),
    );
    unsafe { Box::from_raw(raw as *mut [Box<[GreedyFace]>; 6]) }
}

pub fn greedy_face(face: &mut [u32; CHUNK_SIZE], depth: u8) -> Box<[GreedyFace]> {
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
                });
            }

            y += h;
        }
    }

    quads.into_boxed_slice()
}
