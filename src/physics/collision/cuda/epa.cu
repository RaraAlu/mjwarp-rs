// G10@P5 G5：EPA 迭代设备版，与 host `gjk/epa.rs` 逐式对应。
// Adapted from frozen MJWarp collision_gjk.py, revision 71da24d. Apache-2.0.
#pragma once
#include "polytope_build.cu"

#define G5_MIN_EPATOL 1.0e-7f
#define G5_OVERFLOW_EPA_HORIZON (1u << 8)

struct d_epa_result {
  float dist;
  v3 x1;
  v3 x2;
  int face_idx;
  unsigned overflow;
};

__device__ __forceinline__ void g5_get_edge(int edge, int* out) {
  out[0] = edge & 0x3FF;
  out[1] = (edge >> 10) & 0x3FF;
}

// 上游 `_add_edge`：重复边与末元素交换移除；返回新计数，-1 表示失败。
__device__ __forceinline__ int g5_add_edge(d_polytope* pt, int e1, int e2) {
  int n = pt->nhorizon;
  if (n < 0) return -1;
  int low = e1 < e2 ? e1 : e2;
  int high = e1 < e2 ? e2 : e1;
  int edge = (low << 10) | high;
  for (int i = 0; i < n; ++i) {
    if (edge == pt->horizon[i]) {
      pt->horizon[i] = pt->horizon[n - 1];
      return n - 1;
    }
  }
  if (n == pt->max_horizon) return -1;
  pt->horizon[n] = edge;
  return n + 1;
}

// 上游 `_epa()`：从初始 polytope 求穿透距离与 witness；polytope 原地更新。
__device__ __forceinline__ d_epa_result g5_epa(d_polytope* pt, float tolerance,
                                               int epa_iterations, const d_geom& geom1,
                                               const d_geom& geom2, int geomtype1,
                                               int geomtype2, bool is_discrete) {
  float upper = G5_FLOAT_MAX;
  float upper2 = G5_FLOAT_MAX;
  int idx = -1;
  int pidx = -1;
  unsigned overflow = 0u;
  float epsilon = is_discrete ? G5_MIN_EPATOL : tolerance;
  int nvalid = pt->nface;
  if (epa_iterations > 1000) epa_iterations = 1000;

  for (int iteration = 0; iteration < epa_iterations; ++iteration) {
    pidx = idx;
    idx = -1;
    float lower2 = G5_FLOAT_MAX;

    // 找离原点最近的面（穿透深度下界）。
    for (int i = 0; i < pt->nface; ++i) {
      if (!g5_is_invalid_face(pt->face[i]) && pt->face_norm2[i] < lower2) {
        idx = i;
        lower2 = pt->face_norm2[i];
      }
    }
    // 无有效面：回退到上一面。
    if (lower2 > upper2 || idx < 0) {
      idx = pidx;
      break;
    }
    if (lower2 <= 0.0f) break;

    // 由最近面的法线取支撑点 w。
    float lower = sqrtf(lower2);
    int wi = pt->nvert;
    v3 face_pr = pt->face_pr[idx];
    v3 dir = v3_make(face_pr.x / lower, face_pr.y / lower, face_pr.z / lower);
    int support_index1 = 0;
    int support_index2 = 0;
    g5_epa_support(pt, wi, geom1, geom2, geomtype1, geomtype2, dir, &support_index1,
                   &support_index2);
    v3 w = v3_sub(pt->vert[2 * wi], pt->vert[2 * wi + 1]);
    pt->nvert += 1;

    // 第 k 次迭代上界（归一化前点积，精度更好）。
    float upper_k = v3_dot(face_pr, w) / lower;
    if (upper_k < upper) {
      upper = upper_k;
      upper2 = upper * upper;
    }
    if (upper - lower < epsilon) break;

    // 检查 wi 是否为重复支撑点（离散几何提前终止）。
    if (is_discrete) {
      bool found_repeated = false;
      for (int i = 0; i < pt->nvert - 1; ++i) {
        if (pt->vert_index[2 * i] == pt->vert_index[2 * wi] &&
            pt->vert_index[2 * i + 1] == pt->vert_index[2 * wi + 1]) {
          found_repeated = true;
          break;
        }
      }
      if (found_repeated) break;
    }

    nvalid -= 1;
    pt->face[idx] = g5_delete_face(pt->face[idx]);
    int face[3];
    g5_get_face_verts(pt->face[idx], face);
    pt->nhorizon = g5_add_edge(pt, face[0], face[1]);
    pt->nhorizon = g5_add_edge(pt, face[1], face[2]);
    pt->nhorizon = g5_add_edge(pt, face[2], face[0]);
    if (pt->nhorizon == -1) {
      overflow |= G5_OVERFLOW_EPA_HORIZON;
      idx = -1;
      break;
    }

    // 计算 w 的 horizon。
    for (int i = 0; i < pt->nface; ++i) {
      if (g5_is_face_deleted(pt->face[i])) continue;
      if (v3_dot(pt->face_pr[i], w) - pt->face_norm2[i] > 1e-10f) {
        if (!g5_is_invalid_face(pt->face[i])) nvalid -= 1;
        pt->face[i] = g5_delete_face(pt->face[i]);
        int inner[3];
        g5_get_face_verts(pt->face[i], inner);
        pt->nhorizon = g5_add_edge(pt, inner[0], inner[1]);
        pt->nhorizon = g5_add_edge(pt, inner[1], inner[2]);
        pt->nhorizon = g5_add_edge(pt, inner[2], inner[0]);
        if (pt->nhorizon == -1) {
          overflow |= G5_OVERFLOW_EPA_HORIZON;
          idx = -1;
          break;
        }
      }
    }

    // 插入新顶点并沿 horizon 附面。
    int i = 0;
    while (i < pt->nhorizon) {
      int edge[2];
      g5_get_edge(pt->horizon[i], edge);
      int nface = pt->nface;
      float dist2 = g5_attach_face(pt, nface, wi, edge[0], edge[1]);
      if (dist2 == 0.0f) {
        idx = -1;
        break;
      }
      pt->nface += 1;
      if (dist2 >= lower2 && dist2 <= upper2) {
        nvalid += 1;
      } else {
        pt->face[pt->nface - 1] = g5_invalidate_face(pt->face[pt->nface - 1]);
      }
      i += 1;
    }
    if (nvalid == 0 || idx == -1) break;
    pt->nhorizon = 0;
  }

  d_epa_result result;
  result.overflow = overflow;
  if (idx > -1) {
    v3 x1;
    v3 x2;
    float dist;
    g5_epa_witness(pt, geom1, geom2, geomtype1, geomtype2, idx, &x1, &x2, &dist);
    result.dist = dist;
    result.x1 = x1;
    result.x2 = x2;
    result.face_idx = idx;
    return result;
  }
  result.dist = 0.0f;
  result.x1 = v3_make(0.0f, 0.0f, 0.0f);
  result.x2 = v3_make(0.0f, 0.0f, 0.0f);
  result.face_idx = -1;
  return result;
}
