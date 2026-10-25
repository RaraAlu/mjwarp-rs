// G10@P5 G5：hfield prism CCD 设备版，与 host `hfield/{filter,driver}.rs` 逐式对应。
// Adapted from frozen MJWarp collision_convex.py (`_hfield_filter` / `ccd_hfield_kernel`),
// revision 71da24d. Apache-2.0.
// 世界→hfield 帧一律用 `common.cu::relative_matrix`（上游 `transpose(rot1)*rot2`），
// 不复制 host 测试 helper 的旧转置约定（详见 `src/physics/collision/relmat.rs`）。
#pragma once
#include "ccd.cu"

// 上游 `MJ_MAXCONPAIR`（`mujoco.mjMAXCONPAIR`）。
#define G5_MJ_MAXCONPAIR 50
// 上游 `MJ_MAXVAL`（`mujoco.mjMAXVAL`）。
#define G5_MJ_MAXVAL 1.0e10f
// 上游 `MIN_DIST_TO_NEXT_CONTACT`。
#define G5_MIN_DIST_TO_NEXT_CONTACT 1.0e-3f
// 上游 `OverflowType.HFIELD`。
#define G5_OVERFLOW_HFIELD (1u << 5)

// hfield 模型视图（G1 上传口径：`hfield_size`/`hfield_nrow`/`hfield_ncol`/
// `hfield_adr`/`hfield_data`，全 f32）。
struct d_hfield_data {
  // (x, y, z_top, z_bottom)。
  float size[4];
  int nrow;
  int ncol;
  // `hfield_data` 起始下标。
  int adr;
  const float* data;
};

// 上游 `_hfield_filter` 的紧 AABB 边界（hfield 局部帧）。
struct d_hfield_bounds {
  float xmin;
  float xmax;
  float ymin;
  float ymax;
  float zmin;
  float zmax;
};

// 上游 `_hfield_filter`：返回 true 表示存在可碰撞棱柱（上游 `no_hf_collision=False`）。
// `geom2_in` 为世界系视图（与上游一致，在函数内变换到 hfield 帧）；
// `other_rbound` = `geom_rbound[g2]`，`margin` = 两侧 `geom_margin` 之和
// （模型数组汇总由宿主 launcher 预计算，与 host `hfield_filter` 入参一致）。
__device__ __forceinline__ bool g5_hfield_filter(const d_hfield_data& hfield,
                                                 const d_geom& geom2_in, int geomtype2,
                                                 float other_rbound, float margin, v3 hf_pos,
                                                 const float* hf_mat,
                                                 d_hfield_bounds* bounds) {
  const float* size1 = hfield.size;
  // 上游 `pos = mat1T @ (pos2 - pos1)`。
  v3 pos = transpose_mul_mat3_vec(hf_mat, v3_sub(geom2_in.pos, hf_pos));

  // box-sphere test: horizontal plane（`size[0]`、`size[1]` 为半宽）。
  for (int axis = 0; axis < 2; ++axis) {
    float p = axis == 0 ? pos.x : pos.y;
    if (size1[axis] < p - other_rbound - margin || -size1[axis] > p + other_rbound + margin) {
      return false;
    }
  }
  // box-sphere test: vertical direction（`size[2]` 顶、`size[3]` 底）。
  if (size1[2] < pos.z - other_rbound - margin) return false;
  if (-size1[3] > pos.z + other_rbound + margin) return false;

  // 支撑紧 AABB：geom2 在 hfield 帧内、margin=0（上游同）。
  d_geom geom2 = geom2_in;
  geom2.pos = pos;
  // 上游 `geom2.rot = mat1T @ mat2`。
  relative_matrix(hf_mat, geom2_in.rot, geom2.rot);
  geom2.margin = 0.0f;
  geom2.index = -1;

  float xmax = support(geom2, geomtype2, v3_make(1.0f, 0.0f, 0.0f)).point.x;
  float xmin = support(geom2, geomtype2, v3_make(-1.0f, 0.0f, 0.0f)).point.x;
  float ymax = support(geom2, geomtype2, v3_make(0.0f, 1.0f, 0.0f)).point.y;
  float ymin = support(geom2, geomtype2, v3_make(0.0f, -1.0f, 0.0f)).point.y;
  float zmax = support(geom2, geomtype2, v3_make(0.0f, 0.0f, 1.0f)).point.z;
  float zmin = support(geom2, geomtype2, v3_make(0.0f, 0.0f, -1.0f)).point.z;

  // box-box test。
  if (xmin - margin > size1[0] || xmax + margin < -size1[0] ||
      ymin - margin > size1[1] || ymax + margin < -size1[1] ||
      zmin - margin > size1[2] || zmax + margin < -size1[3]) {
    return false;
  }
  bounds->xmin = xmin;
  bounds->xmax = xmax;
  bounds->ymin = ymin;
  bounds->ymax = ymax;
  bounds->zmin = zmin;
  bounds->zmax = zmax;
  return true;
}

