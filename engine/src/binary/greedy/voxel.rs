#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlockType {
    Air,
    Grass,
    // etc.
}

#[derive(Clone, Copy, Debug)]
pub struct Voxel {
    block_type: BlockType,
}

impl Voxel {
    pub fn new(block_type: BlockType) -> Self {
        Self { block_type }
    }

    pub fn is_solid(&self) -> bool {
        self.block_type != BlockType::Air
    }
    pub fn set_type(&mut self, block_type: BlockType) {
        self.block_type = block_type;
    }
}

impl greedy_voxel::Vertex {
    pub fn new(v_pos: [f32; 3]) -> Self {
        Self { v_pos }
    }
}

shaders::program!(greedy_voxel, {
    #vertex vert
    #fragment frag

    uniform mat4 projectionMatrix;
    uniform mat4 viewMatrix;
    uniform mat4 modelMatrix;

    #include "shaders/lighting.glsl"

    struct vIn {
        vec3 v_pos;
    }

    struct iIn {
        uint data;
    }

    struct v2f {
        vec4 color;
    }

    v2f vert(vIn v, iIn i) {
        v2f o;

        mat4 vp = projectionMatrix * viewMatrix;
        mat4 mvp = vp * modelMatrix;

        float v_x = v.v_pos.x;
        float v_y = v.v_pos.y;
        float v_z = v.v_pos.z;

        float in_x = float((i.data >> 10) & 31);
        float in_y = float((i.data >> 5) & 31);
        float in_z = float(i.data & 31);

        uint direction = (i.data >> 15) & 7;

        uint width = (i.data >> 18) & 31;
        uint height = (i.data >> 23) & 31;

        float w = float(width + 1);
        float h = float(height + 1);

        float x;
        float y;
        float z;

        vec3 normal = vec3(0.0, 0.0, 0.0);

        // left right up down forward back
        switch (direction) {
            // Left
            case 0: {
                x = 0;
                y = (1-v_x) * h;
                z = v_z * w;
                // Magenta
                o.color = vec4(1.0, 0.0, 1.0, 1.0);

                normal.x = -1.0;
                break;
            }
            // Right
            case 1: {
                x = 1;
                y = v_x * h;
                z = v_z * w;
                // Cyan
                o.color = vec4(0.0, 1.0, 1.0, 1.0);

                normal.x = 1.0;
                break;
            }
            // Up
            case 2: {
                x = v_x * w;
                y = 0;
                z = v_z * h;
                o.color = vec4(1.0, 0.0, 0.0, 1.0);

                normal.y = 1.0;
                break;
            }
            // Down
            case 3: {
                x = (1-v_x) * w;
                y = 1;
                z = v_z * h;
                o.color = vec4(1.0, 1.0, 0.0, 1.0);

                normal.y = -1.0;
                break;
            }
            // Forward
            case 4: {
                z = 0;
                x = (1-v_z) * w;
                y = (1-v_x) * h;
                o.color = vec4(0.0, 1.0, 0.0, 1.0);

                normal.z = -1.0;
                break;
            }
            // Backward
            case 5: {
                z = 1;
                x = v_x * w;
                y = (1-v_z) * h;
                o.color = vec4(v_x, 0.0, 1.0, 1.0);

                normal.z = 1.0;
                break;
            }
        }

        float o_x = x + in_x;
        float o_y = y + in_y;
        float o_z = z + in_z;

        vec3 position = vec3(o_x, o_y, o_z);

        o.color = apply_sky_lighting(o.color, normal, position);


        gl_Position = mvp * vec4(position, 1.0);

        return o;
    }

    vec4 frag(v2f i) {
        return i.color;
    }
});
