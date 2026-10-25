// G10@P5 G5：ccd 两阶段驱动设备版，与 host `gjk/phase.rs` + `gjk/ccd.rs` 逐式对应。
// Adapted from frozen MJWarp collision_gjk.py, revision 71da24d. Apache-2.0.
#pragma once
#include "epa.cu"

#define G5_OVERFLOW_CCD (1u << 4)

struct d_ccd_options {
  float tolerance;
  float cutoff;
  int gjk_iterations;
  int epa_iterations;
};

struct d_ccd_result {
  float dist;
  int ncontact;
  v3 x1;
  v3 x2;
  int multiccd_idx;
  unsigned overflow;
};

struct d_gjk_phase {
  bool needs_epa;
  float dist;
  int ncontact;
  v3 x1;
  v3 x2;
  d_gjk_result result;
  d_geom geom1;
  d_geom geom2;
};

struct d_epa_phase {
  float dist;
  int ncontact;
  v3 x1;
  v3 x2;
  int multiccd_idx;
  bool has_polytope;
  unsigned overflow;
};

// 上游 `_inflate`：浅穿透把 witness 沿法线膨胀回真实表面。
__device__ __forceinline__ void g5_inflate(const d_gjk_result* result, const d_geom& geom1,
                                           const d_geom& geom2, int geomtype1,
                                           int geomtype2, float margin1, float margin2,
                                           float* out_dist, v3* out_x1, v3* out_x2) {
  float dist = result->dist;
  v3 x1 = result->x1;
  v3 x2 = result->x2;
  if (geomtype1 == G5_HFIELD) {
    int v = result->simplex_index1[0];
    bool is_side = false;
    for (int i = 0; i < result->dim; ++i) {
      if (result->simplex_index1[i] != v) {
        is_side = true;
        break;
      }
    }
    if (is_side) {
      v3 n = v3_make(0.0f, 0.0f, 1.0f);
      d_support_point sp = support(geom2, geomtype2, x2);
      x2 = v3_sub(sp.point, v3_scale(n, margin2));
      v3 a = v3_make(geom1.polyvert[9], geom1.polyvert[10], geom1.polyvert[11]);
      v3 b = v3_make(geom1.polyvert[12], geom1.polyvert[13], geom1.polyvert[14]);
      v3 c = v3_make(geom1.polyvert[15], geom1.polyvert[16], geom1.polyvert[17]);
      float coordinates[3];
      g5_tri_affine_coord(a, b, c, x2, coordinates);
      if (coordinates[0] > 0.0f && coordinates[1] > 0.0f && coordinates[2] > 0.0f) {
        x1 = v3_make(coordinates[0] * a.x + coordinates[1] * b.x + coordinates[2] * c.x,
                     coordinates[0] * a.y + coordinates[1] * b.y + coordinates[2] * c.y,
                     coordinates[0] * a.z + coordinates[1] * b.z + coordinates[2] * c.z);
      } else {
        v3 p = c;
        if (coordinates[1] > 0.0f) p = b;
        if (coordinates[0] > 0.0f) p = a;
        float d = v3_dot(v3_sub(x2, p), n);
        x1 = v3_make(x2.x - d * n.x, x2.y - d * n.y, x2.z - d * n.z);
      }
      dist = -v3_length(v3_sub(x1, x2));
      *out_dist = dist;
      *out_x1 = x1;
      *out_x2 = x2;
      return;
    }
  }
  v3 n = v3_normalize(v3_sub(x2, x1));
  if (margin1 > 0.0f) x1 = v3_add(x1, v3_scale(n, margin1));
  if (margin2 > 0.0f) x2 = v3_sub(x2, v3_scale(n, margin2));
  dist -= margin1 + margin2;
  *out_dist = dist;
  *out_x1 = x1;
  *out_x2 = x2;
}

