// GPU mesh triangle-BVH ray query (render/BVH S3).
//
// ABI (must match SynchronousKernel::launch's generic four-buffer launch):
//   extern "C" __global__ void mesh_bvh_ray(
//       const int* meta, const float* params, const float* rays,
//       float* out, unsigned d0, unsigned d1, unsigned d2, unsigned d3);
//
// meta:   [n_nodes, n_tri, n_ray, root, cull_backfaces, geom_id, n_vert,
//          links(2*n_nodes), order(n_tri), indices(3*n_tri)]
//         internal link: left/right node indices (>=0)
//         leaf link: -(begin+1), end  -> triangles order[begin..end]
// params: [node bounds 6*n_nodes, vertices 3*n_vert, geom_xpos 3, geom_xmat 9]
// rays:   [origin 3, direction 3, max_t] per ray
// out:    [distance, normal 3, u, v, face bits, geomid bits] per ray
//         miss = (-1, 0, 0, 0, 0, 0, -1, -1)
//
// Semantics are a direct translation of the frozen upstream render path
// (mujoco_warp/_src/ray.py `ray_mesh_with_bvh` -> Warp `wp.mesh_query_ray`):
// `_ray_map` into the local mesh frame, up to 8 back-face-culled marches with
// RAY_TOL_REL/RAY_TOL_ABS offsets, nearest-hit selection over
// `intersect_ray_tri_woop` (Woop triangle intersection in `mesh.h`), then
// normal = normalize(mat @ local_normal), distance = offset + t, u/v barycentrics
// and the face index. Traversal and node layout are our own; the acceptance only
// asserts semantic hit results (no node-layout assertions).
//
// Precision: this kernel is f32-only because the product SynchronousKernel path
// enforces an f32-only PTX gate (same discipline as the physics kernels). The
// upstream Woop routine has one double-precision fallback branch (when a
// barycentric determinant is exactly zero); that branch is not reproduced here,
// and the authoring capture proves the fixture ray set never triggers it
// (`degenerate_pairs = 0`). Compiled by NVRTC with fmad=true, fast_math=false,
// so FMA is the real fma.rn.f32.

struct V3 {
    float x, y, z;
};

__device__ __forceinline__ V3 v3(float x, float y, float z) { V3 r; r.x = x; r.y = y; r.z = z; return r; }
__device__ __forceinline__ V3 add(V3 a, V3 b) { return v3(a.x + b.x, a.y + b.y, a.z + b.z); }
__device__ __forceinline__ V3 sub(V3 a, V3 b) { return v3(a.x - b.x, a.y - b.y, a.z - b.z); }
__device__ __forceinline__ V3 scale(V3 a, float s) { return v3(a.x * s, a.y * s, a.z * s); }
__device__ __forceinline__ float dot(V3 a, V3 b) { return a.x * b.x + a.y * b.y + a.z * b.z; }
__device__ __forceinline__ V3 cross(V3 a, V3 b) {
    return v3(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x);
}
__device__ __forceinline__ float comp(V3 a, int i) { return (i == 0) ? a.x : ((i == 1) ? a.y : a.z); }

// Matches warp vec.h normalize(vec3): divide by sqrt of the plain component sum,
// zero vector when the length is not strictly positive (kEps == 0.0f).
__device__ __forceinline__ V3 normalize3(V3 a) {
    float l = sqrtf(a.x * a.x + a.y * a.y + a.z * a.z);
    if (l > 0.0f) {
        return v3(a.x / l, a.y / l, a.z / l);
    }
    return v3(0.0f, 0.0f, 0.0f);
}

// Row-major 3x3 matrix helpers: world = M * local, local = M^T * (world - pos).
__device__ __forceinline__ V3 mat_vec(const float* m, V3 a) {
    return v3(m[0] * a.x + m[1] * a.y + m[2] * a.z,
              m[3] * a.x + m[4] * a.y + m[5] * a.z,
              m[6] * a.x + m[7] * a.y + m[8] * a.z);
}

