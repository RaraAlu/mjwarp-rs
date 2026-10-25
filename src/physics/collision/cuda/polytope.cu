// G10@P5 G5：EPA polytope 构造与 witness 恢复设备版，与 host `gjk/polytope.rs` 逐式对应。
// Adapted from frozen MJWarp collision_gjk.py, revision 71da24d. Apache-2.0.
#pragma once
#include "gjk_core.cu"

#define G5_FACE_DELETED_BIT 0x80000000u
#define G5_FACE_INVALID_BIT 0x40000000u
#define G5_FACE_INVALID_OR_DELETED_MASK 0xC0000000u

// 缓冲区由调用方（宿主 launcher）按 epa_iterations 分配，容量在 max_face 中给出。
struct d_polytope {
  int nvert;
  int nface;
  int nhorizon;
  int status;
  v3* vert;        // 2*i = geom1 支撑点，2*i+1 = geom2 支撑点
  int* vert_index;
  unsigned* face;
  v3* face_pr;
  float* face_norm2;
  int* horizon;
  v3 center;
  int max_face;
  int max_horizon;
};

__device__ __forceinline__ void g5_get_face_verts(unsigned face, int* out) {
  out[0] = (int)(face & 0x3FFu);
  out[1] = (int)((face >> 10) & 0x3FFu);
  out[2] = (int)((face >> 20) & 0x3FFu);
}

__device__ __forceinline__ unsigned g5_delete_face(unsigned face) {
  return face | G5_FACE_DELETED_BIT;
}

__device__ __forceinline__ bool g5_is_face_deleted(unsigned face) {
  return (face & G5_FACE_DELETED_BIT) != 0u;
}

__device__ __forceinline__ unsigned g5_invalidate_face(unsigned face) {
  return face | G5_FACE_INVALID_BIT;
}

__device__ __forceinline__ bool g5_is_invalid_face(unsigned face) {
  return (face & G5_FACE_INVALID_OR_DELETED_MASK) != 0u;
}

// 上游 `_attach_face`：返回原点投影距离平方（0 = 失败/退化）。
__device__ __forceinline__ float g5_attach_face(d_polytope* pt, int idx, int idx1,
                                                int idx2, int idx3) {
  if (pt->nface == pt->max_face) return 0.0f;
  v3 p1 = v3_sub(pt->vert[2 * idx1], pt->vert[2 * idx1 + 1]);
  v3 p2 = v3_sub(pt->vert[2 * idx2], pt->vert[2 * idx2 + 1]);
  v3 p3 = v3_sub(pt->vert[2 * idx3], pt->vert[2 * idx3 + 1]);
  v3 r;
  bool degenerate = g5_project_origin_plane(p3, p2, p1, &r);
  if (degenerate) return 0.0f;
  if (v3_dot(r, v3_sub(p1, pt->center)) < 0.0f) r = v3_neg(r);
  pt->face[idx] = (unsigned)idx1 + ((unsigned)idx2 << 10) + ((unsigned)idx3 << 20);
  pt->face_pr[idx] = r;
  // 对齐 pinned ccd PTX 的 FMA 收缩：dot(r,r) 为「首项乘积 + 逐项 fma」链。
  float head = r.x * r.x;
  float tail = fmaf(r.y, r.y, head);
  pt->face_norm2[idx] = fmaf(r.z, r.z, tail);
  return pt->face_norm2[idx];
}

// 上游 `_epa_support`：写入 polytope 第 idx 对顶点，返回两侧缓存索引。
__device__ __forceinline__ void g5_epa_support(d_polytope* pt, int idx,
                                               const d_geom& geom1, const d_geom& geom2,
                                               int geomtype1, int geomtype2, v3 dir,
                                               int* index1, int* index2) {
  d_support_point sp = support(geom1, geomtype1, dir);
  pt->vert[2 * idx] = sp.point;
  pt->vert_index[2 * idx] = sp.vertex_index;
  *index1 = sp.cached_index;
  sp = support(geom2, geomtype2, v3_neg(dir));
  pt->vert[2 * idx + 1] = sp.point;
  pt->vert_index[2 * idx + 1] = sp.vertex_index;
  *index2 = sp.cached_index;
}