// 上游 `gjk_phase`。
__device__ __forceinline__ void g5_gjk_phase(float tolerance, float cutoff,
                                             int gjk_iterations, d_geom geom1, d_geom geom2,
                                             int geomtype1, int geomtype2, v3 x_1, v3 x_2,
                                             d_gjk_phase* out) {
  float orig_margin1 = geom1.margin;
  float orig_margin2 = geom2.margin;
  v3 orig_size1 = geom1.size;
  v3 orig_size2 = geom2.size;
  float full_margin1 = 0.0f;
  float full_margin2 = 0.0f;
  float size1 = 0.0f;
  float size2 = 0.0f;
  d_gjk_result empty = g5_gjk_result_zero();
  bool is_discrete = g5_is_discrete(geomtype1, geomtype2, geom1.margin, geom2.margin);

  if (geomtype1 == G5_SPHERE || geomtype1 == G5_CAPSULE) {
    size1 = geom1.size.x;
    full_margin1 = size1 + 0.5f * orig_margin1;
    geom1.margin = 0.0f;
    geom1.size = v3_make(0.0f, geom1.size.y, geom1.size.z);
  }
  if (geomtype2 == G5_SPHERE || geomtype2 == G5_CAPSULE) {
    size2 = geom2.size.x;
    full_margin2 = size2 + 0.5f * orig_margin2;
    geom2.margin = 0.0f;
    geom2.size = v3_make(0.0f, geom2.size.y, geom2.size.z);
  }

  if (size1 + size2 > 0.0f) {
    cutoff += full_margin1 + full_margin2;
    d_gjk_result result =
        g5_gjk(tolerance, gjk_iterations, geom1, geom2, x_1, x_2, geomtype1, geomtype2,
               cutoff, is_discrete);
    geom1.index = result.index1;
    geom2.index = result.index2;
    if (result.dist > tolerance) {
      out->needs_epa = false;
      out->ncontact = 1;
      out->result = empty;
      out->geom1 = geom1;
      out->geom2 = geom2;
      if (result.dist == G5_FLOAT_MAX) {
        out->dist = result.dist;
        out->x1 = result.x1;
        out->x2 = result.x2;
        return;
      }
      g5_inflate(&result, geom1, geom2, geomtype1, geomtype2, full_margin1, full_margin2,
                 &out->dist, &out->x1, &out->x2);
      return;
    }
    geom1.margin = orig_margin1;
    geom1.size = orig_size1;
    geom2.margin = orig_margin2;
    geom2.size = orig_size2;
    cutoff -= full_margin1 + full_margin2;
  }

  d_gjk_result result =
      g5_gjk(tolerance, gjk_iterations, geom1, geom2, x_1, x_2, geomtype1, geomtype2, cutoff,
             is_discrete);
  geom1.index = result.index1;
  geom2.index = result.index2;
  out->geom1 = geom1;
  out->geom2 = geom2;
  if (result.dist > tolerance || result.dim < 2 || result.separated) {
    out->needs_epa = false;
    out->dist = result.dist;
    out->ncontact = 1;
    out->x1 = result.x1;
    out->x2 = result.x2;
    out->result = empty;
    return;
  }
  out->needs_epa = true;
  out->dist = result.dist;
  out->ncontact = 1;
  out->x1 = result.x1;
  out->x2 = result.x2;
  out->result = result;
}

