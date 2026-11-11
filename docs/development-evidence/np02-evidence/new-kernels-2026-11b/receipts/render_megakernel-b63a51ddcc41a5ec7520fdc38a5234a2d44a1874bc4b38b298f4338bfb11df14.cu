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
// R06-S3a：flex 渲染设备侧（渲染/BVH R06）。
//
// 拼接位置：`src/render/kernel/mod.rs::MEGAKERNEL_SOURCE` 中 `scene.cu` + `render_math.cu`
// 之后、`render_common.cu` 之前——因此可直接复用：
//   - `scene.cu`：`V3` 辅助（`v3`/`add`/`sub`/`scale`/`dot`/`mat_vec`/`mat_t_vec`/`ray_map`）
//     与 `ray_capsule`（1D flex 胶囊求交；上游 `ray.py::ray_capsule` 的逐式翻译）；
//   - `render_math.cu`：`RAY_TOL_REL`/`RAY_TOL_ABS` 等共享常量。
//
// 语义锚点（冻结上游 `71da24d`，逐项对表见证据 README §pre-check）：
//   - `render_util.py` L490-512：1D flex 每条边一个胶囊图元（`edgeid = flex_edgeadr[f] + e`）；
//     2D/3D 每 flex 一个 mesh 图元（`edgeid = -1`）。R06-S3c 起两条路径都在设备侧接通：
//     1D 走本文件的胶囊求交；2D/3D 走 `bvh/flex_scene` 打包的三角 BVH（世界空间、identity
//     位姿、无内部背面剔除），设备条目索引 = `flex_bvh_id[flexid] + worldid`。
//   - `render.py` L762-800：flex 叶命中分派与胶囊几何（`pos = 0.5*(v0+v1)`、
//     `vec = v1-v0`、`quat_z2vec`、`size = (radius, 0.5*|vec|, 0)`、`ray_capsule`）；
//     命中后 `hit_geom_id = -2`、`hit_mesh_id = flexid`；`flex_rgba[flexid][3] == 0` 跳过。
//   - `ray.py` L759-802：flex 已在世界空间，**无 `_ray_map`、无多段 march、无内部背面剔除**；
//     背面剔除由调用方 `cast_ray` 的统一后判据处理（`render.py` L802-806）。
//   - `types.py` L532：`GeomType.FLEX = mjGEOM_FLEX`；`mjtype.h` L292-301：`mjOBJ_FLEX = 9`
//     （seg 编码 `(flexid, 9)` 用）。
//
// 精度：只做 f32 算术；`sinf`/`cosf` 的 f64 归约属渲染门禁精确放行项。`warp_length3`
// 按 NVRTC `fmad=true` 的收缩顺序显式用 `fmaf` 固定（与 `render_common.cu::normalize3_rays`
// 同一口径），避免依赖推断收缩。

// 上游 `types.MJ_MINVAL`（mujoco `mjMINVAL`，`mjtype.h` L27）。
#define FLEX_MINVAL 1.0e-15f

// `warp vec.h length(vec3)`：`sqrt(dot(v,v))`，收缩序显式固定为 fma(z,z,fma(y,y,x*x))。
__device__ __forceinline__ float warp_length3(V3 v) {
    return sqrtf(fmaf(v.z, v.z, fmaf(v.y, v.y, v.x * v.x)));
}

// 上游 `math.py::quat_z2vec`：从 z 轴旋到给定向量的四元数（分量序 x,y,z,w）。
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
        // 近平行：上游 `math.py::quat_z2vec` 是**整体替换**为单位四元数；反向时 `quat = (1,0,0,0)`，
        // 同向保持初始单位四元数 `(0,0,0,1)`。z2vec 修复（2026-11-11）：必须显式写全四位——
        // 旧版只写 `quat[0] = 1.0f`，遗留初始化的 `quat[3] = 1.0f`，得到 `(1,0,0,1)`（矩阵差 2.0）。
        if (unit.z < 0.0f) {
            quat[0] = 1.0f;
            quat[1] = 0.0f;
            quat[2] = 0.0f;
            quat[3] = 0.0f;
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

// 上游 `math.py::quat_to_mat`：四元数 → 行主序 3x3（与 `geom_xmat` / `ray_map` 同口径）。
__device__ __forceinline__ void flex_quat_to_mat(const float* quat, float* mat) {
    float x = quat[0];
    float y = quat[1];
    float z = quat[2];
    float w = quat[3];
    float q00 = x * x;
    float q01 = x * y;
    float q02 = x * z;
    float q03 = x * w;
    float q11 = y * y;
    float q12 = y * z;
    float q13 = y * w;
    float q22 = z * z;
    float q23 = z * w;
    float q33 = w * w;
    mat[0] = q00 + q11 - q22 - q33;
    mat[1] = 2.0f * (q12 - q03);
    mat[2] = 2.0f * (q13 + q02);
    mat[3] = 2.0f * (q12 + q03);
    mat[4] = q00 - q11 + q22 - q33;
    mat[5] = 2.0f * (q23 - q01);
    mat[6] = 2.0f * (q13 - q02);
    mat[7] = 2.0f * (q23 + q01);
    mat[8] = q00 - q11 - q22 + q33;
}

// flex 通道的设备视图（81 槽中 9 个 flex 形参的裸指针视图；`xpos` 是 2-D 展平）。
//
// 注意：本文件在 `MEGAKERNEL_SOURCE` 里位于 `render_common.cu`（其 `#include "builtin.h"`
// 引入 `wp::` 命名空间）**之前**，所以这里只用 `scene.cu` 的 `V3` 与裸指针，不用 `wp::` 类型；
// `render_megakernel.cu` 负责把 `wp::array_t` 的 `.data` 转成这些裸指针。
struct FlexView {
    const int* vertadr;
    const int* edge;
    const float* radius;
    const float* xpos;
    const float* rgba;
    const int* geom_flexid;
    const int* geom_edgeid;
    // R06-S3c：2D/3D mesh 条目的 blob 绝对条目基址（`u64::MAX` = 该 flex 走 1D 胶囊）。
    const unsigned long long* bvh_id;
    int nflexvert;
};

// 单个 flex 图元叶的求交（上游 `render.py` L762-800 的 flex 分支）。
//
// `flex_local` 是该叶在 `bvh_nflexgeom` 里的局部号（scene 叶的 `leaf_geom = -1 - flex_local`）。
// 命中返回 true 并填 `out_dist`/`out_normal`/`out_mesh_id`（= flexid，着色与 seg 用）；
// miss / 透明（`rgba[3] == 0`）返回 false。2D/3D mesh 图元（`edgeid < 0`）由
// `render_common.cu::flex_mesh_leaf_hit` 走三角 BVH 路径（本函数只服务 1D 胶囊叶）。
__device__ __forceinline__ bool flex_leaf_hit(const FlexView& flex, int worldid, int flex_local,
                                              V3 origin, V3 direction, float* out_dist,
                                              V3* out_normal, int* out_mesh_id) {
    int flexid = flex.geom_flexid[flex_local];
    if (flexid < 0) {
        return false;
    }
    // 上游 L766-767：透明 flex（alpha == 0）跳过（胶囊与 BVH 两路都跳）。
    if (flex.rgba[4 * flexid + 3] == 0.0f) {
        return false;
    }
    int edge_id = flex.geom_edgeid[flex_local];
    if (edge_id < 0) {
        return false;
    }
    int vert_adr = flex.vertadr[flexid];
    int local0 = flex.edge[2 * edge_id];
    int local1 = flex.edge[2 * edge_id + 1];
    const float* verts = flex.xpos + (long)worldid * (long)flex.nflexvert * 3;
    const float* first = verts + 3 * (long)(vert_adr + local0);
    const float* second = verts + 3 * (long)(vert_adr + local1);
    V3 v0 = v3(first[0], first[1], first[2]);
    V3 v1 = v3(second[0], second[1], second[2]);
    // 上游 L775-781：胶囊中心/轴向/长度/半径（`0.5 * (v0 + v1)` 与 `0.5 * length`）。
    V3 pos = scale(add(v0, v1), 0.5f);
    V3 vec = sub(v1, v0);
    float length = warp_length3(vec);
    float quat[4];
    flex_quat_z2vec(vec, quat);
    float mat[9];
    flex_quat_to_mat(quat, mat);
    float size[3] = {flex.radius[flexid], 0.5f * length, 0.0f};
    float dist = -1.0f;
    V3 normal = v3(0.0f, 0.0f, 0.0f);
    ray_capsule(pos, mat, size, origin, direction, &dist, &normal);
    if (dist < 0.0f) {
        return false;
    }
    *out_dist = dist;
    *out_normal = normal;
    *out_mesh_id = flexid;
    return true;
}
// S7b2 全签名渲染内核的公共部分（渲染/BVH S7b2）。
//
// 本文件被 `src/render/kernel/mod.rs::MEGAKERNEL_SOURCE` 用 `include_str!` 接在
// S2b `scene.cu` 与 `cuda/render_math.cu` 之后、`cuda/render_shade.cu` /
// `cuda/render_megakernel.cu` 之前。因此可以直接复用：
//   - `scene.cu`：`V3` 辅助、`ray_quad`/primitive 求交、`mat_vec`/`mat_t_vec`/`ray_map`；
//   - `render_math.cu`：mesh woop 数学、`RAY_TOL_REL`/`RAY_TOL_ABS`、球面运算顺序对齐实现
//     （`render_sphere_hit` / `render_intersect_primitive`）。
//
// 与 S7b1 `render_depth_seg` 的差异只在“数据怎么来”：S7b2 用 81 槽上游签名，自有 BVH
// 打包通过 `bvh_id`（u64 标量）以设备指针传入（见 `MegakernelBlob`），不消费 Warp BVH
// 句柄；其余语义与 S7b1 一致（S7b1 的 depth/seg 位等是 S7b2 的回归门）。

#define WP_NO_CRT
#define WP_NO_BFLOAT16
#include "builtin.h"

#define RENDER_DEG_TO_RAD 0.017453292519943295f
#define MJ_MAXVAL 1.0e10f

// warp vec.h normalize(vec3)：组件平方和按 fma(z,z,fma(y,y,x*x)) 收缩（S5 已验证位等）。
__device__ __forceinline__ V3 normalize3_rays(V3 a) {
    float length = sqrtf(fmaf(a.z, a.z, fmaf(a.y, a.y, a.x * a.x)));
    if (length > 0.0f) {
        return v3(a.x / length, a.y / length, a.z / length);
    }
    return v3(0.0f, 0.0f, 0.0f);
}

// 上游 `vec.h:1113-1115` 的 normalize：平方和中**中间项单独舍入**（mul y*y → fma x*x → fma z*z）。
// G-L2 残余像素的 PTX 次序 pin（P3）：仅用于 mesh 顶点法线一处；
// `normalize3_rays` 是 S5 已验路径，保持字面不变。
__device__ __forceinline__ V3 normalize3_midpref(V3 a) {
    float l = sqrtf(fmaf(a.z, a.z, fmaf(a.x, a.x, a.y * a.y)));
    if (l > 0.0f) {
        return v3(a.x / l, a.y / l, a.z / l);
    }
    return v3(0.0f, 0.0f, 0.0f);
}

// 上游 `wp.length(vec3)`：平方和中**中间项单独舍入**（mul dy*dy → fma dx*dx → fma dz*dz）。
// G-L2 残余像素的 PTX 次序 pin（P6）：仅用于点/spot 灯 L 的模；`scene.cu::length3` 其它调用方共用，保持不动。
__device__ __forceinline__ float length3_midpref(V3 a) {
    return sqrtf(__fmaf_rn(a.z, a.z, __fmaf_rn(a.x, a.x, __fmul_rn(a.y, a.y))));
}

// 上游 `wp.dot(normal, L)`（ndotl）的收缩次序是**首项单独舍入**：
// mul(nx*Lx) → fma(ny*Ly) → fma(nz*Lz)。G-L2 残余像素的 PTX 次序 pin（P5）：仅用于 ndotl；
// `dot3_ba` 的中间项单独舍入次序保持不变。
__device__ __forceinline__ float dot3_first(float a0, float a1, float a2, float b0, float b1,
                                            float b2) {
    return __fmaf_rn(a2, b2, __fmaf_rn(a1, b1, __fmul_rn(a0, b0)));
}

// `render_util.py::compute_ray` 的 f32 翻译（与 S5 `build_rays.cu` 逐字一致）。
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

// warp `wp::vec3` → 本编译单元的 `V3`（81 槽签名里的 vec3 数组是 warp 类型）。
__device__ __forceinline__ V3 to_v3(wp::vec3 v) { return v3(v[0], v[1], v[2]); }
// 自有 BVH/网格打包 blob（host `MegakernelArrays::pack_blob` 按同一布局写入）：
//   header(u32 × 10)：五段的字节偏移/字节长度对（scene_meta、scene_params、
//                     mesh_meta、mesh_params、mesh_offsets）
//   随后是各段数据（4 字节对齐）。`bvh_id` 标量即该 blob 的设备地址。
// 布局与 S7b1 `SceneDevice`/`MeshDevice` 的 meta/params 完全一致（见 `cast.rs`）。
struct MegakernelBlob {
    const int* scene_meta;
    const float* scene_params;
    const int* mesh_meta;
    const float* mesh_params;
    const wp::vec2i* mesh_offsets;
};

__device__ __forceinline__ MegakernelBlob megakernel_blob(wp::uint64 bvh_id) {
    const unsigned int* header = (const unsigned int*)(size_t)bvh_id;
    const unsigned char* base = (const unsigned char*)bvh_id;
    MegakernelBlob blob;
    blob.scene_meta = (const int*)(base + header[0]);
    blob.scene_params = (const float*)(base + header[2]);
    blob.mesh_meta = (const int*)(base + header[4]);
    blob.mesh_params = (const float*)(base + header[6]);
    blob.mesh_offsets = (const wp::vec2i*)(base + header[8]);
    return blob;
}

// 场景 BVH 设备视图：与 S7b1 `render.cu::SceneView` 同布局。
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

// 网格三角 BVH 设备布局（`render_math.cu::mesh_query_ray` 消费）：
//   meta   = [n_nodes, n_tri, n_vert, root, links(2*n_nodes), order(n_tri), indices(3*n_tri)]
//   params = [node bounds 6*n_nodes, vertices 3*n_vert]
__device__ __forceinline__ float ray_mesh_leaf(const int* mesh_meta, const float* mesh_params,
                                               const wp::vec2i* mesh_offsets, int mesh_index, V3 pos,
                                               const float* mat, V3 pnt, V3 vec, float max_t,
                                               bool cull_backfaces, V3* normal, float* bary_u,
                                               float* bary_v, int* face_out) {
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
            *bary_u = tu;
            *bary_v = tv;
            *face_out = face;
            break;
        }
        offset += t * (1.0f + RAY_TOL_REL) + RAY_TOL_ABS;
    }
    return distance;
}