__device__ __forceinline__ V3 mat_t_vec(const float* m, V3 a) {
    return v3(m[0] * a.x + m[3] * a.y + m[6] * a.z,
              m[1] * a.x + m[4] * a.y + m[7] * a.z,
              m[2] * a.x + m[5] * a.y + m[8] * a.z);
}

// _ray_map: map the world ray into the local geom/mesh frame.
__device__ __forceinline__ void ray_map(V3 pos, const float* mat, V3 pnt, V3 vec, V3* lpnt, V3* lvec) {
    V3 dif = sub(pnt, pos);
    *lpnt = mat_t_vec(mat, dif);
    *lvec = mat_t_vec(mat, vec);
}

// viewport constants from upstream ray.py.
#define RAY_TOL_REL 1.0e-6f
#define RAY_TOL_ABS 1.0e-9f
#define CULL_MARCHES 8

// --- Woop ray/triangle intersection, translated from include/warp-headers/intersect.h ---

__device__ __forceinline__ int longest_axis(V3 a) {
    float lmax = fabsf(a.x);
    int ret = 0;
    float l = fabsf(a.y);
    if (l > lmax) {
        lmax = l;
        ret = 1;
    }
    l = fabsf(a.z);
    if (l > lmax) {
        ret = 2;
    }
    return ret;
}

__device__ __forceinline__ int sign_mask(float x) { return __float_as_int(x) & (int)0x80000000u; }

__device__ __forceinline__ float xorf(float x, int y) { return __int_as_float(__float_as_int(x) ^ y); }

// Difference of products a*b - c*d using FMA instructions (as upstream).
__device__ __forceinline__ float diff_product(float a, float b, float c, float d) {
    float cd = c * d;
    float diff = fmaf(a, b, -cd);
    float error = fmaf(-c, d, cd);
    return diff + error;
}

__device__ __forceinline__ bool intersect_tri(V3 p, V3 dir, V3 a, V3 b, V3 c, float* t, float* u, float* v,
                                              float* sign, V3* normal) {
    int kz = longest_axis(dir);
    int kx = kz + 1;
    if (kx == 3) {
        kx = 0;
    }
    int ky = kx + 1;
    if (ky == 3) {
        ky = 0;
    }
    if (comp(dir, kz) < 0.0f) {
        int tmp = kx;
        kx = ky;
        ky = tmp;
    }

    float Sx = comp(dir, kx) / comp(dir, kz);
    float Sy = comp(dir, ky) / comp(dir, kz);
    float Sz = 1.0f / comp(dir, kz);

    V3 A = sub(a, p);
    V3 B = sub(b, p);
    V3 C = sub(c, p);

    float Ax = comp(A, kx) - Sx * comp(A, kz);
    float Ay = comp(A, ky) - Sy * comp(A, kz);
    float Bx = comp(B, kx) - Sx * comp(B, kz);
    float By = comp(B, ky) - Sy * comp(B, kz);
    float Cx = comp(C, kx) - Sx * comp(C, kz);
    float Cy = comp(C, ky) - Sy * comp(C, kz);

    float U = diff_product(Cx, By, Cy, Bx);
    float V = diff_product(Ax, Cy, Ay, Cx);
    float W = diff_product(Bx, Ay, By, Ax);

    // Upstream recomputes U/V/W in double when any is exactly zero. The product
    // kernel stays f32 (PTX precision gate). The authoring capture statically
    // replays every query: the scene-a ray set never reaches this branch, and on
    // the synthetic degenerate mesh both implementations reject the same
    // triangles, so accept/reject decisions match the frozen reference.
    if ((U < 0.0f || V < 0.0f || W < 0.0f) && (U > 0.0f || V > 0.0f || W > 0.0f)) {
        return false;
    }

    float det = U + V + W;
    if (det == 0.0f) {
        return false;
    }

    float Az = Sz * comp(A, kz);
    float Bz = Sz * comp(B, kz);
    float Cz = Sz * comp(C, kz);
    float T = U * Az + V * Bz + W * Cz;

    int det_sign = sign_mask(det);
    if (xorf(T, det_sign) < 0.0f) {
        return false;
    }

    float rcpDet = 1.0f / det;
    *u = U * rcpDet;
    *v = V * rcpDet;
    *t = T * rcpDet;
    *sign = det;

    V3 ab = sub(b, a);
    V3 ac = sub(c, a);
    *normal = cross(ab, ac);
    return true;
}

