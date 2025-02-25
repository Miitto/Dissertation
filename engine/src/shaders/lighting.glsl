uniform vec4 sky_light_color;
uniform vec3 sky_light_direction;
uniform float ambient_light;

vec4 apply_sky_lighting(vec4 color, vec3 normal, vec3 position) {
    float diffuse = max(dot(normal, sky_light_direction), 0.0);
    return color * (ambient_light + diffuse);
}
