impl greedy_voxel::Vertex {
    pub fn new(v_pos: [i32; 3]) -> Self {
        Self { v_pos }
    }
}

shaders::program!(greedy_voxel, {
    #vertex vert
    #fragment frag

    uniform mat4 projectionMatrix;
    uniform mat4 viewMatrix;

    uniform ivec3 chunk_position;

    #include "shaders/lighting.glsl"

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

        mat4 vp = projectionMatrix * viewMatrix;

        int v_x = v.v_pos.x;
        int v_y = v.v_pos.y;
        int v_z = v.v_pos.z;

        uint in_x = (i.data >> 10) & 31;
        uint in_y = (i.data >> 5) & 31;
        uint in_z = i.data & 31;

        uint direction = (i.data >> 15) & 7;

        uint width = (i.data >> 18) & 31;
        uint height = (i.data >> 23) & 31;

        uint block_type = (i.data >> 28);

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

        vec3 color = vec3(0.0, 0.0, 0.0);

        switch (block_type) {
            case 1: {
                color = vec3(0.1, 0.75, 0.1);
                break;
            }
            case 2: {
                color = vec3(0.25, 0.25, 0.25);
                break;
            }
            case 3: {
                color = vec3(0.8, 0.8, 0.8);
            }
        }

        int o_x = x + int(in_x) + chunk_position.x;
        int o_y = y + int(in_y) + chunk_position.y;
        int o_z = z + int(in_z) + chunk_position.z;

        vec3 position = vec3(float(o_x), float(o_y), float(o_z));

        o.color = apply_sky_lighting(vec4(color, 1.0), normal, position);


        gl_Position = vp * vec4(position, 1.0);

        return o;
    }

    vec4 frag(v2f i) {
        return i.color;
    }
});
