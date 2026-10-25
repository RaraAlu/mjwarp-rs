// G10@P5 G5：SDF 数学设备版，与 host `sdf/primitives.rs` + `sdf/descent.rs` 逐式对应。
// Adapted from frozen MJWarp collision_sdf.py, revision 71da24d. Apache-2.0.
// 注意：rel_mat 一律按上游 `transpose(rot1) * rot2`（common.cu 的 relative_matrix 已按上游语义）。
#pragma once
#include "sdf_octree.cu"

#define G5_MINVAL 1.0e-15f

// SDF 几何类型（与 host `SdfType` 一一对应）。
#define G5_SDF_PLANE 0
#define G5_SDF_SPHERE 1
#define G5_SDF_CAPSULE 2
#define G5_SDF_CYLINDER 3
#define G5_SDF_BOX 4
#define G5_SDF_ELLIPSOID 5
#define G5_SDF_MESH 6
#define G5_SDF_SDF 7

struct d_sdf_shape {
  int geom;
  v3 size;
  const d_volume_data* volume;
  bool has_volume;
};

struct d_opt_params {
  float rel_mat[9];
  v3 rel_pos;
  v3 attr1;
  v3 attr2;
};

// ---- 原语 SDF 与梯度（与 host `sdf/primitives.rs` 一致） ----

__device__ __forceinline__ v3 g5_abs3(v3 a) {
  return v3_make(fabsf(a.x), fabsf(a.y), fabsf(a.z));
}

__device__ __forceinline__ v3 g5_cw_div3(v3 a, v3 b) {
  return v3_make(a.x / b.x, a.y / b.y, a.z / b.z);
}

__device__ __forceinline__ v3 g5_max3(v3 a, v3 b) {
  return v3_make(fmaxf(a.x, b.x), fmaxf(a.y, b.y), fmaxf(a.z, b.z));
}

__device__ __forceinline__ v3 g5_radial_field(v3 a, v3 x, v3 size) {
  v3 field = v3_normalize(g5_cw_div3(v3_neg(size), a));
  // 上游 `wp.sign` 语义：`x < 0 ? -1 : 1`（零/-0.0 → +1，不返回 0）。
  return v3_make(field.x * (x.x < 0.0f ? -1.0f : 1.0f),
                 field.y * (x.y < 0.0f ? -1.0f : 1.0f),
                 field.z * (x.z < 0.0f ? -1.0f : 1.0f));
}

__device__ __forceinline__ float g5_prim_sphere(v3 p, v3 size) {
  return v3_length(p) - size.x;
}

__device__ __forceinline__ float g5_prim_box(v3 p, v3 size) {
  v3 a = v3_sub(g5_abs3(p), size);
  if (a.x >= 0.0f || a.y >= 0.0f || a.z >= 0.0f) {
    v3 b = g5_max3(a, v3_make(0.0f, 0.0f, 0.0f));
    return v3_length(b) + fminf(fmaxf(a.x, fmaxf(a.y, a.z)), 0.0f);
  }
  v3 b = g5_radial_field(a, p, size);
  v3 t = g5_cw_div3(v3_neg(a), g5_abs3(b));
  return -fminf(t.x, fminf(t.y, t.z)) * v3_length(b);
}

__device__ __forceinline__ float g5_prim_ellipsoid(v3 p, v3 size) {
  v3 scaled = v3_make(p.x / size.x, p.y / size.y, p.z / size.z);
  float k0 = v3_length(scaled);
  float k1 = v3_length(v3_make(p.x / (size.x * size.x), p.y / (size.y * size.y),
                               p.z / (size.z * size.z)));
  float denom = k1 != 0.0f ? k1 : 1.0e-12f;
  return k0 * (k0 - 1.0f) / denom;
}

__device__ __forceinline__ float g5_prim_capsule(v3 p, v3 size) {
  float r = size.x;
  float h = size.y;
  float pz = fminf(fmaxf(p.z, -h), h);
  return v3_length(v3_make(p.x, p.y, p.z - pz)) - r;
}