// R06-S3c：2D/3D flex mesh 叶求交（上游 `ray.py` L759-802 `ray_flex_with_bvh` 语义）。
//
// flex mesh 已在世界空间：无 `_ray_map`、无多段 march、无内部背面剔除（统一在 `cast_ray`
// 后判据处理）。每个 `(flex, world)` 一棵三角 BVH，`flex.bvh_id[flexid]` 是该 flex 首条目的
// blob 绝对条目号（`u64::MAX` = 1D/无 mesh → miss），条目索引 = `bvh_id + worldid`。
__device__ __forceinline__ float flex_mesh_leaf_hit(const FlexView& flex, int worldid, int flexid,
                                                    const int* mesh_meta,
                                                    const float* mesh_params,
                                                    const wp::vec2i* mesh_offsets, V3 pnt, V3 vec,
                                                    float max_t, V3* normal, float* bary_u,
                                                    float* bary_v, int* face_out) {
    unsigned long long base = flex.bvh_id[flexid];
    if (base == 0xFFFFFFFFFFFFFFFFull) {
        return -1.0f;
    }
    int mesh_index = (int)(base + (unsigned long long)worldid);
    const float identity[9] = {1.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 1.0f};
    return ray_mesh_leaf(mesh_meta, mesh_params, mesh_offsets, mesh_index, v3(0.0f, 0.0f, 0.0f),
                         identity, pnt, vec, max_t, false, normal, bary_u, bary_v, face_out);
}

// 上游 `_geom_rgba`：geom_rgba 为默认 (0.5,0.5,0.5,1) 且 matid>=0 时改用 mat_rgba。
__device__ __forceinline__ void render_geom_rgba(const int* geom_matid, const float* geom_rgba,
                                          const float* mat_rgba, int geom_id, float* rgba,
                                          int* mat_id) {
    float r = geom_rgba[4 * geom_id + 0];
    float g = geom_rgba[4 * geom_id + 1];
    float b = geom_rgba[4 * geom_id + 2];
    float a = geom_rgba[4 * geom_id + 3];
    int id = geom_matid[geom_id];
    *mat_id = id;
    if (id >= 0 && r == 0.5f && g == 0.5f && b == 0.5f && a == 1.0f) {
        rgba[0] = mat_rgba[4 * id + 0];
        rgba[1] = mat_rgba[4 * id + 1];
        rgba[2] = mat_rgba[4 * id + 2];
        rgba[3] = mat_rgba[4 * id + 3];
        return;
    }
    rgba[0] = r;
    rgba[1] = g;
    rgba[2] = b;
    rgba[3] = a;
}

__device__ __forceinline__ float geom_alpha(const int* geom_matid, const float* geom_rgba,
                                            const float* mat_rgba, int geom_id) {
    float rgba[4];
    int mat_id = -1;
    render_geom_rgba(geom_matid, geom_rgba, mat_rgba, geom_id, rgba, &mat_id);
    return rgba[3];
}

// 候选节点最小堆：按 AABB 进入距离升序给出候选叶子（与 S7b1 `render.cu` 同语义）。
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

// 上游 `cast_ray`（closest-hit）的 S7b2 子集：候选 leaf 按进入距离升序；透明 geom 跳过；
// primitive 与 mesh 同池；背面剔除后按 `RAY_TOL_REL` 近局取更大 geom id。
// 返回 geom_id/dist/normal/face/bary（mesh 贴图需要）。
struct CastHit {
    int geom_id;
    float dist;
    V3 normal;
    int face;
    float bary_u;
    float bary_v;
    int mesh_id;
};

