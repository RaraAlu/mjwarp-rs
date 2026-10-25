// G10@P5 G5：SDF narrowphase 设备版，与 host `sdf/narrowphase.rs`（上游 `_sdf_narrowphase` 子集）对应。
// Adapted from frozen MJWarp collision_sdf.py, revision 71da24d. Apache-2.0.
// 设备结果直接对同配方上游 f32 比较（不依赖 host 侧 rel_mat 旧约定）。
#pragma once
#include "sdf_math.cu"

struct d_sdf_pair {
  d_sdf_shape shape1;
  d_sdf_shape shape2;
  v3 pos1;
  float rot1[9];
  v3 pos2;
  float rot2[9];
  // 局部 AABB：(center.xyz, half_size.xyz)。
  float aabb1[6];
  float aabb2[6];
  int sdf_initpoints;
  int sdf_iterations;
};

struct d_sdf_contact {
  float dist;
  v3 pos;
  float frame[9];
};

// 上游 `_sdf_narrowphase` 的单 pair 子集：AABB 交集 + 每初始点一次梯度下降 → 接触候选。
// `capacity` 为调用方给出的接触容量（>= sdf_initpoints 时不会截断）；返回写入数量。
__device__ __forceinline__ int g5_sdf_pair_contacts(const d_sdf_pair* pair,
                                                    d_sdf_contact* out, int capacity) {
  const float identity[9] = {1.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 1.0f};
  v3 aabb1_center = v3_make(pair->aabb1[0], pair->aabb1[1], pair->aabb1[2]);
  v3 aabb1_half = v3_make(pair->aabb1[3], pair->aabb1[4], pair->aabb1[5]);
  v3 aabb1_min;
  v3 aabb1_max;
  g5_transform_aabb(aabb1_center, aabb1_half, v3_make(0.0f, 0.0f, 0.0f), identity,
                    &aabb1_min, &aabb1_max);

  float rel_mat[9];
  relative_matrix(pair->rot1, pair->rot2, rel_mat);
  v3 rel_pos = transpose_mul_mat3_vec(pair->rot1, v3_sub(pair->pos2, pair->pos1));
  v3 aabb2_center = v3_make(pair->aabb2[0], pair->aabb2[1], pair->aabb2[2]);
  v3 aabb2_half = v3_make(pair->aabb2[3], pair->aabb2[4], pair->aabb2[5]);
  v3 aabb2_min;
  v3 aabb2_max;
  g5_transform_aabb(aabb2_center, aabb2_half, rel_pos, rel_mat, &aabb2_min, &aabb2_max);

  v3 min_v = v3_make(fmaxf(aabb1_min.x, aabb2_min.x), fmaxf(aabb1_min.y, aabb2_min.y),
                     fmaxf(aabb1_min.z, aabb2_min.z));
  v3 max_v = v3_make(fminf(aabb1_max.x, aabb2_max.x), fminf(aabb1_max.y, aabb2_max.y),
                     fminf(aabb1_max.z, aabb2_max.z));
  if (min_v.x > max_v.x || min_v.y > max_v.y || min_v.z > max_v.z) return 0;

  int count = 0;
  int limit = pair->sdf_initpoints < capacity ? pair->sdf_initpoints : capacity;
  for (int index = 0; index < limit; ++index) {
    unsigned i = (unsigned)index;
    v3 x_g2 = v3_make(min_v.x + (max_v.x - min_v.x) * g5_halton(i, 2),
                      min_v.y + (max_v.y - min_v.y) * g5_halton(i, 3),
                      min_v.z + (max_v.z - min_v.z) * g5_halton(i, 5));
    v3 x = v3_add(mul_mat3_vec(pair->rot1, x_g2), pair->pos1);
    v3 x0_initial = transpose_mul_mat3_vec(pair->rot2, v3_sub(x, pair->pos2));
    float dist = 0.0f;
    v3 pos;
    v3 normal;
    g5_gradient_descent(&pair->shape1, &pair->shape2, x0_initial, pair->pos1, pair->rot1,
                        pair->pos2, pair->rot2, pair->sdf_iterations, &dist, &pos, &normal);
    out[count].dist = dist;
    out[count].pos = pos;
    make_frame(normal, out[count].frame);
    count += 1;
  }
  return count;
}
