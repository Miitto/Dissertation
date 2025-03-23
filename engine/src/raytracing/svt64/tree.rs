use std::collections::HashMap;

use glam::{IVec3, ivec3};

use crate::common::BlockType;

use super::svt64::structs::Node;

impl Node {
    pub fn is_leaf(&self) -> bool {
        self.data_offset & 1 == 1
    }

    pub fn set_child_ptr(&mut self, val: u32) {
        let is_leaf = self.is_leaf();

        self.data_offset = val << 1;
        if is_leaf {
            self.data_offset |= 1;
        }
    }

    pub fn set_is_leaf(&mut self) {
        self.data_offset |= 1;
    }
}

pub fn generate_tree(voxels: &HashMap<[i32; 3], BlockType>) -> (Vec<Node>, Vec<u32>) {
    let mut nodes = vec![];
    let mut children = vec![];

    let max_axis = voxels.iter().fold(0, |acc, (p, _)| {
        p.iter().map(|v| v.abs()).max().unwrap().max(acc)
    }) as usize;

    // Inline of usize::div_ceil as it's unstable
    let max_scale = {
        let this = max_axis;
        let rhs = 4;
        let d = this / rhs;
        let r = this % rhs;
        let correction = 1 + ((this ^ rhs) >> (i32::BITS - 1));
        if r != 0 { d + correction } else { d }
    }
    .max(2);

    nodes[0] = recurse_tree(voxels, &mut nodes, &mut children, max_scale, ivec3(0, 0, 0));

    (nodes, children)
}

fn recurse_tree(
    voxels: &HashMap<[i32; 3], BlockType>,
    nodes: &mut Vec<Node>,
    child_data: &mut Vec<u32>,
    scale: usize,
    pos: IVec3,
) -> Node {
    let mut node = Node {
        data_offset: 0,
        mask_lower: 0,
        mask_upper: 0,
    };

    if scale == 2 {
        make_leaf(voxels, &mut node, child_data, pos);
    } else {
        let scale = scale - 2;

        let mut children = vec![];
        for x in -2..2 {
            for y in -2..2 {
                for z in -2..2 {
                    let mut child = recurse_tree(
                        voxels,
                        nodes,
                        child_data,
                        scale,
                        pos + ivec3(x << scale, y << scale, z << scale),
                    );

                    if child.mask_lower != 0 && child.mask_upper != 0 {
                        let offset = ((y + 2) * 16) + ((z + 2) * 8) + (x + 2);
                        if offset >= 32 {
                            child.mask_upper |= 1 << (offset - 32);
                        } else {
                            child.mask_lower |= 1 << offset;
                        }
                        children.push(child);
                    }
                }
            }
        }

        node.set_child_ptr(nodes.len() as u32);
        nodes.append(&mut children);
    }

    node
}

fn make_leaf(
    voxels: &HashMap<[i32; 3], BlockType>,
    tree: &mut Node,
    child_data: &mut Vec<u32>,
    pos: IVec3,
) {
    assert_eq!((pos.x | pos.y | pos.z) % 4, 0);

    // Repack Voxels into 4x4x4 tile
    // Using same index as reference for ease: `x + z*4 + y*16`
    let mut voxel_cube = Vec::with_capacity(64);

    for y in -2..2 {
        for z in -2..2 {
            for x in -2..2 {
                let position = [pos.x + x, pos.y + y, pos.z + z];

                let block = voxels.get(&position).unwrap_or(&BlockType::Air);
                voxel_cube.push(block);
            }
        }
    }

    tree.set_is_leaf();
    tree.set_child_ptr(child_data.len() as u32);
    child_data.extend(voxel_cube.iter().filter_map(|&v| {
        if v.is_solid() {
            Some(Into::<u32>::into(*v))
        } else {
            None
        }
    }));
}
