#include "../../../lygia/generative/snoise.glsl"
uint terrain_function2(ivec3 pos) {
    vec3 noise_pos = vec3(pos) * 0.025;

    float noise_value = snoise(noise_pos) * 15.;
    noise_value -= length(pos);

    uint block_type = 0;
    if (noise_value - pos.y > 0.) {
        block_type = 1;
    }

    return block_type;
}

uint terrain_function(ivec3 pos) {
    vec3 noise_pos = vec3(pos) * 0.025;

    for(int i = 0; i < 15; i++) {
        noise_pos += snoise(noise_pos + vec3(100., 250., 73.)) * 0.1;
    }


    float noise_value = snoise(noise_pos) * 15.;


    uint block_type = 0;
    if (noise_value - pos.y > 0.) {
        block_type = 1;
    }

    return block_type;
}