// G10@P5 G5：上游 primitive narrowphase 数值内核（sphere-box / capsule-box）设备版。
// Adapted from frozen MJWarp `collision_primitive_core.py::sphere_box` / `capsule_box`,
// revision 71da24d. Apache-2.0.
//
// 逐表达式按 f32 转写（含循环展开顺序、整数编码与 `safe_div`/`normalize_with_norm`
// 语义）；上层 wrapper 与接触写入见 `primitive_wrapper.cu`。
#include "common.cu"

// 上游 `types.MJ_MAXVAL`（`mujoco.mjMAXVAL = 1e10`）：capsule-box 未生成第二接触时使用。
#define G5_PRIMITIVE_MAXVAL 1.0e10f
// 上游 `types.MJ_MINVAL`（`mujoco.mjMINVAL = 1e-15`）。
#define G5_PRIMITIVE_MINVAL 1.0e-15f

// 单条 primitive 接触（对应上游 `sphere_box` 返回的 `(dist, pos, normal)`）。
struct g5_primitive_contact {
  float dist;
  v3 pos;
  v3 normal;
};

// 两条 primitive 接触（对应上游 `capsule_box` 返回的 `vec2` + 两个 `mat23f` 行）。
struct g5_primitive_pair {
  g5_primitive_contact contact[2];
};

// 上游 `math.normalize_with_norm`：范数为零时返回原向量与 0.0（不除零）。
__device__ __forceinline__ void g5_primitive_normalize_with_norm(v3 x, v3* direction,
                                                                 float* norm) {
  const float length = sqrtf(v3_dot(x, x));
  *norm = length;
  if (length == 0.0f) {
    *direction = x;
    return;
  }
  *direction = v3_make(x.x / length, x.y / length, x.z / length);
}

// 上游 `math.safe_div`：`x / where(y != 0.0, y, MJ_MINVAL)`。
__device__ __forceinline__ float g5_primitive_safe_div(float numerator, float denominator) {
  return numerator / (denominator != 0.0f ? denominator : G5_PRIMITIVE_MINVAL);
}

// 上游 `collision_primitive_core.sphere_box`：
//   center = transpose(box_rot) @ (sphere_pos - box_pos)
//   盒内分支按 6 个面距离取最近面；盒外按最近点与最深点中点。
__device__ __forceinline__ g5_primitive_contact g5_sphere_box(
    v3 sphere_pos, float sphere_radius, v3 box_pos, const float* box_rot, v3 box_size) {
  const v3 center = transpose_mul_mat3_vec(box_rot, v3_sub(sphere_pos, box_pos));

  // 上游 `wp.max(-box_size, wp.min(box_size, center))`（分量式）。
  const v3 clamped = v3_make(fmaxf(-box_size.x, fminf(box_size.x, center.x)),
                             fmaxf(-box_size.y, fminf(box_size.y, center.y)),
                             fmaxf(-box_size.z, fminf(box_size.z, center.z)));
  v3 clamped_dir;
  float dist;
  g5_primitive_normalize_with_norm(v3_sub(clamped, center), &clamped_dir, &dist);

  g5_primitive_contact contact;
  if (dist <= G5_PRIMITIVE_MINVAL) {
    // 球心在盒内：枚举 6 个面，取 `closest > face_dist` 的严格更小者（同距取首个）。
    float closest = 2.0f * (box_size.x + box_size.y + box_size.z);
    const float half[3] = {box_size.x, box_size.y, box_size.z};
    const float center_axis[3] = {center.x, center.y, center.z};
    int k = 0;
    for (int i = 0; i < 6; ++i) {
      const float sign = (i % 2) ? 1.0f : -1.0f;
      const float face_dist = fabsf(sign * half[i / 2] - center_axis[i / 2]);
      if (closest > face_dist) {
        closest = face_dist;
        k = i;
      }
    }
    float nearest[3] = {0.0f, 0.0f, 0.0f};
    nearest[k / 2] = (k % 2) ? -1.0f : 1.0f;
    const v3 nearest_v = v3_make(nearest[0], nearest[1], nearest[2]);
    const v3 scaled = v3_scale(nearest_v, sphere_radius - closest);
    const v3 offset = v3_make(scaled.x / 2.0f, scaled.y / 2.0f, scaled.z / 2.0f);
    const v3 pos = v3_add(center, offset);
    contact.dist = -closest - sphere_radius;
    contact.normal = mul_mat3_vec(box_rot, nearest_v);
    contact.pos = v3_add(box_pos, mul_mat3_vec(box_rot, pos));
  } else {
    // 球心在盒外：沿最近面方向把球心推进球半径，取最近点与最深点中点。
    const v3 deepest = v3_add(center, v3_scale(clamped_dir, sphere_radius));
    const v3 pos = v3_scale(v3_add(clamped, deepest), 0.5f);
    contact.dist = dist - sphere_radius;
    contact.normal = mul_mat3_vec(box_rot, clamped_dir);
    contact.pos = v3_add(box_pos, mul_mat3_vec(box_rot, pos));
  }
  return contact;
}

