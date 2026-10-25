// G10@P5 G5：距离子算法设备版，与 host `gjk/simplex.rs` 逐式对应。
// Adapted from frozen MJWarp collision_gjk.py, revision 71da24d. Apache-2.0.
#pragma once
#include "common.cu"

#define G5_FLOAT_MAX 1.0e30f
#define G5_MINVAL 1.0e-15f

__device__ __forceinline__ float g5_det3(v3 v1, v3 v2, v3 v3c) {
  return v3_dot(v1, v3_cross(v2, v3c));
}

__device__ __forceinline__ int g5_same_sign(float a, float b) {
  if (a > 0.0f && b > 0.0f) return 1;
  if (a < 0.0f && b < 0.0f) return -1;
  return 0;
}

__device__ __forceinline__ v3 g5_project_origin_line(v3 v1, v3 v2) {
  v3 diff = v3_sub(v2, v1);
  float scl = -(v3_dot(v2, diff) / v3_dot(diff, diff));
  return v3_make(v2.x + scl * diff.x, v2.y + scl * diff.y, v2.z + scl * diff.z);
}

__device__ __forceinline__ void g5_scale_plane(v3 n, float nv, float nn, v3* out) {
  float scl = nv / nn;
  *out = v3_make(scl * n.x, scl * n.y, scl * n.z);
}

// 返回 true 表示退化（与 host `project_origin_plane` 的 bool 一致）。
__device__ __forceinline__ bool g5_project_origin_plane(v3 v1, v3 v2, v3 v3c,
                                                       v3* out) {
  v3 zero = v3_make(0.0f, 0.0f, 0.0f);
  v3 diff21 = v3_sub(v2, v1);
  v3 diff31 = v3_sub(v3c, v1);
  v3 diff32 = v3_sub(v3c, v2);

  v3 n = v3_cross(diff32, diff21);
  float nv = v3_dot(n, v2);
  float nn = v3_dot(n, n);
  if (nn == 0.0f) {
    *out = zero;
    return true;
  }
  if (nv != 0.0f && nn > G5_MINVAL) {
    g5_scale_plane(n, nv, nn, out);
    return false;
  }

  n = v3_cross(diff21, diff31);
  nv = v3_dot(n, v1);
  nn = v3_dot(n, n);
  if (nn == 0.0f) {
    *out = zero;
    return true;
  }
  if (nv != 0.0f && nn > G5_MINVAL) {
    g5_scale_plane(n, nv, nn, out);
    return false;
  }

  n = v3_cross(diff31, diff32);
  nv = v3_dot(n, v3c);
  nn = v3_dot(n, n);
  g5_scale_plane(n, nv, nn, out);
  return false;
}

__device__ __forceinline__ void g5_s1d(v3 s1, v3 s2, float* out) {
  v3 p_o = g5_project_origin_line(s1, s2);
  float mu_max = s1.x - s2.x;
  int index = 0;
  float mu = s1.y - s2.y;
  if (fabsf(mu) >= fabsf(mu_max)) {
    mu_max = mu;
    index = 1;
  }
  mu = s1.z - s2.z;
  if (fabsf(mu) >= fabsf(mu_max)) {
    mu_max = mu;
    index = 2;
  }
  const float* po = reinterpret_cast<const float*>(&p_o.x);
  const float* a = reinterpret_cast<const float*>(&s1.x);
  const float* b = reinterpret_cast<const float*>(&s2.x);
  float c1 = po[index] - b[index];
  float c2 = a[index] - po[index];
  if (g5_same_sign(mu_max, c1) != 0 && g5_same_sign(mu_max, c2) != 0) {
    out[0] = c1 / mu_max;
    out[1] = c2 / mu_max;
    return;
  }
  out[0] = 0.0f;
  out[1] = 1.0f;
}

