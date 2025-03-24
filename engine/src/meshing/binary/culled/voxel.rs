impl culled_voxel::Vertex {
    pub fn new(v_pos: [i32; 3]) -> Self {
        Self { v_pos }
    }
}
impl culled_voxel_combined::Vertex {
    pub fn new(v_pos: [i32; 3]) -> Self {
        Self { v_pos }
    }
}

shaders::program!(culled_voxel, {
    #vertex vert
    #fragment frag

    uniform ivec3 chunk_position;

    #snippet renderer::camera_matrices
    #snippet crate::meshing::binary::common::get_pos

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

shaders::program!(culled_voxel_combined, {
    #vertex vert
    #fragment frag

    #bind 1
    buffer ChunkData {
        ivec3 pos[];
    } chunk_positions;

    #snippet renderer::camera_matrices
    #snippet crate::meshing::binary::common::get_pos

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