// `shadow_ray`：阴影射线按上游 `cast_ray_first_hit`（any_hit 分支）语义处理背面剔除——
// mesh 命中在 any-hit 下 `n` 为零向量（上游不填 surface data）→ cull 判据不生效；
// primitive 命中仍按 cull 剔除。primary ray 传 false，行为与 S7b1/S7b2 完全一致。
__device__ __forceinline__ CastHit cast_ray(const SceneView& scene, const MegakernelBlob& blob,
                                            const FlexView& flex, const int* geom_type,
                                            const int* geom_dataid, const int* geom_matid,
                                            const float* geom_rgba, const float* mat_rgba,
                                            const wp::uint64* hfield_bvh_id, int worldid, V3 origin,
                                            V3 direction, float max_dist, bool cull_backfaces,
                                            bool shadow_ray) {
    CastHit hit;
    hit.geom_id = -1;
    hit.dist = max_dist;
    hit.normal = v3(0.0f, 0.0f, 0.0f);
    hit.face = -1;
    hit.bary_u = 0.0f;
    hit.bary_v = 0.0f;
    hit.mesh_id = -1;
    float initial_bound = hit.dist * (1.0f + RAY_TOL_REL) + RAY_TOL_ABS;
    int root = scene.roots[worldid];
    NodeCandidate heap[MAX_CANDIDATES];
    int count = 0;
    float entry = 0.0f;
    if (!ray_aabb(origin, direction, scene.node_bounds + 6 * root, initial_bound, &entry)) {
        return hit;
    }
    push_candidate(heap, &count, root, entry);
    while (count > 0) {
        NodeCandidate candidate = pop_candidate(heap, &count);
        float bound = hit.dist * (1.0f + RAY_TOL_REL) + RAY_TOL_ABS;
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
        int leaf_geom = scene.leaf_geom[leaf];
        int geom = leaf_geom;
        // 负 `leaf_geom` = flex 叶；按上游 `GeomType.FLEX`（mjGEOM_FLEX = 9）走 flex 分派。
        int gtype = (leaf_geom < 0) ? 9 : geom_type[geom];
        // 上游 tie-break 的 `hit_geom_id`：flex 命中是 -2（render.py L764），geom 命中是 geom id。
        int candidate_id = (leaf_geom < 0) ? -2 : geom;
        int flexid = -1;
        if (leaf_geom >= 0 && geom_alpha(geom_matid, geom_rgba, mat_rgba, geom) == 0.0f) {
            continue;
        }
        const float* leaf_size = scene.leaf_size + 3 * leaf;
        const float* leaf_pos = scene.leaf_xpos + 3 * leaf;
        const float* leaf_mat = scene.leaf_xmat + 9 * leaf;
        float query_dist = hit.dist * (1.0f + RAY_TOL_REL) + RAY_TOL_ABS;
        float d = -1.0f;
        V3 n = v3(0.0f, 0.0f, 0.0f);
        V3 pos = v3(leaf_pos[0], leaf_pos[1], leaf_pos[2]);
        float pass_bary_u = 0.0f;
        float pass_bary_v = 0.0f;
        int pass_face = -1;
        int mesh_index = scene.leaf_mesh[leaf];
        // R06-S3c：2D/3D mesh 路径在 shadow any-hit 下按上游「n 为零向量」语义不参与背面剔除。
        bool flex_mesh_path = false;
        if (leaf_geom < 0) {
            // R06-S3a/S3c：flex 叶（上游 render.py L762-800）。1D 胶囊有真实法向；2D/3D mesh
            // 走世界空间三角 BVH（`flex_bvh_id` 条目基址）。`candidate = -2` 与上游一致。
            int flex_local = -1 - leaf_geom;
            int flex_id = flex.geom_flexid[flex_local];
            if (flex_id < 0 || flex.rgba[4 * flex_id + 3] == 0.0f) {
                continue;
            }
            if (flex.geom_edgeid[flex_local] >= 0) {
                if (!flex_leaf_hit(flex, worldid, flex_local, origin, direction, &d, &n, &flexid)) {
                    continue;
                }
            } else {
                flex_mesh_path = true;
                d = flex_mesh_leaf_hit(flex, worldid, flex_id, blob.mesh_meta, blob.mesh_params,
                                       blob.mesh_offsets, origin, direction, query_dist, &n,
                                       &pass_bary_u, &pass_bary_v, &pass_face);
                flexid = flex_id;
                if (d < 0.0f) {
                    continue;
                }
            }
        } else if (gtype == 7 || gtype == 1 || gtype == 8) {
            // R05-S2：hfield（gtype==1）与 mesh 同走 `ray_mesh_leaf`；hfield 的 mesh 索引
            // 从 `hfield_bvh_id[geom_dataid[geom]]` 取（blob 内索引，追加在既有 mesh 末尾；
            // `u64::MAX` 哨兵=未建/未用 → 跳过）。上游 `render.py` L682-693 同义。
            // SDF（gtype==8）与 mesh 同路：上游按底层 mesh 渲染（render.py L738-761、
            // render_test.py L1251 原文）；8 直接使用 `scene.leaf_mesh[leaf]`（= geom_dataid）。
            int query_mesh = mesh_index;
            if (gtype == 1) {
                int hfield_index = geom_dataid[geom];
                query_mesh = -1;
                if (hfield_index >= 0) {
                    wp::uint64 id = hfield_bvh_id[hfield_index];
                    if (id != 0xFFFFFFFFFFFFFFFFull) {
                        query_mesh = (int)id;
                    }
                }
            }
            if (query_mesh >= 0) {
                d = ray_mesh_leaf(blob.mesh_meta, blob.mesh_params, blob.mesh_offsets, query_mesh,
                                  pos, leaf_mat, origin, direction, query_dist,
                                  cull_backfaces && !shadow_ray, &n,
                                  &pass_bary_u, &pass_bary_v, &pass_face);
            }
        } else if (gtype != 8) {
            // gtype==8 已进上一分支（上游 SDF=底层 mesh 渲染）；此处条件保留为防御：
            // host 仍拒绝 8（`megakernel.rs` 的 `reject_sdf_geometry`），本片改动今天不可达。
            render_intersect_primitive(gtype, pos, leaf_size, leaf_mat, origin, direction, &d, &n);
        }
        // 上游阴影射线走 any-hit 分支（mesh/hfield/SDF 的 n 为零向量 → cull 不生效）；
        // primitive 仍 cull。SDF（gtype==8）按上游 any-hit 语义也不填 normal ⇒ 排除
        // （latent 对齐，见 gap 卡 `2026-10-28-render-shadow-sdf-gtype-cull-gap`；
        // 设备侧 SDF 求交尚未解锁，host 仍拒绝，见 `megakernel.rs` 的 `reject_sdf_geometry`）。
        bool cull_this =
            cull_backfaces && !(shadow_ray && (gtype == 7 || gtype == 1 || gtype == 8 ||
                                               flex_mesh_path));
        if (cull_this && d >= 0.0f && dot(direction, n) > 0.0f) {
            d = -1.0f;
        }
        float tol = RAY_TOL_REL * (hit.dist > 1.0f ? hit.dist : 1.0f);
        bool closer = (d < hit.dist - tol) ||
                      (fabsf(d - hit.dist) <= tol && (hit.geom_id < 0 || candidate_id > hit.geom_id));
        if (d >= 0.0f && closer) {
            hit.dist = d;
            hit.geom_id = candidate_id;
            hit.normal = n;
            hit.face = pass_face;
            hit.bary_u = pass_bary_u;
            hit.bary_v = pass_bary_v;
            if (leaf_geom < 0) {
                hit.mesh_id = flexid;
            } else {
                hit.mesh_id = (gtype == 7 || gtype == 8) ? mesh_index : -1;
            }
        }
    }
    return hit;
}
// R09-S2 阴影射线可见度（渲染/BVH R09；方案 A：复用 first-hit `cast_ray`）。
//
// 上游锚点（冻结 MJWarp 3.15 `render.py`，`71da24d`）：
// - L919-961：仅当 `use_shadows and lightcastshadow` 才发射阴影射线；
//   `shadow_origin = hitpoint + normal * 1.0e-4`（自相交规避）；
//   非方向光 `max_t = dist_to_light - 1.0e-3`，方向光 `max_t = 1.0e8`；
//   命中（`geom_id != -1`）时 `visible = shadow_light_fraction`，否则 1.0。
// - L1457-1459：headlight 调用传 `lightcastshadow=False`（headlight 不投影）。
//
// 实现路径：复用 first-hit `cast_ray`（非 any-hit）；遮挡判定目标与上游 `max_t` 内任意命中
// 一致，但实现路径不同；本片不作任何数值结论，数值验收在 S3b 窗。
//
// 依赖拼接顺序：`render_common.cu`（`SceneView`/`MegakernelBlob`/`CastHit`/`cast_ray`）在前。

__device__ __forceinline__ float shadow_visibility(
    const SceneView& scene, const MegakernelBlob& blob, const FlexView& flex, const int* geom_type,
    const int* geom_dataid, const int* geom_matid, const float* geom_rgba, const float* mat_rgba,
    const wp::uint64* hfield_bvh_id, int worldid, bool cull_backfaces, V3 normal, V3 hitpoint,
    V3 to_light, int lighttype, float dist_to_light, bool use_shadows, bool cast_shadow,
    float shadow_light_fraction) {
    if (!use_shadows || !cast_shadow) {
        return 1.0f;
    }
    // 上游 L922：沿法线微移，避免阴影射线自相交。
    V3 shadow_origin = add(hitpoint, scale(normal, 1.0e-4f));
    // 上游 L924-926：非方向光距离上限为到光距离；方向光按 1e8 近似无穷。
    float max_t = (lighttype == 1) ? 1.0e8f : (dist_to_light - 1.0e-3f);
    CastHit hit = cast_ray(scene, blob, flex, geom_type, geom_dataid, geom_matid, geom_rgba,
                           mat_rgba, hfield_bvh_id, worldid, shadow_origin, to_light, max_t,
                           cull_backfaces, true);
    return (hit.geom_id != -1) ? shadow_light_fraction : 1.0f;
}
// S7b2 着色阶段（渲染/BVH S7b2）：逐式翻译冻结上游 `render.py` 的着色/纹理/光照子集。
//
// 上游对应代码：`_geom_rgba` / `_texture_*` / `_make_sample_texture` / `vertex_normal`
// / `_make_compute_lighting`（7 个 primitive 分支里本片实际用到的 5 类）+ `_render_megakernel`
// 的 shading 段（headlight 三组、环境光、逐灯漫反射/高光、自发光、min/max、pack_rgba）。
//
// 精度对齐方法沿用 S7b1：参考 megakernel 由 warp 1.15 自带 NVRTC（CUDA 12.9、sm_89）编译，
// 产品是 CUDA 12.8.1 / compute_70，同一 C++ 表达式的 fma 收缩顺序可能不同。因此涉及
// dot/normalize/atan2/asin/pow 的关键表达式用显式 `__fmul_rn`/`__fmaf_rn`/`__fsub_rn`
// 固定上游顺序（点积见 `dot3_ba`，normalize 见 `normalize3_rays`），其余交给
// 8 位量化吸收；对比口径是「解包后每通道 ≤1 LSB」，不做任何容差放宽。
//
// 本文件被 `mod.rs::MEGAKERNEL_SOURCE` 接在 `render_common.cu` 之后、`render_megakernel.cu`
// 之前，因此可用 `V3` 辅助、`normalize3_rays`、`dot3_ba`、`geom_rgba`。

__device__ __forceinline__ float safe_div_render(float x, float y) {
    return x / (y != 0.0f ? y : 1.0e-15f);
}

// 上游 `_texture_plane`。
__device__ __forceinline__ void texture_plane(V3 local, const float* size, const float* repeat,
                                              bool tex_uniform, V3* uv, V3* offset, V3* rep) {
    float u = 0.5f * local.x;
    float rep_u = repeat[0];
    if (size[0] > 0.0f) {
        float s0 = fmaxf(size[0], 1.0e-6f);
        u = (local.x + size[0]) / (2.0f * s0);
        if (tex_uniform) {
            rep_u = repeat[0] * size[0];
        }
    }
    float v = -0.5f * local.y;
    float rep_v = repeat[1];
    if (size[1] > 0.0f) {
        float s1 = fmaxf(size[1], 1.0e-6f);
        v = (size[1] - local.y) / (2.0f * s1);
        if (tex_uniform) {
            rep_v = repeat[1] * size[1];
        }
    }
    float off = (size[0] <= 0.0f || size[1] <= 0.0f) ? -0.5f : 0.0f;
    *uv = v3(u, v, 0.0f);
    *offset = v3(off, off, 0.0f);
    *rep = v3(rep_u, rep_v, 0.0f);
}

