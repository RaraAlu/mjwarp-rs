// 渲染内核的 mesh 三角求交数学（渲染/BVH S7b1）。
//
// 逐式复制 S3 `src/render/bvh/cuda/mesh.cu`（该文件只读、未修改）的 woop 三角求交、
// 保守 AABB 剪枝与 `wp.mesh_query_ray` 遍历；与 S3 的唯一差异是 `normalize3` 改名为
// `normalize3_mesh`（S2b `scene.cu` 已在本编译单元前置、定义了不同语义的同名函数），
// 表达式、运算顺序、比较方向与 S3 逐字一致。
//
// 本文件被 `src/render/kernel/cast.rs::DEPTH_SEG_SOURCE` 用 `include_str!` 接在
// S2b `scene.cu` 之后、`cuda/render.cu` 之前，因此可直接复用 `scene.cu` 的 `V3`
// 辅助函数（`v3`/`add`/`sub`/`scale`/`dot`/`cross`/`mat_vec`/`mat_t_vec`/`ray_map`）。
//
// 文件末尾另有球面（geom_type 2）求交的实现：它不属于 S3 复制面，而是用显式
// `__fmul_rn`/`__fmaf_rn`/`__fsub_rn` 固定冻结上游 megakernel 的运算顺序（见该节注释）。

__device__ __forceinline__ float comp(V3 a, int i) { return (i == 0) ? a.x : ((i == 1) ? a.y : a.z); }

// Matches warp vec.h normalize(vec3): divide by sqrt of the plain component sum,
// zero vector when the length is not strictly positive (kEps == 0.0f).
__device__ __forceinline__ V3 normalize3_mesh(V3 a) {
    float l = sqrtf(a.x * a.x + a.y * a.y + a.z * a.z);
    if (l > 0.0f) {
        return v3(a.x / l, a.y / l, a.z / l);
    }
    return v3(0.0f, 0.0f, 0.0f);
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
    // 上游 mesh 法线链（G-L2 判据 A RCA H2）：cross(ab,ac) 未归一化 → 消费端 mat@n → normalize 一次。
    // 本函数返回未归一化的三角面法线；唯一一次归一化由 render_common.cu / render.cu 的消费端完成。
    *out_normal = min_normal;
    *out_face = min_face;
    return true;
}

// --- 球面求交：对齐冻结上游 megakernel 的编译产物（S7b1 depth 口径） --------------------
//
// 上游 `cast_ray` 内联 `ray.py::ray_sphere` 后，warp 1.15（其自带 NVRTC、sm_89）生成的
// f32 指令顺序是：
//   a  = dot(vec, vec)   : mul(d1*d1) → fma(d0*d0) → fma(d2*d2)        （1 号分量先乘）
//   b  = dot(vec, dif)   : mul(d1*dif1) → fma(d0*dif0) → fma(d2*dif2)  （1 号分量先乘）
//   dd = dot(dif, dif)   : mul(dif1*dif1) → fma(dif0*dif0) → fma(dif2*dif2)
//   det = b*b - a*c      : fma(b, b, -(a*c))                          （融合）
// **`a` 与 `b`/`dd` 同序（1 号分量先乘）**——U4 sphere 轮廓 depth RCA（2026-10-31，
// `target/sphere-depth-rca.md`）在 44 个诊断像素上枚举 3 个点积次序 × det 融合共 432 个变体，
// 只有 `a=(1,0,2) b=(1,0,2) det=fused` 能位级复现参考 44/44，而本文件当时的 `a=(0,1,2)`
// （`dot3_ch0`）复现参考 0/44。此前「`a` 用 0 号分量先乘」的结论来自 S7b1 的 747 像素 probe，
// 那是**样本无反例**的误判：`a` 只依赖射线方向，两种次序在多数方向上舍入相同，S7b1/scene-a 的
// 像素恰好没有同时满足「a 差 1 ULP」与「det ≈ 0」，而 scene-sdf cam-front 的球轮廓满足，
// 于是 1 ULP 的 a 差经 sqrt(det) 被放大成 7–191 ULP 的 depth 差。
// 产品渲染配方是冻结的 CUDA 12.8.1 NVRTC / compute_70：显式 intrinsics 固定上述次序，与
// 本地 NVRTC 收缩选择无关；其余 primitive 仍复用 S2b `scene.cu`（该文件只读、未修改）。
// 这是对齐上游编译产物的运算顺序，不是放宽任何容差。

__device__ __forceinline__ float dot3_ba(float a0, float a1, float a2, float b0, float b1, float b2) {
    return __fmaf_rn(a2, b2, __fmaf_rn(a0, b0, __fmul_rn(a1, b1)));
}

// `ray_sphere(pos, size[0]*size[0], pnt, vec)` 的等价实现；`normal` 仍用 `scene.cu` 的
// `normalize3`（与现行产品路径一致，背面剔除判定不变）。
__device__ __forceinline__ void render_sphere_hit(V3 pos, float radius, V3 pnt, V3 vec, float* dist,
                                                  V3* normal) {
    float r2 = __fmul_rn(radius, radius);
    float dx = __fsub_rn(pnt.x, pos.x);
    float dy = __fsub_rn(pnt.y, pos.y);
    float dz = __fsub_rn(pnt.z, pos.z);
    float a = dot3_ba(vec.x, vec.y, vec.z, vec.x, vec.y, vec.z);
    float b = dot3_ba(vec.x, vec.y, vec.z, dx, dy, dz);
    float dd = dot3_ba(dx, dy, dz, dx, dy, dz);
    float c = __fsub_rn(dd, r2);
    float det = __fmaf_rn(b, b, -__fmul_rn(a, c));
    *dist = -1.0f;
    *normal = v3(0.0f, 0.0f, 0.0f);
    if (det < 1.0e-15f) {
        return;
    }
    float sq = sqrtf(det);
    float den = (a != 0.0f) ? __frcp_rn(a) : 1.0e-15f;
    float nb = __fsub_rn(0.0f, b);
    float x0 = __fmul_rn(den, __fsub_rn(nb, sq));
    float x1 = __fmul_rn(den, __fsub_rn(sq, b));
    float sol = (x0 >= 0.0f) ? x0 : ((x1 >= 0.0f) ? x1 : -1.0f);
    if (sol < 0.0f) {
        return;
    }
    *dist = sol;
    V3 hit = add(pnt, scale(vec, sol));
    *normal = normalize3(sub(hit, pos));
}

// `cast_ray` 的 primitive 分支：球面走上面对齐上游的实现，其余保持 S2b `scene.cu` 语义。
__device__ __forceinline__ void render_intersect_primitive(int geom_type, V3 pos, const float* size,
                                                           const float* mat, V3 pnt, V3 vec,
                                                           float* dist, V3* normal) {
    if (geom_type == 2) {
        render_sphere_hit(pos, size[0], pnt, vec, dist, normal);
        return;
    }
    intersect_primitive(geom_type, pos, size, mat, pnt, vec, dist, normal);
}
