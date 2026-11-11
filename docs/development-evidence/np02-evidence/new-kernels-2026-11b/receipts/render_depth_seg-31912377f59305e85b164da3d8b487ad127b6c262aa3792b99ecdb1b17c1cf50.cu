// GPU scene-BVH ray query (render/BVH S2b).
//
// ABI (must match SynchronousKernel::launch's generic four-buffer launch):
//   extern "C" __global__ void scene_bvh_ray(
//       const int* meta, const float* params, const float* rays,
//       float* out, unsigned d0, unsigned d1, unsigned d2, unsigned d3);
//
// meta:   [n_nodes, n_leaf, n_ray, root, links(2*n_nodes), leaf_geom(n_leaf),
//          leaf_type(n_leaf)]
//         internal link: left/right node indices (>=0), leaf link: -(leaf+1)
// params: [node bounds 6*n_nodes, leaf size 3*n_leaf, leaf xpos 3*n_leaf,
//          leaf xmat(row-major) 9*n_leaf]
// rays:   [origin 3, direction 3] per ray
// out:    [distance, geomid bits, normal 3] per ray; miss = (-1, -1, 0,0,0)
//
// Primitive intersections are translated from the frozen upstream
// mujoco_warp/_src/ray.py (71da24d956378a87a703b6e1442b13aec0c4ac29):
// ray_plane / ray_sphere / ray_capsule / ray_ellipsoid / ray_cylinder /
// ray_box, with _ray_map / _ray_quad / safe_div. Mesh (geom_type 7/8) hits are
// deliberately deferred to S3: the kernel reports a miss for them, and the S2b
// test excludes the one reference mesh-hit ray and declares it.
//
// Device side uses plain float math compiled by NVRTC with fmad enabled (the
// SynchronousKernel recipe: fmad=true, fast_math=false), so FMA is the real
// fma.rn.f32; there is no host f64 emulation on the device path.
//
// Default query semantics match the frozen capture call: bodyexclude = -1, all
// geoms visible (no rgba elimination), flg_static = true, geomgroup all -1.
// Those flags are not parameterized in this slice.

struct V3 {
    float x, y, z;
};

__device__ __forceinline__ V3 v3(float x, float y, float z) { V3 r; r.x = x; r.y = y; r.z = z; return r; }
__device__ __forceinline__ V3 add(V3 a, V3 b) { return v3(a.x + b.x, a.y + b.y, a.z + b.z); }
__device__ __forceinline__ V3 sub(V3 a, V3 b) { return v3(a.x - b.x, a.y - b.y, a.z - b.z); }
__device__ __forceinline__ V3 scale(V3 a, float s) { return v3(a.x * s, a.y * s, a.z * s); }
__device__ __forceinline__ float dot(V3 a, V3 b) { return a.x * b.x + a.y * b.y + a.z * b.z; }
__device__ __forceinline__ V3 cw_mul(V3 a, V3 b) { return v3(a.x * b.x, a.y * b.y, a.z * b.z); }
__device__ __forceinline__ V3 cross(V3 a, V3 b) {
    return v3(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x);
}
__device__ __forceinline__ float length3(V3 a) { return sqrtf(dot(a, a)); }
__device__ __forceinline__ V3 normalize3(V3 a) { return scale(a, 1.0f / length3(a)); }

// Matches warp's safe_div and MJ_MINVAL (MuJoCo mjMINVAL).
__device__ __forceinline__ float safe_div(float x, float y) { return x / (y != 0.0f ? y : 1.0e-15f); }

// Row-major 3x3 matrix helpers: world = M * local, local = M^T * (world - pos).
// 矩阵×向量（行主序）：显式固定冻结上游的 `fma` 收缩序——三项和里「中间项」单独舍入为
// `mul`，首项/末项分别融入 `fma`（`fmaf(m2,z, fmaf(m0,x, m1*y))`）。依据：冻结上游 flex
// 副本 PTX 与本卡 warp GPU 回放 oracle（251/251 像素逐位复现参考 depth）；不显式固定时
// NVRTC 会按内联上下文选成「舍入首项」，flex 路径因此与参考差 1 ULP（D8：cam2 depth 321 px）。
__device__ __forceinline__ V3 mat_vec(const float* m, V3 a) {
    return v3(fmaf(m[2], a.z, fmaf(m[0], a.x, m[1] * a.y)),
              fmaf(m[5], a.z, fmaf(m[3], a.x, m[4] * a.y)),
              fmaf(m[8], a.z, fmaf(m[6], a.x, m[7] * a.y)));
}

__device__ __forceinline__ V3 mat_t_vec(const float* m, V3 a) {
    return v3(fmaf(m[6], a.z, fmaf(m[0], a.x, m[3] * a.y)),
              fmaf(m[7], a.z, fmaf(m[1], a.x, m[4] * a.y)),
              fmaf(m[8], a.z, fmaf(m[2], a.x, m[5] * a.y)));
}

// _ray_map: map world ray into the local geom frame.
__device__ __forceinline__ void ray_map(V3 pos, const float* mat, V3 pnt, V3 vec, V3* lpnt, V3* lvec) {
    V3 dif = sub(pnt, pos);
    *lpnt = mat_t_vec(mat, dif);
    *lvec = mat_t_vec(mat, vec);
}

// _ray_quad: solve a*x^2 + 2*b*x + c = 0; sol is the first non-negative root.
__device__ __forceinline__ void ray_quad(float a, float b, float c, float* sol, float* x0, float* x1) {
    float det = b * b - a * c;
    if (det < 1.0e-15f) {
        *sol = -1.0f;
        *x0 = -1.0f;
        *x1 = -1.0f;
        return;
    }
    det = sqrtf(det);
    float den = safe_div(1.0f, a);
    float first = (-b - det) * den;
    float second = (-b + det) * den;
    *x0 = first;
    *x1 = second;
    *sol = (first >= 0.0f) ? first : ((second >= 0.0f) ? second : -1.0f);
}

