// G10@P5 G5：GJK 主循环设备版，与 host `gjk/core.rs` 逐式对应。
// Adapted from frozen MJWarp collision_gjk.py, revision 71da24d. Apache-2.0.
#pragma once
#include "support.cu"
#include "simplex.cu"

#define G5_FLOAT_MAX 1.0e30f
#define G5_MINVAL2 1.0e-30f

struct d_gjk_result {
  bool separated;
  float dist;
  v3 x1;
  v3 x2;
  int dim;
  v3 simplex[4];
  v3 simplex1[4];
  v3 simplex2[4];
  int simplex_index1[4];
  int simplex_index2[4];
  int index1;
  int index2;
};

__device__ __forceinline__ d_gjk_result g5_gjk_result_zero() {
  d_gjk_result result;
  result.separated = false;
  result.dist = 0.0f;
  result.x1 = v3_make(0.0f, 0.0f, 0.0f);
  result.x2 = v3_make(0.0f, 0.0f, 0.0f);
  result.dim = 0;
  for (int i = 0; i < 4; ++i) {
    result.simplex[i] = v3_make(0.0f, 0.0f, 0.0f);
    result.simplex1[i] = v3_make(0.0f, 0.0f, 0.0f);
    result.simplex2[i] = v3_make(0.0f, 0.0f, 0.0f);
    result.simplex_index1[i] = 0;
    result.simplex_index2[i] = 0;
  }
  result.index1 = 0;
  result.index2 = 0;
  return result;
}

// 上游 `_gjk_support`：按当前 `x_k` 取两侧支撑点，离散几何做方向调优。
__device__ __forceinline__ void g5_gjk_support(const d_geom& geom1, const d_geom& geom2,
                                               int geomtype1, int geomtype2, v3 x_k,
                                               float x_norm, const v3* simplex, int n,
                                               bool is_discrete, d_support_point* sp1,
                                               d_support_point* sp2) {
  v3 dir_neg = v3_make(x_k.x / x_norm, x_k.y / x_norm, x_k.z / x_norm);
  if (is_discrete && x_norm < 1e-4f) {
    if (n == 2) {
      v3 edge = v3_sub(simplex[1], simplex[0]);
      float edge_norm2 = v3_dot(edge, edge);
      if (edge_norm2 > G5_MINVAL2) {
        float proj = v3_dot(dir_neg, edge) / edge_norm2;
        dir_neg = v3_sub(dir_neg, v3_scale(edge, proj));
        float dir_norm = v3_length(dir_neg);
        if (dir_norm > G5_MINVAL) {
          dir_neg = v3_make(dir_neg.x / dir_norm, dir_neg.y / dir_norm, dir_neg.z / dir_norm);
        }
      }
    } else if (n == 3) {
      v3 e1 = v3_sub(simplex[1], simplex[0]);
      v3 e2 = v3_sub(simplex[2], simplex[0]);
      v3 normal = v3_cross(e1, e2);
      float normal_norm = v3_length(normal);
      if (normal_norm > G5_MINVAL) {
        float sign = v3_dot(dir_neg, normal) < 0.0f ? -1.0f : 1.0f;
        dir_neg = v3_make(sign * normal.x / normal_norm, sign * normal.y / normal_norm,
                          sign * normal.z / normal_norm);
      }
    }
  }
  *sp1 = support(geom1, geomtype1, v3_neg(dir_neg));
  *sp2 = support(geom2, geomtype2, dir_neg);
}

