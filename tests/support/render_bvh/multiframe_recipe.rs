//! R06 多帧实体 A：recipe 参数 28 项口径补强（7 → 28；CPU-only，零 GPU）。
//!
//! 背景：backlog 卡 `2026-11-05-render-r06-flex-multiframe-recipe-params-backlog`——本实体的 recipe
//! 比对原先只覆盖 7 项口径，而单帧验收（`tests/render_flex_validate.rs`）为 28 项。本模块把比对补到
//! 单帧同口径：**纯测试断言，不改渲染/数值路径、不改 capture JSON、不改判据/参考/spec/相机/recipe**。
//!
//! **28 项口径** = capture `parameters` 的 26 个 recorded 键 + `nworld`（capture 未记录，多帧固定
//! 单 world）+ `cam_res`（由 model/camera 隐式推导；单帧验收亦为隐式比对，本模块不新增字面断言）。
//! 防漏守卫：断言内「已比对键集合」必须等于 [`RECIPE_PARAM_KEYS`]；
//! `capture_parameter_keys_match_28_item_scope` 再守卫 capture 侧键集合不漂移。
//!
//! 抽出为子模块的原因：`tests/support/render_bvh/multiframe.rs`（496 行）与
//! `tests/render_flex_multiframe.rs`（499 行）都贴近单文件 500 行棘轮，28 项断言与 recipe CPU 用例按功能
//! 抽出到本文件；测试仍由同一 test target 执行（由 `tests/render_flex_multiframe.rs` 以 `#[path]` 声明）。
//! 零 GPU、零窗口；不重冻 guards、不占窗。

use mjwarp_rs::render::CameraActive;
use mjwarp_rs::render::context::{ContextParams, RenderFlagSpec};
use serde_json::Value;

/// capture `parameters` 的全部 26 个 recorded 键（字典序；= 单帧验收 28 项口径中的 recorded 部分）。
pub const RECIPE_PARAM_KEYS: [&str; 26] = [
    "background_color",
    "cam_active",
    "enable_backface_culling",
    "enable_emission",
    "enable_per_light_ambient",
    "enable_specular",
    "enable_vertex_normals",
    "enabled_geom_groups",
    "flex_render_smooth",
    "render_depth",
    "render_rgb",
    "render_seg",
    "render_skybox",
    "samples_per_pixel",
    "shadow_light_fraction",
    "splat_adr",
    "splat_group_id",
    "splat_position",
    "splat_rgba",
    "splat_rotation",
    "splat_scale",
    "use_ambient_lighting",
    "use_fast_math",
    "use_precomputed_rays",
    "use_shadows",
    "use_textures",
];

/// `context_params()` 与 `capture.json.parameters` 逐参数比对（28 项口径；`cam_res` 隐式）。
///
/// `nworld` 不在 capture 里（多帧固定单 world），单独断言；其余 26 个 recorded 键逐项比对。
/// 函数末尾断言「已比对键集合 == [`RECIPE_PARAM_KEYS`]」，漏比/多比都会失败（防再次回退到少项）。
#[rustfmt::skip]
pub fn assert_params_match_capture(params: &ContextParams<'_>, recipe: &Value) {
    let mut compared: Vec<&'static str> = Vec::new();
    {
        let boolean = |name: &'static str| -> bool {
            recipe[name].as_bool().unwrap_or_else(|| panic!("{name} 非布尔"))
        };
        // nworld：capture.parameters 不记录（多帧固定单 world；resolved.nworld == 1 另有用例覆盖）。
        assert_eq!(params.nworld, 1, "nworld（capture 未记录；多帧固定单 world）");
        // 三个 render flag：Option<RenderFlagSpec> 与单帧验收同写法。
        for (name, spec) in [
            ("render_rgb", &params.render_rgb),
            ("render_depth", &params.render_depth),
            ("render_seg", &params.render_seg),
        ] {
            assert_eq!(*spec, Some(RenderFlagSpec::Bool(boolean(name))), "{name} 与 recipe 不一致");
            compared.push(name);
        }
        // 12 个布尔（含既有的 use_fast_math/use_shadows 与 10 个新增项）。
        for (name, value) in [
            ("use_textures", params.use_textures),
            ("use_fast_math", params.use_fast_math),
            ("use_shadows", params.use_shadows),
            ("use_ambient_lighting", params.use_ambient_lighting),
            ("flex_render_smooth", params.flex_render_smooth),
            ("use_precomputed_rays", params.use_precomputed_rays),
            ("render_skybox", params.render_skybox),
            ("enable_backface_culling", params.enable_backface_culling),
            ("enable_vertex_normals", params.enable_vertex_normals),
            ("enable_specular", params.enable_specular),
            ("enable_emission", params.enable_emission),
            ("enable_per_light_ambient", params.enable_per_light_ambient),
        ] {
            assert_eq!(value, boolean(name), "{name} 与 recipe 不一致");
            compared.push(name);
        }
    }
    // 非布尔项（与单帧验收同口径）。
    compared.push("enabled_geom_groups");
    assert_eq!(params.enabled_geom_groups, recipe["enabled_geom_groups"].as_array().expect("groups").iter().map(|v| v.as_i64().expect("i32") as i32).collect::<Vec<_>>().as_slice(), "enabled_geom_groups");
    compared.push("cam_active");
    assert!(recipe["cam_active"].is_null() && matches!(params.cam_active, CameraActive::All), "cam_active");
    compared.push("background_color");
    assert_eq!(params.background_color.to_vec(), recipe["background_color"].as_array().expect("bg").iter().map(|v| v.as_f64().expect("f64")).collect::<Vec<_>>(), "background_color");
    compared.push("shadow_light_fraction");
    assert_eq!(params.shadow_light_fraction, recipe["shadow_light_fraction"].as_f64().expect("f64"), "shadow_light_fraction");
    compared.push("samples_per_pixel");
    assert_eq!(params.samples_per_pixel as u64, recipe["samples_per_pixel"].as_u64().expect("u32"), "samples_per_pixel");
    // splat 六项：recipe 必须为 null，且 params.splat 必须全空。
    for name in ["splat_position", "splat_rotation", "splat_scale", "splat_rgba", "splat_adr", "splat_group_id"] {
        compared.push(name);
        assert!(recipe[name].is_null(), "{name} recipe 必须为空");
    }
    assert!(!params.splat.any_present(), "splat 输入必须为空");
    // 防漏守卫：已比对键集合 == RECIPE_PARAM_KEYS（26 个 recorded 键，漏比/多比即失败）。
    let mut actual = compared;
    actual.sort_unstable();
    let mut expected = RECIPE_PARAM_KEYS;
    expected.sort_unstable();
    assert_eq!(actual.as_slice(), expected.as_slice(), "recipe 比对键集合必须等于 26 个 recorded 键（防漏）");
}

