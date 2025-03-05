impl culled_voxel::Vertex {
    pub fn new(v_pos: [f32; 3]) -> Self {
        Self { v_pos }
    }
}

shaders::program!(culled_voxel, {
    #vertex vert
    #fragment frag

    uniform mat4 projectionMatrix;
    uniform mat4 viewMatrix;
    uniform mat4 modelMatrix;

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

        float in_x = float(i.data >> 10 & 31);
        float in_y = float((i.data >> 5) & 31);
        float in_z = float(i.data & 31);

        uint direction = (i.data >> 15) & 7;

        float x;
        float y;
        float z;

        // left right up down forward back
        switch (direction) {
            case 0: {
                x = 0;
                y = 1-v_x;
                z = v_z;
                o.color = vec4(1.0, 0.0, 1.0, 1.0);
                break;
            }
            case 1: {
                x = 1;
                y = v_x;
                z = v_z;
                o.color = vec4(0.0, 1.0, 1.0, 1.0);
                break;
            }
            case 2: {
                x = v_x;
                y = 0;
                z = v_z;
                o.color = vec4(1.0, 0.0, 0.0, 1.0);
                break;
            }
            case 3: {
                x = 1-v_x;
                y = 1;
                z = v_z;
                o.color = vec4(1.0, 1.0, 0.0, 1.0);
                break;
            }
            case 4: {
                z = 0;
                x = v_x;
                y = v_z;
                o.color = vec4(0.0, 1.0, 0.0, 1.0);
                break;
            }
            case 5: {
                z = 1;
                x = 1-v_x;
                y = v_z;
                o.color = vec4(0.0, 0.0, 1.0, 1.0);
                break;
            }
        }

        float o_x = x + in_x;
        float o_y = y + in_y;
        float o_z = z + in_z;

        gl_Position = mvp * vec4(o_x, o_y, o_z, 1.0);


        return o;
    }

    vec4 frag(v2f i) {
        return i.color;
    }
});