__device__ __forceinline__ void g5_s2d(v3 s1, v3 s2, v3 s3, float* out) {
  v3 p_o;
  bool degenerate = g5_project_origin_plane(s1, s2, s3, &p_o);
  if (degenerate) {
    float values[2];
    g5_s1d(s1, s2, values);
    out[0] = values[0];
    out[1] = values[1];
    out[2] = 0.0f;
    return;
  }
  float m_14 = s2.y * s3.z - s2.z * s3.y - s1.y * s3.z + s1.z * s3.y + s1.y * s2.z -
               s1.z * s2.y;
  float m_24 = s2.x * s3.z - s2.z * s3.x - s1.x * s3.z + s1.z * s3.x + s1.x * s2.z -
               s1.z * s2.x;
  float m_34 = s2.x * s3.y - s2.y * s3.x - s1.x * s3.y + s1.y * s3.x + s1.x * s2.y -
               s1.y * s2.x;
  float mu1 = fabsf(m_14);
  float mu2 = fabsf(m_24);
  float mu3 = fabsf(m_34);
  float m_max;
  int x;
  int y;
  if (mu1 >= mu2 && mu1 >= mu3) {
    m_max = m_14;
    x = 1;
    y = 2;
  } else if (mu2 >= mu3) {
    m_max = m_24;
    x = 0;
    y = 2;
  } else {
    m_max = m_34;
    x = 0;
    y = 1;
  }
  const float* s1v = reinterpret_cast<const float*>(&s1.x);
  const float* s2v = reinterpret_cast<const float*>(&s2.x);
  const float* s3v = reinterpret_cast<const float*>(&s3.x);
  const float* pov = reinterpret_cast<const float*>(&p_o.x);
  float s1_2d[2] = {s1v[x], s1v[y]};
  float s2_2d[2] = {s2v[x], s2v[y]};
  float s3_2d[2] = {s3v[x], s3v[y]};
  float p_o_2d[2] = {pov[x], pov[y]};
  float c31 = p_o_2d[0] * s2_2d[1] + p_o_2d[1] * s3_2d[0] + s2_2d[0] * s3_2d[1] -
              p_o_2d[0] * s3_2d[1] - p_o_2d[1] * s2_2d[0] - s3_2d[0] * s2_2d[1];
  float c32 = p_o_2d[0] * s3_2d[1] + p_o_2d[1] * s1_2d[0] + s3_2d[0] * s1_2d[1] -
              p_o_2d[0] * s1_2d[1] - p_o_2d[1] * s3_2d[0] - s1_2d[0] * s3_2d[1];
  float c33 = p_o_2d[0] * s1_2d[1] + p_o_2d[1] * s2_2d[0] + s1_2d[0] * s2_2d[1] -
              p_o_2d[0] * s2_2d[1] - p_o_2d[1] * s1_2d[0] - s2_2d[0] * s1_2d[1];
  int comp1 = g5_same_sign(m_max, c31);
  int comp2 = g5_same_sign(m_max, c32);
  int comp3 = g5_same_sign(m_max, c33);
  if (comp1 != 0 && comp2 != 0 && comp3 != 0) {
    out[0] = c31 / m_max;
    out[1] = c32 / m_max;
    out[2] = c33 / m_max;
    return;
  }
  float dmin = G5_FLOAT_MAX;
  out[0] = 0.0f;
  out[1] = 0.0f;
  out[2] = 0.0f;
  if (comp1 == 0) {
    float sub[2];
    g5_s1d(s2, s3, sub);
    v3 x = v3_make(sub[0] * s2.x + sub[1] * s3.x, sub[0] * s2.y + sub[1] * s3.y,
                   sub[0] * s2.z + sub[1] * s3.z);
    float d = v3_dot(x, x);
    out[0] = 0.0f;
    out[1] = sub[0];
    out[2] = sub[1];
    dmin = d;
  }
  if (comp2 == 0) {
    float sub[2];
    g5_s1d(s1, s3, sub);
    v3 x = v3_make(sub[0] * s1.x + sub[1] * s3.x, sub[0] * s1.y + sub[1] * s3.y,
                   sub[0] * s1.z + sub[1] * s3.z);
    float d = v3_dot(x, x);
    if (d < dmin) {
      out[0] = sub[0];
      out[1] = 0.0f;
      out[2] = sub[1];
      dmin = d;
    }
  }
  if (comp3 == 0) {
    float sub[2];
    g5_s1d(s1, s2, sub);
    v3 x = v3_make(sub[0] * s1.x + sub[1] * s2.x, sub[0] * s1.y + sub[1] * s2.y,
                   sub[0] * s1.z + sub[1] * s2.z);
    float d = v3_dot(x, x);
    if (d < dmin) {
      out[0] = sub[0];
      out[1] = sub[1];
      out[2] = 0.0f;
    }
  }
}

