const int CHUNK_SIZE = 64;
const int CHUNK_VOLUME = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

int rem_euclid_int(int dividend, int divisor) {
    return int(mod(dividend, divisor));
}

ivec3 rem_euclid_ivec3(ivec3 dividend, int divisor){
    return ivec3(
        rem_euclid_int(dividend.x, divisor),
        rem_euclid_int(dividend.y, divisor),
        rem_euclid_int(dividend.z, divisor)
    );
}

vec2 chunk_AABB_test(ivec3 chunk_pos, vec3 ro, vec3 rd) {
    vec3 bmin = vec3(chunk_pos * CHUNK_SIZE);
    vec3 bmax = vec3((chunk_pos + ivec3(1)) * CHUNK_SIZE);

    vec3 inv_rd = 1. / rd;

    vec3 t1 = (bmin - ro) * inv_rd;
    vec3 t2 = (bmax - ro) * inv_rd;

    vec3 tmin = min(t1, t2);
    vec3 tmax = max(t1, t2);

    float t_enter = max(max(tmin.x, tmin.y), tmin.z);
    float t_exit = min(min(tmax.x, tmax.y), tmax.z);

    return vec2(t_enter, t_exit);
}

int argmin(vec3 args) {
    int min_index = 0;
    float min_value = args.x;

    if (args.y < min_value) {
        min_value = args.y;
        min_index = 1;
    }

    if (args.z < min_value) {
        min_index = 2;
    }

    return min_index;
}


uint compute_1D_index(uvec3 invocation_id) {
    return uint(dot(invocation_id, ivec3(1, CHUNK_SIZE, CHUNK_SIZE * CHUNK_SIZE)));
}


uint chebyshev_length(ivec3 pos) {
    return
    max(
        max(
            abs(pos.x),
            abs(pos.y)
        ),
        abs(pos.z)
    );
}

bool is_in_chunk_bounds(ivec3 pos) {
    return all(lessThanEqual(ivec3(0), pos)) && all(lessThan(pos, ivec3(CHUNK_SIZE)));
}