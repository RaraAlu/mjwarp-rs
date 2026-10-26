// G10@P5 B2 sphere/capsule 族 primitive 接触写入设备版。
// Adapted from frozen MJWarp `collision_primitive.py`
// （`sphere_sphere_wrapper` L357-429、`sphere_capsule_wrapper` L432-507、
// `capsule_capsule_wrapper` L510-597、`sphere_cylinder_wrapper` L915-1000），
// revision 71da24d956378a87a703b6e1442b13aec0c4ac29。Apache-2.0。
//
// 只做「取几何字段 → 调数值核心 → 逐接触 make_frame + write_contact」；margin/gap/condim/
// friction/... 已在 `d_contact_params` 里。写入门/顺序按 pinned，不加特判；0 新 ABI 字段。
#include "contact.cu"
#include "primitive.cu"
#include "primitive_sphere_capsule_shapes.cu"

// 上游 `sphere_sphere_wrapper`：单接触，`write_contact(id_=0)`。
__device__ __forceinline__ void g5_sphere_sphere_wrapper(const d_geom& sphere1,
                                                         const d_geom& sphere2,
                                                         const d_contact_params& params,
                                                         int naconmax, d_contact_out* out) {
  const g5_primitive_contact contact =
      g5_sphere_sphere(sphere1.pos, sphere1.size.x, sphere2.pos, sphere2.size.x);
  float frame[9];
  make_frame(contact.normal, frame);
  g5_write_contact(params, naconmax, 0, contact.dist, contact.pos, frame, out);
}

// 上游 `sphere_capsule_wrapper`：单接触，胶囊轴 = 旋转矩阵第三列。
__device__ __forceinline__ void g5_sphere_capsule_wrapper(const d_geom& sphere,
                                                          const d_geom& capsule,
                                                          const d_contact_params& params,
                                                          int naconmax, d_contact_out* out) {
  // 上游 `axis = vec3(cap.rot[0, 2], cap.rot[1, 2], cap.rot[2, 2])`（行优先 mat33 的第三列）。
  const v3 axis = v3_make(capsule.rot[2], capsule.rot[5], capsule.rot[8]);
  const g5_primitive_contact contact =
      g5_sphere_capsule(sphere.pos, sphere.size.x, capsule.pos, axis, capsule.size.x,
                        capsule.size.y);
  float frame[9];
  make_frame(contact.normal, frame);
  g5_write_contact(params, naconmax, 0, contact.dist, contact.pos, frame, out);
}

// 上游 `capsule_capsule_wrapper`：两接触（`id_ = 0/1`）；无效槽 `dist = wp.inf` 由
// `write_contact` 的 `detected` 门跳过，与上游计数语义一致。
__device__ __forceinline__ void g5_capsule_capsule_wrapper(const d_geom& cap1,
                                                           const d_geom& cap2,
                                                           const d_contact_params& params,
                                                           int naconmax, d_contact_out* out) {
  const v3 cap1_axis = v3_make(cap1.rot[2], cap1.rot[5], cap1.rot[8]);
  const v3 cap2_axis = v3_make(cap2.rot[2], cap2.rot[5], cap2.rot[8]);
  g5_primitive_pair pair;
  g5_capsule_capsule_core(cap1.pos, cap1_axis, cap1.size.x, cap1.size.y, cap2.pos, cap2_axis,
                          cap2.size.x, cap2.size.y, params.margin, &pair);
  for (int i = 0; i < 2; ++i) {
    const g5_primitive_contact& contact = pair.contact[i];
    float frame[9];
    make_frame(contact.normal, frame);
    g5_write_contact(params, naconmax, i, contact.dist, contact.pos, frame, out);
  }
}

// 上游 `sphere_cylinder_wrapper`：单接触，圆柱轴 = 旋转矩阵第三列。
__device__ __forceinline__ void g5_sphere_cylinder_wrapper(const d_geom& sphere,
                                                           const d_geom& cylinder,
                                                           const d_contact_params& params,
                                                           int naconmax, d_contact_out* out) {
  const v3 axis = v3_make(cylinder.rot[2], cylinder.rot[5], cylinder.rot[8]);
  const g5_primitive_contact contact =
      g5_sphere_cylinder(sphere.pos, sphere.size.x, cylinder.pos, axis, cylinder.size.x,
                         cylinder.size.y);
  float frame[9];
  make_frame(contact.normal, frame);
  g5_write_contact(params, naconmax, 0, contact.dist, contact.pos, frame, out);
}
