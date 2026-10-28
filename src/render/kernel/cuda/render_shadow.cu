// R09-S2 阴影射线可见度（渲染/BVH R09；方案 A：复用 first-hit `cast_ray`）。
//
// 上游锚点（冻结 MJWarp 3.15 `render.py`，`71da24d`）：
// - L919-961：仅当 `use_shadows and lightcastshadow` 才发射阴影射线；
//   `shadow_origin = hitpoint + normal * 1.0e-4`（自相交规避）；
//   非方向光 `max_t = dist_to_light - 1.0e-3`，方向光 `max_t = 1.0e8`；
//   命中（`geom_id != -1`）时 `visible = shadow_light_fraction`，否则 1.0。
// - L1457-1459：headlight 调用传 `lightcastshadow=False`（headlight 不投影）。
//
// 实现路径：复用 first-hit `cast_ray`（非 any-hit）；遮挡判定目标与上游 `max_t` 内任意命中
// 一致，但实现路径不同；本片不作任何数值结论，数值验收在 S3b 窗。
//
// 依赖拼接顺序：`render_common.cu`（`SceneView`/`MegakernelBlob`/`CastHit`/`cast_ray`）在前。

__device__ __forceinline__ float shadow_visibility(
    const SceneView& scene, const MegakernelBlob& blob, const int* geom_type, const int* geom_matid,
    const float* geom_rgba, const float* mat_rgba, int worldid, bool cull_backfaces, V3 normal,
    V3 hitpoint, V3 to_light, int lighttype, float dist_to_light, bool use_shadows,
    bool cast_shadow, float shadow_light_fraction) {
    if (!use_shadows || !cast_shadow) {
        return 1.0f;
    }
    // 上游 L922：沿法线微移，避免阴影射线自相交。
    V3 shadow_origin = add(hitpoint, scale(normal, 1.0e-4f));
    // 上游 L924-926：非方向光距离上限为到光距离；方向光按 1e8 近似无穷。
    float max_t = (lighttype == 1) ? 1.0e8f : (dist_to_light - 1.0e-3f);
    CastHit hit = cast_ray(scene, blob, geom_type, geom_matid, geom_rgba, mat_rgba, worldid,
                           shadow_origin, to_light, max_t, cull_backfaces, true);
    return (hit.geom_id != -1) ? shadow_light_fraction : 1.0f;
}