__device__ __forceinline__ float g5_prim_cylinder(v3 p, v3 size) {
  float r = size.x;
  float h = size.y;
  float dx = v3_length(v3_make(p.x, p.y, 0.0f)) - r;
  float dy = fabsf(p.z) - h;
  return fminf(fmaxf(dx, dy), 0.0f) +
         v3_length(v3_make(fmaxf(dx, 0.0f), fmaxf(dy, 0.0f), 0.0f));
}

__device__ __forceinline__ v3 g5_grad_sphere(v3 p) {
  float c = v3_length(p);
  if (c > 1.0e-9f) return v3_make(p.x / c, p.y / c, p.z / c);
  return v3_make(0.0f, 0.0f, 0.0f);
}

__device__ __forceinline__ v3 g5_grad_box(v3 p, v3 size) {
  v3 a = v3_sub(g5_abs3(p), size);
  if (fmaxf(a.x, fmaxf(a.y, a.z)) < 0.0f) return g5_radial_field(a, p, size);
  v3 b = g5_max3(a, v3_make(0.0f, 0.0f, 0.0f));
  float c = v3_length(b);
  v3 g = v3_make(b.x / c * (p.x / fabsf(p.x)), b.y / c * (p.y / fabsf(p.y)),
                 b.z / c * (p.z / fabsf(p.z)));
  if (a.x <= 0.0f) g.x = 0.0f;
  if (a.y <= 0.0f) g.y = 0.0f;
  if (a.z <= 0.0f) g.z = 0.0f;
  return g;
}

__device__ __forceinline__ v3 g5_grad_ellipsoid(v3 p, v3 size) {
  v3 a = v3_make(p.x / size.x, p.y / size.y, p.z / size.z);
  v3 b = v3_make(a.x / size.x, a.y / size.y, a.z / size.z);
  float k0 = v3_length(a);
  float k1 = v3_length(b);
  float inv_k0 = 1.0f / k0;
  float inv_k1 = 1.0f / k1;
  v3 gk0 = v3_make(b.x * inv_k0, b.y * inv_k0, b.z * inv_k0);
  v3 gk1 = v3_make(b.x * inv_k1 / (size.x * size.x), b.y * inv_k1 / (size.y * size.y),
                   b.z * inv_k1 / (size.z * size.z));
  float df_dk0 = (2.0f * k0 - 1.0f) * inv_k1;
  float df_dk1 = k0 * (k0 - 1.0f) * inv_k1 * inv_k1;
  v3 raw = v3_make(gk0.x * df_dk0 - gk1.x * df_dk1, gk0.y * df_dk0 - gk1.y * df_dk1,
                   gk0.z * df_dk0 - gk1.z * df_dk1);
  return v3_normalize(raw);
}

__device__ __forceinline__ v3 g5_grad_capsule(v3 p, v3 size) {
  float h = size.y;
  float pz = fminf(fmaxf(p.z, -h), h);
  v3 diff = v3_make(p.x, p.y, p.z - pz);
  float c = v3_length(diff);
  if (c > G5_MINVAL) return v3_make(diff.x / c, diff.y / c, diff.z / c);
  return v3_make(0.0f, 0.0f, 0.0f);
}

__device__ __forceinline__ v3 g5_grad_cylinder(v3 p, v3 size) {
  float r = size.x;
  float h = size.y;
  float radial_dist = v3_length(v3_make(p.x, p.y, 0.0f));
  v3 u = radial_dist > G5_MINVAL
             ? v3_make(p.x / radial_dist, p.y / radial_dist, 0.0f)
             : v3_make(0.0f, 0.0f, 0.0f);
  // 上游 `wp.sign(p[2])`（零/-0.0 → +1）。
  v3 w = v3_make(0.0f, 0.0f, p.z < 0.0f ? -1.0f : 1.0f);
  float dx = radial_dist - r;
  float dy = fabsf(p.z) - h;
  if (dx > 0.0f && dy > 0.0f) {
    float len_v = v3_length(v3_make(dx, dy, 0.0f));
    if (len_v > G5_MINVAL) {
      return v3_make(u.x * (dx / len_v) + w.x * (dy / len_v),
                     u.y * (dx / len_v) + w.y * (dy / len_v),
                     u.z * (dx / len_v) + w.z * (dy / len_v));
    }
    return v3_make(0.0f, 0.0f, 0.0f);
  }
  if (dx > 0.0f) return u;
  if (dy > 0.0f) return w;
  return dx > dy ? u : w;
}

