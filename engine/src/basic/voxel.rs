use std::{cell::RefCell, rc::Rc};

use glam::{IVec3, Mat4, Vec3, mat4, vec4};
use renderer::{Renderable, State, draw::draw, mesh::basic::BasicMesh};
use shaders::Program;

use crate::common::BlockType;

pub struct Voxel {
    position: IVec3,
    pub block_type: BlockType,
    mesh: Rc<RefCell<BasicMesh<basic_voxel::Vertex>>>,
}

impl Voxel {
    pub fn new(
        position: IVec3,
        block_type: BlockType,
        mesh: Rc<RefCell<BasicMesh<basic_voxel::Vertex>>>,
    ) -> Self {
        Self {
            position,
            block_type,
            mesh,
        }
    }
}

impl basic_voxel::Vertex {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self {
            position: [x, y, z],
        }
    }
}

impl Voxel {
    pub fn get_vertices() -> Vec<basic_voxel::Vertex> {
        // 0
        let fbl = basic_voxel::Vertex::new(0, 0, 0);
        // 1
        let ftl = basic_voxel::Vertex::new(0, 1, 0);
        // 2
        let ftr = basic_voxel::Vertex::new(1, 1, 0);
        // 3
        let fbr = basic_voxel::Vertex::new(1, 0, 0);
        // 4
        let bbl = basic_voxel::Vertex::new(0, 0, 1);
        // 5
        let btl = basic_voxel::Vertex::new(0, 1, 1);
        // 6
        let btr = basic_voxel::Vertex::new(1, 1, 1);
        // 7
        let bbr = basic_voxel::Vertex::new(1, 0, 1);

        vec![fbl, ftl, ftr, fbr, bbl, btl, btr, bbr]
    }

    pub fn get_indices() -> Vec<u32> {
        vec![
            0, 1, 2, 2, 3, 0, // Front
            7, 6, 5, 5, 4, 7, // Back
            4, 5, 1, 1, 0, 4, // Left
            3, 2, 6, 6, 7, 3, // Right
            1, 5, 6, 6, 2, 1, // Top
            4, 0, 3, 3, 7, 4, // Bottom
        ]
    }

    pub fn get_model_matrix(&self) -> Mat4 {
        let position = vec4(
            self.position[0] as f32,
            self.position[1] as f32,
            self.position[2] as f32,
            1.0,
        );

        let forward = vec4(0., 0., -1., 0.);
        let right = vec4(1., 0., 0., 0.);
        let up = vec4(0., 1., 0., 0.);

        mat4(right, up, forward, position)
    }

    pub fn get_position(&self) -> IVec3 {
        self.position
    }
}

impl Renderable for Voxel {
    fn render(&mut self, state: &mut State) {
        let uniforms = basic_voxel::Uniforms {
            modelMatrix: self.get_model_matrix().to_cols_array_2d(),
            viewMatrix: state.cameras.active().get_view().to_cols_array_2d(),
            projectionMatrix: state.cameras.active().get_projection().to_cols_array_2d(),
            block_type: self.block_type.into(),
        };

        let program = basic_voxel::Program::get();

        draw(&mut *self.mesh.borrow_mut(), &program, &uniforms, state);
    }
}

shaders::program!(basic_voxel, {
#vertex vertex
#fragment frag

uniform mat4 modelMatrix;
uniform mat4 viewMatrix;
uniform mat4 projectionMatrix;
uniform uint block_type;

struct vIn {
    ivec3 position;
}

struct v2f {
    vec4 color;
}

#include "shaders/block.glsl"

v2f vertex(vIn i) {
    v2f o;
    mat4 pv = projectionMatrix * viewMatrix;

    mat4 mvp = pv * modelMatrix;

    o.color = get_block_color(block_type);

    gl_Position = mvp * vec4(i.position, 1.0);
    return o;
}

vec4 frag(v2f i) {
    return i.color;
}
});

shaders::program!(instanced_voxel, {
    #vertex vert
    #fragment frag

    uniform mat4 viewMatrix;
    uniform mat4 projectionMatrix;

    struct vIn {
        ivec3 position;
    }

    struct iIn {
        ivec3 pos;
        uint block_type;
    }

    struct v2f {
        vec4 color;
    }

    #include "shaders/block.glsl"

    v2f vert(vIn i, iIn ii) {
        v2f o;
        mat4 pv = projectionMatrix * viewMatrix;


        o.color = get_block_color(ii.block_type);

        vec3 pos = i.position + ii.pos;

        gl_Position = pv * vec4(pos, 1.0);
        return o;
    }

    vec4 frag(v2f i) {
        return i.color;
    }
});