__device__ __forceinline__ void g5_s3d(v3 s1, v3 s2, v3 s3, v3 s4, float* out) {
  float c41 = -g5_det3(s2, s3, s4);
  float c42 = g5_det3(s1, s3, s4);
  float c43 = -g5_det3(s1, s2, s4);
  float c44 = g5_det3(s1, s2, s3);
  float m_det = c41 + c42 + c43 + c44;
  int comp1 = g5_same_sign(m_det, c41);
  int comp2 = g5_same_sign(m_det, c42);
  int comp3 = g5_same_sign(m_det, c43);
  int comp4 = g5_same_sign(m_det, c44);
  if (comp1 != 0 && comp2 != 0 && comp3 != 0 && comp4 != 0) {
    out[0] = c41 / m_det;
    out[1] = c42 / m_det;
    out[2] = c43 / m_det;
    out[3] = c44 / m_det;
    return;
  }
  float lmbda[4] = {0.0f, 0.0f, 0.0f, 0.0f};
  float dmin = G5_FLOAT_MAX;
  if (comp1 == 0) {
    float sub[3];
    g5_s2d(s2, s3, s4, sub);
    v3 x = v3_make(sub[0] * s2.x + sub[1] * s3.x + sub[2] * s4.x,
                   sub[0] * s2.y + sub[1] * s3.y + sub[2] * s4.y,
                   sub[0] * s2.z + sub[1] * s3.z + sub[2] * s4.z);
    float d = v3_dot(x, x);
    lmbda[0] = 0.0f;
    lmbda[1] = sub[0];
    lmbda[2] = sub[1];
    lmbda[3] = sub[2];
    dmin = d;
  }
  if (comp2 == 0) {
    float sub[3];
    g5_s2d(s1, s3, s4, sub);
    v3 x = v3_make(sub[0] * s1.x + sub[1] * s3.x + sub[2] * s4.x,
                   sub[0] * s1.y + sub[1] * s3.y + sub[2] * s4.y,
                   sub[0] * s1.z + sub[1] * s3.z + sub[2] * s4.z);
    float d = v3_dot(x, x);
    if (d < dmin) {
      lmbda[0] = sub[0];
      lmbda[1] = 0.0f;
      lmbda[2] = sub[1];
      lmbda[3] = sub[2];
      dmin = d;
    }
  }
  if (comp3 == 0) {
    float sub[3];
    g5_s2d(s1, s2, s4, sub);
    v3 x = v3_make(sub[0] * s1.x + sub[1] * s2.x + sub[2] * s4.x,
                   sub[0] * s1.y + sub[1] * s2.y + sub[2] * s4.y,
                   sub[0] * s1.z + sub[1] * s2.z + sub[2] * s4.z);
    float d = v3_dot(x, x);
    if (d < dmin) {
      lmbda[0] = sub[0];
      lmbda[1] = sub[1];
      lmbda[2] = 0.0f;
      lmbda[3] = sub[2];
      dmin = d;
    }
  }
  if (comp4 == 0) {
    float sub[3];
    g5_s2d(s1, s2, s3, sub);
    v3 x = v3_make(sub[0] * s1.x + sub[1] * s2.x + sub[2] * s3.x,
                   sub[0] * s1.y + sub[1] * s2.y + sub[2] * s3.y,
                   sub[0] * s1.z + sub[1] * s2.z + sub[2] * s3.z);
    float d = v3_dot(x, x);
    if (d < dmin) {
      lmbda[0] = sub[0];
      lmbda[1] = sub[1];
      lmbda[2] = sub[2];
      lmbda[3] = 0.0f;
    }
  }
  out[0] = lmbda[0];
  out[1] = lmbda[1];
  out[2] = lmbda[2];
  out[3] = lmbda[3];
}

__device__ __forceinline__ v3 g5_linear_combine(int n, const float* scl,
                                                const v3* matrix) {
  if (n == 1) {
    return v3_make(scl[0] * matrix[0].x, scl[0] * matrix[0].y, scl[0] * matrix[0].z);
  }
  if (n == 2) {
    return v3_make(scl[0] * matrix[0].x + scl[1] * matrix[1].x,
                   scl[0] * matrix[0].y + scl[1] * matrix[1].y,
                   scl[0] * matrix[0].z + scl[1] * matrix[1].z);
  }
  if (n == 3) {
    return v3_make(
        scl[0] * matrix[0].x + scl[1] * matrix[1].x + scl[2] * matrix[2].x,
        scl[0] * matrix[0].y + scl[1] * matrix[1].y + scl[2] * matrix[2].y,
        scl[0] * matrix[0].z + scl[1] * matrix[1].z + scl[2] * matrix[2].z);
  }
  return v3_make(scl[0] * matrix[0].x + scl[1] * matrix[1].x + scl[2] * matrix[2].x +
                     scl[3] * matrix[3].x,
                 scl[0] * matrix[0].y + scl[1] * matrix[1].y + scl[2] * matrix[2].y +
                     scl[3] * matrix[3].y,
                 scl[0] * matrix[0].z + scl[1] * matrix[1].z + scl[2] * matrix[2].z +
                     scl[3] * matrix[3].z);
}

