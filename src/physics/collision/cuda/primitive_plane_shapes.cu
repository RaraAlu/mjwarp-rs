// G10@P5 B1 plane 族数值核心（shapes：sphere / box / capsule；ellipsoid / cylinder 下一块）。
// 语义来源 pinned collision_primitive_core.py；复用既有 g5_write_contact/make_frame（不改 ABI）。
#include "common.cu"

// 上游 `collision_primitive_core.plane_sphere`：
//   dist = dot(sphere_pos - plane_pos, plane_normal) - radius
//   pos  = sphere_pos - plane_normal * (radius + 0.5 * dist)
__device__ __forceinline__ void g5_plane_sphere(v3 plane_normal, v3 plane_pos, v3 sphere_pos,
                                                float sphere_radius, float* dist, v3* pos) {
  *dist = v3_dot(v3_sub(sphere_pos, plane_pos), plane_normal) - sphere_radius;
  *pos = v3_sub(sphere_pos, v3_scale(plane_normal, sphere_radius + 0.5f * (*dist)));
}



// 上游 `plane_box_wrapper`（pinned L761-837）几何核心：8 角点逐点距离与位置。
// 每角点：corner = box_pos + box_rot * (size * sign)；dist = dot(corner - plane_pos, normal)；
// pos = corner - 0.5 * dist * normal（pinned plane_convex L269-271 同式）。
__device__ __forceinline__ int g5_plane_box(v3 plane_normal, v3 plane_pos, v3 box_pos,
                                            const float* box_rot, v3 box_size,
                                            g5_plane_contact out[8]) {
  // pinned `plane_box`（core L366-390）：center_dist = dot(box_pos - plane_pos, n)；
  // ldist = dot(n, rot@corner_local)；cdist = center_dist + ldist；
  // pos = rot@corner_local + box_pos - (0.5*plane_normal)*cdist（左结合，逐式对齐 pinned）。
  const float center_dist = v3_dot(v3_sub(box_pos, plane_pos), plane_normal);
  int count = 0;
  for (int i = 0; i < 8; ++i) {
    const float sx = (i & 1) ? 1.0f : -1.0f;
    const float sy = (i & 2) ? 1.0f : -1.0f;
    const float sz = (i & 4) ? 1.0f : -1.0f;
    const v3 local = v3_make(sx * box_size.x, sy * box_size.y, sz * box_size.z);
    const v3 rotated = mul_mat3_vec(box_rot, local);
    const float ldist = v3_dot(plane_normal, rotated);
    const float cdist = center_dist + ldist;
    out[count].dist = cdist;
    out[count].pos = v3_sub(v3_add(rotated, box_pos),
                            v3_scale(v3_scale(plane_normal, 0.5f), cdist));
    out[count].normal = plane_normal;
    count++;
  }
  return count;
}




__device__ __forceinline__ int g5_plane_capsule(v3 plane_normal, v3 plane_pos, v3 capsule_pos,
                                                v3 capsule_axis, float capsule_radius,
                                                float capsule_half_length,
                                                g5_plane_contact out[2], float frame[9]) {
  const v3 n = plane_normal;
  const v3 axis = capsule_axis;

  // b = normalize(axis - n*dot(n,axis))，零保护沿用上游 `normalize_with_norm`（L287）。
  v3 b = v3_make(0.0f, 0.0f, 0.0f);
  float b_norm = 0.0f;
  const v3 perpendicular = v3_sub(axis, v3_scale(n, v3_dot(n, axis)));
  const float length = sqrtf(v3_dot(perpendicular, perpendicular));
  b_norm = length;
  if (length == 0.0f) {
    b = perpendicular;
  } else {
    b = v3_make(perpendicular.x / length, perpendicular.y / length, perpendicular.z / length);
  }
  if (b_norm < 0.5f) {
    if (-0.5f < n.y && n.y < 0.5f) {
      b = v3_make(0.0f, 1.0f, 0.0f);
    } else {
      b = v3_make(0.0f, 0.0f, 1.0f);
    }
  }
  const v3 c = v3_cross(n, b);
  g5_plane_frame(n, b, c, frame);

  const v3 segment = v3_scale(axis, capsule_half_length);
  g5_plane_sphere(n, plane_pos, v3_add(capsule_pos, segment), capsule_radius, &out[0].dist, &out[0].pos);
  out[0].normal = n;
  g5_plane_sphere(n, plane_pos, v3_sub(capsule_pos, segment), capsule_radius, &out[1].dist, &out[1].pos);
  out[1].normal = n;
  return 2;
}


// ---------------------------------------------------------------------------------------------
// plane_ellipsoid / plane_cylinder 数值核心（pinned collision_primitive_core.py L311-339 / L466-559）
// ---------------------------------------------------------------------------------------------