// ---- sdf / sdf_grad 分发 ----

__device__ __forceinline__ bool g5_sdf_volume(const d_sdf_shape* shape, v3 p, float* out) {
  if (!shape->has_volume) return false;
  return g5_sample_volume_sdf(shape->volume, p, out);
}

__device__ __forceinline__ bool g5_sdf_volume_grad(const d_sdf_shape* shape, v3 p, v3* out) {
  if (!shape->has_volume) return false;
  return g5_sample_volume_grad(shape->volume, p, out);
}

__device__ __forceinline__ bool g5_sdf(const d_sdf_shape* shape, v3 p, float* out) {
  switch (shape->geom) {
    case G5_SDF_PLANE: *out = p.z; return true;
    case G5_SDF_SPHERE: *out = g5_prim_sphere(p, shape->size); return true;
    case G5_SDF_CAPSULE: *out = g5_prim_capsule(p, shape->size); return true;
    case G5_SDF_CYLINDER: *out = g5_prim_cylinder(p, shape->size); return true;
    case G5_SDF_BOX: *out = g5_prim_box(p, shape->size); return true;
    case G5_SDF_ELLIPSOID: *out = g5_prim_ellipsoid(p, shape->size); return true;
    case G5_SDF_MESH:
    case G5_SDF_SDF: return g5_sdf_volume(shape, p, out);
    default: return false;
  }
}

__device__ __forceinline__ bool g5_sdf_grad(const d_sdf_shape* shape, v3 p, v3* out) {
  switch (shape->geom) {
    case G5_SDF_PLANE: *out = v3_make(0.0f, 0.0f, 1.0f); return true;
    case G5_SDF_SPHERE: *out = g5_grad_sphere(p); return true;
    case G5_SDF_CAPSULE: *out = g5_grad_capsule(p, shape->size); return true;
    case G5_SDF_CYLINDER: *out = g5_grad_cylinder(p, shape->size); return true;
    case G5_SDF_BOX: *out = g5_grad_box(p, shape->size); return true;
    case G5_SDF_ELLIPSOID: *out = g5_grad_ellipsoid(p, shape->size); return true;
    case G5_SDF_MESH:
    case G5_SDF_SDF: return g5_sdf_volume_grad(shape, p, out);
    default: return false;
  }
}

// 上游对 geom2 强制 `GeomType.SDF` 的调用。
__device__ __forceinline__ bool g5_forced_sdf(const d_sdf_shape* shape, v3 p, float* out) {
  return g5_sdf_volume(shape, p, out);
}

__device__ __forceinline__ bool g5_forced_sdf_grad(const d_sdf_shape* shape, v3 p, v3* out) {
  return g5_sdf_volume_grad(shape, p, out);
}

// 上游 `clearance`。
__device__ __forceinline__ float g5_clearance(const d_sdf_shape* shape1,
                                              const d_sdf_shape* shape2, v3 p1, v3 p2,
                                              bool sfd_intersection) {
  float sdf1 = 0.0f;
  float sdf2 = 0.0f;
  g5_sdf(shape1, p1, &sdf1);
  g5_forced_sdf(shape2, p2, &sdf2);
  float max_val = fmaxf(sdf1, sdf2);
  return sfd_intersection ? max_val : sdf1 + sdf2 + fabsf(max_val);
}

// 上游 `compute_grad`。
__device__ __forceinline__ v3 g5_compute_grad(const d_sdf_shape* shape1,
                                              const d_sdf_shape* shape2, v3 p1, v3 p2,
                                              const d_opt_params* params,
                                              bool sfd_intersection) {
  float a_ = 0.0f;
  float b_ = 0.0f;
  v3 grad1 = v3_make(0.0f, 0.0f, 0.0f);
  v3 grad2 = v3_make(0.0f, 0.0f, 0.0f);
  g5_sdf(shape1, p1, &a_);
  g5_forced_sdf(shape2, p2, &b_);
  g5_sdf_grad(shape1, p1, &grad1);
  g5_forced_sdf_grad(shape2, p2, &grad2);
  v3 grad1_transformed = transpose_mul_mat3_vec(params->rel_mat, grad1);
  if (sfd_intersection) return a_ > b_ ? grad1_transformed : grad2;
  v3 gradient = v3_add(grad2, grad1_transformed);
  float max_val = fmaxf(a_, b_);
  v3 max_grad = a_ > b_ ? grad1_transformed : grad2;
  // 上游 `wp.sign(max_val)`（零/-0.0 → +1）。
  float sign = max_val < 0.0f ? -1.0f : 1.0f;
  return v3_add(gradient, v3_scale(max_grad, sign));
}