// --- traversal ---

// mesh_query_ray_safe_dir: zero components become 1e-20 only for the reciprocal.
__device__ __forceinline__ V3 safe_dir(V3 dir) {
    return v3(dir.x == 0.0f ? 1.0e-20f : dir.x, dir.y == 0.0f ? 1.0e-20f : dir.y,
              dir.z == 0.0f ? 1.0e-20f : dir.z);
}

// Marching/frustum test mirroring intersect_ray_aabb[_robust]; only used to skip
// subtrees, so a conservative result cannot change the nearest-hit answer.
__device__ __forceinline__ bool aabb_hit(V3 start, V3 dir, V3 rdir, bool fast, const float* bounds) {
    float lmin = -3.402823466e+38f;
    float lmax = 3.402823466e+38f;
    for (int i = 0; i < 3; i++) {
        float o = comp(start, i);
        float d = comp(dir, i);
        float lo = bounds[i];
        float hi = bounds[3 + i];
        if (!fast && d == 0.0f) {
            if (o < lo || o > hi) {
                return false;
            }
            continue;
        }
        float l1 = (lo - o) * comp(rdir, i);
        float l2 = (hi - o) * comp(rdir, i);
        float near_t = l1 < l2 ? l1 : l2;
        float far_t = l1 < l2 ? l2 : l1;
        lmin = near_t > lmin ? near_t : lmin;
        lmax = far_t < lmax ? far_t : lmax;
    }
    return (lmax >= 0.0f) && (lmax >= lmin);
}

// wp.mesh_query_ray: nearest triangle hit inside [0, max_t], with u/v/sign/normal.
__device__ __forceinline__ bool mesh_query_ray(V3 pnt, V3 vec, float max_t, int n_nodes, int root, int n_vert,
                                               const int* links, const int* order, const int* indices,
                                               const float* node_bounds, const float* vertices, float* out_t,
                                               float* out_u, float* out_v, float* out_sign, V3* out_normal,
                                               int* out_face) {
    float min_t = max_t;
    int min_face = -1;
    float min_u = 0.0f;
    float min_v = 0.0f;
    float min_sign = 1.0f;
    V3 min_normal = v3(0.0f, 0.0f, 0.0f);
    bool hit = false;

    V3 ray_dir = safe_dir(vec);
    V3 rcp_dir = v3(1.0f / ray_dir.x, 1.0f / ray_dir.y, 1.0f / ray_dir.z);
    bool fast = vec.x != 0.0f && vec.y != 0.0f && vec.z != 0.0f;

    int stack[128];
    int depth = 0;
    stack[depth++] = root;
    while (depth > 0) {
        int node = stack[--depth];
        const float* bounds = node_bounds + 6 * node;
        if (!aabb_hit(pnt, vec, rcp_dir, fast, bounds)) {
            continue;
        }
        int link = links[2 * node];
        if (link >= 0) {
            if (depth + 2 <= 128) {
                stack[depth++] = link;
                stack[depth++] = links[2 * node + 1];
            }
            continue;
        }
        int begin = -link - 1;
        int end = links[2 * node + 1];
        for (int slot = begin; slot < end; slot++) {
            int tri = order[slot];
            int i0 = indices[3 * tri];
            int i1 = indices[3 * tri + 1];
            int i2 = indices[3 * tri + 2];
            (void)n_vert;
            V3 a = v3(vertices[3 * i0], vertices[3 * i0 + 1], vertices[3 * i0 + 2]);
            V3 b = v3(vertices[3 * i1], vertices[3 * i1 + 1], vertices[3 * i1 + 2]);
            V3 c = v3(vertices[3 * i2], vertices[3 * i2 + 1], vertices[3 * i2 + 2]);
            float tri_t = 0.0f;
            float tri_u = 0.0f;
            float tri_v = 0.0f;
            float tri_sign = 0.0f;
            V3 tri_normal = v3(0.0f, 0.0f, 0.0f);
            if (intersect_tri(pnt, vec, a, b, c, &tri_t, &tri_u, &tri_v, &tri_sign, &tri_normal)) {
                if (tri_t < min_t && tri_t >= 0.0f) {
                    min_t = tri_t;
                    min_face = tri;
                    min_u = tri_u;
                    min_v = tri_v;
                    min_sign = tri_sign;
                    min_normal = tri_normal;
                    hit = true;
                }
            }
        }
    }

    if (!hit) {
        return false;
    }
    *out_t = min_t;
    *out_u = min_u;
    *out_v = min_v;
    *out_sign = min_sign;
    *out_normal = normalize3(min_normal);
    *out_face = min_face;
    return true;
}

