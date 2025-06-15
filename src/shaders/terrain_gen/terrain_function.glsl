#include "../../../lygia/generative/snoise.glsl"
uint terrain_function(ivec3 pos) {
    vec3 noise_pos = vec3(pos) * 0.025;
    noise_pos.x -= snoise(noise_pos);
    float noise_value = snoise(noise_pos) * 15.;

    uint block_type = 0;
    if (noise_value - pos.y > 0.) {
        block_type = 1;
    }

    return block_type;
}