// 上游 `_texture_sphere_ellipsoid`。
__device__ __forceinline__ void texture_sphere_ellipsoid(V3 local, const float* size,
                                                          const float* repeat, bool tex_uniform,
                                                          bool is_sphere, V3* uv, V3* offset,
                                                          V3* rep) {
    float rx = fmaxf(is_sphere ? size[0] : size[0], 1.0e-6f);
    float ry = fmaxf(is_sphere ? size[0] : size[1], 1.0e-6f);
    float rz = fmaxf(is_sphere ? size[0] : size[2], 1.0e-6f);
    V3 pt = v3(local.x / rx, local.y / ry, local.z / rz);
    float inv_len = 1.0f / fmaxf(length3(pt), 1.0e-6f);
    V3 d = scale(pt, inv_len);
    float azimuth = atan2f(d.y, d.x);
    if (azimuth < 0.0f) {
        azimuth += 6.283185307179586f;
    }
    float elevation = asinf(fminf(fmaxf(d.z, -1.0f), 1.0f));
    float u = azimuth * (0.5f / 3.141592653589793f);
    float v = 0.5f - elevation * (1.0f / 3.141592653589793f);
    float rep_u = repeat[0];
    float rep_v = repeat[1];
    if (tex_uniform) {
        rep_u = repeat[0] * rx;
        rep_v = repeat[1] * ry;
    }
    *uv = v3(u, v, 0.0f);
    *offset = v3(0.0f, 0.0f, 0.0f);
    *rep = v3(rep_u, rep_v, 0.0f);
}

// 上游 `_texture_cylinder`。
__device__ __forceinline__ void texture_cylinder(V3 local, V3 local_normal, const float* size,
                                                  const float* repeat, bool tex_uniform, V3* uv,
                                                  V3* offset, V3* rep) {
    float r = fmaxf(size[0], 1.0e-6f);
    float h = fmaxf(size[1], 1.0e-6f);
    float u;
    float v;
    if (fabsf(local_normal.z) > 0.7071f) {
        u = 0.5f + 0.5f * (local.x / r);
        v = 0.5f + 0.5f * (local.y / r);
    } else {
        float azimuth = atan2f(local.y, local.x);
        if (azimuth < 0.0f) {
            azimuth += 6.283185307179586f;
        }
        u = azimuth * (0.5f / 3.141592653589793f);
        v = (h - local.z) / (2.0f * h);
    }
    float rep_u = repeat[0];
    float rep_v = repeat[1];
    if (tex_uniform) {
        rep_u = repeat[0] * size[0];
        rep_v = repeat[1] * size[0];
    }
    *uv = v3(u, v, 0.0f);
    *offset = v3(0.0f, 0.0f, 0.0f);
    *rep = v3(rep_u, rep_v, 0.0f);
}

// 上游 `_texture_capsule`。
__device__ __forceinline__ void texture_capsule(V3 local, const float* size, const float* repeat,
                                                 bool tex_uniform, V3* uv, V3* offset, V3* rep) {
    float r = fmaxf(size[0], 1.0e-6f);
    float h = fmaxf(size[1], 1.0e-6f);
    float z = local.z;
    float azimuth = atan2f(local.y, local.x);
    if (azimuth < 0.0f) {
        azimuth += 6.283185307179586f;
    }
    float u = azimuth * (0.5f / 3.141592653589793f);
    float v;
    if (z > h) {
        float dz = fminf(fmaxf((z - h) / r, 0.0f), 1.0f);
        v = 1.0f - asinf(dz) * (2.0f / 3.141592653589793f);
    } else if (z < -h) {
        float dz = fminf(fmaxf((-h - z) / r, 0.0f), 1.0f);
        v = asinf(dz) * (2.0f / 3.141592653589793f);
    } else {
        v = (h - z) / (2.0f * h);
    }
    float rep_u = repeat[0];
    float rep_v = repeat[1];
    if (tex_uniform) {
        rep_u = repeat[0] * size[0];
        rep_v = repeat[1] * size[0];
    }
    *uv = v3(u, v, 0.0f);
    *offset = v3(0.0f, 0.0f, 0.0f);
    *rep = v3(rep_u, rep_v, 0.0f);
}

// 上游 `_texture_box`。
__device__ __forceinline__ void texture_box(V3 local, V3 local_normal, const float* size,
                                             const float* repeat, bool tex_uniform, V3* uv,
                                             V3* offset, V3* rep) {
    float sx = fmaxf(size[0], 1.0e-6f);
    float sy = fmaxf(size[1], 1.0e-6f);
    float sz = fmaxf(size[2], 1.0e-6f);
    float ax = fabsf(local_normal.x);
    float ay = fabsf(local_normal.y);
    float az = fabsf(local_normal.z);
    float u;
    float v;
    if (az >= ax && az >= ay) {
        if (local_normal.z > 0.0f) {
            u = (local.x + sx) / (2.0f * sx);
            v = (sy - local.y) / (2.0f * sy);
        } else {
            u = (local.x + sx) / (2.0f * sx);
            v = (local.y + sy) / (2.0f * sy);
        }
    } else if (ax >= ay && ax >= az) {
        if (local_normal.x > 0.0f) {
            u = (local.y + sy) / (2.0f * sy);
            v = (sz - local.z) / (2.0f * sz);
        } else {
            u = (sy - local.y) / (2.0f * sy);
            v = (sz - local.z) / (2.0f * sz);
        }
    } else if (local_normal.y < 0.0f) {
        u = (local.x + sx) / (2.0f * sx);
        v = (sz - local.z) / (2.0f * sz);
    } else {
        u = (sx - local.x) / (2.0f * sx);
        v = (sz - local.z) / (2.0f * sz);
    }
    float rep_u = repeat[0];
    float rep_v = repeat[1];
    if (tex_uniform) {
        rep_u = repeat[0] * size[0];
        rep_v = repeat[1] * size[1];
    }
    *uv = v3(u, v, 0.0f);
    *offset = v3(0.0f, 0.0f, 0.0f);
    *rep = v3(rep_u, rep_v, 0.0f);
}

// 上游 `_texture_hfield`（render.py L332-350）：水平范围 `[-size[0], size[0]] x [-size[1], size[1]]`，
// P10 诊断 helper 的前向声明（定义在 `render_megakernel.cu` 的 DIAG 段；仅 opt-in 时被调用）。
__device__ __forceinline__ void diag_store_tex(float* d, float u, float v, V3 tex_color);
__device__ __forceinline__ void diag_store_light(float* d, float weight, float visible, float ndotl);
__device__ __forceinline__ void diag_store_light_spec(float* d, float ndoth, V3 half, float spec_pow);

// `s0 = max(size[0], 1e-6)`、`s1 = max(size[1], 1e-6)`；UV 不依赖命中面/法线。
__device__ __forceinline__ void texture_hfield(V3 local, const float* size, const float* repeat,
                                                bool tex_uniform, V3* uv, V3* offset, V3* rep) {
    float s0 = fmaxf(size[0], 1.0e-6f);
    float s1 = fmaxf(size[1], 1.0e-6f);
    float u = (local.x + size[0]) / (2.0f * s0);
    float v = (size[1] - local.y) / (2.0f * s1);
    float rep_u = repeat[0];
    float rep_v = repeat[1];
    if (tex_uniform) {
        rep_u = repeat[0] * size[0];
        rep_v = repeat[1] * size[1];
    }
    *uv = v3(u, v, 0.0f);
    *offset = v3(0.0f, 0.0f, 0.0f);
    *rep = v3(rep_u, rep_v, 0.0f);
}

// 上游 `_texture_mesh`（s7b 场景只有 1 个 mesh 且无 texcoord：走 OBJECT_PLANE fallback）。
__device__ __forceinline__ void texture_mesh(const int* mesh_faceadr, V3 local, const float* size,
                                              const float* repeat, bool tex_uniform, int face,
                                              int mesh_id, float bary_u, float bary_v,
                                              const int* mesh_facetexcoord, const float* mesh_texcoord,
                                              const int* mesh_texcoord_offsets, V3* uv, V3* offset,
                                              V3* rep) {
    bool has_uv = false;
    *uv = v3(0.0f, 0.0f, 0.0f);
    *offset = v3(0.0f, 0.0f, 0.0f);
    *rep = v3(repeat[0], repeat[1], 0.0f);
    if (face >= 0 && mesh_id >= 0) {
        int texcoord_offset = mesh_texcoord_offsets[mesh_id];
        if (texcoord_offset >= 0) {
            int face_adr = mesh_faceadr[mesh_id] + face;
            const int* coords = mesh_facetexcoord + 3 * face_adr;
            V3 uv0 = v3(mesh_texcoord[2 * (texcoord_offset + coords[0]) + 0],
                        mesh_texcoord[2 * (texcoord_offset + coords[0]) + 1], 0.0f);
            V3 uv1 = v3(mesh_texcoord[2 * (texcoord_offset + coords[1]) + 0],
                        mesh_texcoord[2 * (texcoord_offset + coords[1]) + 1], 0.0f);
            V3 uv2 = v3(mesh_texcoord[2 * (texcoord_offset + coords[2]) + 0],
                        mesh_texcoord[2 * (texcoord_offset + coords[2]) + 1], 0.0f);
            float w0 = bary_u;
            float w1 = bary_v;
            float w2 = 1.0f - bary_u - bary_v;
            // P7：对齐参考核 `vec.h` 的 mul→fma→fma 收缩——单独舍入项是**中间项** `uv1*w1`；
            // 产品原为 `fmaf(uv0,w0, fmaf(uv1,w1, uv2*w2))`（单独舍入第三项 `uv2*w2`），会差 1 ULP。
            float u = __fmaf_rn(uv2.x, w2, __fmaf_rn(uv0.x, w0, __fmul_rn(uv1.x, w1)));
            float v = __fmaf_rn(uv2.y, w2, __fmaf_rn(uv0.y, w0, __fmul_rn(uv1.y, w1)));
            *uv = v3(u, v, 0.0f);
            has_uv = true;
        }
    }
    if (!has_uv) {
        *uv = v3(0.5f * local.x, -0.5f * local.y, 0.0f);
        *offset = v3(-0.5f, -0.5f, 0.0f);
        float rep_u = repeat[0];
        float rep_v = repeat[1];
        if (size[0] > 0.0f) {
            rep_u = rep_u / size[0];
        }
        if (size[1] > 0.0f) {
            rep_v = rep_v / size[1];
        }
        if (tex_uniform) {
            if (size[0] > 0.0f) {
                rep_u = rep_u * size[0];
            }
            if (size[1] > 0.0f) {
                rep_v = rep_v * size[1];
            }
        }
        *rep = v3(rep_u, rep_v, 0.0f);
    }
}

