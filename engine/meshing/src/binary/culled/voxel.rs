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

renderer::program!(culled_voxel, {
    #vertex vert
    #fragment frag

    uniform ivec3 chunk_position;

    #snippet renderer::camera_matrices
    #snippet crate::binary::common::get_pos

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

renderer::snippet!(vertex_pull_face_data, {
    const ivec3 vertices[4] = ivec3[](
        ivec3(0, 0, 0),
        ivec3(1, 0, 0),
        ivec3(0, 0, 1),
        ivec3(1, 0, 1)
    );

    const int indices[6] = int[](
        0, 1, 2,
        1, 3, 2
    );

    #bind 2
    buffer FaceData {
        uint face_data[];
    };
});

renderer::program!(culled_voxel_vertex_pull, {
    #vertex vert
    #fragment frag

    uniform ivec3 chunk_position;

    #snippet crate::binary::culled::voxel::vertex_pull_face_data
    #snippet renderer::camera_matrices
    #snippet crate::binary::common::get_pos

    struct v2f {
        vec4 color;
    }

    v2f vert() {
        v2f o;
        mat4 vp = camera.projection * camera.inverse_view;

        ivec3 v_pos = vertices[indices[gl_VertexID % 6]];
        uint face = face_data[gl_VertexID / 6];

        PlaneData data = unpack_data(v_pos, face, chunk_position);

        gl_Position = vp * vec4(data.position, 1.0);

        o.color = data.color;

        return o;
    }

    vec4 frag(v2f i) {
        return i.color;
    }
});

renderer::program!(culled_voxel_vertex_pull_combined, {
    #vertex vert
    #fragment frag

    #snippet crate::binary::culled::voxel::vertex_pull_face_data
    #snippet crate::binary::culled::voxel::combined_chunk_data
    #snippet renderer::camera_matrices
    #snippet crate::binary::common::get_pos

    struct v2f {
        vec4 color;
    }

    v2f vert() {
        v2f o;
        mat4 vp = camera.projection * camera.inverse_view;

        ivec3 v_pos = vertices[indices[gl_VertexID % 6]];
        uint face = face_data[gl_VertexID / 6];

        ivec3 chunk_position = chunk_positions[gl_DrawID];

        PlaneData data = unpack_data(v_pos, face, chunk_position);

        gl_Position = vp * vec4(data.position, 1.0);

        o.color = data.color;

        return o;
    }

    vec4 frag(v2f i) {
        return i.color;
    }
});

renderer::snippet!(combined_chunk_data, {
    #bind 1
    buffer ChunkData {
        ivec3 chunk_positions[];
    };
});

renderer::program!(culled_voxel_combined, {
    #vertex vert
    #fragment frag

    #snippet crate::binary::culled::voxel::combined_chunk_data
    #snippet renderer::camera_matrices
    #snippet crate::binary::common::get_pos

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

        ivec3 chunk_pos = chunk_positions[gl_DrawID];

        PlaneData data = unpack_data(v.v_pos, i.data, chunk_pos);

        gl_Position = vp * vec4(data.position, 1.0);

        o.color = data.color;

        return o;
    }

    vec4 frag(v2f i) {
        return i.color;
    }
});