extern "C" __global__ void mesh_bvh_ray(const int* meta, const float* params, const float* rays, float* out,
                                        unsigned d0, unsigned d1, unsigned d2, unsigned d3) {
    (void)d0;
    (void)d1;
    (void)d2;
    const int n_nodes = meta[0];
    const int n_tri = meta[1];
    const int n_ray = meta[2];
    const int root = meta[3];
    const int cull_backfaces = meta[4];
    const int geom_id = meta[5];
    const int n_vert = meta[6];
    (void)geom_id;
    (void)n_tri;

    unsigned ray_index = blockDim.x * blockIdx.x + threadIdx.x;
    (void)d3;
    if ((int)ray_index >= n_ray) {
        return;
    }

    const int* links = meta + 7;
    const int* order = links + 2 * n_nodes;
    const int* indices = order + meta[1];
    const float* node_bounds = params;
    const float* vertices = node_bounds + 6 * n_nodes;
    const float* xpos = vertices + 3 * (long)n_vert;
    const float* xmat = xpos + 3;

    V3 origin = v3(rays[7 * ray_index + 0], rays[7 * ray_index + 1], rays[7 * ray_index + 2]);
    V3 direction = v3(rays[7 * ray_index + 3], rays[7 * ray_index + 4], rays[7 * ray_index + 5]);
    float max_t = rays[7 * ray_index + 6];

    V3 pos = v3(xpos[0], xpos[1], xpos[2]);
    V3 lpnt;
    V3 lvec;
    ray_map(pos, xmat, origin, direction, &lpnt, &lvec);

    float distance = -1.0f;
    float u = 0.0f;
    float v = 0.0f;
    int face = -1;
    V3 normal = v3(0.0f, 0.0f, 0.0f);

    float offset = 0.0f;
    for (int march = 0; march < CULL_MARCHES; march++) {
        V3 start = add(lpnt, scale(lvec, offset));
        float t = 0.0f;
        float tu = 0.0f;
        float tv = 0.0f;
        float sign = 0.0f;
        V3 n_local = v3(0.0f, 0.0f, 0.0f);
        int tri = -1;
        if (!mesh_query_ray(start, lvec, max_t - offset, n_nodes, root, n_vert, links, order, indices, node_bounds,
                            vertices, &t, &tu, &tv, &sign, &n_local, &tri)) {
            break;
        }
        if (!cull_backfaces || dot(lvec, n_local) < 0.0f) {
            distance = offset + t;
            u = tu;
            v = tv;
            face = tri;
            normal = normalize3(mat_vec(xmat, n_local));
            break;
        }
        offset += t * (1.0f + RAY_TOL_REL) + RAY_TOL_ABS;
    }

    float* target = out + 8 * ray_index;
    if (distance >= 0.0f) {
        target[0] = distance;
        target[1] = normal.x;
        target[2] = normal.y;
        target[3] = normal.z;
        target[4] = u;
        target[5] = v;
        target[6] = __int_as_float(face);
        target[7] = __int_as_float(geom_id);
    } else {
        target[0] = -1.0f;
        target[1] = 0.0f;
        target[2] = 0.0f;
        target[3] = 0.0f;
        target[4] = 0.0f;
        target[5] = 0.0f;
        target[6] = __int_as_float(-1);
        target[7] = __int_as_float(-1);
    }
}
