// G10@P5 B2 sphere/capsule 族 primitive 数值核心（sphere-sphere / sphere-capsule /
// capsule-capsule / sphere-cylinder）。
// Adapted from frozen MJWarp `collision_primitive_core.py`
// （`sphere_sphere` L58-87、`sphere_capsule` L90-125、`capsule_capsule` L128-255、
// `sphere_cylinder` L393-462），revision 71da24d956378a87a703b6e1442b13aec0c4ac29。Apache-2.0。
//
// 逐表达式按 pinned 顺序转写（含 zero-distance fallback、平行/退化分支、端面+侧面+角点分支）；
// 上层 wrapper 与接触写入见 `primitive_sphere_capsule_wrapper.cu`。0 新 ABI 字段。
#include "primitive.cu"
#include "primitive_plane_shapes.cu"

// 上游 `wp.inf`（`capsule_capsule` 无效接触距离；`write_contact` 的 `detected` 门会跳过）。
#define G5_SC_INF __int_as_float(0x7f800000)

// 上游 `collision_primitive_core.sphere_sphere`：
//   dir = pos2 - pos1；dist = |dir|；dist == 0 时 n = (1,0,0)，否则 n = dir / dist；
//   dist -= (radius1 + radius2)；pos = pos1 + n * (radius1 + 0.5 * dist)。
__device__ __forceinline__ g5_primitive_contact g5_sphere_sphere(v3 pos1, float radius1, v3 pos2,
                                                                 float radius2) {
  const v3 dir = v3_sub(pos2, pos1);
  float dist = v3_length(dir);
  v3 n;
  if (dist == 0.0f) {
    n = v3_make(1.0f, 0.0f, 0.0f);
  } else {
    // pinned `dir / dist`：逐分量除法（不是乘倒数）。
    n = v3_make(dir.x / dist, dir.y / dist, dir.z / dist);
  }
  dist = dist - (radius1 + radius2);
  const v3 pos = v3_add(pos1, v3_scale(n, radius1 + 0.5f * dist));
  g5_primitive_contact contact;
  contact.dist = dist;
  contact.pos = pos;
  contact.normal = n;
  return contact;
}

// 上游 `collision_primitive_core.sphere_capsule`：把球心投影到胶囊中心线并 clamp，
// 再对最近点调 `sphere_sphere`。
__device__ __forceinline__ g5_primitive_contact g5_sphere_capsule(v3 sphere_pos, float sphere_radius,
                                                                 v3 capsule_pos, v3 capsule_axis,
                                                                 float capsule_radius,
                                                                 float capsule_half_length) {
  const v3 offset = v3_sub(sphere_pos, capsule_pos);
  // 上游 `wp.clamp(dot(axis, offset), -half_length, half_length)`。
  const float projection = fminf(fmaxf(v3_dot(capsule_axis, offset), -capsule_half_length),
                                 capsule_half_length);
  const v3 pt = v3_add(capsule_pos, v3_scale(capsule_axis, projection));
  return g5_sphere_sphere(sphere_pos, sphere_radius, pt, capsule_radius);
}