// 上游 `epa_phase`：调用方提供已清零计数的 polytope 缓冲。
__device__ __forceinline__ void g5_epa_phase(float tolerance, int epa_iterations,
                                             d_gjk_result result, const d_geom& geom1,
                                             const d_geom& geom2, int geomtype1,
                                             int geomtype2, d_polytope* pt,
                                             d_epa_phase* out) {
  d_gjk_result next = g5_gjk_result_zero();
  pt->nvert = 0;
  pt->nface = 0;
  pt->nhorizon = 0;
  pt->status = 0;
  pt->horizon[0] = 0;
  if (result.dim == 2) {
    g5_polytope2(pt, result.simplex, result.simplex1, result.simplex2, result.simplex_index1,
                 result.simplex_index2, geom1, geom2, geomtype1, geomtype2, &next);
    if (pt->status == -1) {
      result.simplex[0] = next.simplex[0];
      result.simplex[1] = next.simplex[1];
      result.simplex[2] = next.simplex[2];
      result.simplex1[0] = next.simplex1[0];
      result.simplex1[1] = next.simplex1[1];
      result.simplex1[2] = next.simplex1[2];
      result.simplex2[0] = next.simplex2[0];
      result.simplex2[1] = next.simplex2[1];
      result.simplex2[2] = next.simplex2[2];
      result.simplex_index1[0] = next.simplex_index1[0];
      result.simplex_index1[1] = next.simplex_index1[1];
      result.simplex_index1[2] = next.simplex_index1[2];
      result.simplex_index2[0] = next.simplex_index2[0];
      result.simplex_index2[1] = next.simplex_index2[1];
      result.simplex_index2[2] = next.simplex_index2[2];
      result.dim = 3;
    }
  } else if (result.dim == 4) {
    g5_polytope4(pt, result.simplex, result.simplex1, result.simplex2, result.simplex_index1,
                 result.simplex_index2, &next);
    if (pt->status == -1) {
      result.simplex[0] = next.simplex[0];
      result.simplex[1] = next.simplex[1];
      result.simplex[2] = next.simplex[2];
      result.simplex1[0] = next.simplex1[0];
      result.simplex1[1] = next.simplex1[1];
      result.simplex1[2] = next.simplex1[2];
      result.simplex2[0] = next.simplex2[0];
      result.simplex2[1] = next.simplex2[1];
      result.simplex2[2] = next.simplex2[2];
      result.simplex_index1[0] = next.simplex_index1[0];
      result.simplex_index1[1] = next.simplex_index1[1];
      result.simplex_index1[2] = next.simplex_index1[2];
      result.simplex_index2[0] = next.simplex_index2[0];
      result.simplex_index2[1] = next.simplex_index2[1];
      result.simplex_index2[2] = next.simplex_index2[2];
      result.dim = 3;
    }
  }
  if (result.dim == 3) {
    g5_polytope3(pt, result.dist, result.simplex, result.simplex1, result.simplex2,
                 result.simplex_index1, result.simplex_index2, geom1, geom2, geomtype1,
                 geomtype2);
  }

  // 原点在边界上：不算穿透。
  if (pt->status != 0) {
    out->dist = result.dist;
    out->ncontact = 1;
    out->x1 = result.x1;
    out->x2 = result.x2;
    out->multiccd_idx = -1;
    out->has_polytope = false;
    out->overflow = 0;
    return;
  }

  bool is_discrete = g5_is_discrete(geomtype1, geomtype2, geom1.margin, geom2.margin);
  d_epa_result epa_result =
      g5_epa(pt, tolerance, epa_iterations, geom1, geom2, geomtype1, geomtype2, is_discrete);
  if (epa_result.face_idx == -1) {
    out->dist = G5_FLOAT_MAX;
    out->ncontact = 0;
    out->x1 = v3_make(0.0f, 0.0f, 0.0f);
    out->x2 = v3_make(0.0f, 0.0f, 0.0f);
    out->multiccd_idx = -1;
    out->has_polytope = false;
    out->overflow = epa_result.overflow;
    return;
  }
  int multiccd_idx = epa_result.face_idx;
  if (geom1.margin != 0.0f || geom2.margin != 0.0f) multiccd_idx = -1;
  bool supported = (geomtype1 == G5_BOX || geomtype1 == G5_MESH ||
                    geomtype1 == G5_CYLINDER) &&
                   (geomtype2 == G5_BOX || geomtype2 == G5_MESH ||
                    geomtype2 == G5_CYLINDER);
  if (!supported) multiccd_idx = -1;
  out->dist = epa_result.dist;
  out->ncontact = 1;
  out->x1 = epa_result.x1;
  out->x2 = epa_result.x2;
  out->multiccd_idx = multiccd_idx;
  out->has_polytope = true;
  out->overflow = epa_result.overflow;
}

// 上游 `ccd()`：通用凸碰撞入口；polytope 缓冲由调用方按 epa_iterations 分配。
__device__ __forceinline__ void g5_ccd(d_geom geom1, d_geom geom2, int geomtype1,
                                       int geomtype2, v3 x_1, v3 x_2, d_ccd_options options,
                                       d_polytope* pt, d_ccd_result* out) {
  d_gjk_phase phase;
  g5_gjk_phase(options.tolerance, options.cutoff, options.gjk_iterations, geom1, geom2,
               geomtype1, geomtype2, x_1, x_2, &phase);
  if (!phase.needs_epa) {
    out->dist = phase.dist;
    out->ncontact = phase.ncontact;
    out->x1 = phase.x1;
    out->x2 = phase.x2;
    out->multiccd_idx = -1;
    out->overflow = 0;
    return;
  }
  d_epa_phase epa_phase_result;
  g5_epa_phase(options.tolerance, options.epa_iterations, phase.result, phase.geom1,
               phase.geom2, geomtype1, geomtype2, pt, &epa_phase_result);
  unsigned overflow = epa_phase_result.overflow;
  if (epa_phase_result.dist == G5_FLOAT_MAX) overflow |= G5_OVERFLOW_CCD;
  out->dist = epa_phase_result.dist;
  out->ncontact = epa_phase_result.ncontact;
  out->x1 = epa_phase_result.x1;
  out->x2 = epa_phase_result.x2;
  out->multiccd_idx = epa_phase_result.multiccd_idx;
  out->overflow = overflow;
}
