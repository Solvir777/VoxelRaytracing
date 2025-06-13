layout(local_size_x = 16, local_size_y = 16, local_size_z = 1) in;
layout(r16ui, set = 0, binding = 1) readonly uniform uimage3D block_data;
//layout(r16ui, set = 0, binding = 2) readonly uniform uimage3D distance_data;

layout(push_constant) uniform PushConstants {
    mat4 cam_transform;
} push;


#include "../util.glsl"

const vec3[] debug_colors = {vec3(1., 0., 0.), vec3(0., 1., 0.), vec3(0., 0., 1.)};

vec3 player_position = push.cam_transform[3].xyz;
int voxel_data_size = imageSize(block_data).x;

bool is_inside_loaded_area(ivec3 pos) {
    ivec3 player_chunk_middle = ivec3(floor(player_position / float(CHUNK_SIZE))) * CHUNK_SIZE + CHUNK_SIZE / 2;
    ivec3 lower_bound = player_chunk_middle - imageSize(block_data) / 2;
    ivec3 upper_bound = player_chunk_middle + imageSize(block_data) / 2 - ivec3(1);
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

        uint block_type = imageLoad(block_data, rem_euclid_ivec3(pos, voxel_data_size)).r;
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
    const int voxel_data_size = imageSize(block_data).x;
    const vec2 render_img_size = imageSize(render_target).xy;
    const vec2 norm_coordinates = vec2(((gl_GlobalInvocationID.xy) / render_img_size.x) - vec2(0.5, render_img_size.y / render_img_size.x * 0.5));
    const vec3 rd = normalize((vec4(norm_coordinates, 1., 1.) * push.cam_transform).xyz);
    const vec3 ro = player_position;

    uint block_id;
    vec3 surface_normal;
    vec3 hit_point;

    if(single_ray(ro, rd, block_id, surface_normal, hit_point)) {
        return surface_normal + 0.5 * vec3(sin(floor(hit_point.x + 0.001) * 2.), sin(floor(hit_point.y + 0.001) * 2.), sin((floor(hit_point.z + 0.001)) * 2.));
    }


    return surface_normal;
}