/// 守卫 capture 侧键集合：`capture.json.parameters` 的键集必须等于 [`RECIPE_PARAM_KEYS`]（防夹具漂移）。
#[rustfmt::skip]
pub fn assert_capture_key_set(parameters: &Value) {
    let object = parameters.as_object().expect("capture.parameters 必须是对象");
    let mut actual: Vec<&str> = object.keys().map(String::as_str).collect();
    actual.sort_unstable();
    let mut expected = RECIPE_PARAM_KEYS;
    expected.sort_unstable();
    assert_eq!(actual, expected.as_slice(), "capture.parameters 键集合必须等于 26 个 recorded 键（28 项 = 26 recorded + nworld + cam_res 隐式）");
}

// ----------------------------------------------------------------------------------
// CPU 自证（零 GPU）：正例 / 反例（非空转证明）/ 键集守卫。

/// 正例：`context_params()` 对 capture 26 recorded + nworld 全过（28 项口径；`cam_res` 隐式）。
#[test]
fn recipe_params_match_capture_28_item_cpu_self_check() {
    let params = crate::multiframe::context_params();
    assert_params_match_capture(&params, &crate::multiframe::capture()["parameters"]);
}

/// 反例（**非空转证明**）：**既有键**与**新增键**各翻转一次，断言必须 panic；未改动时必须通过。
#[test]
fn recipe_param_mismatch_is_detected() {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let params = crate::multiframe::context_params();
    let base = crate::multiframe::capture()["parameters"].clone();
    // 反例 1（**既有键**，7 项口径时代就已比对）：改动 samples_per_pixel。
    let mut bumped = base.clone();
    bumped["samples_per_pixel"] = Value::from(base["samples_per_pixel"].as_u64().expect("u32") + 1);
    let caught = catch_unwind(AssertUnwindSafe(|| {
        assert_params_match_capture(&params, &bumped)
    }));
    assert!(
        caught.is_err(),
        "既有键 samples_per_pixel 改变后断言必须 panic（证明既有比对非空转）"
    );
    // 反例 2（**本片新增的 21 项断言之一**）：翻转 use_textures，证明新增断言非空转。
    let mut flipped = base.clone();
    flipped["use_textures"] = Value::Bool(!base["use_textures"].as_bool().expect("bool"));
    let caught = catch_unwind(AssertUnwindSafe(|| {
        assert_params_match_capture(&params, &flipped)
    }));
    assert!(
        caught.is_err(),
        "新增键 use_textures 翻转后断言必须 panic（证明新增 21 项断言非空转）"
    );
    // 正例对照：未改动时必须通过。
    assert_params_match_capture(&params, &base);
}

/// 键集守卫：capture 侧 26 键 == `RECIPE_PARAM_KEYS`；28 项 = 26 recorded + nworld + cam_res(隐式)。
#[test]
fn capture_parameter_keys_match_28_item_scope() {
    assert_capture_key_set(&crate::multiframe::capture()["parameters"]);
    assert_eq!(
        RECIPE_PARAM_KEYS.len() + 2,
        28,
        "28 项口径 = 26 recorded + nworld + cam_res"
    );
}

/// 既有 CPU 自证（原在 `tests/render_flex_multiframe.rs`，按单文件 500 行棘轮随 recipe 用例一并移入）：
/// 28 项比对 + context 解析（nworld / 4 个 geom 全开）。
#[test]
fn context_params_match_capture_recipe() {
    let params = crate::multiframe::context_params();
    assert_params_match_capture(&params, &crate::multiframe::capture()["parameters"]);
    let resolved = crate::resolve_context(&crate::render_fields(), &params).expect("context 解析");
    assert_eq!(resolved.nworld, 1);
    assert_eq!(resolved.enabled_geom_ids.len(), 4, "4 个 geom 全开");
}