__device__ __forceinline__ void g5_subdistance(int n, const v3* simplex,
                                               float* out) {
  out[0] = 1.0f;
  out[1] = 0.0f;
  out[2] = 0.0f;
  out[3] = 0.0f;
  if (n == 4) {
    g5_s3d(simplex[0], simplex[1], simplex[2], simplex[3], out);
  } else if (n == 3) {
    float values[3];
    g5_s2d(simplex[0], simplex[1], simplex[2], values);
    out[0] = values[0];
    out[1] = values[1];
    out[2] = values[2];
  } else if (n == 2) {
    float values[2];
    g5_s1d(simplex[0], simplex[1], values);
    out[0] = values[0];
    out[1] = values[1];
  }
}

__device__ __forceinline__ void g5_tri_affine_coord(v3 v1, v3 v2, v3 v3c, v3 p,
                                                    float* out) {
  float m_14 = v2.y * v3c.z - v2.z * v3c.y - v1.y * v3c.z + v1.z * v3c.y + v1.y * v2.z -
               v1.z * v2.y;
  float m_24 = v2.x * v3c.z - v2.z * v3c.x - v1.x * v3c.z + v1.z * v3c.x + v1.x * v2.z -
               v1.z * v2.x;
  float m_34 = v2.x * v3c.y - v2.y * v3c.x - v1.x * v3c.y + v1.y * v3c.x + v1.x * v2.y -
               v1.y * v2.x;
  float mu1 = fabsf(m_14);
  float mu2 = fabsf(m_24);
  float mu3 = fabsf(m_34);
  float m_max;
  int x;
  int y;
  if (mu1 >= mu2 && mu1 >= mu3) {
    m_max = m_14;
    x = 1;
    y = 2;
  } else if (mu2 >= mu3) {
    m_max = m_24;
    x = 0;
    y = 2;
  } else {
    m_max = m_34;
    x = 0;
    y = 1;
  }
  const float* pv = reinterpret_cast<const float*>(&p.x);
  const float* v1v = reinterpret_cast<const float*>(&v1.x);
  const float* v2v = reinterpret_cast<const float*>(&v2.x);
  const float* v3v = reinterpret_cast<const float*>(&v3c.x);
  float c31 = pv[x] * v2v[y] + pv[y] * v3v[x] + v2v[x] * v3v[y] - pv[x] * v3v[y] -
              pv[y] * v2v[x] - v3v[x] * v2v[y];
  float c32 = pv[x] * v3v[y] + pv[y] * v1v[x] + v3v[x] * v1v[y] - pv[x] * v1v[y] -
              pv[y] * v3v[x] - v1v[x] * v3v[y];
  float c33 = pv[x] * v1v[y] + pv[y] * v2v[x] + v1v[x] * v2v[y] - pv[x] * v2v[y] -
              pv[y] * v1v[x] - v2v[x] * v1v[y];
  out[0] = c31 / m_max;
  out[1] = c32 / m_max;
  out[2] = c33 / m_max;
}

__device__ __forceinline__ bool g5_tri_point_intersect(v3 v1, v3 v2, v3 v3c, v3 p) {
  float coordinates[3];
  g5_tri_affine_coord(v1, v2, v3c, p, coordinates);
  float l1 = coordinates[0];
  float l2 = coordinates[1];
  float l3 = coordinates[2];
  if (l1 < 0.0f || l2 < 0.0f || l3 < 0.0f) return false;
  v3 pr = v3_make(v1.x * l1 + v2.x * l2 + v3c.x * l3,
                  v1.y * l1 + v2.y * l2 + v3c.y * l3,
                  v1.z * l1 + v2.z * l2 + v3c.z * l3);
  return v3_length(v3_sub(pr, p)) < G5_MINVAL;
}

__device__ __forceinline__ bool g5_same_side(v3 p0, v3 p1, v3 p2, v3 p3) {
  v3 n = v3_cross(v3_sub(p1, p0), v3_sub(p2, p0));
  float dot1 = v3_dot(n, v3_sub(p3, p0));
  float dot2 = v3_dot(n, v3_neg(p0));
  return (dot1 > 0.0f && dot2 > 0.0f) || (dot1 < 0.0f && dot2 < 0.0f);
}

__device__ __forceinline__ bool g5_test_tetra(v3 p0, v3 p1, v3 p2, v3 p3) {
  return g5_same_side(p0, p1, p2, p3) && g5_same_side(p1, p2, p3, p0) &&
         g5_same_side(p2, p3, p0, p1) && g5_same_side(p3, p0, p1, p2);
}
