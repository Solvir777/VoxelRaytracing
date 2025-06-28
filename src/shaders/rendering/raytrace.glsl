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



layout(push_constant) uniform PushConstants {
    mat4 cam_transform;
} push;


#include "../util.glsl"

const vec3[] debug_colors = {vec3(1., 0., 0.), vec3(0., 1., 0.), vec3(0., 0., 1.)};

vec3 player_position = push.cam_transform[3].xyz;

uint read_value(ivec3 pos, uint render_dist) {
    uint chunks_length = render_dist * 2 + 1;
    ivec3 storage_pos = rem_euclid_ivec3(pos, int(chunks_length) * CHUNK_SIZE);
    ivec3 in_chunk_pos = rem_euclid_ivec3(pos, CHUNK_SIZE);

    uvec3 chunk_address = uvec3(floor(storage_pos / CHUNK_SIZE));
    uint chunk_index = uint(dot(chunk_address, ivec3(1, chunks_length, chunks_length * chunks_length)));

    uint value = imageLoad(block_data[chunk_index], in_chunk_pos).x;
    return value;
}
uint read_distance(ivec3 pos, uint render_dist) {
    uint chunks_length = render_dist * 2 + 1;
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


bool single_ray(in vec3 ro, in vec3 rd, out uint block_id, out vec3 surface_normal, out vec3 hit_point) {
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

        uint block_type = read_value(pos, render_distance);
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

    if(gl_GlobalInvocationID.x == render_img_size.x / 2 && gl_GlobalInvocationID.y == render_img_size.y / 2) {
        uint d = read_distance(ivec3(floor(ro)), render_distance);
        debugPrintfEXT("distance to nearest block: %i", d);
    }

    if(single_ray(ro, rd, block_id, surface_normal, hit_point)) {
        if(gl_GlobalInvocationID.x == render_img_size.x / 2 && gl_GlobalInvocationID.y == render_img_size.y / 2) {
            looking_at.hit_point = hit_point;
            looking_at.hit_normal = surface_normal;
            looking_at.block_id = block_id;
        }
        return surface_normal + 0.5 * vec3(sin(floor(hit_point.x + 0.001) * 2.), sin(floor(hit_point.y + 0.001) * 2.), sin((floor(hit_point.z + 0.001)) * 2.));
    }

    return vec3(.51, 0.7, 0.9);
}

