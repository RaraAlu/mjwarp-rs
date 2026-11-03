#![allow(dead_code)]
//! R09-S3b 阴影验收的共享夹具：scene-shadow 产品场景装配、参考比较、运行哈希。
//!
//! - 场景口径：`fixtures/render-bvh/scene-shadow/scene.xml` 派生自 scene-a，几何/相机/材质
//!   相同，唯一语义差异是三灯 `castshadow=1`（scene-a 的 lamp 为 0）→ 产品字段复用
//!   `s7b2::fields_from_model()`，只覆盖 `light_castshadow`。
//! - 判据口径：复用 S7b1/S7b2 的 diff 原语（RGB 每通道 ≤1 LSB、depth ≤4 ULP 位级优先、
//!   seg 全等）；本模块不新增/不放宽任何容差。
//! - 全部函数都是作者侧测试辅助；GPU 用例在 `tests/render_shadow_reference.rs` 且 `#[ignore]`。

use mjwarp_rs::model::render::{FieldData, RENDER_FIELDS, RenderFields};
use mjwarp_rs::render::CameraModel;
use mjwarp_rs::render::RayBuffer;
use mjwarp_rs::render::RenderContext;
use mjwarp_rs::render::kernel::MegakernelRenderer;
use mjwarp_rs::render::kernel::cast::CameraPose;
use mjwarp_rs::render::kernel::output::{DepthDiff, MAX_DEPTH_ULP, SegDiff, diff_seg};
use mjwarp_rs::runtime::TransferSession;
use sha2::{Digest, Sha256};

use super::s7b1::outline::{Arm, diff_depth_with_outline};
use super::s7b1::{camera_data_poses, read_depth, read_seg};
use super::s7b2::{
    RgbStats, create_context, default_params, diff_rgb, precomputed_rays, read_rgb,
    recipe_depth_seg, render,
};
use super::shadow::ShadowReference;

/// S3a 捕获时间（两组同一 `MJWARP_CAPTURE_AT`；仅作窗口证据追溯用）。
pub const CAPTURE_AT: &str = "2026-10-28T19:09:01+08:00";

/// 产品侧 scene-shadow 字段：scene-a 几何/相机/材质 + 三灯 `castshadow=1`。
pub fn shadow_fields() -> RenderFields {
    let base = super::s7b2::fields_from_model();
    let data = RENDER_FIELDS
        .iter()
        .enumerate()
        .map(|(index, spec)| match spec.name {
            "light_castshadow" => FieldData::Flag(vec![1, 1, 1]),
            _ => base.get(index).expect("字段按 RENDER_FIELDS 顺序").clone(),
        })
        .collect();
    RenderFields::new(data)
}

/// 捕获配方对应的 context 参数（spp=1、预计算射线、三输出；`use_shadows` 按组切换）。
pub fn shadow_params(use_shadows: bool) -> mjwarp_rs::render::ContextParams<'static> {
    mjwarp_rs::render::ContextParams {
        use_shadows,
        ..default_params()
    }
}

/// 一帧产品输出（与 `OutputLayout` 的 adr/pixels 对齐）。
#[derive(Clone, Debug)]
pub struct Rendered {
    pub rgb: Vec<u32>,
    pub depth: Vec<f32>,
    pub seg: Vec<[i32; 2]>,
}

pub struct ShadowScene {
    pub context: RenderContext,
    pub model: CameraModel,
    pub poses: Vec<CameraPose>,
    pub rays: RayBuffer,
}

pub fn build_scene(
    session: &TransferSession,
    fields: &RenderFields,
    use_shadows: bool,
) -> ShadowScene {
    let context = create_context(session, fields, &shadow_params(use_shadows));
    let model = CameraModel::from_fields(fields).expect("相机模型");
    let recipe = recipe_depth_seg();
    let poses = camera_data_poses(&recipe, &model, context.resolved());
    let rays = precomputed_rays(session, fields);
    ShadowScene {
        context,
        model,
        poses,
        rays,
    }
}

pub fn fire(
    session: &TransferSession,
    renderer: &MegakernelRenderer,
    fields: &RenderFields,
    scene: &ShadowScene,
) -> Rendered {
    render(
        session,
        renderer,
        &scene.context,
        fields,
        &scene.model,
        &scene.poses,
        &scene.rays,
    );
    Rendered {
        rgb: read_rgb(&scene.context),
        depth: read_depth(&scene.context),
        seg: read_seg(&scene.context),
    }
}