__device__ __forceinline__ void ray_plane(V3 pos, const float* mat, const float* size, V3 pnt, V3 vec,
                                          float* dist, V3* normal) {
    V3 lpnt, lvec;
    ray_map(pos, mat, pnt, vec, &lpnt, &lvec);
    if (lvec.z > -1.0e-15f) {
        *dist = -1.0f;
        *normal = v3(0.0f, 0.0f, 0.0f);
        return;
    }
    float x = -lpnt.z / lvec.z;
    if (x < 0.0f) {
        *dist = -1.0f;
        *normal = v3(0.0f, 0.0f, 0.0f);
        return;
    }
    float px = lpnt.x + x * lvec.x;
    float py = lpnt.y + x * lvec.y;
    if ((size[0] <= 0.0f || fabsf(px) <= size[0]) && (size[1] <= 0.0f || fabsf(py) <= size[1])) {
        *dist = x;
        *normal = v3(mat[2], mat[5], mat[8]);
    } else {
        *dist = -1.0f;
        *normal = v3(0.0f, 0.0f, 0.0f);
    }
}

__device__ __forceinline__ void ray_sphere(V3 pos, float dist_sqr, V3 pnt, V3 vec, float* dist, V3* normal) {
    V3 dif = sub(pnt, pos);
    float a = dot(vec, vec);
    float b = dot(vec, dif);
    float c = dot(dif, dif) - dist_sqr;
    float sol, x0, x1;
    ray_quad(a, b, c, &sol, &x0, &x1);
    if (sol >= 0.0f) {
        *dist = sol;
        V3 s = add(pnt, scale(vec, sol));
        *normal = normalize3(sub(s, pos));
    } else {
        *dist = -1.0f;
        *normal = v3(0.0f, 0.0f, 0.0f);
    }
}

__device__ __forceinline__ void ray_capsule(V3 pos, const float* mat, const float* size, V3 pnt, V3 vec,
                                            float* dist, V3* normal) {
    float ssz = size[0] + size[1];
    float dist_sphere;
    V3 normal_sphere;
    ray_sphere(pos, ssz * ssz, pnt, vec, &dist_sphere, &normal_sphere);
    if (dist_sphere < 0.0f) {
        *dist = -1.0f;
        *normal = v3(0.0f, 0.0f, 0.0f);
        return;
    }
    V3 lpnt, lvec;
    ray_map(pos, mat, pnt, vec, &lpnt, &lvec);
    float x = -1.0f;
    float sq_size0 = size[0] * size[0];
    float a = lvec.x * lvec.x + lvec.y * lvec.y;
    float b = lvec.x * lpnt.x + lvec.y * lpnt.y;
    float c = lpnt.x * lpnt.x + lpnt.y * lpnt.y - sq_size0;
    float sol, x0, x1;
    ray_quad(a, b, c, &sol, &x0, &x1);
    int part = 0;
    if (sol >= 0.0f && fabsf(lpnt.z + sol * lvec.z) <= size[1]) {
        if (x < 0.0f || sol < x) {
            x = sol;
        }
    }
    V3 ldif = v3(lpnt.x, lpnt.y, lpnt.z - size[1]);
    a += lvec.z * lvec.z;
    b = dot(lvec, ldif);
    c = dot(ldif, ldif) - sq_size0;
    ray_quad(a, b, c, &sol, &x0, &x1);
    if (x0 >= 0.0f && lpnt.z + x0 * lvec.z >= size[1] && (x < 0.0f || x0 < x)) {
        x = x0;
        part = 1;
    }
    if (x1 >= 0.0f && lpnt.z + x1 * lvec.z >= size[1] && (x < 0.0f || x1 < x)) {
        x = x1;
        part = 1;
    }
    ldif = v3(ldif.x, ldif.y, lpnt.z + size[1]);
    b = dot(lvec, ldif);
    c = dot(ldif, ldif) - sq_size0;
    ray_quad(a, b, c, &sol, &x0, &x1);
    if (x0 >= 0.0f && lpnt.z + x0 * lvec.z <= -size[1] && (x < 0.0f || x0 < x)) {
        x = x0;
        part = -1;
    }
    if (x1 >= 0.0f && lpnt.z + x1 * lvec.z <= -size[1] && (x < 0.0f || x1 < x)) {
        x = x1;
        part = -1;
    }
    if (x >= 0.0f) {
        V3 n = v3(lpnt.x + lvec.x * x, lpnt.y + lvec.y * x,
                  part == 0 ? 0.0f : lpnt.z + lvec.z * x - size[1] * (float)part);
        *dist = x;
        *normal = mat_vec(mat, normalize3(n));
    } else {
        *dist = -1.0f;
        *normal = v3(0.0f, 0.0f, 0.0f);
    }
}

__device__ __forceinline__ void ray_ellipsoid(V3 pos, const float* mat, const float* size, V3 pnt, V3 vec,
                                             float* dist, V3* normal) {
    V3 lpnt, lvec;
    ray_map(pos, mat, pnt, vec, &lpnt, &lvec);
    V3 s = v3(safe_div(1.0f, size[0] * size[0]), safe_div(1.0f, size[1] * size[1]),
              safe_div(1.0f, size[2] * size[2]));
    V3 slvec = cw_mul(s, lvec);
    float a = dot(slvec, lvec);
    float b = dot(slvec, lpnt);
    float c = dot(cw_mul(s, lpnt), lpnt) - 1.0f;
    float sol, x0, x1;
    ray_quad(a, b, c, &sol, &x0, &x1);
    if (sol >= 0.0f) {
        V3 l = add(lpnt, scale(lvec, sol));
        *dist = sol;
        *normal = mat_vec(mat, normalize3(cw_mul(s, l)));
    } else {
        *dist = -1.0f;
        *normal = v3(0.0f, 0.0f, 0.0f);
    }
}