// 上游 `gjk()`：两凸几何距离；返回结构与 host `GjkResult` 对应。
__device__ __forceinline__ d_gjk_result g5_gjk(float tolerance, int gjk_iterations,
                                               d_geom geom1, d_geom geom2, v3 x1_0,
                                               v3 x2_0, int geomtype1, int geomtype2,
                                               float cutoff, bool is_discrete) {
  d_gjk_result result = g5_gjk_result_zero();
  v3 simplex[4];
  v3 simplex1[4];
  v3 simplex2[4];
  int simplex_index1[4];
  int simplex_index2[4];
  for (int i = 0; i < 4; ++i) {
    simplex[i] = v3_make(0.0f, 0.0f, 0.0f);
    simplex1[i] = v3_make(0.0f, 0.0f, 0.0f);
    simplex2[i] = v3_make(0.0f, 0.0f, 0.0f);
    simplex_index1[i] = 0;
    simplex_index2[i] = 0;
  }
  int n = 0;
  float lmbda[4] = {1.0f, 0.0f, 0.0f, 0.0f};

  float epsilon = is_discrete ? 0.0f : 0.5f * tolerance * tolerance;
  float min_norm = is_discrete ? G5_MINVAL : tolerance;

  v3 x_k = v3_sub(x1_0, x2_0);
  float xnorm = sqrtf(v3_dot(x_k, x_k));
  float xnorm_prev = 0.0f;

  for (int step = 0; step < gjk_iterations; ++step) {
    if (xnorm < min_norm || fabsf(xnorm_prev - xnorm) < G5_MINVAL) break;

    d_support_point sp1;
    d_support_point sp2;
    g5_gjk_support(geom1, geom2, geomtype1, geomtype2, x_k, xnorm, simplex, n,
                   is_discrete, &sp1, &sp2);
    simplex1[n] = sp1.point;
    geom1.index = sp1.cached_index;
    simplex_index1[n] = sp1.vertex_index;
    simplex2[n] = sp2.point;
    geom2.index = sp2.cached_index;
    simplex_index2[n] = sp2.vertex_index;
    simplex[n] = v3_sub(simplex1[n], simplex2[n]);

    // Frank-Wolfe 对偶间隙停止判据。
    if (v3_dot(x_k, v3_sub(x_k, simplex[n])) < epsilon) break;

    // 距离下界：lower > 0 表示分离。
    float lower = v3_dot(x_k, simplex[n]);
    if (cutoff == 0.0f) {
      if (lower > 0.0f) {
        result.separated = true;
        result.dim = 0;
        result.dist = G5_FLOAT_MAX;
        result.index1 = geom1.index;
        result.index2 = geom2.index;
        return result;
      }
    } else if (cutoff < G5_FLOAT_MAX && lower > 0.0f && lower >= cutoff * xnorm) {
      result.separated = true;
      result.dim = 0;
      result.dist = G5_FLOAT_MAX;
      result.index1 = geom1.index;
      result.index2 = geom2.index;
      return result;
    }

    // 距离子算法：simplex 中离原点最近点的重心坐标。
    g5_subdistance(n + 1, simplex, lmbda);

    // 剔除不再需要的顶点。
    n = 0;
    for (int i = 0; i < 4; ++i) {
      if (lmbda[i] == 0.0f) continue;
      simplex[n] = simplex[i];
      simplex1[n] = simplex1[i];
      simplex2[n] = simplex2[i];
      simplex_index1[n] = simplex_index1[i];
      simplex_index2[n] = simplex_index2[i];
      lmbda[n] = lmbda[i];
      n += 1;
    }
    if (n < 1) break;

    x_k = g5_linear_combine(n, lmbda, simplex);
    xnorm_prev = xnorm;
    xnorm = sqrtf(v3_dot(x_k, x_k));
    if (n == 4) break;
  }

  result.separated = false;
  if (n == 0) {
    result.x1 = x1_0;
    result.x2 = x2_0;
  } else {
    result.x1 = g5_linear_combine(n, lmbda, simplex1);
    result.x2 = g5_linear_combine(n, lmbda, simplex2);
  }
  if (xnorm > 0.0f) {
    v3 dir = v3_make(x_k.x / xnorm, x_k.y / xnorm, x_k.z / xnorm);
    d_support_point sp1 = support(geom1, geomtype1, v3_neg(dir));
    d_support_point sp2 = support(geom2, geomtype2, dir);
    result.separated = v3_dot(x_k, v3_sub(sp1.point, sp2.point)) > 0.0f;
  }
  result.dist = (n == 4 && !result.separated) ? 0.0f : xnorm;
  result.dim = n;
  for (int i = 0; i < 4; ++i) {
    result.simplex[i] = simplex[i];
    result.simplex1[i] = simplex1[i];
    result.simplex2[i] = simplex2[i];
    result.simplex_index1[i] = simplex_index1[i];
    result.simplex_index2[i] = simplex_index2[i];
  }
  result.index1 = geom1.index;
  result.index2 = geom2.index;
  return result;
}

// 上游 `_gjk_support` 的离散几何判定（供 `phase` 复用）。
__device__ __forceinline__ bool g5_is_discrete(int geomtype1, int geomtype2, float margin1,
                                               float margin2) {
  return discrete_geoms(geomtype1, geomtype2) && margin1 == 0.0f && margin2 == 0.0f;
}
