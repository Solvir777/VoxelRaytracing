layout (constant_id = 0) const int render_distance = 4;

layout(local_size_x = 16, local_size_y = 16, local_size_z = 1) in;
layout(set = 0, binding = 1) writeonly buffer LookingAtBlock{
    vec3 hit_point;
    uint block_id; // 0 means no hit (air)
    vec3 hit_normal;
} looking_at;
layout(set = 0, binding = 2) readonly uniform GpuGraphicsSettings{
    float fov;
} settings;
layout(r16ui, set = 0, binding = 3) readonly uniform uimage3D block_data[(render_distance * 2 + 1) * (render_distance * 2 + 1) * (render_distance * 2 + 1)];
layout(r8ui, set = 0, binding = 4) readonly uniform uimage3D distance_data[(render_distance * 2 + 1) * (render_distance * 2 + 1) * (render_distance * 2 + 1)];
layout(set = 0, binding = 5) uniform texture2DArray textures;
layout(set = 0, binding = 6) uniform sampler texture_sampler;


layout(push_constant) uniform PushConstants {
    mat4 cam_transform;
} push;

const vec3 sun_ray = normalize(vec3(0.267, 0.886, -0.16));


#include "../util.glsl"

const vec3[] debug_colors = {vec3(1., 0., 0.), vec3(0., 1., 0.), vec3(0., 0., 1.)};

vec3 player_position = push.cam_transform[3].xyz;

uint read_block(ivec3 pos) {
    uint chunks_length = render_distance * 2 + 1;
    ivec3 storage_pos = rem_euclid_ivec3(pos, int(chunks_length) * CHUNK_SIZE);
    ivec3 in_chunk_pos = rem_euclid_ivec3(pos, CHUNK_SIZE);

    uvec3 chunk_address = uvec3(floor(storage_pos / CHUNK_SIZE));
    uint chunk_index = uint(dot(chunk_address, ivec3(1, chunks_length, chunks_length * chunks_length)));

    uint value = imageLoad(block_data[chunk_index], in_chunk_pos).x;
    return value;
}
uint read_distance(ivec3 pos) {
    uint chunks_length = render_distance * 2 + 1;
    ivec3 storage_pos = rem_euclid_ivec3(pos, int(chunks_length) * CHUNK_SIZE);
    ivec3 in_chunk_pos = rem_euclid_ivec3(pos, CHUNK_SIZE);

    uvec3 chunk_address = uvec3(floor(storage_pos / CHUNK_SIZE));
    uint chunk_index = uint(dot(chunk_address, ivec3(1, chunks_length, chunks_length * chunks_length)));

    uint d = imageLoad(distance_data[chunk_index], in_chunk_pos).x;
    return d;
}


bool is_inside_loaded_area(ivec3 pos) {
    ivec3 player_chunk_middle = ivec3(floor(player_position / float(CHUNK_SIZE))) * CHUNK_SIZE + CHUNK_SIZE / 2;
    ivec3 lower_bound = player_chunk_middle - CHUNK_SIZE * render_distance - CHUNK_SIZE / 2;
    ivec3 upper_bound = player_chunk_middle + CHUNK_SIZE * render_distance + CHUNK_SIZE / 2 - 1;
    return all(lessThan(lower_bound, pos)) && all(lessThan(pos, upper_bound));
}

bool single_ray_df(in vec3 ro, in vec3 rd, out uint block_id, out vec3 surface_normal, out vec3 hit_point) {
    const vec3 inv_rd = 1. / rd;

    ivec3 oct_rd01 = ivec3(greaterThan(rd, vec3(0.)));
    ivec3 oct_rd11 = (oct_rd01 * 2) - ivec3(1);

    vec3 t_dist_to_next = (vec3(oct_rd01) - fract(ro)) * inv_rd;

    ivec3 offset = ivec3(0);
    ivec3 pos = ivec3(floor(ro));
    uint free_dist = 0;
    ivec3 last_read_pos = pos;

    while(is_inside_loaded_area(pos)) {
        int next_xyz = argmin(t_dist_to_next);
        float old_distance = t_dist_to_next[next_xyz];
        t_dist_to_next[next_xyz] += abs(inv_rd[next_xyz]);
        offset[next_xyz] += 1;


        pos = ivec3(floor(ro)) + offset * oct_rd11;

        if (chebyshev_length(last_read_pos - pos) >= free_dist){
            last_read_pos = pos;
            free_dist = read_distance(pos);
            uint block_type = read_block(pos);
            if(block_type > 0) {
                block_id = block_type;
                vec3 normal = vec3(0);
                normal[next_xyz] = -oct_rd11[next_xyz];
                surface_normal = normal;
                hit_point = ro + rd * old_distance;
                return true;
            }
        }
    }

    return false;
}

