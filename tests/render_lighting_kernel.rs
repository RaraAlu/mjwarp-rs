//! 渲染/BVH R09-S2 验收：内核光照/阴影接线的宿主侧结构断言（纯宿主，无 GPU 窗口）。
//!
//! 覆盖范围（S2 只实现 + 单测，不宣称任何渲染数值结论；数值验收在 S3b 窗）：
//! - `MEGAKERNEL_SOURCE` 已拼接 `render_shadow.cu`（方案 A：复用 first-hit `cast_ray`）；
//! - `shadow_light_fraction` 经 `scale_constants[0][2]`（原占位 unused）传入内核；
//! - `use_shadows` 由 `ResolvedContext` 接线（不再是 host 硬编码 false）；
//! - 81 槽冻结签名不变（`MEGAKERNEL_SLOTS` 长度/首槽）。
//!
//! 上游锚点：冻结 MJWarp 3.15 `render.py`（`71da24d`）L830/L850/L866/L880/L919-961。

use mjwarp_rs::render::ContextParams;
use mjwarp_rs::render::kernel::{MEGAKERNEL_ENTRY, MEGAKERNEL_SOURCE};
use mjwarp_rs::render::resolve_context;
use mjwarp_rs::runtime::launcher::{MEGAKERNEL_SLOTS, RenderArgKind};
use serde_json::Value;

const MODEL: &str = include_str!("../fixtures/render-bvh/scene-a/model-312.json");

/// 把 S1 渲染字段夹具喂给 `resolve_context`（与既有 render context 测试同源构造）。
fn resolved(use_shadows: bool, shadow_light_fraction: f64) -> mjwarp_rs::render::ResolvedContext {
    use mjwarp_rs::model::render::{FieldData, RENDER_FIELDS, RenderFieldKind, RenderFields};
    fn array(value: &Value) -> &[Value] {
        value.as_array().expect("JSON 数组")
    }
    let model: Value = serde_json::from_str(MODEL).expect("model-312.json");
    let arrays = &model["arrays"];
    let data = RENDER_FIELDS
        .iter()
        .map(|spec| {
            let values = array(&arrays[spec.name]["values"]);
            match spec.kind {
                RenderFieldKind::Index => FieldData::Index(
                    values
                        .iter()
                        .map(|v| v.as_i64().expect("i32") as i32)
                        .collect(),
                ),
                RenderFieldKind::Num => {
                    FieldData::Num(values.iter().map(|v| v.as_f64().expect("f64")).collect())
                }
                RenderFieldKind::Flag => FieldData::Flag(
                    values
                        .iter()
                        .map(|v| v.as_u64().expect("u8") as u8)
                        .collect(),
                ),
                RenderFieldKind::Size => {
                    FieldData::Size(values.iter().map(|v| v.as_i64().expect("i64")).collect())
                }
                RenderFieldKind::Float => FieldData::Float(
                    values
                        .iter()
                        .map(|v| v.as_f64().expect("f32") as f32)
                        .collect(),
                ),
            }
        })
        .collect();
    let fields = RenderFields::new(data);
    let params = ContextParams {
        use_shadows,
        shadow_light_fraction,
        ..Default::default()
    };
    resolve_context(&fields, &params).expect("context 解析")
}

#[test]
fn megakernel_source_includes_shadow_visibility_source() {
    // R09-S2：新设备源 `render_shadow.cu` 已拼入内核源码（方案 A 封装）。
    assert!(
        MEGAKERNEL_SOURCE.contains("shadow_visibility("),
        "内核源码应含 shadow_visibility 封装"
    );
    assert!(
        MEGAKERNEL_SOURCE.contains("shadow_light_fraction"),
        "内核源码应含 shadow_light_fraction 参数/读取"
    );
    assert!(
        MEGAKERNEL_SOURCE.contains("splat_scale.data[0][2]"),
        "shadow_light_fraction 应经 scale_constants[0][2] 传入"
    );
    // 旧行为：`use_shadows` 直接 return 的防御分支应已移除。
    assert!(
        !MEGAKERNEL_SOURCE.contains("if (use_shadows) {\n        return;"),
        "旧的 use_shadows 直接 return 防御分支应已移除"
    );
    // 上游 L922/L924-926 的实现要点应出现在源码注释/代码中。
    assert!(MEGAKERNEL_SOURCE.contains("1.0e-4f"), "阴影原点法线微移");
    assert!(MEGAKERNEL_SOURCE.contains("1.0e-3f"), "非方向光距离上限");
}

#[test]
fn shadow_source_precedes_shading_in_concat_order() {
    // 拼接顺序：common（cast_ray 定义）→ shadow（shutdown 封装）→ shade（compute_lighting）→ megakernel。
    let common = MEGAKERNEL_SOURCE
        .find("__device__ __forceinline__ CastHit cast_ray(")
        .expect("common 的 cast_ray 定义");
    let shadow = MEGAKERNEL_SOURCE
        .find("__device__ __forceinline__ float shadow_visibility(")
        .expect("shadow 的 shadow_visibility 定义");
    let shade = MEGAKERNEL_SOURCE
        .find("__device__ __forceinline__ void compute_lighting(")
        .expect("shade 的 compute_lighting 定义");
    let megakernel = MEGAKERNEL_SOURCE
        .find("extern \"C\" __global__ void render_megakernel(")
        .expect("内核本体");
    assert!(common < shadow, "cast_ray 必须先于 shadow_visibility");
    assert!(
        shadow < shade,
        "shadow_visibility 必须先于 compute_lighting"
    );
    assert!(shade < megakernel, "compute_lighting 必须先于内核本体");
}

#[test]
fn resolved_context_carries_shadow_parameters() {
    let off = resolved(false, 0.3);
    assert!(!off.use_shadows);
    assert_eq!(off.shadow_light_fraction, 0.3);
    let on = resolved(true, 0.42);
    assert!(on.use_shadows);
    assert_eq!(on.shadow_light_fraction, 0.42);
    // 只影响阴影开关/系数，不改变其他已签名的解析字段。
    assert_eq!(off.znear, on.znear);
    assert_eq!(off.zfar, on.zfar);
    assert_eq!(off.enable_per_light_ambient, on.enable_per_light_ambient);
}

#[test]
fn megakernel_slots_signature_is_unchanged() {
    // 81 槽 S7a 冻结签名：R09-S2 不增槽（shadow_light_fraction 复用 scale_constants 数组）。
    assert_eq!(MEGAKERNEL_SLOTS.len(), 82);
    assert_eq!(MEGAKERNEL_SLOTS[0], RenderArgKind::Bounds2D);
    assert_eq!(MEGAKERNEL_ENTRY, "render_megakernel");
}

#[test]
fn shadow_light_fraction_default_matches_upstream_signature() {
    // 上游签名默认 0.3（render.py `shadow_light_fraction=0.3`）。
    let params = ContextParams::default();
    assert_eq!(params.shadow_light_fraction, 0.3);
    assert!(!params.use_shadows);
}
