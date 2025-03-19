impl greedy_voxel::Vertex {
    pub fn new(v_pos: [i32; 3]) -> Self {
        Self { v_pos }
    }
}
impl greedy_voxel_combined::Vertex {
    pub fn new(v_pos: [i32; 3]) -> Self {
        Self { v_pos }
    }
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
                x = v_x * w;
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

shaders::program!(greedy_voxel, {
    #vertex vert
    #fragment frag

    uniform ivec3 chunk_position;

    #snippet renderer::camera_matrices
    #snippet crate::meshing::binary::greedy::voxel::get_pos

    struct vIn {
        ivec3 v_pos;
    }

    struct iIn {
        uint data;
    }

    struct v2f {
        vec4 color;
    }

    v2f vert(vIn v, iIn i) {
        v2f o;

        mat4 vp = camera.projection * camera.inverse_view;

        PlaneData data = unpack_data(v.v_pos, i.data, chunk_position);

        gl_Position = vp * vec4(data.position, 1.0);

        o.color = data.color;

        return o;
    }

    vec4 frag(v2f i) {
        return i.color;
    }
});

shaders::program!(greedy_voxel_combined, {
    #vertex vert
    #fragment frag

    #bind 1
    buffer ChunkData {
        ivec3 pos[];
    } chunk_positions;

    #snippet renderer::camera_matrices
    #snippet crate::meshing::binary::greedy::voxel::get_pos

    struct vIn {
        ivec3 v_pos;
    }

    struct iIn {
        uint data;
    }

    struct v2f {
        vec4 color;
    }

    v2f vert(vIn v, iIn i) {
        v2f o;

        mat4 vp = camera.projection * camera.inverse_view;

        ivec3 chunk_pos = chunk_positions.pos[gl_DrawID];

        PlaneData data = unpack_data(v.v_pos, i.data, chunk_pos);


        gl_Position = vp * vec4(data.position, 1.0);

        o.color = data.color;

        return o;
    }

    vec4 frag(v2f i) {
        return i.color;
    }
});