// 上游 `_rotmat`：绕轴旋转 120° 的矩阵（行优先）。
__device__ __forceinline__ void g5_rotmat(v3 axis, float* out) {
  float n = v3_length(axis);
  float u1 = axis.x / n;
  float u2 = axis.y / n;
  float u3 = axis.z / n;
  float sin_v = 0.86602540378f;
  float cos_v = -0.5f;
  float one_minus = 1.0f - cos_v;
  out[0] = cos_v + u1 * u1 * one_minus;
  out[1] = u1 * u2 * one_minus - u3 * sin_v;
  out[2] = u1 * u3 * one_minus + u2 * sin_v;
  out[3] = u2 * u1 * one_minus + u3 * sin_v;
  out[4] = cos_v + u2 * u2 * one_minus;
  out[5] = u2 * u3 * one_minus - u1 * sin_v;
  out[6] = u1 * u3 * one_minus - u2 * sin_v;
  out[7] = u2 * u3 * one_minus + u1 * sin_v;
  out[8] = cos_v + u3 * u3 * one_minus;
}

// 上游 `_ray_triangle`：返回 1/-1/0。
__device__ __forceinline__ int g5_ray_triangle(v3 p1, v3 p2, v3 p3, v3 p4, v3 p5) {
  float vol1 = g5_det3(v3_sub(p3, p1), v3_sub(p4, p1), v3_sub(p2, p1));
  float vol2 = g5_det3(v3_sub(p4, p1), v3_sub(p5, p1), v3_sub(p2, p1));
  float vol3 = g5_det3(v3_sub(p5, p1), v3_sub(p3, p1), v3_sub(p2, p1));
  if (vol1 >= 0.0f && vol2 >= 0.0f && vol3 >= 0.0f) return 1;
  if (vol1 <= 0.0f && vol2 <= 0.0f && vol3 <= 0.0f) return -1;
  return 0;
}

// 上游 `_replace_simplex3`：用 polytope 的 3 顶点重置 GJK 结果。
__device__ __forceinline__ d_gjk_result g5_replace_simplex3(const d_polytope* pt, int idx1,
                                                            int idx2, int idx3) {
  d_gjk_result result = g5_gjk_result_zero();
  result.simplex1[0] = pt->vert[2 * idx1];
  result.simplex1[1] = pt->vert[2 * idx2];
  result.simplex1[2] = pt->vert[2 * idx3];
  result.simplex2[0] = pt->vert[2 * idx1 + 1];
  result.simplex2[1] = pt->vert[2 * idx2 + 1];
  result.simplex2[2] = pt->vert[2 * idx3 + 1];
  result.simplex[0] = v3_sub(result.simplex1[0], result.simplex2[0]);
  result.simplex[1] = v3_sub(result.simplex1[1], result.simplex2[1]);
  result.simplex[2] = v3_sub(result.simplex1[2], result.simplex2[2]);
  result.simplex_index1[0] = pt->vert_index[2 * idx1];
  result.simplex_index1[1] = pt->vert_index[2 * idx2];
  result.simplex_index1[2] = pt->vert_index[2 * idx3];
  result.simplex_index2[0] = pt->vert_index[2 * idx1 + 1];
  result.simplex_index2[1] = pt->vert_index[2 * idx2 + 1];
  result.simplex_index2[2] = pt->vert_index[2 * idx3 + 1];
  return result;
}

