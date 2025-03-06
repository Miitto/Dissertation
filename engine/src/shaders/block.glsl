vec4 get_block_color(uint block_type) {
    vec3 color = vec3(0.0, 0.0, 0.0);
    switch (block_type) {
        case 1:
        {
            color = vec3(0.1, 0.5, 0.1);
            break;
        }
        case 2:
        {
            color = vec3(0.3, 0.3, 0.3);
            break;
        }
        case 3:
        {
            color = vec3(0.7, 0.7, 0.7);
        }
    }

    return vec4(color, 1.0);
}