// 上游 `collision_primitive_core.capsule_capsule`：非平行轴取 1 个接触（含 3 处分段
// clamp 修正），平行轴最多取 2 个端点接触；`margin` 过滤（`dist <= margin` 才写）。
// 返回两个接触槽；无效槽 `dist = wp.inf`（由 `write_contact` 跳过）。
__device__ __forceinline__ void g5_capsule_capsule_core(
    v3 cap1_pos, v3 cap1_axis, float cap1_radius, float cap1_half_length, v3 cap2_pos,
    v3 cap2_axis, float cap2_radius, float cap2_half_length, float margin,
    g5_primitive_pair* pair) {
  // 上游 `contact_dist = vec2(inf, inf)`、`contact_pos/contact_normal = mat23f()`（全零）。
  pair->contact[0].dist = G5_SC_INF;
  pair->contact[0].pos = v3_make(0.0f, 0.0f, 0.0f);
  pair->contact[0].normal = v3_make(0.0f, 0.0f, 0.0f);
  pair->contact[1] = pair->contact[0];

  // 缩放轴与中心差。
  const v3 axis1 = v3_scale(cap1_axis, cap1_half_length);
  const v3 axis2 = v3_scale(cap2_axis, cap2_half_length);
  const v3 dif = v3_sub(cap1_pos, cap2_pos);

  // 矩阵系数与行列式。
  const float ma = v3_dot(axis1, axis1);
  const float mb = -v3_dot(axis1, axis2);
  const float mc = v3_dot(axis2, axis2);
  const float u = -v3_dot(axis1, dif);
  const float v = v3_dot(axis2, dif);
  const float det = ma * mc - mb * mb;

  if (fabsf(det) >= G5_PRIMITIVE_MINVAL) {
    // 非平行轴：1 个接触。
    const float inv_det = 1.0f / det;
    float x1 = (mc * u - mb * v) * inv_det;
    float x2 = (ma * v - mb * u) * inv_det;

    if (x1 > 1.0f) {
      x1 = 1.0f;
      x2 = (v - mb) / mc;
    } else if (x1 < -1.0f) {
      x1 = -1.0f;
      x2 = (v + mb) / mc;
    }

    if (x2 > 1.0f) {
      x2 = 1.0f;
      x1 = fminf(fmaxf((u - mb) / ma, -1.0f), 1.0f);
    } else if (x2 < -1.0f) {
      x2 = -1.0f;
      x1 = fminf(fmaxf((u + mb) / ma, -1.0f), 1.0f);
    }

    const v3 vec1 = v3_add(cap1_pos, v3_scale(axis1, x1));
    const v3 vec2 = v3_add(cap2_pos, v3_scale(axis2, x2));
    const g5_primitive_contact contact = g5_sphere_sphere(vec1, cap1_radius, vec2, cap2_radius);
    if (contact.dist <= margin) {
      pair->contact[0] = contact;
    }
  } else {
    // 平行轴：逐对测试 4 个端点，保留通过 margin 检查的前 2 个。
    int contact_count = 0;

    // x1 = 1：胶囊 1 正端。
    {
      const v3 vec1 = v3_add(cap1_pos, axis1);
      const float x2 = fminf(fmaxf((v - mb) / mc, -1.0f), 1.0f);
      const v3 vec2 = v3_add(cap2_pos, v3_scale(axis2, x2));
      const g5_primitive_contact contact = g5_sphere_sphere(vec1, cap1_radius, vec2, cap2_radius);
      if (contact.dist <= margin) {
        pair->contact[contact_count] = contact;
        contact_count += 1;
      }
    }

    // x1 = -1：胶囊 1 负端。
    {
      const v3 vec1 = v3_sub(cap1_pos, axis1);
      const float x2 = fminf(fmaxf((v + mb) / mc, -1.0f), 1.0f);
      const v3 vec2 = v3_add(cap2_pos, v3_scale(axis2, x2));
      const g5_primitive_contact contact = g5_sphere_sphere(vec1, cap1_radius, vec2, cap2_radius);
      if (contact.dist <= margin) {
        pair->contact[contact_count] = contact;
        contact_count += 1;
      }
    }

    // x2 = 1：胶囊 2 正端（仅在仍缺第 2 个接触时测试）。
    if (contact_count < 2) {
      const v3 vec2 = v3_add(cap2_pos, axis2);
      const float x1 = fminf(fmaxf((u - mb) / ma, -1.0f), 1.0f);
      const v3 vec1 = v3_add(cap1_pos, v3_scale(axis1, x1));
      const g5_primitive_contact contact = g5_sphere_sphere(vec1, cap1_radius, vec2, cap2_radius);
      if (contact.dist <= margin) {
        pair->contact[contact_count] = contact;
        contact_count += 1;
      }
    }

    // x2 = -1：胶囊 2 负端（仅在仍缺第 2 个接触时测试）。
    if (contact_count < 2) {
      const v3 vec2 = v3_sub(cap2_pos, axis2);
      const float x1 = fminf(fmaxf((u + mb) / ma, -1.0f), 1.0f);
      const v3 vec1 = v3_add(cap1_pos, v3_scale(axis1, x1));
      const g5_primitive_contact contact = g5_sphere_sphere(vec1, cap1_radius, vec2, cap2_radius);
      if (contact.dist <= margin) {
        pair->contact[contact_count] = contact;
      }
    }
  }
}