bool single_ray_basic(in vec3 ro, in vec3 rd, out uint block_id, out vec3 surface_normal, out vec3 hit_point) {
    const vec3 inv_rd = 1. / rd;

    ivec3 oct_rd01 = ivec3(greaterThan(rd, vec3(0.)));
    ivec3 oct_rd11 = (oct_rd01 * 2) - ivec3(1);

    vec3 t_dist_to_next = (vec3(oct_rd01) - fract(ro)) * inv_rd;

    ivec3 offset = ivec3(0);
    ivec3 pos = ivec3(floor(ro));
    while(is_inside_loaded_area(pos)) {
        int next_xyz = argmin(t_dist_to_next);
        float old_distance = t_dist_to_next[next_xyz];
        t_dist_to_next[next_xyz] += abs(inv_rd[next_xyz]);
        offset[next_xyz] += 1;


        pos = ivec3(floor(ro)) + offset * oct_rd11;

        uint block_type = read_block(pos);
        if(block_type > 0) {
            block_id = block_type;
            vec3 normal = vec3(0);
            normal[next_xyz] = -oct_rd11[next_xyz];
            surface_normal = normal;
            hit_point = ro + rd * old_distance;
            return true;
        }
    }

    return false;
}

vec4 get_texture(uint block_id, vec3 hit_normal, vec3 position) {
    vec2 uv;
    uint side = 1;
    if(hit_normal.y == 0.) {
        uv = abs(hit_normal.x) == 1. ? position.zy : position.xy;
    }
    else {
        uv = position.xz;
        side = hit_normal.y > 0. ? 0 : 2;
    }

    return texture(sampler2DArray(textures, texture_sampler), vec3(fract(-uv), 3 * (block_id - 1) + side));
}

vec3 raycast() {
    const vec2 render_img_size = imageSize(render_target).xy;
    if (gl_GlobalInvocationID.x == render_img_size.x / 2 && gl_GlobalInvocationID.y == render_img_size.y / 2) {
        looking_at.block_id = 0;
    }
    const vec2 norm_coordinates = vec2(((gl_GlobalInvocationID.xy) / render_img_size.x) - vec2(0.5, render_img_size.y / render_img_size.x * 0.5));
    vec3 rd = normalize((vec4(norm_coordinates * tan(settings.fov / 2.), 1., 1.) * push.cam_transform).xyz);
    vec3 ro = player_position;

    uint block_id;
    vec3 surface_normal;
    vec3 hit_point;

    float rest_multiplier = 1.;
    vec3 color = vec3(0);
    for(int i = 0; i < 5 && single_ray_df(ro, rd, block_id, surface_normal, hit_point); i++) {
        if(i == 0) {
            if(gl_GlobalInvocationID.x == render_img_size.x / 2 && gl_GlobalInvocationID.y == render_img_size.y / 2) {
                looking_at.hit_point = hit_point;
                looking_at.hit_normal = surface_normal;
                looking_at.block_id = block_id;
            }
        }


        ro = hit_point - rd * 0.0001;
        rd = rd - 2 * dot(rd, surface_normal) * surface_normal;

        vec4 tex = get_texture(block_id, surface_normal, hit_point);

        color += rest_multiplier * tex.rgb;
        rest_multiplier *= 1. - tex.a;
    }
    return color;
}

/*
vec3 raycast() {
    const vec2 render_img_size = imageSize(render_target).xy;
    if(gl_GlobalInvocationID.x == render_img_size.x / 2 && gl_GlobalInvocationID.y == render_img_size.y / 2) {
        looking_at.block_id = 0;
    }
    const vec2 norm_coordinates = vec2(((gl_GlobalInvocationID.xy) / render_img_size.x) - vec2(0.5, render_img_size.y / render_img_size.x * 0.5));
    const vec3 rd = normalize((vec4(norm_coordinates * tan(settings.fov / 2.), 1., 1.) * push.cam_transform).xyz);
    const vec3 ro = player_position;

    uint block_id;
    vec3 surface_normal;
    vec3 hit_point;

    if(single_ray_df(ro, rd, block_id, surface_normal, hit_point)) {


        vec2 uv = vec2(0);
        uint side = 1;
        if(surface_normal.y == -1.) {
            uv = hit_point.xz;
            side = 2;
        }
        else if(surface_normal.y == 1.) {
            uv = hit_point.xz;
            side = 0;
        }
        else if (abs(surface_normal.x) == 1.) {
            uv = -hit_point.zy;
        }
        else if (abs(surface_normal.z) == 1.) {
            uv = -hit_point.xy;
        }
        vec4 color = texture(sampler2DArray(textures, texture_sampler), vec3(fract(uv), 3 * (block_id - 1) + side));
        vec3 _normal;
        // lighting
        if(single_ray_df(hit_point - rd * 0.0001, sun_ray, block_id, _normal, hit_point)) {
            return color.xyz * 0.6;
        }
        return color.xyz;
    }

    return vec3(0.5, 0.7, 0.9);
}

*/