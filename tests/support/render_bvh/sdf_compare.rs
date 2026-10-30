#![allow(dead_code)]
//! U4 SDF 参考比较与判据断言（作者侧测试辅助；GPU 用例在
//! `tests/render_sdf_reference.rs` 且 `#[ignore]`）。
//!
//! 判据（沿用 R09-S3b / R05-S3b，不新增、不放宽）：
//! - RGB 每通道 **≤ 1 LSB**（`beyond_1_lsb=0`、`max_byte_diff<=1`）；
//! - depth **位级优先**（`mismatched==0`）且 `max_ulp<=4`、`beyond_ulp==0`；
//! - seg 逐像素**完全相等**（`mismatched=0`）。
//!
//! 两条**一致性**臂（没有上游 shadow-off 参考，不宣称数值参考）：
//! - shadow 开/关：depth/seg 位级相等；rgb `off >= on`（逆序超 1 LSB 的通道=0）且差异像素>0；
//! - DepthSeg 跨路径：`render_depth_seg` 与 megakernel 的 depth/seg 位级相等。

use mjwarp_rs::render::kernel::output::{
    DepthDiff, MAX_DEPTH_ULP, RefCamera, SegDiff, diff_depth, diff_seg,
};
use sha2::{Digest, Sha256};

use super::s7b2::{RgbStats, diff_rgb};
use super::sdf::SdfReference;
use super::sdf_scene::{Rendered, SdfScene};

/// 对一组参考逐相机比较（复用 S7b1/S7b2 的 diff 口径）。
pub fn compare_group(
    rendered: &Rendered,
    scene: &SdfScene,
    reference: &SdfReference,
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
        depth_stats.push(diff_depth(depth_actual, camera).expect("depth diff"));
        let seg_start = layout.seg_adr[camera.index] as usize;
        let seg_actual = &rendered.seg[seg_start..seg_start + camera.pixels];
        seg_stats.push(diff_seg(seg_actual, camera).expect("seg diff"));
    }
    (rgb_stats, depth_stats, seg_stats)
}

/// 判据断言（同 R09-S3b `assert_group_criteria`，不新增/不放宽）。
pub fn assert_group_criteria(
    label: &str,
    reference: &SdfReference,
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

/// 参考相机深度统计摘要（打印用）。
pub fn camera_summary(camera: &RefCamera, depth: &DepthDiff, seg: &SegDiff) -> String {
    format!(
        "cam{} pixels={} depth_mismatch={} max_ulp={} beyond_ulp={} seg_mismatch={} \
         depth_zero_pixels={} depth_max={}",
        camera.index,
        camera.pixels,
        depth.mismatched,
        depth.max_ulp,
        depth.beyond_ulp,
        seg.mismatched,
        camera.depth_zero_pixels,
        camera.depth_max_hex
    )
}

/// shadow 开/关一致性（**只做一致性，没有上游 shadow-off 参考**）：
/// depth/seg 两臂位级相等；rgb 每通道 `off >= on`（逆序超 1 LSB 的通道 = 0）；
/// 且必须有像素发生变化（证明阴影确实生效）。返回 (changed_pixels, reversed_channels)。
pub fn assert_shadow_consistency(
    label: &str,
    on: &Rendered,
    off: &Rendered,
    scene_on: &SdfScene,
    camera_index: usize,
) -> (usize, usize) {
    let layout = &scene_on.context.resolved().output;
    let pixels = grid_pixels(scene_on, camera_index);
    let rgb_start = layout.rgb_adr[camera_index] as usize;
    let rgb_on = &on.rgb[rgb_start..rgb_start + pixels];
    let rgb_off = &off.rgb[rgb_start..rgb_start + pixels];
    let mut changed = 0usize;
    let mut reversed = 0usize;
    for (on_pixel, off_pixel) in rgb_on.iter().zip(rgb_off) {
        let mut pixel_changed = false;
        for channel in 0..3 {
            let on_byte = packed_byte(*on_pixel, channel);
            let off_byte = packed_byte(*off_pixel, channel);
            if off_byte != on_byte {
                pixel_changed = true;
            }
            if off_byte + 1 < on_byte {
                reversed += 1;
            }
        }
        if pixel_changed {
            changed += 1;
        }
    }
    let depth_start = layout.depth_adr[camera_index] as usize;
    let depth_on = &on.depth[depth_start..depth_start + pixels];
    let depth_off = &off.depth[depth_start..depth_start + pixels];
    assert_eq!(
        depth_bits(depth_on),
        depth_bits(depth_off),
        "{label} cam{camera_index} shadow 开/关 depth 必须位级相等（阴影不改变命中几何）"
    );
    let seg_start = layout.seg_adr[camera_index] as usize;
    let seg_on = &on.seg[seg_start..seg_start + pixels];
    let seg_off = &off.seg[seg_start..seg_start + pixels];
    assert_eq!(
        seg_on, seg_off,
        "{label} cam{camera_index} shadow 开/关 seg 必须逐像素相等"
    );
    assert_eq!(
        reversed, 0,
        "{label} cam{camera_index} shadow-off 出现 {reversed} 个通道比 shadow-on 暗 >1 LSB（应单调不暗）"
    );
    assert!(
        changed > 0,
        "{label} cam{camera_index} shadow 开/关 rgb 完全相同（阴影未生效，无法作为一致性证据）"
    );
    (changed, reversed)
}

/// DepthSeg 跨路径一致性：同一 context/输入下 megakernel 与 `render_depth_seg` 的
/// depth/seg **位级相等**（cam 全量）。
pub fn assert_depth_seg_cross_path(
    label: &str,
    megakernel: &Rendered,
    depth_seg: &Rendered,
    scene: &SdfScene,
) {
    let layout = &scene.context.resolved().output;
    for camera in 0..scene.model.ncam() {
        let pixels = grid_pixels(scene, camera);
        let depth_start = layout.depth_adr[camera] as usize;
        assert_eq!(
            depth_bits(&megakernel.depth[depth_start..depth_start + pixels]),
            depth_bits(&depth_seg.depth[depth_start..depth_start + pixels]),
            "{label} cam{camera} DepthSeg 路径与 megakernel 的 depth 必须位级相等"
        );
        let seg_start = layout.seg_adr[camera] as usize;
        assert_eq!(
            &megakernel.seg[seg_start..seg_start + pixels],
            &depth_seg.seg[seg_start..seg_start + pixels],
            "{label} cam{camera} DepthSeg 路径与 megakernel 的 seg 必须逐像素相等"
        );
    }
}

fn depth_bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

/// 某相机像素数（由 context 布局的 adr 差推导；最后一台用总像素）。
fn grid_pixels(scene: &SdfScene, camera: usize) -> usize {
    let layout = &scene.context.resolved().output;
    let current = layout.rgb_adr.get(camera).copied().expect("rgb_adr") as usize;
    let next = layout
        .rgb_adr
        .get(camera + 1)
        .map(|adr| *adr as usize)
        .unwrap_or(layout.rgb_pixels);
    next - current
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

/// packed RGB 的通道字节（与 S7b2 同语义）。
pub fn packed_byte(packed: u32, channel: usize) -> u8 {
    match channel {
        0 => ((packed >> 16) & 0xFF) as u8,
        1 => ((packed >> 8) & 0xFF) as u8,
        _ => (packed & 0xFF) as u8,
    }
}

/// 逐像素 packed RGB 不同像素数（shadow 臂"差异像素 > 0"辅助）。
pub fn rgb_changed_pixels(a: &[u32], b: &[u32]) -> usize {
    a.iter()
        .zip(b)
        .filter(|(left, right)| left != right)
        .count()
}
