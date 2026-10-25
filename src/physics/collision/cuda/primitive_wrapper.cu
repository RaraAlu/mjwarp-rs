// G10@P5 G5：上游 `sphere_box_wrapper` / `capsule_box_wrapper` 的接触写入设备版。
// Adapted from frozen MJWarp `collision_primitive.py` (sphere_box_wrapper L1087-1157,
// capsule_box_wrapper L1160-1246), revision 71da24d. Apache-2.0.
//
// 只做「取几何字段 → 调 primitive 数值内核 → 逐接触 make_frame + write_contact」，
// 不复制上游 wrapper 参数列表（margin/gap/condim/friction/... 已在 `d_contact_params` 里）。
#include "contact.cu"
#include "primitive.cu"

// 上游 `sphere_box_wrapper`：单接触，`write_contact(id_=0)`。
__device__ __forceinline__ void g5_sphere_box_wrapper(const d_geom& sphere, const d_geom& box,
                                                      const d_contact_params& params,
                                                      int naconmax, d_contact_out* out) {
  const g5_primitive_contact contact =
      g5_sphere_box(sphere.pos, sphere.size.x, box.pos, box.rot, box.size);
  float frame[9];
  make_frame(contact.normal, frame);
  g5_write_contact(params, naconmax, 0, contact.dist, contact.pos, frame, out);
}

// 上游 `capsule_box_wrapper`：两接触（`id_ = 0/1`）；无效第二接触的 `dist = MJ_MAXVAL`
// 由 `write_contact` 的 `detected` 门跳过，与上游计数语义一致。
__device__ __forceinline__ void g5_capsule_box_wrapper(const d_geom& capsule, const d_geom& box,
                                                       const d_contact_params& params,
                                                       int naconmax, d_contact_out* out) {
  // 上游 `axis = vec3(cap.rot[0, 2], cap.rot[1, 2], cap.rot[2, 2])`（行优先 mat33 的第三列）。
  const v3 axis = v3_make(capsule.rot[2], capsule.rot[5], capsule.rot[8]);
  const g5_primitive_pair pair =
      g5_capsule_box(capsule.pos, axis, capsule.size.x, capsule.size.y, box.pos, box.rot,
                     box.size);
  for (int i = 0; i < 2; ++i) {
    const g5_primitive_contact& contact = pair.contact[i];
    float frame[9];
    make_frame(contact.normal, frame);
    g5_write_contact(params, naconmax, i, contact.dist, contact.pos, frame, out);
  }
}
