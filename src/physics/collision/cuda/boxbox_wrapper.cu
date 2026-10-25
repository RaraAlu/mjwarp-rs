#pragma once
// G10@P5 G5：上游 `box_box_wrapper`（collision_primitive.py L1250-1352）设备版。
// Adapted from frozen MJWarp collision_primitive.py, revision 71da24d. Apache-2.0.
//
// 调用方保证 contact.cu（`g5_write_contact` / `make_frame`）已先包含；本文件只做
// 「取几何字段 → g5_box_box → 8 槽逐 i make_frame + write_contact」。未填充槽的
// dist=MJ_MAXVAL 由 `g5_write_contact` 的 detected 门过滤，与上游计数语义一致。
#include "boxbox.cu"

__device__ __forceinline__ void g5_box_box_wrapper(
    const d_geom& box1, const d_geom& box2, const d_contact_params& params,
    int naconmax, d_contact_out* out) {
    const g5_box_box_result result = g5_box_box(
        box1.pos, box1.rot, box1.size, box2.pos, box2.rot, box2.size, params.margin);
    for (int i = 0; i < 8; ++i) {
        float frame[9];
        make_frame(result.normal[i], frame);
        g5_write_contact(params, naconmax, i, result.dist[i], result.pos[i], frame, out);
    }
}
