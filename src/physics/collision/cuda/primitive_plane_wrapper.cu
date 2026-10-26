// G10@P5 B1：plane 族 6 个 wrapper 的接触写入门（设备版）。
// 语义来源 pinned `collision_primitive.py`：plane_sphere_wrapper L281、plane_capsule_wrapper L600、
// plane_ellipsoid_wrapper L686、plane_cylinder_wrapper L1000、plane_box_wrapper L761、
// plane_convex_wrapper L838-914；revision 71da24d。设备保持 pinned GPU fma.rn 融合语义。
//
// 写入约定与既有 `primitive_wrapper.cu` 完全一致（不新增写入器、不改 ABI）：
//   `g5_write_contact(params, naconmax, id_, dist, pos, frame, out)`；无效接触的 `dist = MJ_MAXVAL`
//   由 `g5_write_contact` 的 `detected` 门跳过（与上游计数语义一致）。`make_frame` 参数顺序为
//   法线在前（`make_frame(normal, frame)`），与既有 primitive 片相同。
//
// 入口复用：product.cu 的 `mjwarp_collision_primitive` 按 type pair 调用下面 wrapper；
// 本文件不新增 `extern "C"` 入口、不新增 ABI 字段。
#include "contact.cu"
#include "primitive.cu"
#include "primitive_plane_common.cu"
#include "primitive_plane_convex.cu"
#include "primitive_plane_shapes.cu"

// plane 族 wrapper 需要的 mesh 访问（product 参数里已有：mesh_vert / mesh_graph 指针）。
struct g5_plane_mesh_view {
  const float* vert;
  const int* graph;
};

// 上游 `plane_sphere_wrapper`（pinned L281-355）：单接触，id_=0。
__device__ __forceinline__ void g5_plane_sphere_wrapper(const d_geom& plane, const d_geom& sphere,
                                                        const d_contact_params& params,
                                                        int naconmax, d_contact_out* out) {
  const v3 normal = v3_make(plane.rot[2], plane.rot[5], plane.rot[8]);
  g5_plane_contact contact;
  g5_plane_sphere(normal, plane.pos, sphere.pos, sphere.size.x, &contact.dist, &contact.pos);
  contact.normal = normal;
  float frame[9];
  make_frame(contact.normal, frame);
  g5_write_contact(params, naconmax, 0, contact.dist, contact.pos, frame, out);
}

// 上游 `plane_capsule_wrapper`（pinned L600-685）：两接触 id_=0/1，共享 (n,b,c) 帧。
__device__ __forceinline__ void g5_plane_capsule_wrapper(const d_geom& plane, const d_geom& capsule,
                                                         const d_contact_params& params,
                                                         int naconmax, d_contact_out* out) {
  const v3 normal = v3_make(plane.rot[2], plane.rot[5], plane.rot[8]);
  const v3 axis = v3_make(capsule.rot[2], capsule.rot[5], capsule.rot[8]);
  g5_plane_contact contacts[2];
  float plane_frame[9];
  g5_plane_capsule(normal, plane.pos, capsule.pos, axis, capsule.size.x, capsule.size.y, contacts,
                   plane_frame);
  for (int i = 0; i < 2; ++i) {
    g5_write_contact(params, naconmax, i, contacts[i].dist, contacts[i].pos, plane_frame, out);
  }
}

// 上游 `plane_ellipsoid_wrapper`（pinned L686-760）：单接触 id_=0，frame = make_frame(normal)。
__device__ __forceinline__ void g5_plane_ellipsoid_wrapper(const d_geom& plane,
                                                           const d_geom& ellipsoid,
                                                           const d_contact_params& params,
                                                           int naconmax, d_contact_out* out) {
  const v3 normal = v3_make(plane.rot[2], plane.rot[5], plane.rot[8]);
  g5_plane_contact contacts[1];
  g5_plane_ellipsoid(normal, plane.pos, ellipsoid.pos, ellipsoid.rot, ellipsoid.size, contacts);
  float frame[9];
  make_frame(normal, frame);
  g5_write_contact(params, naconmax, 0, contacts[0].dist, contacts[0].pos, frame, out);
}

// 上游 `plane_cylinder_wrapper`（pinned L1000-1086）：4 条候选 id_=0..3，frame = make_frame(normal)；
// 无效候选的 MJ_MAXVAL 由 write_contact detected 门跳过。
__device__ __forceinline__ void g5_plane_cylinder_wrapper(const d_geom& plane,
                                                          const d_geom& cylinder,
                                                          const d_contact_params& params,
                                                          int naconmax, d_contact_out* out) {
  const v3 normal = v3_make(plane.rot[2], plane.rot[5], plane.rot[8]);
  const v3 axis = v3_make(cylinder.rot[2], cylinder.rot[5], cylinder.rot[8]);
  g5_plane_contact contacts[4];
  g5_plane_cylinder(normal, plane.pos, cylinder.pos, axis, cylinder.size.x, cylinder.size.y,
                    contacts);
  float frame[9];
  make_frame(normal, frame);
  for (int i = 0; i < 4; ++i) {
    g5_write_contact(params, naconmax, i, contacts[i].dist, contacts[i].pos, frame, out);
  }
}

// 上游 `plane_box_wrapper`（pinned L761-837）：8 角点候选 id_=0..7，frame = make_frame(normal)。
__device__ __forceinline__ void g5_plane_box_wrapper(const d_geom& plane, const d_geom& box,
                                                     const d_contact_params& params,
                                                     int naconmax, d_contact_out* out) {
  const v3 normal = v3_make(plane.rot[2], plane.rot[5], plane.rot[8]);
  g5_plane_contact contacts[8];
  const int count = g5_plane_box(normal, plane.pos, box.pos, box.rot, box.size, contacts);
  float frame[9];
  make_frame(normal, frame);
  for (int i = 0; i < count; ++i) {
    g5_write_contact(params, naconmax, i, contacts[i].dist, contacts[i].pos, frame, out);
  }
}

// 上游 `plane_convex_wrapper`（pinned L838-914）：mesh 走 `plane_convex` 核心，≤4 接触 id_=0..3，
// frame = make_frame(plane normal)。
__device__ __forceinline__ void g5_plane_convex_wrapper(const d_geom& plane, const d_geom& convex,
                                                        const g5_plane_mesh_view& mesh,
                                                        const d_contact_params& params, int naconmax,
                                                        d_contact_out* out) {
  const v3 normal = v3_make(plane.rot[2], plane.rot[5], plane.rot[8]);
  g5_plane_contact contacts[4];
  const int count = g5_plane_convex(normal, plane.pos, convex, mesh.vert, mesh.graph, contacts);
  float frame[9];
  make_frame(normal, frame);
  for (int i = 0; i < count; ++i) {
    g5_write_contact(params, naconmax, i, contacts[i].dist, contacts[i].pos, frame, out);
  }
}

