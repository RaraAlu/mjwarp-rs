struct V3 { float x; float y; float z; };
__device__ __forceinline__ V3 v3(float x, float y, float z) { V3 r; r.x = x; r.y = y; r.z = z; return r; }
#define FLEX_MINVAL 1.0e-15f

__device__ __forceinline__ float warp_length3(V3 v) {
    return sqrtf(fmaf(v.z, v.z, fmaf(v.y, v.y, v.x * v.x)));
}

__device__ __forceinline__ void flex_quat_z2vec(V3 vec, float* quat) {
    quat[0] = 0.0f;
    quat[1] = 0.0f;
    quat[2] = 0.0f;
    quat[3] = 1.0f;
    float norm = warp_length3(vec);
    if (norm < FLEX_MINVAL) {
        return;
    }
    V3 unit = v3(vec.x / norm, vec.y / norm, vec.z / norm);
    V3 axis = v3(-unit.y, unit.x, 0.0f);
    float a = warp_length3(axis);
    if (a < FLEX_MINVAL) {
        // 近平行：反向时绕 x 轴 180°；同向保持单位四元数。
        if (unit.z < 0.0f) {
            quat[0] = 1.0f;
        }
        return;
    }
    axis = v3(axis.x / a, axis.y / a, axis.z / a);
    float angle = atan2f(a, unit.z);
    float half = angle * 0.5f;
    float s = sinf(half);
    float c = cosf(half);
    // 上游 `axis_angle_to_quat`：`quat(c, axis*s)`。
    quat[0] = c;
    quat[1] = axis.x * s;
    quat[2] = axis.y * s;
    quat[3] = axis.z * s;
}


extern "C" __global__ void probe_z2vec(float* out) {
    float quat[4];
    flex_quat_z2vec(v3(0.0f, 0.0f, -1.0f), quat);
    out[0] = quat[0];
    out[1] = quat[1];
    out[2] = quat[2];
    out[3] = quat[3];
}