/// 对一组参考逐相机比较（复用 S7b1/S7b2 的 diff 口径）。
pub fn compare_group(
    rendered: &Rendered,
    scene: &ShadowScene,
    reference: &ShadowReference,
) -> (Vec<RgbStats>, Vec<DepthDiff>, Vec<SegDiff>) {
    let layout = &scene.context.resolved().output;
    let mut rgb_stats = Vec::with_capacity(reference.rgb.len());
    for camera in &reference.rgb {
        let start = layout.rgb_adr[camera.index] as usize;
        let actual = &rendered.rgb[start..start + camera.pixels];
        rgb_stats.push(diff_rgb(actual, camera));
    }
    let mut depth_stats = Vec::with_capacity(reference.depth_seg.len());
    let mut seg_stats = Vec::with_capacity(reference.depth_seg.len());
    for camera in &reference.depth_seg {
        let depth_start = layout.depth_adr[camera.index] as usize;
        let depth_actual = &rendered.depth[depth_start..depth_start + camera.pixels];
        // criterion v2.2：cam1 球面轮廓比较走已签清单豁免（env 未启用时 helper 返回原始 diff，行为不变）。
        depth_stats.push(diff_depth_with_outline(
            depth_actual,
            camera,
            Some(Arm::Recipe),
        ));
        let seg_start = layout.seg_adr[camera.index] as usize;
        let seg_actual = &rendered.seg[seg_start..seg_start + camera.pixels];
        seg_stats.push(diff_seg(seg_actual, camera).expect("seg diff"));
    }
    (rgb_stats, depth_stats, seg_stats)
}

/// 判据断言：RGB 每通道 ≤1 LSB、depth 位级优先（`mismatched==0`）且 ≤4 ULP 兜底、seg 全等。
pub fn assert_group_criteria(
    label: &str,
    reference: &ShadowReference,
    rgb_stats: &[RgbStats],
    depth_stats: &[DepthDiff],
    seg_stats: &[SegDiff],
) {
    for (camera, stats) in reference.rgb.iter().zip(rgb_stats) {
        assert_eq!(
            stats.beyond_one_lsb_pixels,
            0,
            "{label} cam{} RGB 超 1 LSB：{} / {} 像素，max_byte_diff={} first={:?}",
            camera.index,
            stats.beyond_one_lsb_pixels,
            stats.compared,
            stats.max_byte_diff,
            stats.first
        );
    }
    for (camera, diff) in reference.depth_seg.iter().zip(depth_stats) {
        assert_eq!(
            diff.mismatched, 0,
            "{label} cam{} depth 必须与参考位级相同（first={:?}）",
            camera.index, diff.first
        );
        assert!(
            diff.max_ulp <= MAX_DEPTH_ULP,
            "{label} cam{} depth max_ulp={} 超 {}",
            camera.index,
            diff.max_ulp,
            MAX_DEPTH_ULP
        );
        assert_eq!(
            diff.beyond_ulp, 0,
            "{label} cam{} depth 超 4 ULP 像素 {}",
            camera.index, diff.beyond_ulp
        );
    }
    for (camera, diff) in reference.depth_seg.iter().zip(seg_stats) {
        assert_eq!(
            diff.mismatched, 0,
            "{label} cam{} seg 必须逐像素全等（first={:?}）",
            camera.index, diff.first
        );
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn hash_rgb(values: &[u32]) -> String {
    let mut buffer = Vec::with_capacity(values.len() * 4);
    for value in values {
        buffer.extend_from_slice(&value.to_le_bytes());
    }
    sha256_hex(&buffer)
}

pub fn hash_depth(values: &[f32]) -> String {
    let mut buffer = Vec::with_capacity(values.len() * 4);
    for value in values {
        buffer.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    sha256_hex(&buffer)
}

pub fn hash_seg(values: &[[i32; 2]]) -> String {
    let mut buffer = Vec::with_capacity(values.len() * 8);
    for pair in values {
        buffer.extend_from_slice(&pair[0].to_le_bytes());
        buffer.extend_from_slice(&pair[1].to_le_bytes());
    }
    sha256_hex(&buffer)
}

/// 产品层 RGB 差异像素数（任意通道 packed 不同）。
pub fn product_rgb_difference(left: &Rendered, right: &Rendered) -> usize {
    left.rgb
        .iter()
        .zip(&right.rgb)
        .filter(|(a, b)| a != b)
        .count()
}

/// 产品层 depth 差异像素数（f32 位型不同）。
pub fn product_depth_difference(left: &Rendered, right: &Rendered) -> usize {
    left.depth
        .iter()
        .zip(&right.depth)
        .filter(|(a, b)| a.to_bits() != b.to_bits())
        .count()
}

/// 产品层 seg 差异像素数。
pub fn product_seg_difference(left: &Rendered, right: &Rendered) -> usize {
    left.seg
        .iter()
        .zip(&right.seg)
        .filter(|(a, b)| a != b)
        .count()
}