// 上游 `sample_texture`：按 geom 类型给出 UV，wrap 后采样。
__device__ __forceinline__ V3 sample_texture(const int* geom_type, const int* mesh_faceadr,
                                              int geom_id, const float* size, const float* repeat,
                                              bool tex_uniform, wp::texture2d_t tex,
                                              V3 pos, const float* rot, const int* mesh_facetexcoord,
                                              const float* mesh_texcoord,
                                              const int* mesh_texcoord_offsets, V3 hit_point,
                                              V3 normal, float bary_u, float bary_v, int face,
                                              int mesh_id, float* diag_tex) {
    V3 uv = v3(0.0f, 0.0f, 0.0f);
    V3 offset = v3(0.0f, 0.0f, 0.0f);
    V3 rep = v3(repeat[0], repeat[1], 0.0f);
    int gtype = geom_type[geom_id];
    V3 local = mat_t_vec(rot, sub(hit_point, pos));
    V3 local_normal = mat_t_vec(rot, normal);
    if (gtype == 0) {
        texture_plane(local, size, repeat, tex_uniform, &uv, &offset, &rep);
    } else if (gtype == 2 || gtype == 4) {
        texture_sphere_ellipsoid(local, size, repeat, tex_uniform, gtype == 2, &uv, &offset, &rep);
    } else if (gtype == 5) {
        texture_cylinder(local, local_normal, size, repeat, tex_uniform, &uv, &offset, &rep);
    } else if (gtype == 3) {
        texture_capsule(local, size, repeat, tex_uniform, &uv, &offset, &rep);
    } else if (gtype == 6) {
        texture_box(local, local_normal, size, repeat, tex_uniform, &uv, &offset, &rep);
    } else if (gtype == 1) {
        // R05-S2：hfield 纹理（上游 render.py L462-464 分支）。
        texture_hfield(local, size, repeat, tex_uniform, &uv, &offset, &rep);
    } else if (gtype == 7 || gtype == 8) {
        texture_mesh(mesh_faceadr, local, size, repeat, tex_uniform, face, mesh_id, bary_u, bary_v,
                     mesh_facetexcoord, mesh_texcoord, mesh_texcoord_offsets, &uv, &offset, &rep);
    }
    float u = uv.x * rep.x + offset.x;
    float v = uv.y * rep.y + offset.y;
    u = u - floorf(u);
    v = v - floorf(v);
    wp::vec4 tex_color = wp::texture_sample<wp::vec4>(tex, wp::vec2(u, v), 0.0f);
    diag_store_tex(diag_tex, u, v, v3(tex_color[0], tex_color[1], tex_color[2]));
    return v3(tex_color[0], tex_color[1], tex_color[2]);
}

// R02-S1：上游 `sample_skybox`（`render.py` L494-556）。
//
// 语义：天空盒在 MuJoCo 里是**竖排 2D** 纹理（`tex_height = 6*tex_width`，face 顺序
// +X,-X,+Y,-Y,+Z,-Z；上游 `render_util.py` L514-535 只取 `tex_width` 作 face 宽度、
// 用普通 `wp.Texture2D` 上传，L554 用 `v=(face+t)/6` 落到竖排）。世界方向先按 GL3
// 约定旋转（S=x, T=z, R=-y），按最大轴选 face，再按该轴归一化出 sc/tc：
//   face 0 (+X): sc=-rz, tc=-ry    face 1 (-X): sc= rz, tc=-ry
//   face 2 (+Y): sc= rx, tc= rz    face 3 (-Y): sc= rx, tc=-rz
//   face 4 (+Z): sc= rx, tc=-ry    face 5 (-Z): sc=-rx, tc=-ry
// `s/t=(safe_div(x,ma)+1)*0.5`，`t` 夹在 `[0.5*face_width_inv, 1-0.5*face_width_inv]`，
// 防止竖排相邻面在 LINEAR 过滤下串色（上游 L550-552 注释同义）。
__device__ __forceinline__ V3 sample_skybox(wp::texture2d_t tex, float face_width_inv,
                                            V3 ray_dir_world) {
    // L503-505：GL3 依据（render_gl3.c: S=x, T=z, R=-y）。
    float rx = ray_dir_world.x;
    float ry = ray_dir_world.z;
    float rz = -ray_dir_world.y;
    float arx = fabsf(rx);
    float ary = fabsf(ry);
    float arz = fabsf(rz);
    // L511-545：face / sc / tc / ma。
    int face = 0;
    float sc = 0.0f;
    float tc = 0.0f;
    float ma = 1.0f;
    if (arx >= ary && arx >= arz) {
        ma = arx;
        if (rx > 0.0f) {
            face = 0;
            sc = -rz;
            tc = -ry;
        } else {
            face = 1;
            sc = rz;
            tc = -ry;
        }
    } else if (ary >= arz) {
        ma = ary;
        if (ry > 0.0f) {
            face = 2;
            sc = rx;
            tc = rz;
        } else {
            face = 3;
            sc = rx;
            tc = -rz;
        }
    } else {
        ma = arz;
        if (rz > 0.0f) {
            face = 4;
            sc = rx;
            tc = -ry;
        } else {
            face = 5;
            sc = -rx;
            tc = -ry;
        }
    }
    // L547-548：`math.safe_div` = `x / (y != 0 ? y : MJ_MINVAL)`（`math.py` L318-319，
    // MJ_MINVAL = mjMINVAL = 1e-15）；本文件 `safe_div_render` 同义。
    float s = (safe_div_render(sc, ma) + 1.0f) * 0.5f;
    float t = (safe_div_render(tc, ma) + 1.0f) * 0.5f;
    // L550-552：竖排相邻面防串色（半 texel）。
    float t_min = 0.5f * face_width_inv;
    t = fminf(fmaxf(t, t_min), 1.0f - t_min);
    // L554：`v = (float(face) + t) * wp.static(1.0/6.0)`；1/6 是 warp 编译期常量，
    // double `1.0/6.0` 舍入到 f32 后为 `0x3E2AAAAB`，与 `1.0f/6.0f` 逐位相同。
    float v = ((float)face + t) * (1.0f / 6.0f);
    // L555：`wp.texture_sample(skybox_tex, vec2(s, v), dtype=vec4)`。
    wp::vec4 tex_color = wp::texture_sample<wp::vec4>(tex, wp::vec2(s, v), 0.0f);
    return v3(tex_color[0], tex_color[1], tex_color[2]);
}

// 上游 `vertex_normal`：顶点法线与面法线夹角超过 ~37° 时用面法线。
__device__ __forceinline__ V3 vertex_normal(V3 n, V3 face) {
    if (dot3_ba(n.x, n.y, n.z, face.x, face.y, face.z) < 0.8f) {
        return face;
    }
    return n;
}

// 上游 `_make_compute_lighting`（本片：R09-S2 起支持 `use_shadows`；逐灯参数与 headlight
// 由调用方给）。`has_spot` 为场景属性：lighttype==0 时走聚光衰减（其余类型不受影响）。
// 阴影：仅 `use_shadows && lightcastshadow` 时经 `shadow_visibility`（R09）发射阴影射线
// （方案 A：复用 first-hit `cast_ray`；上游 L919-961 语义，实现路径不同）。
__device__ __forceinline__ void compute_lighting(const SceneView& scene, const MegakernelBlob& blob,
                                                  const FlexView& flex, const int* geom_type,
                                                  const int* geom_dataid, const int* geom_matid,
                                                  const float* geom_rgba, const float* mat_rgba,
                                                  const wp::uint64* hfield_bvh_id, int worldid,
                                                  bool cull_backfaces, bool lightactive,
                                                  int lighttype, V3 lightpos, V3 lightdir,
                                                  V3 lightattenuation, float lightcutoff_rad,
                                                  float lightexp, V3 lightdiff, V3 lightspec,
                                                  V3 normal, V3 hitpoint, V3 view_dir, float mat_spec,
                                                  float mat_shin_exp, bool enable_specular,
                                                  bool use_shadows, bool cast_shadow,
                                                  float shadow_light_fraction, V3* diff_rgb,
                                                  V3* spec_rgb, float* diag_light) {
    *diff_rgb = v3(0.0f, 0.0f, 0.0f);
    *spec_rgb = v3(0.0f, 0.0f, 0.0f);
    if (!lightactive) {
        return;
    }
    V3 L = v3(0.0f, 0.0f, 0.0f);
    float dist_to_light = MJ_MAXVAL;
    float attenuation = 1.0f;
    if (lighttype == 1) {
        L = scale(lightdir, -1.0f);
    } else {
        V3 delta = sub(lightpos, hitpoint);
        // P6：对齐上游 `wp.length` 的中间项单独舍入次序（原为 `length3`，未 pin）。
        float len = length3_midpref(delta);
        float norm = len;
        if (norm > 0.0f) {
            // P4：对齐上游 `normalize_with_norm` 的三次除法（原为 `scale(delta, 1.0f / norm)` 倒数乘）。
            L = v3(delta.x / norm, delta.y / norm, delta.z / norm);
        } else {
            L = v3(0.0f, 0.0f, 0.0f);
        }
        dist_to_light = norm;
        V3 factor = v3(1.0f, dist_to_light, dist_to_light * dist_to_light);
        attenuation = safe_div_render(
            1.0f, dot3_ba(factor.x, factor.y, factor.z, lightattenuation.x, lightattenuation.y,
                          lightattenuation.z));
        if (lighttype == 0) {
            // P9：参考真值 `wp.dot(-L, lightdir)` 为 3×plain `mul.f32` + 1 `neg` + 2 `sub.f32`（**无 fma**），
            // 数值 = `-(rnd(rnd(Lx·dx + Ly·dy) + Lz·dz))`；`neg` 精确，可外提。
            float cos_theta = -__fadd_rn(
                __fadd_rn(__fmul_rn(L.x, lightdir.x), __fmul_rn(L.y, lightdir.y)),
                __fmul_rn(L.z, lightdir.z));
            float cos_cutoff = cosf(lightcutoff_rad);
            if (cos_theta < cos_cutoff) {
                return;
            }
            attenuation = attenuation * powf(fmaxf(cos_theta, 0.0f), lightexp);
        }
    }
    // P5：对齐上游 `wp.dot(normal, L)` 的首项单独舍入次序（原为 `dot3_ba` 的中间项）。
    float ndotl = fmaxf(0.0f, dot3_first(normal.x, normal.y, normal.z, L.x, L.y, L.z));
    if (ndotl == 0.0f) {
        return;
    }
    // R09-S2 阴影可见度：仅 `use_shadows && cast_shadow` 时发射阴影射线（上游 L919）；
    // 命中时 `visible = shadow_light_fraction`，否则 1.0。
    float visible = shadow_visibility(
                                        scene, blob, flex, geom_type, geom_dataid, geom_matid,
                                        geom_rgba, mat_rgba, hfield_bvh_id, worldid,
                                        cull_backfaces, normal, hitpoint, L, lighttype,
                                        dist_to_light, use_shadows, cast_shadow,
                                        shadow_light_fraction);
    float weight = attenuation * visible;
    diag_store_light(diag_light, weight, visible, ndotl);
    *diff_rgb = v3(lightdiff.x * (ndotl * weight), lightdiff.y * (ndotl * weight),
                   lightdiff.z * (ndotl * weight));
    if (enable_specular) {
        if (mat_spec > 0.0f && mat_shin_exp > 0.0f) {
            // R1：参考核 half-vector 归一化是**首项**单独舍入（Arch 附录 D），保持 `normalize3_rays`。
            V3 half = normalize3_rays(add(L, view_dir));
            float ndoth = fmaxf(0.0f, dot3_ba(normal.x, normal.y, normal.z, half.x, half.y, half.z));
            float spec_pow = powf(ndoth, mat_shin_exp);
            float energy = mat_spec * spec_pow * weight;
            diag_store_light_spec(diag_light, ndoth, half, spec_pow);
            *spec_rgb = v3(lightspec.x * energy, lightspec.y * energy, lightspec.z * energy);
        }
    }
}