// 一个 hfield 接触（世界系，与上游 `write_contact` 入参一致）。
struct d_hfield_contact {
  float dist;
  v3 pos;
  // 行优先 3×3 frame（由 `make_frame(normal)` 生成）。
  float frame[9];
};

// 单对 hfield 结果；`overflow` 汇总本对产生的溢出位（HFIELD + 逐棱柱 ccd 溢出）。
struct d_hfield_result {
  // 实际写入的接触数（0..4）。
  int ncontact;
  // 上游 `count`：通过筛选且进入 `ccd()` 并命中接触的棱柱数（可 >= 4）。
  int prism_count;
  unsigned overflow;
  d_hfield_contact contact[4];
};

// 上游 `write_contact` 的接触装配子集（frame 由法线生成；nacon 写入归第五批）。
__device__ __forceinline__ void g5_write_hfield_contact(d_hfield_contact* contact, float dist,
                                                        v3 pos, v3 normal) {
  contact->dist = dist;
  contact->pos = pos;
  make_frame(normal, contact->frame);
}

// 上游 `ccd_hfield_kernel` 的单对主体：hfield 子网格棱柱扫描 + `ccd()` + 0..3 接触选择。
// `pt` 为调用方按 `epa_iterations` 提供的 EPA polytope 缓冲（上游按 ccdid 复用同一缓冲）。
__device__ __forceinline__ void g5_hfield_pair_contacts(
    const d_hfield_data& hfield, v3 hf_pos, const float* hf_mat, const d_geom& geom2_in,
    int geomtype2, float other_rbound, float margin, float gap, d_ccd_options options,
    d_polytope* pt, d_hfield_result* out) {
  out->ncontact = 0;
  out->prism_count = 0;
  out->overflow = 0u;
  for (int i = 0; i < 4; ++i) {
    out->contact[i].dist = 0.0f;
    out->contact[i].pos = v3_make(0.0f, 0.0f, 0.0f);
    for (int k = 0; k < 9; ++k) out->contact[i].frame[k] = 0.0f;
  }

  d_hfield_bounds bounds;
  if (!g5_hfield_filter(hfield, geom2_in, geomtype2, other_rbound, margin, hf_pos, hf_mat,
                        &bounds)) {
    return;
  }

  // 上游：geom2 变换到 hfield 帧；geom1 恒为单位位姿。
  d_geom geom2 = geom2_in;
  geom2.pos = transpose_mul_mat3_vec(hf_mat, v3_sub(geom2_in.pos, hf_pos));
  relative_matrix(hf_mat, geom2_in.rot, geom2.rot);

  d_geom geom1;
  geom1.pos = v3_make(0.0f, 0.0f, 0.0f);
  const float identity[9] = {1.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 1.0f};
  for (int k = 0; k < 9; ++k) geom1.rot[k] = identity[k];
  geom1.size = v3_make(0.0f, 0.0f, 0.0f);
  geom1.margin = 0.0f;
  for (int k = 0; k < 18; ++k) geom1.polyvert[k] = 0.0f;
  geom1.vertadr = -1;
  geom1.vertnum = -1;
  geom1.graphadr = -1;
  geom1.index = -1;
  geom1.mesh.vert = nullptr;
  geom1.mesh.graph = nullptr;

  const float size[4] = {hfield.size[0], hfield.size[1], hfield.size[2], hfield.size[3]};
  int nrow = hfield.nrow;
  int ncol = hfield.ncol;

  // 上游 hfield 子网格。
  float x_scale = 0.5f * (float)(ncol - 1) / size[0];
  float y_scale = 0.5f * (float)(nrow - 1) / size[1];
  int cmin = (int)floorf((bounds.xmin + size[0]) * x_scale);
  if (cmin < 0) cmin = 0;
  int cmax = (int)ceilf((bounds.xmax + size[0]) * x_scale);
  if (cmax > ncol - 1) cmax = ncol - 1;
  int rmin = (int)floorf((bounds.ymin + size[1]) * y_scale);
  if (rmin < 0) rmin = 0;
  int rmax = (int)ceilf((bounds.ymax + size[1]) * y_scale);
  if (rmax > nrow - 1) rmax = nrow - 1;

  float dx = (2.0f * size[0]) / (float)(ncol - 1);
  float dy = (2.0f * size[1]) / (float)(nrow - 1);
  const int dr[2] = {1, 0};

  // prism 六顶点（行优先 6×3）：0..2 底三角（z=-size[3]），3..5 顶三角。
  float prism[18] = {0.0f};
  prism[2] = -size[3];
  prism[5] = -size[3];
  prism[8] = -size[3];

  int adr = hfield.adr;

  float contact_dist[G5_MJ_MAXCONPAIR];
  v3 contact_pos[G5_MJ_MAXCONPAIR];
  v3 contact_normal[G5_MJ_MAXCONPAIR];
  float min_dist = G5_MJ_MAXVAL;
  v3 min_normal = v3_make(G5_MJ_MAXVAL, G5_MJ_MAXVAL, G5_MJ_MAXVAL);
  v3 min_pos = v3_make(G5_MJ_MAXVAL, G5_MJ_MAXVAL, G5_MJ_MAXVAL);
  int min_id = -1;

  // 上游在本阶段把 geom2.margin 设为两侧 margin 之和（geom1 margin 恒 0）。
  geom2.margin = margin;

  int count = 0;
  int noverflow = 0;
  for (int r = rmin; r < rmax; ++r) {
    if (noverflow > 0) break;

    // 预置本行前两个顶点（行首列）。
    for (int init_i = 0; init_i < 2; ++init_i) {
      float x = dx * (float)cmin - size[0];
      float y = dy * (float)(r + dr[init_i]) - size[1];
      float z = hfield.data[adr + (r + dr[init_i]) * ncol + cmin] * size[2] + margin;

      for (int k = 0; k < 3; ++k) prism[k] = prism[3 + k];
      for (int k = 0; k < 3; ++k) prism[3 + k] = prism[6 + k];
      for (int k = 0; k < 3; ++k) prism[9 + k] = prism[12 + k];
      for (int k = 0; k < 3; ++k) prism[12 + k] = prism[15 + k];
      prism[6] = x;
      prism[15] = x;
      prism[7] = y;
      prism[16] = y;
      prism[17] = z;
    }

    for (int c = cmin + 1; c <= cmax; ++c) {
      if (noverflow > 0) break;

      // 每格两个三角棱柱。
      for (int i = 0; i < 2; ++i) {
        if (count >= G5_MJ_MAXCONPAIR) {
          noverflow = 1;
          break;
        }

        // 追加本列顶点。
        float x = dx * (float)c - size[0];
        float y = dy * (float)(r + dr[i]) - size[1];
        float z = hfield.data[adr + (r + dr[i]) * ncol + c] * size[2] + margin;

        for (int k = 0; k < 3; ++k) prism[k] = prism[3 + k];
        for (int k = 0; k < 3; ++k) prism[3 + k] = prism[6 + k];
        for (int k = 0; k < 3; ++k) prism[9 + k] = prism[12 + k];
        for (int k = 0; k < 3; ++k) prism[12 + k] = prism[15 + k];
        prism[6] = x;
        prism[15] = x;
        prism[7] = y;
        prism[16] = y;
        prism[17] = z;

        // prism 高度测试（顶三角 z 均低于紧 AABB 下界）。
        if (prism[11] < bounds.zmin && prism[14] < bounds.zmin && prism[17] < bounds.zmin) {
          continue;
        }

        for (int k = 0; k < 18; ++k) geom1.polyvert[k] = prism[k];

        // 上游 `x1 = geom1.pos + geom1.rot @ (sum * 1/6)`。
        v3 sum = v3_make(0.0f, 0.0f, 0.0f);
        for (int k = 0; k < 6; ++k) {
          sum = v3_add(sum, v3_make(prism[3 * k], prism[3 * k + 1], prism[3 * k + 2]));
        }
        v3 center = v3_add(geom1.pos, mul_mat3_vec(geom1.rot, v3_scale(sum, 1.0f / 6.0f)));

        d_ccd_result result;
        g5_ccd(geom1, geom2, G5_HFIELD, geomtype2, center, geom2.pos, options, pt, &result);
        // 上游把 `overflow_out` 透传给 ccd（EPA_HORIZON / FLOAT_MAX→CCD），由调用方汇总。
        out->overflow |= result.overflow;

        // 分离棱柱跳过（ccd 对分离返回正距离）。
        if (result.ncontact == 0 || result.dist >= margin + gap) continue;

        // 上游：`pos_local = 0.5 * (w1 + w2)`。
        v3 pos_local = v3_scale(v3_add(result.x1, result.x2), 0.5f);
        v3 pos = v3_add(mul_mat3_vec(hf_mat, pos_local), hf_pos);

        float frame_local[9];
        make_frame(v3_sub(result.x1, result.x2), frame_local);
        v3 normal_local = v3_make(frame_local[0], frame_local[1], frame_local[2]);
        v3 normal = mul_mat3_vec(hf_mat, normal_local);

        contact_dist[count] = result.dist;
        contact_pos[count] = pos;
        contact_normal[count] = normal;
        if (result.dist < min_dist) {
          min_dist = result.dist;
          min_normal = normal;
          min_pos = pos;
          min_id = count;
        }
        count += 1;
      }
    }
  }

  // 上游：每对只报一次 HFIELD 溢出（`MJ_MAXCONPAIR`）。
  if (noverflow > 0) out->overflow |= G5_OVERFLOW_HFIELD;
  out->prism_count = count;
  if (count == 0) return;

  // contact 0：最小距离接触。
  g5_write_hfield_contact(&out->contact[0], min_dist, min_pos, min_normal);
  out->ncontact = 1;

  // contact 1：离最小距离接触点最远的接触。
  int id1 = -1;
  float dist1 = -G5_MJ_MAXVAL;
  for (int i = 0; i < count; ++i) {
    if (i == min_id) continue;
    float dist = v3_length(v3_sub(contact_pos[i], min_pos));
    if (dist > dist1) {
      id1 = i;
      dist1 = dist;
    }
  }
  if (id1 == -1 || (dist1 > 0.0f && dist1 < G5_MIN_DIST_TO_NEXT_CONTACT)) return;
  v3 pos1 = contact_pos[id1];
  g5_write_hfield_contact(&out->contact[1], contact_dist[id1], pos1, contact_normal[id1]);
  out->ncontact = 2;

  // contact 2：离 min_pos→pos1 直线最远的接触。
  v3 dist_min1 = v3_cross(min_normal, v3_sub(min_pos, pos1));
  int id2 = -1;
  float dist_12 = -G5_MJ_MAXVAL;
  for (int i = 0; i < count; ++i) {
    if (i == min_id || i == id1) continue;
    float dist = fabsf(v3_dot(v3_sub(contact_pos[i], min_pos), dist_min1));
    if (dist > dist_12) {
      id2 = i;
      dist_12 = dist;
    }
  }
  if (id2 == -1 || (dist_12 > 0.0f && dist_12 < G5_MIN_DIST_TO_NEXT_CONTACT)) return;
  v3 pos2 = contact_pos[id2];
  g5_write_hfield_contact(&out->contact[2], contact_dist[id2], pos2, contact_normal[id2]);
  out->ncontact = 3;

  // contact 3：离另一条三角边最远的接触。
  v3 vec_min2 = v3_cross(min_normal, v3_sub(min_pos, pos2));
  v3 vec_12 = v3_cross(min_normal, v3_sub(pos1, pos2));
  int id3 = -1;
  float dist3 = -G5_MJ_MAXVAL;
  for (int i = 0; i < count; ++i) {
    if (i == min_id || i == id1 || i == id2) continue;
    v3 point = contact_pos[i];
    float dist = fabsf(v3_dot(v3_sub(point, min_pos), vec_min2)) +
                 fabsf(v3_dot(v3_sub(pos1, point), vec_12));
    if (dist > dist3) {
      id3 = i;
      dist3 = dist;
    }
  }
  if (id3 == -1 || (dist3 > 0.0f && dist3 < G5_MIN_DIST_TO_NEXT_CONTACT)) return;
  g5_write_hfield_contact(&out->contact[3], contact_dist[id3], contact_pos[id3],
                          contact_normal[id3]);
  out->ncontact = 4;
}