__device__ __forceinline__ void ray_cylinder(V3 pos, const float* mat, const float* size, V3 pnt, V3 vec,
                                             float* dist, V3* normal) {
    float ssz = size[0] * size[0] + size[1] * size[1];
    float dist_sphere;
    V3 normal_sphere;
    ray_sphere(pos, ssz, pnt, vec, &dist_sphere, &normal_sphere);
    if (dist_sphere < 0.0f) {
        *dist = -1.0f;
        *normal = v3(0.0f, 0.0f, 0.0f);
        return;
    }
    V3 lpnt, lvec;
    ray_map(pos, mat, pnt, vec, &lpnt, &lvec);
    float x = -1.0f;
    int part = 0;
    if (fabsf(lvec.z) > 1.0e-15f) {
        for (int side = -1; side <= 1; side += 2) {
            float sol = ((float)side * size[1] - lpnt.z) / lvec.z;
            if (sol >= 0.0f) {
                float px = lpnt.x + sol * lvec.x;
                float py = lpnt.y + sol * lvec.y;
                if (px * px + py * py <= size[0] * size[0] && (x < 0.0f || sol < x)) {
                    x = sol;
                    part = side;
                }
            }
        }
    }
    float a = lvec.x * lvec.x + lvec.y * lvec.y;
    float b = lvec.x * lpnt.x + lvec.y * lpnt.y;
    float c = lpnt.x * lpnt.x + lpnt.y * lpnt.y - size[0] * size[0];
    float sol, x0, x1;
    ray_quad(a, b, c, &sol, &x0, &x1);
    if (sol >= 0.0f && fabsf(lpnt.z + sol * lvec.z) <= size[1] && (x < 0.0f || sol < x)) {
        x = sol;
        part = 0;
    }
    if (x >= 0.0f) {
        V3 n;
        if (part == 0) {
            n = v3(lpnt.x + lvec.x * x, lpnt.y + lvec.y * x, 0.0f);
            n = normalize3(n);
        } else {
            n = v3(0.0f, 0.0f, (float)part);
        }
        *dist = x;
        *normal = mat_vec(mat, n);
    } else {
        *dist = -1.0f;
        *normal = v3(0.0f, 0.0f, 0.0f);
    }
}

__device__ __forceinline__ void ray_box(V3 pos, const float* mat, const float* size, V3 pnt, V3 vec,
                                        float* dist, V3* normal) {
    float ssz = dot(v3(size[0], size[1], size[2]), v3(size[0], size[1], size[2]));
    float dist_sphere;
    V3 normal_sphere;
    ray_sphere(pos, ssz, pnt, vec, &dist_sphere, &normal_sphere);
    if (dist_sphere < 0.0f) {
        *dist = -1.0f;
        *normal = v3(0.0f, 0.0f, 0.0f);
        return;
    }
    V3 lpnt, lvec;
    ray_map(pos, mat, pnt, vec, &lpnt, &lvec);
    float x = -1.0f;
    int face_axis = -1;
    int face_side = -1;
    for (int i = 0; i < 3; i++) {
        float lvi = (i == 0) ? lvec.x : ((i == 1) ? lvec.y : lvec.z);
        if (fabsf(lvi) > 1.0e-15f) {
            float lpi = (i == 0) ? lpnt.x : ((i == 1) ? lpnt.y : lpnt.z);
            int id0 = (i == 0) ? 1 : 0;
            int id1 = (i == 2) ? 1 : 2;
            for (int side = -1; side <= 1; side += 2) {
                float sol = ((float)side * size[i] - lpi) / lvi;
                if (sol >= 0.0f) {
                    float lp0 = (id0 == 0) ? lpnt.x : ((id0 == 1) ? lpnt.y : lpnt.z);
                    float lp1 = (id1 == 0) ? lpnt.x : ((id1 == 1) ? lpnt.y : lpnt.z);
                    float lv0 = (id0 == 0) ? lvec.x : ((id0 == 1) ? lvec.y : lvec.z);
                    float lv1 = (id1 == 0) ? lvec.x : ((id1 == 1) ? lvec.y : lvec.z);
                    float p0 = lp0 + sol * lv0;
                    float p1 = lp1 + sol * lv1;
                    if (fabsf(p0) <= size[id0] && fabsf(p1) <= size[id1]) {
                        if (x < 0.0f || sol < x) {
                            x = sol;
                            face_axis = i;
                            face_side = side;
                        }
                    }
                }
            }
        }
    }
    if (x >= 0.0f) {
        V3 n = v3(0.0f, 0.0f, 0.0f);
        if (face_axis == 0) {
            n.x = (float)face_side;
        } else if (face_axis == 1) {
            n.y = (float)face_side;
        } else {
            n.z = (float)face_side;
        }
        *dist = x;
        *normal = mat_vec(mat, n);
    } else {
        *dist = -1.0f;
        *normal = v3(0.0f, 0.0f, 0.0f);
    }
}

// Dispatch by MuJoCo geom_type; mesh (7/8) and hfield (1) are deferred to S3 and
// report a miss. The S2b reference set only needs primitives plus the one mesh
// ray that is explicitly excluded.
__device__ __forceinline__ void intersect_primitive(int geom_type, V3 pos, const float* size, const float* mat,
                                                    V3 pnt, V3 vec, float* dist, V3* normal) {
    *dist = -1.0f;
    *normal = v3(0.0f, 0.0f, 0.0f);
    if (geom_type == 0) {
        ray_plane(pos, mat, size, pnt, vec, dist, normal);
    } else if (geom_type == 2) {
        ray_sphere(pos, size[0] * size[0], pnt, vec, dist, normal);
    } else if (geom_type == 3) {
        ray_capsule(pos, mat, size, pnt, vec, dist, normal);
    } else if (geom_type == 4) {
        ray_ellipsoid(pos, mat, size, pnt, vec, dist, normal);
    } else if (geom_type == 5) {
        ray_cylinder(pos, mat, size, pnt, vec, dist, normal);
    } else if (geom_type == 6) {
        ray_box(pos, mat, size, pnt, vec, dist, normal);
    }
}

// Slab test with a conservative entry parameter; returns false if the ray cannot
// reach the box within [0, t_limit].
__device__ __forceinline__ bool ray_aabb(V3 origin, V3 direction, const float* node_bounds, float t_limit,
                                         float* t_entry) {
    float t0 = 0.0f;
    float t1 = t_limit;
    for (int axis = 0; axis < 3; axis++) {
        float o = (axis == 0) ? origin.x : ((axis == 1) ? origin.y : origin.z);
        float d = (axis == 0) ? direction.x : ((axis == 1) ? direction.y : direction.z);
        float lo = node_bounds[axis];
        float hi = node_bounds[3 + axis];
        if (d == 0.0f) {
            if (o < lo || o > hi) {
                return false;
            }
            continue;
        }
        float near_t = (lo - o) / d;
        float far_t = (hi - o) / d;
        if (near_t > far_t) {
            float swap = near_t;
            near_t = far_t;
            far_t = swap;
        }
        t0 = near_t > t0 ? near_t : t0;
        t1 = far_t < t1 ? far_t : t1;
        if (t0 > t1) {
            return false;
        }
    }
    *t_entry = t0;
    return true;
}