// 上游 `_epa_witness`：从选中面恢复 witness 点与穿透距离。
__device__ __forceinline__ void g5_epa_witness(const d_polytope* pt, const d_geom& geom1,
                                               const d_geom& geom2, int geomtype1,
                                               int geomtype2, int face_idx, v3* out_x1,
                                               v3* out_x2, float* out_dist) {
  int face[3];
  g5_get_face_verts(pt->face[face_idx], face);
  v3 v1 = v3_sub(pt->vert[2 * face[0]], pt->vert[2 * face[0] + 1]);
  v3 v2 = v3_sub(pt->vert[2 * face[1]], pt->vert[2 * face[1] + 1]);
  v3 v3v = v3_sub(pt->vert[2 * face[2]], pt->vert[2 * face[2] + 1]);
  float coordinates[3];
  g5_tri_affine_coord(v1, v2, v3v, pt->face_pr[face_idx], coordinates);
  float l1 = coordinates[0];
  float l2 = coordinates[1];
  float l3 = coordinates[2];

  v3 w1 = pt->vert[2 * face[0] + 1];
  v3 w2 = pt->vert[2 * face[1] + 1];
  v3 w3 = pt->vert[2 * face[2] + 1];
  v3 x2 = v3_make(w1.x * l1 + w2.x * l2 + w3.x * l3,
                  w1.y * l1 + w2.y * l2 + w3.y * l3,
                  w1.z * l1 + w2.z * l2 + w3.z * l3);

  int i1 = pt->vert_index[2 * face[0]];
  int i2 = pt->vert_index[2 * face[1]];
  int i3 = pt->vert_index[2 * face[2]];
  if (geomtype1 == G5_HFIELD && (i1 != i2 || i1 != i3)) {
    v3 n = v3_make(0.0f, 0.0f, 1.0f);
    v3 a = v3_make(geom1.polyvert[9], geom1.polyvert[10], geom1.polyvert[11]);
    v3 b = v3_make(geom1.polyvert[12], geom1.polyvert[13], geom1.polyvert[14]);
    v3 c = v3_make(geom1.polyvert[15], geom1.polyvert[16], geom1.polyvert[17]);
    v3 shifted_x2;
    if (geomtype2 == G5_CAPSULE || geomtype2 == G5_SPHERE) {
      float radius = geom2.size.x;
      float margin = geom2.margin;
      d_geom geom2_shifted = geom2;
      geom2_shifted.margin = 0.0f;
      geom2_shifted.size = v3_make(0.0f, geom2.size.y, geom2.size.z);
      d_support_point sp = support(geom2_shifted, geomtype2, x2);
      float offset = 0.5f * margin + radius;
      shifted_x2 = v3_make(sp.point.x - offset * n.x, sp.point.y - offset * n.y,
                           sp.point.z - offset * n.z);
    } else {
      shifted_x2 = support(geom2, geomtype2, v3_normalize(x2)).point;
    }
    float coordinates2[3];
    g5_tri_affine_coord(a, b, c, shifted_x2, coordinates2);
    v3 x1;
    if (coordinates2[0] > 0.0f && coordinates2[1] > 0.0f && coordinates2[2] > 0.0f) {
      x1 = v3_make(coordinates2[0] * a.x + coordinates2[1] * b.x + coordinates2[2] * c.x,
                   coordinates2[0] * a.y + coordinates2[1] * b.y + coordinates2[2] * c.y,
                   coordinates2[0] * a.z + coordinates2[1] * b.z + coordinates2[2] * c.z);
    } else {
      v3 p = c;
      if (coordinates2[1] > 0.0f) p = b;
      if (coordinates2[0] > 0.0f) p = a;
      float d = v3_dot(v3_sub(shifted_x2, p), n);
      x1 = v3_make(shifted_x2.x - d * n.x, shifted_x2.y - d * n.y,
                   shifted_x2.z - d * n.z);
    }
    *out_x1 = x1;
    *out_x2 = shifted_x2;
    *out_dist = -v3_length(v3_sub(x1, shifted_x2));
    return;
  }

  v3 p1 = pt->vert[2 * face[0]];
  v3 p2 = pt->vert[2 * face[1]];
  v3 p3 = pt->vert[2 * face[2]];
  v3 x1 = v3_make(p1.x * l1 + p2.x * l2 + p3.x * l3,
                  p1.y * l1 + p2.y * l2 + p3.y * l3,
                  p1.z * l1 + p2.z * l2 + p3.z * l3);
  *out_x1 = x1;
  *out_x2 = x2;
  *out_dist = -sqrtf(pt->face_norm2[face_idx]);
}
