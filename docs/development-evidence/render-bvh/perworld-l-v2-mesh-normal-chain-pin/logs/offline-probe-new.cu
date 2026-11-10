// offline probe only (no GPU, no product change): exact snippets extracted from the repo sources
struct V3 { float x; float y; float z; };
__device__ __forceinline__ V3 v3(float x, float y, float z) { V3 r; r.x = x; r.y = y; r.z = z; return r; }

__device__ __forceinline__ V3 normalize3_mesh(V3 a) {
    float l = sqrtf(a.x * a.x + a.y * a.y + a.z * a.z);
    if (l > 0.0f) {
        return v3(a.x / l, a.y / l, a.z / l);
    }
    return v3(0.0f, 0.0f, 0.0f);
}

__device__ __forceinline__ V3 mat_vec_xpref(const float* m, V3 a) {
    return v3(fmaf(m[2], a.z, fmaf(m[1], a.y, m[0] * a.x)),
              fmaf(m[5], a.z, fmaf(m[4], a.y, m[3] * a.x)),
              fmaf(m[8], a.z, fmaf(m[7], a.y, m[6] * a.x)));
}
extern "C" __global__ void probe_normal_chain(const float* __restrict__ mat, const float* __restrict__ tri, float* __restrict__ out) {
    V3 tn = v3(tri[0], tri[1], tri[2]);
    V3 n_local = normalize3_mesh(tn);
    V3 n = normalize3_mesh(mat_vec_xpref(mat, n_local));
    out[0] = n.x; out[1] = n.y; out[2] = n.z;
}
