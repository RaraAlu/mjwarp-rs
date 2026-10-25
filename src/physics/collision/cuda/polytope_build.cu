// G10@P5 G5：EPA polytope 初始构造设备版，与 host `gjk/polytope_build.rs` 逐式对应。
// Adapted from frozen MJWarp collision_gjk.py, revision 71da24d. Apache-2.0.
#pragma once
#include "polytope.cu"

#define G5_MIN_DIST2 1.0e-10f
#define G5_MIN_DIST3 1.0e-10f
#define G5_MIN_DIST4 1.0e-17f

__device__ __forceinline__ void g5_next_face(d_polytope* pt, int idx1, int idx2, int idx3,
                                             d_gjk_result* next) {
  *next = g5_replace_simplex3(pt, idx1, idx2, idx3);
}

// 上游 `_polytope2`：由 GJK 的 1-单纯形构造 hexahedron。
__device__ __forceinline__ void g5_polytope2(
    d_polytope* pt, const v3* simplex, const v3* simplex1, const v3* simplex2,
    const int* simplex_index1, const int* simplex_index2, const d_geom& geom1,
    const d_geom& geom2, int geomtype1, int geomtype2, d_gjk_result* next) {
  v3 diff = v3_sub(simplex[1], simplex[0]);
  pt->center = v3_make(0.5f * (simplex[0].x + simplex[1].x),
                       0.5f * (simplex[0].y + simplex[1].y),
                       0.5f * (simplex[0].z + simplex[1].z));
  float value = G5_FLOAT_MAX;
  int index = 0;
  for (int i = 0; i < 3; ++i) {
    const float* d = reinterpret_cast<const float*>(&diff.x);
    if (fabsf(d[i]) < value) {
      value = fabsf(d[i]);
      index = i;
    }
  }
  v3 e = v3_make(0.0f, 0.0f, 0.0f);
  float* ep = reinterpret_cast<float*>(&e.x);
  ep[index] = 1.0f;
  v3 d1 = v3_cross(e, diff);
  float r[9];
  g5_rotmat(diff, r);
  v3 d2 = mul_mat3_vec(r, d1);
  v3 d3 = mul_mat3_vec(r, d2);

  pt->vert[0] = simplex1[0];
  pt->vert[1] = simplex2[0];
  pt->vert[2] = simplex1[1];
  pt->vert[3] = simplex2[1];
  pt->vert_index[0] = simplex_index1[0];
  pt->vert_index[1] = simplex_index2[0];
  pt->vert_index[2] = simplex_index1[1];
  pt->vert_index[3] = simplex_index2[1];

  int cache1 = 0;
  int cache2 = 0;
  float n1 = v3_length(d1);
  g5_epa_support(pt, 2, geom1, geom2, geomtype1, geomtype2,
                 v3_make(d1.x / n1, d1.y / n1, d1.z / n1), &cache1, &cache2);
  float n2 = v3_length(d2);
  g5_epa_support(pt, 3, geom1, geom2, geomtype1, geomtype2,
                 v3_make(d2.x / n2, d2.y / n2, d2.z / n2), &cache1, &cache2);
  float n3 = v3_length(d3);
  g5_epa_support(pt, 4, geom1, geom2, geomtype1, geomtype2,
                 v3_make(d3.x / n3, d3.y / n3, d3.z / n3), &cache1, &cache2);

  if (g5_attach_face(pt, 0, 0, 2, 3) < G5_MIN_DIST2) {
    pt->status = -1;
    g5_next_face(pt, 0, 2, 3, next);
    return;
  }
  if (g5_attach_face(pt, 1, 0, 4, 2) < G5_MIN_DIST2) {
    pt->status = -1;
    g5_next_face(pt, 0, 4, 2, next);
    return;
  }
  if (g5_attach_face(pt, 2, 0, 3, 4) < G5_MIN_DIST2) {
    pt->status = -1;
    g5_next_face(pt, 0, 3, 4, next);
    return;
  }
  if (g5_attach_face(pt, 3, 1, 3, 2) < G5_MIN_DIST2) {
    pt->status = -1;
    g5_next_face(pt, 1, 3, 2, next);
    return;
  }
  if (g5_attach_face(pt, 4, 1, 2, 4) < G5_MIN_DIST2) {
    pt->status = -1;
    g5_next_face(pt, 1, 2, 4, next);
    return;
  }
  if (g5_attach_face(pt, 5, 1, 4, 3) < G5_MIN_DIST2) {
    pt->status = -1;
    g5_next_face(pt, 1, 4, 3, next);
    return;
  }

  v3 v2 = v3_sub(pt->vert[4], pt->vert[5]);
  v3 v3v = v3_sub(pt->vert[6], pt->vert[7]);
  v3 v4 = v3_sub(pt->vert[8], pt->vert[9]);
  if (g5_ray_triangle(simplex[0], simplex[1], v2, v3v, v4) == 0) {
    pt->status = 1;
    *next = g5_gjk_result_zero();
    return;
  }
  pt->nvert = 5;
  pt->nface = 6;
  pt->status = 0;
  *next = g5_gjk_result_zero();
}