// 上游 `gradient_step`：Armijo 回溯（amin=1e-4、rho=0.5、c=0.1）。
__device__ __forceinline__ void g5_gradient_step(const d_sdf_shape* shape1,
                                                 const d_sdf_shape* shape2, v3 x0,
                                                 const d_opt_params* params, int niter,
                                                 bool sfd_intersection, float* out_dist,
                                                 v3* out_x) {
  const float amin = 1.0e-4f;
  const float rho = 0.5f;
  const float c = 0.1f;
  float dist = 1.0e10f;
  v3 x = x0;
  for (int iteration = 0; iteration < niter; ++iteration) {
    float alpha = 2.0f;
    v3 x2 = x;
    v3 x1 = v3_add(mul_mat3_vec(params->rel_mat, x2), params->rel_pos);
    v3 grad = g5_compute_grad(shape1, shape2, x1, x2, params, sfd_intersection);
    float dist0 = g5_clearance(shape1, shape2, x1, x, sfd_intersection);
    float grad_dot = v3_dot(grad, grad);
    if (grad_dot < 1.0e-12f) {
      *out_dist = dist0;
      *out_x = x;
      return;
    }
    float wolfe = -c * alpha * grad_dot;
    for (;;) {
      alpha *= rho;
      wolfe *= rho;
      x = v3_sub(x2, v3_scale(grad, alpha));
      x1 = v3_add(mul_mat3_vec(params->rel_mat, x), params->rel_pos);
      dist = g5_clearance(shape1, shape2, x1, x, sfd_intersection);
      if (alpha <= amin || (dist - dist0) <= wolfe) break;
    }
    if (dist > dist0) {
      *out_dist = dist0;
      *out_x = x2;
      return;
    }
  }
  *out_dist = dist;
  *out_x = x;
}

// 上游 `gradient_descent`：sdf_iterations 次普通步 + 1 次 intersection 步。
__device__ __forceinline__ void g5_gradient_descent(const d_sdf_shape* shape1,
                                                    const d_sdf_shape* shape2, v3 x0_initial,
                                                    v3 pos1, const float* rot1, v3 pos2,
                                                    const float* rot2, int sdf_iterations,
                                                    float* out_dist, v3* out_pos,
                                                    v3* out_normal) {
  d_opt_params params;
  relative_matrix(rot1, rot2, params.rel_mat);
  params.rel_pos = transpose_mul_mat3_vec(rot1, v3_sub(pos2, pos1));
  params.attr1 = shape1->size;
  params.attr2 = shape2->size;
  float dist = 0.0f;
  v3 x;
  g5_gradient_step(shape1, shape2, x0_initial, &params, sdf_iterations, false, &dist, &x);
  g5_gradient_step(shape1, shape2, x, &params, 1, true, &dist, &x);
  v3 x1 = v3_add(mul_mat3_vec(params.rel_mat, x), params.rel_pos);
  v3 grad1 = v3_make(0.0f, 0.0f, 0.0f);
  g5_sdf_grad(shape1, x1, &grad1);
  grad1 = v3_normalize(transpose_mul_mat3_vec(params.rel_mat, grad1));
  v3 grad2 = v3_make(0.0f, 0.0f, 0.0f);
  g5_forced_sdf_grad(shape2, x, &grad2);
  grad2 = v3_normalize(grad2);
  v3 n = v3_normalize(v3_sub(grad1, grad2));
  v3 pos = v3_add(mul_mat3_vec(rot2, x), pos2);
  n = mul_mat3_vec(rot2, n);
  *out_dist = dist;
  *out_pos = v3_sub(pos, v3_scale(n, dist / 2.0f));
  *out_normal = n;
}
