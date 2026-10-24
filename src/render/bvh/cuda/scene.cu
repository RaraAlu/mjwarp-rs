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