// 上游 `_polytope3`：由 GJK 的 2-单纯形构造 hexahedron。
__device__ __forceinline__ void g5_polytope3(
    d_polytope* pt, float dist, const v3* simplex, const v3* simplex1, const v3* simplex2,
    const int* simplex_index1, const int* simplex_index2, const d_geom& geom1,
    const d_geom& geom2, int geomtype1, int geomtype2) {
  v3 sum1 = v3_add(v3_add(simplex[0], simplex[1]), simplex[2]);
  pt->center = v3_make(sum1.x * (1.0f / 3.0f), sum1.y * (1.0f / 3.0f),
                       sum1.z * (1.0f / 3.0f));
  v3 n = v3_cross(v3_sub(simplex[1], simplex[0]), v3_sub(simplex[2], simplex[0]));
  float norm = v3_length(n);
  if (norm < G5_MINVAL) {
    pt->status = 2;
    return;
  }
  n = v3_make(n.x / norm, n.y / norm, n.z / norm);

  pt->vert[0] = simplex1[0];
  pt->vert[1] = simplex2[0];
  pt->vert[2] = simplex1[1];
  pt->vert[3] = simplex2[1];
  pt->vert[4] = simplex1[2];
  pt->vert[5] = simplex2[2];
  pt->vert_index[0] = simplex_index1[0];
  pt->vert_index[1] = simplex_index2[0];
  pt->vert_index[2] = simplex_index1[1];
  pt->vert_index[3] = simplex_index2[1];
  pt->vert_index[4] = simplex_index1[2];
  pt->vert_index[5] = simplex_index2[2];

  int cache1 = 0;
  int cache2 = 0;
  g5_epa_support(pt, 3, geom1, geom2, geomtype1, geomtype2, v3_neg(n), &cache1, &cache2);
  g5_epa_support(pt, 4, geom1, geom2, geomtype1, geomtype2, n, &cache1, &cache2);

  v3 v1 = simplex[0];
  v3 v2 = simplex[1];
  v3 v3v = simplex[2];
  v3 v4 = v3_sub(pt->vert[6], pt->vert[7]);
  v3 v5 = v3_sub(pt->vert[8], pt->vert[9]);
  if (g5_tri_point_intersect(v1, v2, v3v, v4)) {
    pt->status = 3;
    return;
  }
  if (g5_tri_point_intersect(v1, v2, v3v, v5)) {
    pt->status = 4;
    return;
  }
  if (dist > 1e-5f && !g5_test_tetra(v1, v2, v3v, v4) && !g5_test_tetra(v1, v2, v3v, v5)) {
    pt->status = 5;
    return;
  }
  if (g5_attach_face(pt, 0, 4, 0, 1) < G5_MIN_DIST3) {
    pt->status = 6;
    return;
  }
  if (g5_attach_face(pt, 1, 4, 2, 0) < G5_MIN_DIST3) {
    pt->status = 7;
    return;
  }
  if (g5_attach_face(pt, 2, 4, 1, 2) < G5_MIN_DIST3) {
    pt->status = 8;
    return;
  }
  if (g5_attach_face(pt, 3, 3, 1, 0) < G5_MIN_DIST3) {
    pt->status = 9;
    return;
  }
  if (g5_attach_face(pt, 4, 3, 0, 2) < G5_MIN_DIST3) {
    pt->status = 10;
    return;
  }
  if (g5_attach_face(pt, 5, 3, 2, 1) < G5_MIN_DIST3) {
    pt->status = 11;
    return;
  }
  pt->nvert = 5;
  pt->nface = 6;
  pt->status = 0;
}