extern "C" __global__ void scene_bvh_ray(const int* meta, const float* params, const float* rays, float* out,
                                         unsigned d0, unsigned d1, unsigned d2, unsigned d3) {
    (void)d0;
    (void)d1;
    (void)d2;
    const int n_nodes = meta[0];
    const int n_leaf = meta[1];
    const int n_ray = meta[2];
    const int root = meta[3];
    (void)n_leaf;

    unsigned ray_index = blockDim.x * blockIdx.x + threadIdx.x;
    if ((int)ray_index >= n_ray) {
        return;
    }

    const int* links = meta + 4;
    const int* leaf_geom = links + 2 * n_nodes;
    const int* leaf_type = leaf_geom + n_leaf;
    const float* node_bounds = params;
    const float* leaf_size = node_bounds + 6 * n_nodes;
    const float* leaf_xpos = leaf_size + 3 * (long)n_leaf;
    const float* leaf_xmat = leaf_xpos + 3 * (long)n_leaf;

    V3 origin = v3(rays[6 * ray_index + 0], rays[6 * ray_index + 1], rays[6 * ray_index + 2]);
    V3 direction = v3(rays[6 * ray_index + 3], rays[6 * ray_index + 4], rays[6 * ray_index + 5]);

    float best_distance = 1.0e10f;
    int best_geom = -1;
    V3 best_normal = v3(0.0f, 0.0f, 0.0f);

    int stack[128];
    int depth = 0;
    stack[depth++] = root;
    while (depth > 0) {
        int node = stack[--depth];
        const float* bounds = node_bounds + 6 * node;
        float entry = 0.0f;
        if (!ray_aabb(origin, direction, bounds, best_distance, &entry)) {
            continue;
        }
        int left = links[2 * node];
        if (left >= 0) {
            int right = links[2 * node + 1];
            // Push the farther child first by entry parameter is not tracked per
            // node; both children are tested, nearest hit is selected.
            if (depth + 2 <= 128) {
                stack[depth++] = left;
                stack[depth++] = right;
            }
            continue;
        }
        int leaf = -left - 1;
        V3 pos = v3(leaf_xpos[3 * leaf + 0], leaf_xpos[3 * leaf + 1], leaf_xpos[3 * leaf + 2]);
        const float* mat = leaf_xmat + 9 * leaf;
        float dist = -1.0f;
        V3 normal = v3(0.0f, 0.0f, 0.0f);
        intersect_primitive(leaf_type[leaf], pos, leaf_size + 3 * leaf, mat, origin, direction, &dist, &normal);
        if (dist >= 0.0f && dist < best_distance) {
            best_distance = dist;
            best_geom = leaf_geom[leaf];
            best_normal = normal;
        }
    }

    float* target = out + 5 * ray_index;
    if (best_distance >= 1.0e10f) {
        target[0] = -1.0f;
        target[1] = __int_as_float(-1);
        target[2] = 0.0f;
        target[3] = 0.0f;
        target[4] = 0.0f;
    } else {
        target[0] = best_distance;
        target[1] = __int_as_float(best_geom);
        target[2] = best_normal.x;
        target[3] = best_normal.y;
        target[4] = best_normal.z;
    }
}
// 渲染内核的 mesh 三角求交数学（渲染/BVH S7b1）。
//
// 逐式复制 S3 `src/render/bvh/cuda/mesh.cu`（该文件只读、未修改）的 woop 三角求交、
// 保守 AABB 剪枝与 `wp.mesh_query_ray` 遍历；与 S3 的唯一差异是 `normalize3` 改名为
// `normalize3_mesh`（S2b `scene.cu` 已在本编译单元前置、定义了不同语义的同名函数），
// 表达式、运算顺序、比较方向与 S3 逐字一致。
//
// P10（2026-11-11）：恢复 `mesh_query_ray` 返回前的**局部归一化**（与 S3 `mesh.cu:290` 一致；
// `f728b2d7` 依错误 H2 前提删除，已由 Arch 离线重放证伪），并新增 `mat_vec_xpref`
// （上游 warp `mat.h:3180 mul(mat,vec)` 的 x 项先单独舍入）供 mesh 法线消费点使用。
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