// 上游 `plane_ellipsoid`（pinned L312-339）：
//   sphere_support = -normalize(cw_mul(transpose(rot) @ n, size))
//   pos = ellipsoid_pos + rot @ cw_mul(sphere_support, size)
//   dist = dot(n, pos - plane_pos); pos -= n * dist * 0.5
// 退化保护（本片追加，非上游路径）：size 或局部法线缩放到 0 时返回 0 接触，不产生 NaN。
__device__ __forceinline__ int g5_plane_ellipsoid(v3 plane_normal, v3 plane_pos, v3 ellipsoid_pos,
                                                  const float* ellipsoid_rot, v3 ellipsoid_size,
                                                  g5_plane_contact out[1]) {
  const v3 local_n = transpose_mul_mat3_vec(ellipsoid_rot, plane_normal);
  const v3 scaled = v3_make(local_n.x * ellipsoid_size.x, local_n.y * ellipsoid_size.y,
                            local_n.z * ellipsoid_size.z);
  const float length = sqrtf(v3_dot(scaled, scaled));
  if (!(length > 0.0f) || !isfinite(length)) {
    return 0;
  }
  const v3 normalized = v3_make(scaled.x / length, scaled.y / length, scaled.z / length);
  const v3 sphere_support = v3_make(-normalized.x, -normalized.y, -normalized.z);
  const v3 local = v3_make(sphere_support.x * ellipsoid_size.x,
                           sphere_support.y * ellipsoid_size.y,
                           sphere_support.z * ellipsoid_size.z);
  v3 pos = v3_add(ellipsoid_pos, mul_mat3_vec(ellipsoid_rot, local));
  const float dist = v3_dot(plane_normal, v3_sub(pos, plane_pos));
  pos = v3_sub(pos, v3_scale(v3_scale(plane_normal, dist), 0.5f));
  out[0].dist = dist;
  out[0].pos = pos;
  out[0].normal = plane_normal;
  return 1;
}

// 上游 `plane_cylinder`（pinned L466-559）：4 条候选（两端面 + 侧面三角两点）。
// 退化保护：`len_sqr >= 1e-12` 走 normalize/safe_div，否则回退 x 轴 * radius（pinned L513-517）。
__device__ __forceinline__ int g5_plane_cylinder(v3 plane_normal, v3 plane_pos, v3 cylinder_center,
                                                 v3 cylinder_axis, float cylinder_radius,
                                                 float cylinder_half_height,
                                                 g5_plane_contact out[4]) {
  const v3 n = plane_normal;
  v3 axis = cylinder_axis;
  float prjaxis = v3_dot(n, axis);
  if (prjaxis > 0.0f) {
    axis = v3_scale(axis, -1.0f);
    prjaxis = -prjaxis;
  }
  const float dist0 = v3_dot(v3_sub(cylinder_center, plane_pos), n);
  v3 vec = v3_sub(v3_scale(axis, prjaxis), n);
  const float len_sqr = v3_dot(vec, vec);
  if (len_sqr >= 1.0e-12f) {
    const float len = sqrtf(len_sqr);
    const float denominator = len != 0.0f ? len : 1.0e-15f;
    vec = v3_scale(vec, cylinder_radius / denominator);
  } else {
    vec = v3_make(cylinder_radius, 0.0f, 0.0f);
  }
  const float prjvec = v3_dot(vec, n);
  axis = v3_scale(axis, cylinder_half_height);
  prjaxis = prjaxis * cylinder_half_height;

  // 接触 1/2：两端面（pinned L526-538）。
  const float dist1 = dist0 + prjaxis + prjvec;
  out[0].dist = dist1;
  out[0].pos = v3_sub(v3_add(v3_add(cylinder_center, vec), axis), v3_scale(n, dist1 * 0.5f));
  out[0].normal = n;
  const float dist2 = dist0 - prjaxis + prjvec;
  out[1].dist = dist2;
  out[1].pos = v3_sub(v3_sub(v3_add(cylinder_center, vec), axis), v3_scale(n, dist2 * 0.5f));
  out[1].normal = n;

  // 接触 3/4：侧面近平面三角两点（pinned L540-557）。
  const float prjvec1 = -prjvec * 0.5f;
  const float dist3 = dist0 + prjaxis + prjvec1;
  v3 vec1 = v3_cross(vec, axis);
  const float vec1_len = sqrtf(v3_dot(vec1, vec1));
  const float vec1_denominator = vec1_len != 0.0f ? vec1_len : 1.0e-15f;
  vec1 = v3_make(vec1.x / vec1_denominator, vec1.y / vec1_denominator, vec1.z / vec1_denominator);
  vec1 = v3_scale(vec1, cylinder_radius * sqrtf(3.0f) * 0.5f);
  const v3 base = v3_sub(v3_add(cylinder_center, axis), v3_scale(vec, 0.5f));
  out[2].dist = dist3;
  out[2].pos = v3_sub(v3_add(base, vec1), v3_scale(n, dist3 * 0.5f));
  out[2].normal = n;
  out[3].dist = dist3;
  out[3].pos = v3_sub(v3_sub(base, vec1), v3_scale(n, dist3 * 0.5f));
  out[3].normal = n;
  return 4;
}

