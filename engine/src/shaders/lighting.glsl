uniform vec4 sky_light_color = vec4(1.0, 1.0, 1.0, 1.0);
uniform vec3 sky_light_direction = normalize(vec3(-1.0, -1.0, -1.0));
uniform float ambient_light = 0.5;

vec4 apply_sky_lighting(vec4 color, vec3 normal, vec3 position) {
    float diffuse = max(dot(normal, sky_light_direction), 0.0);
    return color * (ambient_light + diffuse);
}