// 上游 warp `mat.h:3180 mul(mat,vec)`：**x 项先单独舍入**，再 fma y、fma z（行主序索引与
// `scene.cu::mat_vec` 完全一致，只把独立舍入项从 y 改为 x）。仅供 mesh 法线消费点使用；
// 不改 `scene.cu::mat_vec`（S5 已验路径保持）。
__device__ __forceinline__ V3 mat_vec_xpref(const float* m, V3 a) {
    return v3(fmaf(m[2], a.z, fmaf(m[1], a.y, m[0] * a.x)),
              fmaf(m[5], a.z, fmaf(m[4], a.y, m[3] * a.x)),
              fmaf(m[8], a.z, fmaf(m[7], a.y, m[6] * a.x)));
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
    // 上游 `wp.mesh_query_ray`（warp/native/mesh.h:1880）返回前对**局部法线**归一化（与 S3
    // `mesh.cu:290` 一致）；消费端随后做 `mat @ n`（x 项先舍入）与再一次归一化 ⇒ 上游 mesh
    // 法线链共两次 normalize。
    // 注：G-L2 判据 A RCA 的 H2 前提（「cross 未归一化、消费端只 normalize 一次」）已被 Arch
    // 离线重放证伪并作废；`f728b2d7` 依该前提删除的这次归一化在本卡（P10）恢复。
    *out_normal = normalize3_mesh(min_normal);
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
// 渲染/BVH S7b1 渲染内核：主射线阶段（平面深度 + 分割）。
//
// 源码由 `src/render/kernel/cast.rs::DEPTH_SEG_SOURCE` 用 Rust `concat!` 拼成三段：
//   1) S2b `src/render/bvh/cuda/scene.cu`：`V3` 辅助、`_ray_map`/`_ray_quad` 与六个
//      primitive 求交逐字复用（该文件只读、未修改）；其中同名内核 `scene_bvh_ray`
//      出现在本编译单元但不会被发射；
//   2) `cuda/render_math.cu`：S3 mesh woop 数学逐式复制；
//   3) 本文件：相机系→世界系、去畸变射线重建（dynamic）、场景+网格组合、最近命中
//      tie-break、深度/分割写入。
//
// ABI（`RenderSignature` 槽位见 `kernel/mod.rs::DEPTH_SEG_SLOTS`，按值布局由
// `RenderArrayValue`/`RenderScalar` 校验；首槽位 `launch_bounds_t<2>` 形状
// `[nworld, total_rays]`，`index = worldid * total_rays + rayid`）：
//
//   extern "C" __global__ void render_depth_seg(
//       wp::launch_bounds_t<2> dim,
//       wp::array_t<wp::int32> scene_meta,
//       wp::array_t<float> scene_params,
//       wp::array_t<wp::int32> mesh_meta,
//       wp::array_t<float> mesh_params,
//       wp::array_t<wp::vec2i> mesh_offsets,
//       wp::array_t<wp::int32> geom_type,
//       wp::array_t<wp::vec4> geom_rgba,
//       wp::array_t<wp::int32> geom_matid,
//       wp::array_t<wp::vec4> mat_rgba,
//       wp::array_t<wp::int32> cam_projection,
//       wp::array_t<float> cam_fovy,
//       wp::array_t<wp::vec2> cam_sensorsize,
//       wp::array_t<wp::vec4> cam_intrinsic,
//       wp::array_t<wp::vec3> cam_xpos,
//       wp::array_t<wp::mat33> cam_xmat,
//       wp::array_t<wp::vec2i> cam_res,
//       wp::array_t<wp::int32> depth_adr,
//       wp::array_t<wp::int32> seg_adr,
//       wp::array_t<bool> render_depth,
//       wp::array_t<bool> render_seg,
//       wp::array_t<wp::vec3> ray,
//       wp::array_t<wp::vec3> ray_offset,
//       wp::array_t<float> depth_out,
//       wp::array_t<wp::vec2i> seg_out,
//       wp::int32 nrender, wp::int32 znear_bits, wp::int32 zfar_bits,
//       wp::int32 depth_stride, wp::int32 seg_stride,
//       bool use_precomputed_rays, bool cull_backfaces)
//
// 语义口径（冻结上游 `render.py::_render_megakernel` 71da24d，本片只覆盖主射线/
// 深度/分割）：
//   - 射线：`use_precomputed_rays=true` 读 S5 缓冲；`false` 用 `compute_ray` 逐像素重算
//     （与 S5 `build_rays.cu` 同一份 f32 表达式，spp=1 时逐位等价）。
//   - 世界系：`ray_dir_world = cam_xmat @ ray_dir_local_cam`（行点积、不重归一化）；
//     `ray_origin_world = cam_xpos + cam_xmat @ ray_offset`（透视 offset 恒 0，加法位等；
//     正交相机由 S5 给出 offset，与上游 `has_orthographic_camera` 分支同式）。
//   - `max_cam_dist = zfar / max(-ray_dir_local_cam.z, 1e-6)`（`zfar > 0`），否则 MJ_MAXVAL。
//   - 命中：primitive 复用 S2b 数学（**球面例外**：`render_math.cu::render_sphere_hit` 用
//     显式 fma 序列对齐冻结上游 megakernel 的 d0/det 运算顺序，见该节注释）；mesh 复用 S3
//     数学（局部系 woop + 8 次背面剔除步进）；
//     查询上界 `dist*(1+RAY_TOL_REL)+RAY_TOL_ABS`、透明 geom（`rgba[3]==0`）跳过、
//     背面剔除 `dot(dir, n) > 0` 置 miss、近平局取更大 geom id。
//   - 深度：平面深度 `dist * -ray_dir_local_cam.z`；无命中/`zfar` 外写 0。
//   - 分割：命中写 `(geom_id, 5)`（mjOBJ_GEOM）；无命中原样写 `(-1, -1)`，等价上游
//     `render()` 先 `seg_data.fill_(-1,-1)` 再只写命中（S7b1 每次渲染都显式写背景）。
//   - 本片不含：RGB/着色、AA（nsamples=1、ray_base=0）、阴影/天空盒/splat/flex/hfield。
//   - znear/zfar 以 f32 位型经 `int32` 标量传入：渲染 ABI 的按值标量只有
//     int32/uint64/bool（上游这两个量是 `wp.static` 编译期常量，没有浮点标量位）。
#define WP_NO_CRT
#define WP_NO_BFLOAT16
#include "builtin.h"

// f32(pi / 180.0) exactly as Warp folds `wp.static(wp.pi / 180.0)` (S5 build_rays.cu)。
// 名字加 RENDER_ 前缀：W15 `builtin.h` 已定义同名 `DEG_TO_RAD` 宏。
#define RENDER_DEG_TO_RAD 0.017453292519943295f
// MuJoCo `mjMAXVAL`（上游 `float(MJ_MAXVAL)`）。
#define MJ_MAXVAL 1.0e10f

// warp vec.h normalize(vec3) as contracted by the frozen reference compiler：组件平方和
// 发射为 fma(z,z,fma(y,y,x*x))；显式 fmaf 链让舍入与局部 NVRTC 收缩选择无关
// （逐字取自 S5 `build_rays.cu`，保证 dynamic 与 precomputed 射线逐位一致）。
__device__ __forceinline__ V3 normalize3_rays(V3 a) {
    float length = sqrtf(fmaf(a.z, a.z, fmaf(a.y, a.y, a.x * a.x)));
    if (length > 0.0f) {
        return v3(a.x / length, a.y / length, a.z / length);
    }
    return v3(0.0f, 0.0f, 0.0f);
}

// compute_ray(projection, fovy, sensorsize, intrinsic, img_w, img_h, px, py, znear).
// 逐字复制 S5 `camera/cuda/build_rays.cu`（上游 `render_util.py::compute_ray` 的 f32 翻译）。
__device__ __forceinline__ void compute_ray(int projection, float fovy, float sensor_size_x,
                                            float sensor_size_y, const float* intrinsic, int img_w,
                                            int img_h, int px, int py, float znear, V3* direction,
                                            V3* offset) {
    float inv_img_h = 1.0f / (float)img_h;
    if (projection == 1) {
        *direction = v3(0.0f, 0.0f, -1.0f);
        float aspect = (float)img_w * inv_img_h;
        float sensor_h = fovy;
        float sensor_w = sensor_h * aspect;
        float left = -0.5f * sensor_w;
        float top = 0.5f * sensor_h;
        float bottom = -top;
        float u = ((float)px + 0.5f) / (float)img_w;
        float v = ((float)py + 0.5f) * inv_img_h;
        float x = fmaf(sensor_w, u, left);
        float y = fmaf(bottom - top, v, top);
        *offset = v3(x, y, 0.0f);
        return;
    }

    float aspect = (float)img_w * inv_img_h;
    float sensor_h = sensor_size_y;
    float left;
    float right;
    float top;
    float bottom;
    if (sensor_h != 0.0f) {
        float fx = intrinsic[0];
        float fy = intrinsic[1];
        float cx = intrinsic[2];
        float cy = intrinsic[3];
        float sensor_w = sensor_size_x;
        float target_aspect = aspect;
        float sensor_aspect = sensor_w / sensor_h;
        if (target_aspect > sensor_aspect) {
            sensor_h = sensor_w / target_aspect;
        } else if (target_aspect < sensor_aspect) {
            sensor_w = sensor_h * target_aspect;
        }
        float inv_fx_znear = znear / fx;
        float inv_fy_znear = znear / fy;
        float half_sensor_w = 0.5f * sensor_w;
        float half_sensor_h = 0.5f * sensor_h;
        left = -inv_fx_znear * (half_sensor_w - cx);
        right = inv_fx_znear * (half_sensor_w + cx);
        top = inv_fy_znear * (half_sensor_h - cy);
        bottom = -inv_fy_znear * (half_sensor_h + cy);
    } else {
        float fovy_rad = fovy * RENDER_DEG_TO_RAD;
        float half_height = znear * tanf(0.5f * fovy_rad);
        float half_width = half_height * aspect;
        left = -half_width;
        right = half_width;
        top = half_height;
        bottom = -half_height;
    }
    float u = ((float)px + 0.5f) / (float)img_w;
    float v = ((float)py + 0.5f) * inv_img_h;
    float x = fmaf(right - left, u, left);
    float y = fmaf(bottom - top, v, top);
    *direction = normalize3_rays(v3(x, y, -znear));
    *offset = v3(0.0f, 0.0f, 0.0f);
}
// 场景 BVH 设备布局（`cast.rs` 打包；与 S2b `scene.cu` 的 meta/params 同式，另加
// world 根与 leaf→网格绑定；不再单独存 leaf geom 类型，按上游从 `geom_type[geom_id]` 取）：
//   meta   = [n_nodes, n_leaf, n_world, reserved,
//             roots(n_world), links(2*n_nodes), leaf_geom(n_leaf), leaf_mesh(n_leaf)]
//   params = [node bounds 6*n_nodes, leaf size 3*n_leaf, leaf xpos 3*n_leaf,
//             leaf xmat(row-major) 9*n_leaf]
struct SceneView {
    int n_nodes;
    int n_leaf;
    int n_world;
    const int* roots;
    const int* links;
    const int* leaf_geom;
    const int* leaf_mesh;
    const float* node_bounds;
    const float* leaf_size;
    const float* leaf_xpos;
    const float* leaf_xmat;
};

__device__ __forceinline__ SceneView scene_view(const int* meta, const float* params) {
    SceneView view;
    view.n_nodes = meta[0];
    view.n_leaf = meta[1];
    view.n_world = meta[2];
    view.roots = meta + 4;
    view.links = view.roots + view.n_world;
    view.leaf_geom = view.links + 2 * view.n_nodes;
    view.leaf_mesh = view.leaf_geom + view.n_leaf;
    view.node_bounds = params;
    view.leaf_size = view.node_bounds + 6 * view.n_nodes;
    view.leaf_xpos = view.leaf_size + 3 * view.n_leaf;
    view.leaf_xmat = view.leaf_xpos + 3 * view.n_leaf;
    return view;
}

// 网格三角 BVH 设备布局（`render_math.cu::mesh_query_ray` 消费；`mesh_offsets` 指向
// 每个网格的 (meta 基址, params 基址)）：
//   meta   = [n_nodes, n_tri, n_vert, root, links(2*n_nodes), order(n_tri), indices(3*n_tri)]
//   params = [node bounds 6*n_nodes, vertices 3*n_vert]
// 世界位姿来自场景 leaf（同 S3 `MeshRayGeometry` 的 geom 位姿）。
__device__ __forceinline__ float ray_mesh_leaf(const int* mesh_meta, const float* mesh_params,
                                               const wp::vec2i* mesh_offsets, int mesh_index, V3 pos,
                                               const float* mat, V3 pnt, V3 vec, float max_t,
                                               bool cull_backfaces, V3* normal) {
    *normal = v3(0.0f, 0.0f, 0.0f);
    if (mesh_index < 0) {
        return -1.0f;
    }
    const int* meta = mesh_meta + mesh_offsets[mesh_index][0];
    const float* params = mesh_params + mesh_offsets[mesh_index][1];
    const int n_nodes = meta[0];
    const int n_tri = meta[1];
    const int n_vert = meta[2];
    const int root = meta[3];
    const int* links = meta + 4;
    const int* order = links + 2 * n_nodes;
    const int* indices = order + n_tri;
    const float* node_bounds = params;
    const float* vertices = params + 6 * n_nodes;

    V3 lpnt;
    V3 lvec;
    ray_map(pos, mat, pnt, vec, &lpnt, &lvec);
    float distance = -1.0f;
    float offset = 0.0f;
    for (int march = 0; march < CULL_MARCHES; march++) {
        V3 start = add(lpnt, scale(lvec, offset));
        float t = 0.0f;
        float tu = 0.0f;
        float tv = 0.0f;
        float sign = 0.0f;
        V3 n_local = v3(0.0f, 0.0f, 0.0f);
        int face = -1;
        if (!mesh_query_ray(start, lvec, max_t - offset, n_nodes, root, n_vert, links, order, indices,
                            node_bounds, vertices, &t, &tu, &tv, &sign, &n_local, &face)) {
            break;
        }
        if (!cull_backfaces || dot(lvec, n_local) < 0.0f) {
            distance = offset + t;
            *normal = normalize3_mesh(mat_vec_xpref(mat, n_local));
            break;
        }
        offset += t * (1.0f + RAY_TOL_REL) + RAY_TOL_ABS;
    }
    return distance;
}

// 上游 `_geom_rgba`：geom_rgba 为默认 (0.5,0.5,0.5,1) 且 matid>=0 时改用 mat_rgba。
__device__ __forceinline__ float geom_alpha(const int* geom_matid, const float* geom_rgba,
                                            const float* mat_rgba, int geom_id) {
    float r = geom_rgba[4 * geom_id + 0];
    float g = geom_rgba[4 * geom_id + 1];
    float b = geom_rgba[4 * geom_id + 2];
    float a = geom_rgba[4 * geom_id + 3];
    int mat_id = geom_matid[geom_id];
    if (mat_id >= 0 && r == 0.5f && g == 0.5f && b == 0.5f && a == 1.0f) {
        a = mat_rgba[4 * mat_id + 3];
    }
    return a;
}

// 候选节点最小堆：Warp `wp.bvh_query_ray` 按 AABB 进入距离升序给出候选叶子，上游
// tie-break（`RAY_TOL_REL` 近平局取更大 geom id）依赖该顺序，故这里按 entry 升序
// 出堆复刻候选顺序；S2b 的栈序不满足该语义。
struct NodeCandidate {
    int node;
    float entry;
};

#define MAX_CANDIDATES 128

__device__ __forceinline__ void push_candidate(NodeCandidate* heap, int* count, int node, float entry) {
    if (*count >= MAX_CANDIDATES) {
        return;
    }
    int index = (*count)++;
    while (index > 0) {
        int parent = (index - 1) / 2;
        if (heap[parent].entry <= entry) {
            break;
        }
        heap[index] = heap[parent];
        index = parent;
    }
    heap[index].node = node;
    heap[index].entry = entry;
}

__device__ __forceinline__ NodeCandidate pop_candidate(NodeCandidate* heap, int* count) {
    NodeCandidate top = heap[0];
    (*count)--;
    if (*count > 0) {
        NodeCandidate last = heap[*count];
        int index = 0;
        while (true) {
            int left = 2 * index + 1;
            if (left >= *count) {
                break;
            }
            int right = left + 1;
            int child = (right < *count && heap[right].entry < heap[left].entry) ? right : left;
            if (heap[child].entry >= last.entry) {
                break;
            }
            heap[index] = heap[child];
            index = child;
        }
        heap[index] = last;
    }
    return top;
}

// 上游 `cast_ray` 的最近命中子集：候选 leaf 按进入距离升序；透明 geom 跳过；primitive 与
// mesh 进入同一比较；背面剔除后按 `RAY_TOL_REL` 容差 tie-break（近平局取更大 geom id）。
__device__ __forceinline__ void cast_ray(const SceneView& scene, const int* mesh_meta,
                                         const float* mesh_params, const wp::vec2i* mesh_offsets,
                                         const int* geom_type, const int* geom_matid,
                                         const float* geom_rgba, const float* mat_rgba, int worldid,
                                         V3 origin, V3 direction, float max_dist, bool cull_backfaces,
                                         int* out_geom_id, float* out_dist) {
    float dist = max_dist;
    int geom_id = -1;
    float initial_bound = dist * (1.0f + RAY_TOL_REL) + RAY_TOL_ABS;
    int root = scene.roots[worldid];
    NodeCandidate heap[MAX_CANDIDATES];
    int count = 0;
    float entry = 0.0f;
    if (!ray_aabb(origin, direction, scene.node_bounds + 6 * root, initial_bound, &entry)) {
        *out_geom_id = -1;
        *out_dist = dist;
        return;
    }
    push_candidate(heap, &count, root, entry);
    while (count > 0) {
        NodeCandidate candidate = pop_candidate(heap, &count);
        float bound = dist * (1.0f + RAY_TOL_REL) + RAY_TOL_ABS;
        if (candidate.entry > bound) {
            break;
        }
        int node = candidate.node;
        int left = scene.links[2 * node];
        if (left >= 0) {
            int right = scene.links[2 * node + 1];
            float child_entry = 0.0f;
            if (ray_aabb(origin, direction, scene.node_bounds + 6 * left, bound, &child_entry)) {
                push_candidate(heap, &count, left, child_entry);
            }
            if (ray_aabb(origin, direction, scene.node_bounds + 6 * right, bound, &child_entry)) {
                push_candidate(heap, &count, right, child_entry);
            }
            continue;
        }
        int leaf = -left - 1;
        int geom = scene.leaf_geom[leaf];
        int gtype = geom_type[geom];
        if (geom_alpha(geom_matid, geom_rgba, mat_rgba, geom) == 0.0f) {
            continue;
        }
        const float* leaf_size = scene.leaf_size + 3 * leaf;
        const float* leaf_pos = scene.leaf_xpos + 3 * leaf;
        const float* leaf_mat = scene.leaf_xmat + 9 * leaf;
        float query_dist = dist * (1.0f + RAY_TOL_REL) + RAY_TOL_ABS;
        float d = -1.0f;
        V3 n = v3(0.0f, 0.0f, 0.0f);
        V3 pos = v3(leaf_pos[0], leaf_pos[1], leaf_pos[2]);
        if (gtype == 7 || gtype == 8) {
            // mesh 与 SDF 同路：上游按底层 mesh 渲染（render.py L738-761）；host 仍拒绝 8（今天不可达）。
            d = ray_mesh_leaf(mesh_meta, mesh_params, mesh_offsets, scene.leaf_mesh[leaf], pos,
                              leaf_mat, origin, direction, query_dist, cull_backfaces, &n);
        } else {
            render_intersect_primitive(gtype, pos, leaf_size, leaf_mat, origin, direction, &d, &n);
        }
        if (cull_backfaces && d >= 0.0f && dot(direction, n) > 0.0f) {
            d = -1.0f;
        }
        float tol = RAY_TOL_REL * (dist > 1.0f ? dist : 1.0f);
        bool closer = (d < dist - tol) || (fabsf(d - dist) <= tol && (geom_id < 0 || geom > geom_id));
        if (d >= 0.0f && closer) {
            dist = d;
            geom_id = geom;
        }
    }
    *out_geom_id = geom_id;
    *out_dist = dist;
}

extern "C" __global__ void render_depth_seg(
    wp::launch_bounds_t<2> dim, wp::array_t<wp::int32> scene_meta, wp::array_t<float> scene_params,
    wp::array_t<wp::int32> mesh_meta, wp::array_t<float> mesh_params, wp::array_t<wp::vec2i> mesh_offsets,
    wp::array_t<wp::int32> geom_type, wp::array_t<wp::vec4> geom_rgba, wp::array_t<wp::int32> geom_matid,
    wp::array_t<wp::vec4> mat_rgba, wp::array_t<wp::int32> cam_projection, wp::array_t<float> cam_fovy,
    wp::array_t<wp::vec2> cam_sensorsize, wp::array_t<wp::vec4> cam_intrinsic,
    wp::array_t<wp::vec3> cam_xpos, wp::array_t<wp::mat33> cam_xmat, wp::array_t<wp::vec2i> cam_res,
    wp::array_t<wp::int32> depth_adr, wp::array_t<wp::int32> seg_adr,
    wp::array_t<bool> render_depth, wp::array_t<bool> render_seg,
    wp::array_t<wp::vec3> ray, wp::array_t<wp::vec3> ray_offset, wp::array_t<float> depth_out,
    wp::array_t<wp::vec2i> seg_out, wp::int32 nrender, wp::int32 znear_bits, wp::int32 zfar_bits,
    wp::int32 depth_stride, wp::int32 seg_stride, bool use_precomputed_rays, bool cull_backfaces) {
    size_t index = (size_t)blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= dim.size) {
        return;
    }
    int worldid = 0;
    int rayid = 0;
    wp::tid(worldid, rayid, index, dim);
    if (worldid >= dim.shape[0]) {
        return;
    }

    const int* modes = cam_projection;
    const float* fovys = cam_fovy;
    // `array_t<vec2/vec4/vec3/mat33>` 只能转成元素指针；几何类数组按原始 f32 视图读取
    // （本机序与 Warp 设备布局一致，S7a ABI 证据）。
    const float* sensors = (const float*)cam_sensorsize.data;
    const float* intrinsics = (const float*)cam_intrinsic.data;
    const wp::vec2i* resolutions = cam_res;

    int camid = -1;
    int rayid_local = -1;
    int accum = 0;
    for (int camera = 0; camera < nrender; camera++) {
        int num_i = resolutions[camera][0] * resolutions[camera][1];
        if (rayid < accum + num_i) {
            camid = camera;
            rayid_local = rayid - accum;
            break;
        }
        accum += num_i;
    }
    if (camid == -1 || rayid_local < 0) {
        return;
    }
    const bool* depth_flags = render_depth;
    const bool* seg_flags = render_seg;
    if (!depth_flags[camid] && !seg_flags[camid]) {
        return;
    }
    const float znear = __int_as_float(znear_bits);
    const float zfar = __int_as_float(zfar_bits);
    V3 ray_dir_local_cam;
    V3 ray_offset_local_cam;
    const float* directions = (const float*)ray.data;
    const float* offsets = (const float*)ray_offset.data;
    if (use_precomputed_rays) {
        ray_dir_local_cam = v3(directions[3 * rayid + 0], directions[3 * rayid + 1],
                               directions[3 * rayid + 2]);
        ray_offset_local_cam =
            v3(offsets[3 * rayid + 0], offsets[3 * rayid + 1], offsets[3 * rayid + 2]);
    } else {
        int img_w = resolutions[camid][0];
        int img_h = resolutions[camid][1];
        int px = rayid_local % img_w;
        int py = rayid_local / img_w;
        compute_ray(modes[camid], fovys[camid], sensors[2 * camid], sensors[2 * camid + 1],
                    intrinsics + 4 * camid, img_w, img_h, px, py, znear, &ray_dir_local_cam,
                    &ray_offset_local_cam);
    }

    const float* cam_positions = (const float*)cam_xpos.data;
    const float* cam_matrices = (const float*)cam_xmat.data; int cam_slot = worldid * (int)cam_xpos.shape.dims[1] + camid;
    V3 cam_pos = v3(cam_positions[3 * cam_slot + 0], cam_positions[3 * cam_slot + 1],
                    cam_positions[3 * cam_slot + 2]);
    const float* cam_mat = cam_matrices + 9 * cam_slot;
    V3 ray_origin_world = add(cam_pos, mat_vec(cam_mat, ray_offset_local_cam));
    V3 ray_dir_world = mat_vec(cam_mat, ray_dir_local_cam);

    float max_cam_dist = MJ_MAXVAL;
    if (zfar > 0.0f) {
        max_cam_dist = zfar / fmaxf(-ray_dir_local_cam.z, 1.0e-6f);
    }

    SceneView scene = scene_view(scene_meta, scene_params);
    int geom_id = -1;
    float dist = max_cam_dist;
    cast_ray(scene, mesh_meta, mesh_params, mesh_offsets, (const int*)geom_type.data,
             (const int*)geom_matid.data, (const float*)geom_rgba.data, (const float*)mat_rgba.data,
             worldid, ray_origin_world, ray_dir_world, max_cam_dist, cull_backfaces, &geom_id, &dist);

    if (geom_id != -1 && zfar > 0.0f && (dist * -ray_dir_local_cam.z) > zfar) {
        geom_id = -1;
    }

    if (depth_flags[camid]) {
        int depth_index = depth_adr[camid] + rayid_local;
        float depth = 0.0f;
        if (geom_id != -1) {
            depth = dist * -ray_dir_local_cam.z;
        }
        float* depth_pixels = depth_out;
        depth_pixels[((long)worldid) * depth_stride + depth_index] = depth;
    }
    if (seg_flags[camid]) {
        int seg_index = seg_adr[camid] + rayid_local;
        wp::vec2i value;
        if (geom_id != -1) {
            value[0] = geom_id;
            value[1] = 5;  // mjOBJ_GEOM
        } else {
            value[0] = -1;
            value[1] = -1;
        }
        wp::vec2i* seg_pixels = seg_out;
        seg_pixels[((long)worldid) * seg_stride + seg_index] = value;
    }
}
