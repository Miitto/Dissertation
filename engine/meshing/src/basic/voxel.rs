use std::{cell::RefCell, rc::Rc};

use glam::IVec3;
use renderer::{ProgramSource, Renderable, State, mesh::basic::BasicMesh};

use common::BlockType;

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

    pub fn get_position(&self) -> IVec3 {
        self.position
    }
}

impl Renderable for Voxel {
    fn render(&mut self, state: &mut State) {
        let uniforms = basic_voxel::Uniforms {
            block_position: self.position.into(),
            block_type: self.block_type.into(),
        };

        let program = basic_voxel::Program::get();

        state.draw(&mut *self.mesh.borrow_mut(), &program, &uniforms);
    }

    fn args(&mut self, _args: &common::Args) {
        todo!()
    }
}

renderer::program!(basic_voxel, {
#vertex vertex
#fragment frag

#snippet renderer::camera_matrices

uniform uint block_type;
uniform ivec3 block_position;

struct vIn {
    ivec3 position;
}

struct v2f {
    vec4 color;
}

#include "shaders/block.glsl"

v2f vertex(vIn i) {
    v2f o;
    mat4 pv = camera.projection * camera.inverse_view;

    o.color = get_block_color(block_type);

    ivec3 pos = i.position + block_position;

    o.color.x = float(block_position.x);

    gl_Position = pv * vec4(pos, 1.0);
    return o;
}

vec4 frag(v2f i) {
    return i.color;
}
});

renderer::program!(instanced_voxel, {
    #vertex vert
    #fragment frag

    #snippet renderer::camera_matrices

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
        mat4 pv = camera.projection * camera.inverse_view;

        o.color = get_block_color(ii.block_type);

        vec3 pos = i.position + ii.pos;

        gl_Position = pv * vec4(pos, 1.0);
        return o;
    }

    vec4 frag(v2f i) {
        return i.color;
    }
});

renderer::program!(vertex_pull, {
    #vertex vertex
    #fragment frag

    #snippet renderer::camera_matrices

    #include "shaders/block.glsl"

    const ivec3 vertices[8] = ivec3[](
        ivec3(0, 0, 0),
        ivec3(0, 1, 0),
        ivec3(1, 1, 0),
        ivec3(1, 0, 0),
        ivec3(0, 0, 1),
        ivec3(0, 1, 1),
        ivec3(1, 1, 1),
        ivec3(1, 0, 1)
    );

    const int indices[36] = int[](
        0, 1, 2, 2, 3, 0, // Front
        7, 6, 5, 5, 4, 7, // Back
        4, 5, 1, 1, 0, 4, // Left
        3, 2, 6, 6, 7, 3, // Right
        1, 5, 6, 6, 2, 1, // Top
        4, 0, 3, 3, 7, 4 // Bottom
    );

    #bind 1
    buffer BlockPosition {
        ivec4 block_data[];
    };

    struct v2f {
        vec4 color;
    }

    v2f vertex() {
        v2f o;
        mat4 pv = camera.projection * camera.inverse_view;

        int index = gl_VertexID % 36;
        int offset = indices[index];

        ivec3 vertex = vertices[offset];

        int block = gl_VertexID / 36;
        ivec3 block_position = block_data[block].xyz;
        uint block_type = uint(block_data[block].w);

        o.color = get_block_color(block_type);

        ivec3 pos = vertex + block_position;

        gl_Position = pv * vec4(pos, 1.0);
        return o;
    }

    vec4 frag(v2f i) {
        return i.color;
    }
});
