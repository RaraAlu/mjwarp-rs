// G10@P5 B1 plane 族 primitive 设备实现（拆分：common）
// 语义来源 pinned collision_primitive.py rev 71da24d9；设备保持 GPU fma.rn 融合语义；0 新 ABI 字段。
#include "common.cu"

// G10@P5 B1：plane 族 primitive 设备实现（pinned collision_primitive.py rev 71da24d9；0 新 ABI 字段）。
//
// 语义来源（pinned，只读）：mujoco_warp 3.15 `collision_primitive.py`
//   - `plane_convex(plane_normal, plane_pos, convex)`：L52-278（共享核心）
//   - `plane_sphere_wrapper`：L281-355
//   - `plane_box_wrapper`：L761-837
//   - `plane_convex_wrapper`（mesh 走此 wrapper）：L838-914
// revision 71da24d956378a87a703b6e1442b13aec0c4ac29；设备侧保持 pinned GPU fma.rn 融合语义。
// 接触写入/结构字段复用既有 `primitive_wrapper.cu` 的 `g5_write_contact`/`make_frame`，
// 本文件不新增接触写入器、不改 ABI（`CollisionArgs`/`CollisionContactOut` 零新增字段）。
//


// 上游 `types.MJ_MAXVAL`（未填接触位置用）。
#define G5_PLANE_MAXVAL 1.0e10f
// 上游 `plane_convex` 的 `_HUGE_VAL`。
#define G5_PLANE_HUGE_VAL 1.0e6f
// `plane_convex` 的阈值（pinned L94 `max_support - 1e-3`）。
#define G5_PLANE_CONVEX_THRESHOLD 1.0e-3f

// 单条 plane 族接触候选（对应上游 wrapper 内 `(dist, pos, normal)`）。
struct g5_plane_contact {
  float dist;
  v3 pos;
  v3 normal;
};


// (n,b,c) 行优先 3x3 帧（plane_capsule 使用；与上游 wp.mat33(n,b,c) 行序一致）。
//   contact 2 = plane_sphere(n, plane_pos, capsule_pos-segment, radius)。
// ---------------------------------------------------------------------------------------------

// 把 (n, b, c) 行优先 3x3 帧写成设备侧 float[9]（与上游 `wp.mat33(n,b,c)` 行序一致）。
__device__ __forceinline__ void g5_plane_frame(v3 n, v3 b, v3 c, float frame[9]) {
  frame[0] = n.x; frame[1] = n.y; frame[2] = n.z;
  frame[3] = b.x; frame[4] = b.y; frame[5] = b.z;
  frame[6] = c.x; frame[7] = c.y; frame[8] = c.z;
}