// 上游 `_polytope4`：由 GJK 的 3-单纯形构造 tetrahedron。
__device__ __forceinline__ void g5_polytope4(d_polytope* pt, const v3* simplex,
                                             const v3* simplex1, const v3* simplex2,
                                             const int* simplex_index1,
                                             const int* simplex_index2,
                                             d_gjk_result* next) {
  v3 sum = v3_add(v3_add(simplex[0], simplex[1]), v3_add(simplex[2], simplex[3]));
  pt->center = v3_make(0.25f * sum.x, 0.25f * sum.y, 0.25f * sum.z);
  for (int i = 0; i < 4; ++i) {
    pt->vert[2 * i] = simplex1[i];
    pt->vert[2 * i + 1] = simplex2[i];
    pt->vert_index[2 * i] = simplex_index1[i];
    pt->vert_index[2 * i + 1] = simplex_index2[i];
  }
  float dist[4] = {0.0f, 0.0f, 0.0f, 0.0f};
  int idx = 0;
  dist[0] = g5_attach_face(pt, 0, 0, 1, 2);
  if (dist[0] < G5_MIN_DIST4) {
    pt->status = -1;
    g5_next_face(pt, 0, 1, 2, next);
    return;
  }
  dist[1] = g5_attach_face(pt, 1, 0, 3, 1);
  if (dist[1] < G5_MIN_DIST4) {
    pt->status = -1;
    g5_next_face(pt, 0, 3, 1, next);
    return;
  }
  idx = dist[0] < dist[1] ? 0 : 1;
  dist[2] = g5_attach_face(pt, 2, 0, 2, 3);
  if (dist[2] < G5_MIN_DIST4) {
    pt->status = -1;
    g5_next_face(pt, 0, 2, 3, next);
    return;
  }
  if (dist[2] < dist[idx]) idx = 2;
  dist[3] = g5_attach_face(pt, 3, 3, 2, 1);
  if (dist[3] < G5_MIN_DIST4) {
    pt->status = -1;
    g5_next_face(pt, 3, 2, 1, next);
    return;
  }
  if (dist[3] < dist[idx]) idx = 3;
  if (!g5_test_tetra(simplex[0], simplex[1], simplex[2], simplex[3])) {
    if (dist[idx] > G5_MINVAL) {
      pt->status = 12;
      *next = g5_gjk_result_zero();
      return;
    }
    pt->status = -1;
    if (idx == 0) {
      g5_next_face(pt, 0, 1, 2, next);
    } else if (idx == 1) {
      g5_next_face(pt, 0, 3, 1, next);
    } else if (idx == 2) {
      g5_next_face(pt, 0, 2, 3, next);
    } else {
      g5_next_face(pt, 3, 2, 1, next);
    }
    return;
  }
  pt->nvert = 4;
  pt->nface = 4;
  pt->status = 0;
  *next = g5_gjk_result_zero();
}