// 上游 `collision_primitive_core.sphere_cylinder`：侧面 / 端面 / 角点三分支。
// 端面分支复用 `plane_sphere` 后反转法线（pinned 注释 `flip normal after position calculation`）。
__device__ __forceinline__ g5_primitive_contact g5_sphere_cylinder(
    v3 sphere_pos, float sphere_radius, v3 cylinder_pos, v3 cylinder_axis, float cylinder_radius,
    float cylinder_half_height) {
  const v3 vec = v3_sub(sphere_pos, cylinder_pos);
  const float x = v3_dot(vec, cylinder_axis);

  const v3 a_proj = v3_scale(cylinder_axis, x);
  v3 p_proj = v3_sub(vec, a_proj);
  const float p_proj_sqr = v3_dot(p_proj, p_proj);

  bool collide_side = fabsf(x) < cylinder_half_height;
  bool collide_cap = p_proj_sqr < (cylinder_radius * cylinder_radius);

  if (collide_side && collide_cap) {
    const float dist_cap = cylinder_half_height - fabsf(x);
    const float dist_radius = cylinder_radius - sqrtf(p_proj_sqr);

    if (dist_cap < dist_radius) {
      collide_side = false;
    } else {
      collide_cap = false;
    }
  }

  // 侧面碰撞。
  if (collide_side) {
    const v3 pos_target = v3_add(cylinder_pos, a_proj);
    return g5_sphere_sphere(sphere_pos, sphere_radius, pos_target, cylinder_radius);
  }
  // 端面碰撞。
  if (collide_cap) {
    v3 pos_cap;
    v3 plane_normal;
    if (x > 0.0f) {
      // 顶面。
      pos_cap = v3_add(cylinder_pos, v3_scale(cylinder_axis, cylinder_half_height));
      plane_normal = cylinder_axis;
    } else {
      // 底面。
      pos_cap = v3_sub(cylinder_pos, v3_scale(cylinder_axis, cylinder_half_height));
      plane_normal = v3_scale(cylinder_axis, -1.0f);
    }

    float dist = 0.0f;
    v3 pos = v3_make(0.0f, 0.0f, 0.0f);
    g5_plane_sphere(plane_normal, pos_cap, sphere_pos, sphere_radius, &dist, &pos);
    g5_primitive_contact contact;
    contact.dist = dist;
    contact.pos = pos;
    contact.normal = v3_scale(plane_normal, -1.0f);
    return contact;
  }
  // 角点碰撞。
  const float inv_len = g5_primitive_safe_div(1.0f, sqrtf(p_proj_sqr));
  p_proj = v3_scale(p_proj, cylinder_radius * inv_len);

  // 上游 `wp.sign(x)`：x > 0 → 1，x < 0 → -1，x == 0 → 0。
  const float sign = (x > 0.0f) ? 1.0f : ((x < 0.0f) ? -1.0f : 0.0f);
  const v3 cap_offset = v3_scale(cylinder_axis, sign * cylinder_half_height);
  const v3 pos_corner = v3_add(v3_add(cylinder_pos, cap_offset), p_proj);

  return g5_sphere_sphere(sphere_pos, sphere_radius, pos_corner, 0.0f);
}