// 上游 `pack_rgba_to_uint32`：`int()` 截断（不是四舍五入），通道顺序 ABGR。
__device__ __forceinline__ unsigned int pack_rgba_to_uint32(float r, float g, float b, float a) {
    return ((unsigned int)(int)a << 24) | ((unsigned int)(int)r << 16) | ((unsigned int)(int)g << 8) |
           (unsigned int)(int)b;
}
// S7b2 全签名渲染内核（渲染/BVH S7b2）：`_render_megakernel` 的 81 槽上游签名。
//
// 本文件由 `mod.rs::MEGAKERNEL_SOURCE` 接在 `render_common.cu` / `render_shade.cu` 之后，
// 提供唯一入口 `render_megakernel`（1 bounds + 80 形参，顺序与 S7a 冻结的
// `MEGAKERNEL_SLOTS` 逐槽一致；证据 README 给逐槽「填值/real|placeholder」表）。
//
// 语义边界（任务 JSON 已批）：
//   - 内部 BVH 仍是自有打包：场景/网格五段数据经 `bvh_id`（u64 标量）以设备指针传入，
//     不消费 Warp BVH 句柄；`mesh_bvh_id`/`hfield_bvh_id`/`flex_bvh_id` 等槽为 ABI 占位。
//   - 着色子集：headlight 三组、背景色、base color、纹理采样、顶点/面法线、自发光、高光、
//     背面剔除、阴影（R09；host 自 R09-S2b 起不再前置拒绝）；天空盒/splat/flex/hfield/SDF 不在本片。
//   - 复用 S7b2 自有参数槽（均为上游本片未用的占位槽，证据表标注）：
//       `splat_position` = [headlight_ambient, headlight_diffuse, headlight_specular,
//                           background_color_vec3]（4 × vec3 f32）
//       `splat_scale`    = [znear, zfar, shadow_light_fraction]（第三分量原占位 unused，R09-S2 起改作 shadow_light_fraction）
//       `splat_bvh_id`   = (packed_background_u32 << 32) | flags
//     flags 位见 `MK_FLAG_*`。
//   - 光线固定读 S5 预计算缓冲（`use_precomputed_rays=true`；动态路径由 host 拒绝）。
//   - 深度/分割语义与 S7b1 `render_depth_seg` 完全一致（S7b1 的 120,000/120,000 位等是回归门）。

#define MK_FLAG_TEXTURES 0x1u
#define MK_FLAG_VERTEX_NORMALS 0x2u
#define MK_FLAG_SPECULAR 0x4u
#define MK_FLAG_EMISSION 0x8u
#define MK_FLAG_PER_LIGHT_AMBIENT 0x10u
#define MK_FLAG_AMBIENT_LIGHTING 0x20u
#define MK_FLAG_HEADLIGHT 0x40u
#define MK_FLAG_CULL_BACKFACES 0x80u
#define MK_FLAG_AA 0x100u
// R02-S1：天空盒总开关（`ResolvedContext.render_skybox`，上游 `rc_static["render_skybox"]`，
// render.py L1259）。位 9 空闲（位 0-8 见上表），只在 `hit.geom_id == -1` 的背景分支里读。
#define MK_FLAG_SKYBOX 0x200u

// 上游 `store_pixel` 的 AA 分支（render.py L1017-1031）：
//   if wp.static(aa): aa_accum_out[worldid, adr] += color * 255.0
// `ray_base == 0`（第一个子样本）直接覆盖写：省掉每帧 host 侧清零，逐样本求和顺序与
// 上游 `aa_accum.zero_()` + `+=` 完全一致（每个像素每个 sample 都写，含背景）。
__device__ __forceinline__ void aa_store(wp::array_t<wp::vec3> aa_accum, long offset, V3 color,
                                         bool first) {
    float r = color.x * 255.0f;
    float g = color.y * 255.0f;
    float b = color.z * 255.0f;
    if (first) {
        aa_accum.data[offset] = wp::vec_t<3, wp::float32>(r, g, b);
    } else {
        wp::vec_t<3, wp::float32> current = aa_accum.data[offset];
        aa_accum.data[offset] =
            wp::vec_t<3, wp::float32>(current[0] + r, current[1] + g, current[2] + b);
    }
}

#define MK_MAX_SHININESS 128.0f
#define MK_DEFAULT_MAT_SPECULAR 0.5f
#define MK_DEFAULT_MAT_EMISSION 0.0f
#define MK_NO_LIGHT_AMBIENT_FALLBACK 0.3f

// P10 诊断：opt-in 像素中间量 dump 布局常量与 store helper（仅新增 store/分支；`diag_ptr == 0` 时全部首行返回）。
// 本段只被 `render_shade.cu`（前向声明）与下方 megakernel 的诊断分支调用。
#define DIAG_STRIDE 64
#define DIAG_COLOR 0
#define DIAG_BASE_COLOR 3
#define DIAG_TEX_UV 6
#define DIAG_TEX_COLOR 8
#define DIAG_BARY_U 11
#define DIAG_BARY_V 12
#define DIAG_T 13
#define DIAG_FACE 14
#define DIAG_MESH_ID 15
#define DIAG_HIT_POINT 16
#define DIAG_NORMAL 19
#define DIAG_RAY_DIR 22
#define DIAG_LIGHT_BASE 25
#define DIAG_LIGHT_STRIDE 8
#define DIAG_LIGHT_SLOTS 4

// 纹理链：wrap 后的采样 uv（f32 位型原样）与硬件采样结果。
__device__ __forceinline__ void diag_store_tex(float* d, float u, float v, V3 tex_color) {
    if (d == nullptr) {
        return;
    }
    d[DIAG_TEX_UV + 0] = u;
    d[DIAG_TEX_UV + 1] = v;
    d[DIAG_TEX_COLOR + 0] = tex_color.x;
    d[DIAG_TEX_COLOR + 1] = tex_color.y;
    d[DIAG_TEX_COLOR + 2] = tex_color.z;
}

// 每灯：weight/visible/ndotl 在 `compute_lighting` 尾部写；ndoth/half/pow 只在 specular 分支写。
__device__ __forceinline__ void diag_store_light(float* d, float weight, float visible, float ndotl) {
    if (d == nullptr) {
        return;
    }
    d[0] = weight;
    d[1] = visible;
    d[2] = ndotl;
}

__device__ __forceinline__ void diag_store_light_spec(float* d, float ndoth, V3 half,
                                                      float spec_pow) {
    if (d == nullptr) {
        return;
    }
    d[3] = ndoth;
    d[4] = half.x;
    d[5] = half.y;
    d[6] = half.z;
    d[7] = spec_pow;
}

// 命中链：打包前 f32 颜色、纹理后 base_color、求交量（face/mesh_id 以 i32 位型存）、
// 命中点/法线/ray_dir_world。只在命中分支调用；未命中像素不写记录。
// 存 ray_dir_world 而非 view_dir：view_dir = -ray_dir_world 为精确负号位翻转，可在 host 侧
// 无损派生；避免为落盘额外 materialize（否则会多出 3 条 neg.f32，破坏「算术未变」）。
__device__ __forceinline__ void diag_store_hit(float* d, V3 color, V3 base_color, V3 hit_point,
                                               V3 normal, V3 ray_dir, float bary_u, float bary_v,
                                               float t, int face, int mesh_id) {
    if (d == nullptr) {
        return;
    }
    d[DIAG_COLOR + 0] = color.x;
    d[DIAG_COLOR + 1] = color.y;
    d[DIAG_COLOR + 2] = color.z;
    d[DIAG_BASE_COLOR + 0] = base_color.x;
    d[DIAG_BASE_COLOR + 1] = base_color.y;
    d[DIAG_BASE_COLOR + 2] = base_color.z;
    d[DIAG_BARY_U] = bary_u;
    d[DIAG_BARY_V] = bary_v;
    d[DIAG_T] = t;
    d[DIAG_FACE] = __int_as_float(face);
    d[DIAG_MESH_ID] = __int_as_float(mesh_id);
    d[DIAG_HIT_POINT + 0] = hit_point.x;
    d[DIAG_HIT_POINT + 1] = hit_point.y;
    d[DIAG_HIT_POINT + 2] = hit_point.z;
    d[DIAG_NORMAL + 0] = normal.x;
    d[DIAG_NORMAL + 1] = normal.y;
    d[DIAG_NORMAL + 2] = normal.z;
    d[DIAG_RAY_DIR + 0] = ray_dir.x;
    d[DIAG_RAY_DIR + 1] = ray_dir.y;
    d[DIAG_RAY_DIR + 2] = ray_dir.z;
}