// 上游 `collision_primitive_core.capsule_box`（逐行对照 mjc 实现）：
// 先找与盒面/盒边最近的胶囊端点或边中点，再对 1~2 个球端点调用 `sphere_box`。
__device__ __forceinline__ g5_primitive_pair g5_capsule_box(
    v3 capsule_pos, v3 capsule_axis, float capsule_radius, float capsule_half_length,
    v3 box_pos, const float* box_rot, v3 box_size) {
  const v3 pos = transpose_mul_mat3_vec(box_rot, v3_sub(capsule_pos, box_pos));
  const v3 axis = transpose_mul_mat3_vec(box_rot, capsule_axis);
  const v3 halfaxis = v3_scale(axis, capsule_half_length);
  const int axisdir = (halfaxis.x > 0.0f ? 1 : 0) + 2 * (halfaxis.y > 0.0f ? 1 : 0) +
                      4 * (halfaxis.z > 0.0f ? 1 : 0);

  const float size[3] = {box_size.x, box_size.y, box_size.z};
  const float halfaxis_axis[3] = {halfaxis.x, halfaxis.y, halfaxis.z};
  const float axis_axis[3] = {axis.x, axis.y, axis.z};

  // 最近点跟踪（初值与上游一致）。
  float bestdist = 1.0e32f;
  float bestsegmentpos = -12.0f;
  int cltype = -4;
  int clface = -12;

  // 第一段：考虑盒面最近的配置（胶囊两端点向盒面投影）。
  for (int i = -1; i <= 1; i += 2) {
    const v3 axis_tip = v3_add(pos, v3_scale(halfaxis, (float)i));
    float box_point[3] = {axis_tip.x, axis_tip.y, axis_tip.z};
    int n_out = 0;
    int ax_out = -1;
    for (int j = 0; j < 3; ++j) {
      if (box_point[j] < -size[j]) {
        n_out += 1;
        ax_out = j;
        box_point[j] = -size[j];
      } else if (box_point[j] > size[j]) {
        n_out += 1;
        ax_out = j;
        box_point[j] = size[j];
      }
    }
    if (n_out > 1) continue;
    const v3 box_point_v = v3_make(box_point[0], box_point[1], box_point[2]);
    const v3 delta = v3_sub(box_point_v, axis_tip);
    const float dist = v3_dot(delta, delta);
    if (dist < bestdist) {
      bestdist = dist;
      bestsegmentpos = (float)i;
      cltype = -2 + i;
      clface = ax_out;
    }
  }

  // 第二段：考虑盒边最近的配置（i 编码盒角点，j 为被置零的边轴）。
  int clcorner = -123;
  int cledge = -123;
  float bestboxpos = 0.0f;
  for (int i = 0; i < 8; ++i) {
    for (int j = 0; j < 3; ++j) {
      if ((i & (1 << j)) != 0) continue;
      int c2 = -123;
      float box_pt[3] = {(i & 1) ? size[0] : -size[0], (i & 2) ? size[1] : -size[1],
                         (i & 4) ? size[2] : -size[2]};
      box_pt[j] = 0.0f;
      const v3 box_pt_v = v3_make(box_pt[0], box_pt[1], box_pt[2]);
      v3 dif = v3_sub(box_pt_v, pos);
      float dif_axis[3] = {dif.x, dif.y, dif.z};

      // 胶囊轴与盒边的最近点（两线段参数 x1/x2，clamp 到 [-1, 1]）。
      const float u = -size[j] * dif_axis[j];
      const float v = v3_dot(halfaxis, dif);
      const float ma = size[j] * size[j];
      const float mb = -size[j] * halfaxis_axis[j];
      const float mc = capsule_half_length * capsule_half_length;
      const float det = ma * mc - mb * mb;
      if (fabsf(det) < G5_PRIMITIVE_MINVAL) continue;
      const float idet = 1.0f / det;
      // sX：X=1 表示线段中部，0/2 表示某一端。
      float x1 = (mc * u - mb * v) * idet;
      float x2 = (ma * v - mb * u) * idet;
      int s1 = 1;
      int s2 = 1;
      if (x1 > 1.0f) {
        x1 = 1.0f;
        s1 = 2;
        x2 = g5_primitive_safe_div(v - mb, mc);
      } else if (x1 < -1.0f) {
        x1 = -1.0f;
        s1 = 0;
        x2 = g5_primitive_safe_div(v + mb, mc);
      }
      const bool x2_over = x2 > 1.0f;
      if (x2_over || x2 < -1.0f) {
        if (x2_over) {
          x2 = 1.0f;
          s2 = 2;
          x1 = g5_primitive_safe_div(u - mb, ma);
        } else {
          x2 = -1.0f;
          s2 = 0;
          x1 = g5_primitive_safe_div(u + mb, ma);
        }
        if (x1 > 1.0f) {
          x1 = 1.0f;
          s1 = 2;
        } else if (x1 < -1.0f) {
          x1 = -1.0f;
          s1 = 0;
        }
      }

      dif = v3_sub(dif, v3_scale(halfaxis, x2));
      dif_axis[0] = dif.x;
      dif_axis[1] = dif.y;
      dif_axis[2] = dif.z;
      dif_axis[j] += size[j] * x1;
      const int ct = s1 * 3 + s2;
      const v3 dif_v = v3_make(dif_axis[0], dif_axis[1], dif_axis[2]);
      const float dif_sq = v3_dot(dif_v, dif_v);
      if (dif_sq < bestdist - G5_PRIMITIVE_MINVAL) {
        bestdist = dif_sq;
        bestsegmentpos = x2;
        bestboxpos = x1;
        // ct<6：盒上最近点在边的下端或中部。
        c2 = ct / 6;
        clcorner = i + (1 << j) * c2;
        cledge = j;
        cltype = ct;
      }
    }
  }

  // 平行盒面的方向选择（对应上游 planar 输出帧的 c1 编码）。
  const float p[2] = {pos.x, pos.y};
  const float dd[2] = {halfaxis.x, halfaxis.y};
  const float s[2] = {box_size.x, box_size.y};
  float secondpos = -4.0f;
  const float uu = dd[0] * s[1];
  const float vv = dd[1] * s[0];
  const bool w_neg = dd[0] * p[1] - dd[1] * p[0] < 0.0f;
  float best = -1.0f;
  const float ee1 = uu - vv;
  const float ee2 = uu + vv;
  int c1 = 0;
  if (fabsf(ee1) > best) {
    best = fabsf(ee1);
    c1 = ((ee1 < 0.0f) == w_neg) ? 0 : 3;
  }
  if (fabsf(ee2) > best) {
    best = fabsf(ee2);
    c1 = ((ee2 > 0.0f) == w_neg) ? 1 : 2;
  }

  if (cltype == -4) {
    // 上游无效配置：两个接触距离都是 MJ_MAXVAL，位置/法线为空（wrapper 仍会写两行，
    // `write_contact` 因 `detected == false` 全部跳过）。
    g5_primitive_pair empty;
    empty.contact[0].dist = G5_PRIMITIVE_MAXVAL;
    empty.contact[0].pos = v3_make(0.0f, 0.0f, 0.0f);
    empty.contact[0].normal = v3_make(0.0f, 0.0f, 0.0f);
    empty.contact[1] = empty.contact[0];
    return empty;
  }

  if (cltype >= 0 && cltype / 3 != 1) {
    // 盒角点最近：可能沿胶囊轴再生成第二接触。
    c1 = axisdir ^ clcorner;
    if (c1 != 0 && c1 != 7) {
      int mul;
      if (c1 == 1 || c1 == 2 || c1 == 4) {
        mul = 1;
      } else {
        mul = -1;
        c1 = 7 - c1;
      }
      int ax = 0;
      int ax1 = 0;
      int ax2 = 0;
      if (c1 == 1) {
        ax = 0;
        ax1 = 1;
        ax2 = 2;
      } else if (c1 == 2) {
        ax = 1;
        ax1 = 2;
        ax2 = 0;
      } else if (c1 == 4) {
        ax = 2;
        ax1 = 0;
        ax2 = 1;
      }
      if (axis_axis[ax] * axis_axis[ax] > 0.5f) {
        // 第二点沿盒边。
        const float m = 2.0f * g5_primitive_safe_div(size[ax], fabsf(halfaxis_axis[ax]));
        secondpos = fminf(1.0f - (float)mul * bestsegmentpos, m);
      } else {
        // 第二点沿盒面。
        const float m = 2.0f * fminf(g5_primitive_safe_div(size[ax1], fabsf(halfaxis_axis[ax1])),
                                     g5_primitive_safe_div(size[ax2], fabsf(halfaxis_axis[ax2])));
        secondpos = -fminf(1.0f + (float)mul * bestsegmentpos, m);
      }
      secondpos *= (float)mul;
    }
  } else if (cltype >= 0 && cltype / 3 == 1) {
    // 盒边最近：T 配置（c1 = 2^n）无第二接触，X 配置可能生成第二接触。
    c1 = axisdir ^ clcorner;
    c1 &= 7 - (1 << cledge);
    if (c1 == 1 || c1 == 2 || c1 == 4) {
      int ax1 = 0;
      int ax2 = 0;
      if (cledge == 0) {
        ax1 = 1;
        ax2 = 2;
      }
      if (cledge == 1) {
        ax1 = 2;
        ax2 = 0;
      }
      if (cledge == 2) {
        ax1 = 0;
        ax2 = 1;
      }
      const int ax = cledge;
      if (fabsf(axis_axis[ax1]) > fabsf(axis_axis[ax2])) ax1 = ax2;
      ax2 = 3 - ax - ax1;
      int mul;
      if (c1 & (1 << ax2)) {
        mul = 1;
        secondpos = 1.0f - bestsegmentpos;
      } else {
        mul = -1;
        secondpos = 1.0f + bestsegmentpos;
      }
      float e1 = 2.0f * g5_primitive_safe_div(size[ax2], fabsf(halfaxis_axis[ax2]));
      secondpos = fminf(e1, secondpos);
      float e2;
      if (((axisdir & (1 << ax)) != 0) == ((c1 & (1 << ax2)) != 0)) {
        e2 = 1.0f - bestboxpos;
      } else {
        e2 = 1.0f + bestboxpos;
      }
      e1 = size[ax] * g5_primitive_safe_div(e2, fabsf(halfaxis_axis[ax]));
      secondpos = fminf(e1, secondpos);
      secondpos *= (float)mul;
    }
  } else if (cltype < 0) {
    // 胶囊端点最近于盒面：按第二方向找盒面/边上的最远可用点。
    if (clface != -1) {
      const int mul = (cltype == -3) ? 1 : -1;
      secondpos = 2.0f;
      const v3 tmp1 = v3_sub(pos, v3_scale(halfaxis, (float)mul));
      const float tmp1_axis[3] = {tmp1.x, tmp1.y, tmp1.z};
      for (int i = 0; i < 3; ++i) {
        if (i != clface) {
          const float ha_r = g5_primitive_safe_div((float)mul, halfaxis_axis[i]);
          float e1 = (size[i] - tmp1_axis[i]) * ha_r;
          if (0.0f < e1 && e1 < secondpos) secondpos = e1;
          e1 = (-size[i] - tmp1_axis[i]) * ha_r;
          if (0.0f < e1 && e1 < secondpos) secondpos = e1;
        }
      }
      secondpos *= (float)mul;
    }
  }

  // 第一接触：胶囊轴参数 bestsegmentpos 处的球端点。
  const v3 s1_pos_l = v3_add(pos, v3_scale(halfaxis, bestsegmentpos));
  const v3 s1_pos_g = v3_add(mul_mat3_vec(box_rot, s1_pos_l), box_pos);
  g5_primitive_pair pair;
  pair.contact[0] = g5_sphere_box(s1_pos_g, capsule_radius, box_pos, box_rot, box_size);

  if (secondpos > -3.0f) {
    const v3 s2_pos_l = v3_add(pos, v3_scale(halfaxis, secondpos + bestsegmentpos));
    const v3 s2_pos_g = v3_add(mul_mat3_vec(box_rot, s2_pos_l), box_pos);
    pair.contact[1] = g5_sphere_box(s2_pos_g, capsule_radius, box_pos, box_rot, box_size);
  } else {
    pair.contact[1].dist = G5_PRIMITIVE_MAXVAL;
    pair.contact[1].pos = v3_make(0.0f, 0.0f, 0.0f);
    pair.contact[1].normal = v3_make(0.0f, 0.0f, 0.0f);
  }
  return pair;
}