extern "C" __global__ void render_megakernel(
    wp::launch_bounds_t<2> dim, wp::array_t<wp::int32> geom_type,
    wp::array_t<wp::int32> geom_dataid, wp::array_t<wp::int32> geom_matid,
    wp::array_t<wp::vec3> geom_size, wp::array_t<wp::vec4> geom_rgba,
    wp::array_t<wp::int32> cam_projection, wp::array_t<float> cam_fovy,
    wp::array_t<wp::vec2> cam_sensorsize, wp::array_t<wp::vec4> cam_intrinsic,
    wp::array_t<wp::int32> light_type, wp::array_t<bool> light_castshadow,
    wp::array_t<bool> light_active, wp::array_t<wp::vec3> light_attenuation,
    wp::array_t<float> light_cutoff, wp::array_t<float> light_exponent,
    wp::array_t<wp::vec3> light_ambient, wp::array_t<wp::vec3> light_diffuse,
    wp::array_t<wp::vec3> light_specular, wp::array_t<wp::int32> flex_vertadr,
    wp::array_t<wp::vec2i> flex_edge, wp::array_t<float> flex_radius,
    wp::array_t<wp::int32> mesh_faceadr, wp::array_t<wp::int32> mesh_normaladr,
    wp::array_t<wp::vec3> mesh_normal, wp::array_t<wp::int32> mat_texid,
    wp::array_t<bool> mat_texuniform, wp::array_t<wp::vec2> mat_texrepeat,
    wp::array_t<float> mat_emission, wp::array_t<float> mat_specular,
    wp::array_t<float> mat_shininess, wp::array_t<wp::vec4> mat_rgba,
    wp::array_t<wp::vec3> geom_xpos_in, wp::array_t<wp::mat33> geom_xmat_in,
    wp::array_t<wp::vec3> cam_xpos_in, wp::array_t<wp::mat33> cam_xmat_in,
    wp::array_t<wp::vec3> light_xpos_in, wp::array_t<wp::vec3> light_xdir_in,
    wp::array_t<wp::vec3> flexvert_xpos_in, wp::int32 nrender, bool use_shadows,
    wp::int32 bvh_ngeom, wp::int32 bvh_nflexgeom, wp::array_t<wp::vec2i> cam_res,
    wp::array_t<wp::int32> cam_id_map, wp::array_t<wp::vec3> ray,
    wp::array_t<wp::vec3> ray_offset, wp::int32 ray_base, wp::array_t<wp::int32> rgb_adr,
    wp::array_t<wp::int32> depth_adr, wp::array_t<wp::int32> seg_adr,
    wp::array_t<bool> render_rgb, wp::array_t<bool> render_depth,
    wp::array_t<bool> render_seg, wp::uint64 bvh_id, wp::array_t<wp::int32> group_root,
    wp::array_t<wp::uint64> flex_bvh_id, wp::array_t<wp::int32> flex_group_root,
    wp::array_t<wp::int32> enabled_geom_ids, wp::array_t<wp::uint64> mesh_bvh_id,
    wp::array_t<wp::vec3i> mesh_facetexcoord, wp::array_t<wp::vec3i> mesh_facenormal,
    wp::array_t<wp::vec2> mesh_texcoord, wp::array_t<wp::int32> mesh_texcoord_offsets,
    wp::array_t<wp::uint64> hfield_bvh_id, wp::array_t<wp::vec4> flex_rgba,
    wp::array_t<wp::int32> flex_geom_flexid, wp::array_t<wp::int32> flex_geom_edgeid,
    wp::array_t<wp::int32> skybox_tex_id, wp::array_t<wp::int32> skybox_face_width,
    wp::array_t<wp::texture2d_t> textures, wp::array_t<wp::vec3> splat_position,
    wp::array_t<wp::quat> splat_rotation, wp::array_t<wp::vec3> splat_scale,
    wp::array_t<wp::vec4> splat_rgba, wp::uint64 splat_bvh_id,
    wp::array_t<wp::int32> splat_group_root, wp::array_t<wp::uint32> rgb_out,
    wp::array_t<wp::vec3> aa_accum_out, wp::array_t<float> depth_out,
    wp::array_t<wp::vec2i> seg_out, wp::uint64 diag_ptr) {
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
    // R09-S2：`use_shadows` 已支持（阴影在 compute_lighting 内按灯 castshadow 处理）；
    // splat 仍未实现，由 host 侧拒绝（这里只做防御性判断）。

    const int ncam = nrender;
    int camid = -1;
    int rayid_local = -1;
    int accum = 0;
    for (int camera = 0; camera < ncam; camera++) {
        int width = cam_res.data[camera][0];
        int height = cam_res.data[camera][1];
        int num_i = width * height;
        if (rayid < accum + num_i) {
            camid = camera;
            rayid_local = rayid - accum;
            break;
        }
        accum += num_i;
    }
    if (camid < 0 || rayid_local < 0) {
        return;
    }
    bool do_depth = render_depth.data[camid];
    bool do_seg = render_seg.data[camid];
    bool do_rgb = render_rgb.data[camid];
    if (!do_depth && !do_seg && !do_rgb) {
        return;
    }

    unsigned long long flags = splat_bvh_id;
    unsigned int flag_bits = (unsigned int)(flags & 0xFFFFFFFFull);
    unsigned int packed_background = (unsigned int)(flags >> 32);
    V3 headlight_ambient = to_v3(splat_position.data[0]);
    V3 headlight_diffuse = to_v3(splat_position.data[1]);
    V3 headlight_specular = to_v3(splat_position.data[2]);
    V3 background = to_v3(splat_position.data[3]);
    float zfar = splat_scale.data[0][1];
    // R09-S2：`scale_constants[0][2]` 原占位 unused → `shadow_light_fraction`（上游 L880）。
    float shadow_light_fraction = splat_scale.data[0][2];
    // S7b3：AA（samples_per_pixel>1）时 `store_pixel` 累积到 aa_accum，最后 `aa_resolve` 平均。
    bool aa = (flag_bits & MK_FLAG_AA) != 0u;

    int mujoco_cam_id = cam_id_map.data[camid];
    V3 ray_dir_local_cam = to_v3(ray.data[ray_base + rayid]);
    V3 ray_offset_local_cam = to_v3(ray_offset.data[ray_base + rayid]);
    int cam_slot = worldid * (int)cam_xpos_in.shape.dims[1] + mujoco_cam_id;
    const float* cam_mat = (const float*)(cam_xmat_in.data + cam_slot);
    V3 cam_pos = to_v3(cam_xpos_in.data[cam_slot]);
    V3 ray_origin_world = add(cam_pos, mat_vec(cam_mat, ray_offset_local_cam));
    V3 ray_dir_world = mat_vec(cam_mat, ray_dir_local_cam);

    float max_cam_dist = MJ_MAXVAL;
    if (zfar > 0.0f) {
        max_cam_dist = zfar / fmaxf(-ray_dir_local_cam.z, 1.0e-6f);
    }

    MegakernelBlob blob = megakernel_blob(bvh_id);
    SceneView scene = scene_view(blob.scene_meta, blob.scene_params);
    // R06-S3a：flex 设备视图（81 槽的 9 个 flex 形参）。无 flex 时槽是 1 元素占位，但
    // `bvh_nflexgeom == 0` ⇒ 场景叶不会有负 `leaf_geom`，此视图不会被解引用。
    FlexView flex;
    flex.vertadr = flex_vertadr.data;
    flex.edge = (const int*)flex_edge.data;
    flex.radius = flex_radius.data;
    flex.xpos = (const float*)flexvert_xpos_in.data;
    flex.rgba = (const float*)flex_rgba.data;
    flex.geom_flexid = flex_geom_flexid.data;
    flex.geom_edgeid = flex_geom_edgeid.data;
    // R06-S3c：2D/3D mesh 条目基址（宿主把 `flex_bvh_id` 槽覆盖为 blob 绝对条目号；
    // 1D/无 mesh 的 flex 为 `u64::MAX` 哨兵，设备侧据此走胶囊路径）。
    flex.bvh_id = (const unsigned long long*)flex_bvh_id.data;
    flex.nflexvert = (int)flexvert_xpos_in.shape.dims[1];
    bool cull_backfaces = (flag_bits & MK_FLAG_CULL_BACKFACES) != 0u;
    CastHit hit = cast_ray(scene, blob, flex, geom_type.data, geom_dataid.data, geom_matid.data,
                           (const float*)geom_rgba.data, (const float*)mat_rgba.data,
                           hfield_bvh_id.data, worldid, ray_origin_world, ray_dir_world,
                           max_cam_dist, cull_backfaces, false);

    if (hit.geom_id != -1 && zfar > 0.0f && (hit.dist * -ray_dir_local_cam.z) > zfar) {
        hit.geom_id = -1;
    }

    // 输出 pitch（array2d 的第二维）：上游 `rgb_out[worldid, adr + rayid_local]`。
    int rgb_pitch = rgb_out.shape.dims[1];
    int depth_pitch = depth_out.shape.dims[1];
    int seg_pitch = seg_out.shape.dims[1];
    long rgb_offset = (long)worldid * rgb_pitch + rgb_adr.data[camid] + rayid_local;
    float* diag = (float*)(unsigned long long)diag_ptr;
    float* diag_rec = diag != nullptr ? diag + (long)rayid * DIAG_STRIDE : nullptr;
    // 上游：「Depth and seg are single-sample outputs: only the first pass writes them」
    // （render.py `if ray_base == 0`）。AA 时后续子样本不覆盖；spp=1 时恒真。
    bool first_sample = ray_base == 0;

    if (do_seg && first_sample) {
        wp::vec2i value;
        if (hit.geom_id == -2) {
            // 上游 L1245-1249：flex 的 seg = `(flexid, ObjType.FLEX)`；`mjOBJ_FLEX = 9`
            // （mujoco `mjtype.h` L292-301 枚举序）。
            value[0] = hit.mesh_id;
            value[1] = 9;
        } else {
            value[0] = hit.geom_id;
            value[1] = hit.geom_id == -1 ? -1 : 5;
        }
        seg_out.data[(long)worldid * seg_pitch + seg_adr.data[camid] + rayid_local] = value;
    }

    if (hit.geom_id == -1) {
        if (do_depth && first_sample) {
            depth_out.data[(long)worldid * depth_pitch + depth_adr.data[camid] + rayid_local] = 0.0f;
        }
        if (do_rgb) {
            // R02-S1：上游 L1259-1268 —— `render_skybox` 且该相机渲染 RGB 时，背景像素采
            // 天空盒并按 `store_pixel`（L1017-1031）语义写出（AA 累加 `color*255`，非 AA
            // 直接打包写）。`MK_FLAG_SKYBOX` 未置位时不进入本支，行为与 S7b2 逐字节一致。
            if ((flag_bits & MK_FLAG_SKYBOX) != 0u) {
                int skybox_id = skybox_tex_id.data[worldid % skybox_tex_id.shape.dims[0]];
                // 上游 L1263：`1.0 / float(skybox_face_width[...])`（f32 除法；face 宽度 =
                // 模型 `tex_width`，见 render_util.py L519）。
                float face_width_inv =
                    1.0f / (float)skybox_face_width.data[worldid % skybox_face_width.shape.dims[0]];
                V3 skybox_color =
                    sample_skybox(textures.data[skybox_id], face_width_inv, ray_dir_world);
                if (aa) {
                    aa_store(aa_accum_out, rgb_offset, skybox_color, ray_base == 0);
                } else {
                    rgb_out.data[rgb_offset] =
                        pack_rgba_to_uint32(skybox_color.x * 255.0f, skybox_color.y * 255.0f,
                                            skybox_color.z * 255.0f, 255.0f);
                }
            } else if (aa) {
                aa_store(aa_accum_out, rgb_offset, background, ray_base == 0);
            } else {
                rgb_out.data[rgb_offset] = packed_background;
            }
        }
        return;
    }

    if (do_depth && first_sample) {
        depth_out.data[(long)worldid * depth_pitch + depth_adr.data[camid] + rayid_local] =
            hit.dist * -ray_dir_local_cam.z;
    }
    if (!do_rgb) {
        return;
    }

    int geom_id = hit.geom_id;
    V3 normal = hit.normal;
    int face = hit.face;
    int mesh_id = hit.mesh_id;
    // R06-S3a：flex 命中 `geom_id = -2`，不能索引 geom 模型数组；按上游 `GeomType.FLEX`
    // （mjGEOM_FLEX = 9）参与后续分支（顶点法线分支因此天然跳过）。
    int hit_geom_type = (geom_id == -2) ? 9 : geom_type.data[geom_id];

    // 顶点法线（上游 `enable_vertex_normals` 分支；mesh/SDF 命中且 face>=0 时；
    // R05-S2：hfield 命中不带顶点法线，与上游 render.py L1209-1211 的 gtype 判据一致）。
    bool vertex_normal_gtype = (hit_geom_type == 7 || hit_geom_type == 8);
    if ((flag_bits & MK_FLAG_VERTEX_NORMALS) != 0u && vertex_normal_gtype && mesh_id >= 0 &&
        face >= 0) {
        const float* geom_mat = (const float*)(geom_xmat_in.data + geom_id);
        V3 geom_face = mat_t_vec(geom_mat, normal);
        const int* face_normal = (const int*)(mesh_facenormal.data + (mesh_faceadr.data[mesh_id] + face));
        int adr = mesh_normaladr.data[mesh_id];
        V3 vn = add(
            scale(vertex_normal(to_v3(mesh_normal.data[adr + face_normal[0]]), geom_face), hit.bary_u),
            scale(vertex_normal(to_v3(mesh_normal.data[adr + face_normal[1]]), geom_face), hit.bary_v));
        vn = add(vn, scale(vertex_normal(to_v3(mesh_normal.data[adr + face_normal[2]]), geom_face),
                           1.0f - hit.bary_u - hit.bary_v));
        // P3：mesh 顶点法线归一化对齐参考核的中间项单独舍入次序。
        normal = normalize3_midpref(mat_vec(geom_mat, vn));
    }
    if ((flag_bits & MK_FLAG_CULL_BACKFACES) == 0u && dot3_ba(normal.x, normal.y, normal.z,
                                                               ray_dir_world.x, ray_dir_world.y,
                                                               ray_dir_world.z) > 0.0f) {
        normal = scale(normal, -1.0f);
    }

    // 命中点/材质。
    V3 hit_point = add(ray_origin_world, scale(ray_dir_world, hit.dist));
    float rgba[4];
    int mat_id = -1;
    if (geom_id == -2) {
        // 上游 L1303-1305：flex 命中把 flex id 编码在 mesh_id，颜色取 `flex_rgba[mesh_id]`，
        // 材质 id 保持 -1（无纹理/高光/自发光，走默认值）。
        wp::vec4 flex_color = flex_rgba.data[mesh_id];
        rgba[0] = flex_color[0];
        rgba[1] = flex_color[1];
        rgba[2] = flex_color[2];
        rgba[3] = flex_color[3];
    } else {
        render_geom_rgba(geom_matid.data, (const float*)geom_rgba.data,
                         (const float*)mat_rgba.data, geom_id, rgba, &mat_id);
    }
    V3 base_color = v3(rgba[0], rgba[1], rgba[2]);

    if ((flag_bits & MK_FLAG_TEXTURES) != 0u && mat_id >= 0) {
        int tex_id = mat_texid.data[mat_id * 10 + 1];
        if (tex_id >= 0) {
            const float* geom_pos = (const float*)(geom_xpos_in.data + geom_id);
            const float* geom_mat = (const float*)(geom_xmat_in.data + geom_id);
            bool tex_uniform = mat_texuniform.data[mat_id];
            V3 tex_color = sample_texture(
                geom_type.data, mesh_faceadr.data, geom_id, (const float*)(geom_size.data + geom_id),
                (const float*)(mat_texrepeat.data + mat_id), tex_uniform, textures.data[tex_id],
                v3(geom_pos[0], geom_pos[1], geom_pos[2]), geom_mat,
                (const int*)mesh_facetexcoord.data, (const float*)mesh_texcoord.data,
                mesh_texcoord_offsets.data, hit_point, normal, hit.bary_u, hit.bary_v, face, mesh_id,
                diag_rec);
            base_color = cw_mul(base_color, tex_color);
        }
    }

    float mat_spec = MK_DEFAULT_MAT_SPECULAR;
    float mat_shin_exp = 0.5f * MK_MAX_SHININESS;
    float mat_emis = MK_DEFAULT_MAT_EMISSION;
    if (mat_id >= 0) {
        if ((flag_bits & MK_FLAG_SPECULAR) != 0u) {
            mat_spec = mat_specular.data[mat_id];
            mat_shin_exp = mat_shininess.data[mat_id] * MK_MAX_SHININESS;
        }
        if ((flag_bits & MK_FLAG_EMISSION) != 0u) {
            mat_emis = mat_emission.data[mat_id];
        }
    }

    V3 result = v3(0.0f, 0.0f, 0.0f);
    if ((flag_bits & MK_FLAG_EMISSION) != 0u) {
        result = scale(base_color, mat_emis);
    }

    int nlight = light_type.shape.dims[1];
    if ((flag_bits & MK_FLAG_AMBIENT_LIGHTING) != 0u) {
        if ((flag_bits & MK_FLAG_HEADLIGHT) != 0u) {
            result = add(result, cw_mul(base_color, headlight_ambient));
        } else if (nlight == 0) {
            result = add(result, scale(base_color, MK_NO_LIGHT_AMBIENT_FALLBACK));
        }
        if ((flag_bits & MK_FLAG_PER_LIGHT_AMBIENT) != 0u) {
            for (int light = 0; light < nlight; light++) {
                if (light_active.data[light]) {
                    result = add(result, cw_mul(base_color, to_v3(light_ambient.data[light])));
                }
            }
        }
    }

    V3 view_dir = scale(ray_dir_world, -1.0f);
    bool enable_specular = (flag_bits & MK_FLAG_SPECULAR) != 0u;
    for (int light = 0; light < nlight; light++) {
        bool active = light_active.data[light];
        int type = light_type.data[light];
        V3 lightpos = to_v3(light_xpos_in.data[light]);
        V3 lightdir = to_v3(light_xdir_in.data[light]);
        V3 attenuation = to_v3(light_attenuation.data[light]);
        float cutoff = light_cutoff.data[light] * RENDER_DEG_TO_RAD;
        float exponent = light_exponent.data[light];
        V3 diff_rgb;
        V3 spec_rgb;
        int diag_slot = light < DIAG_LIGHT_SLOTS - 1 ? light : DIAG_LIGHT_SLOTS - 1;
        float* diag_light = diag_rec != nullptr
                                ? diag_rec + DIAG_LIGHT_BASE + diag_slot * DIAG_LIGHT_STRIDE
                                : nullptr;
        compute_lighting(scene, blob, flex, geom_type.data, geom_dataid.data, geom_matid.data,
                         (const float*)geom_rgba.data, (const float*)mat_rgba.data,
                         hfield_bvh_id.data, worldid,
                         cull_backfaces, active, type, lightpos, lightdir, attenuation, cutoff,
                         exponent, to_v3(light_diffuse.data[light]),
                         to_v3(light_specular.data[light]), normal, hit_point, view_dir, mat_spec,
                         mat_shin_exp, enable_specular, use_shadows, light_castshadow.data[light],
                         shadow_light_fraction, &diff_rgb, &spec_rgb, diag_light);
        result = add(add(result, cw_mul(base_color, diff_rgb)), spec_rgb);
    }

    // Headlight：camera 位置 + 光轴方向（上游 `cam_fwd = -cam_mat_world[:, 2]`）。
    // 上游 headlight 调用传的是**材质** `mat_spec`/`mat_shin_exp`（megakernel 行 9419-9432
    // 的 `var_352`/`var_353`），不是固定的 headlight 高光指数。
    if ((flag_bits & MK_FLAG_HEADLIGHT) != 0u) {
        V3 cam_fwd = v3(-cam_mat[2], -cam_mat[5], -cam_mat[8]);
        V3 diff_rgb;
        V3 spec_rgb;
        float* diag_headlight = diag_rec != nullptr
                                    ? diag_rec + DIAG_LIGHT_BASE + (DIAG_LIGHT_SLOTS - 1) * DIAG_LIGHT_STRIDE
                                    : nullptr;
        // R09-S2：headlight 不投影（上游 L1457-1459 传 `lightcastshadow=False`）。
        compute_lighting(scene, blob, flex, geom_type.data, geom_dataid.data, geom_matid.data,
                         (const float*)geom_rgba.data, (const float*)mat_rgba.data,
                         hfield_bvh_id.data, worldid,
                         cull_backfaces, true, 1, ray_origin_world, cam_fwd, v3(1.0f, 0.0f, 0.0f),
                         0.0f, 0.0f, headlight_diffuse, headlight_specular, normal, hit_point,
                         view_dir, mat_spec, mat_shin_exp, enable_specular, use_shadows, false,
                         shadow_light_fraction, &diff_rgb, &spec_rgb, diag_headlight);
        result = add(add(result, cw_mul(base_color, diff_rgb)), spec_rgb);
    }

    V3 hit_color = v3(fminf(result.x, 1.0f), fminf(result.y, 1.0f), fminf(result.z, 1.0f));
    hit_color = v3(fmaxf(hit_color.x, 0.0f), fmaxf(hit_color.y, 0.0f), fmaxf(hit_color.z, 0.0f));
    diag_store_hit(diag_rec, hit_color, base_color, hit_point, normal, ray_dir_world, hit.bary_u,
                   hit.bary_v, hit.dist, face, mesh_id);
    if (aa) {
        aa_store(aa_accum_out, rgb_offset, hit_color, ray_base == 0);
        return;
    }
    float r = hit_color.x * 255.0f;
    float g = hit_color.y * 255.0f;
    float b = hit_color.z * 255.0f;
    unsigned int pixel = pack_rgba_to_uint32(r, g, b, 255.0f);
    rgb_out.data[rgb_offset] = pixel;
}
