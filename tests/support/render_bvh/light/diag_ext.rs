//! P10：扩展像素 dump 的 JSON 编码（仅 opt-in 诊断轮；`MJWARP_DIAG_PIXEL_DUMP` 未设时
//! 调用方不会进入本模块，默认路径零输出）。
//!
//! 记录布局与编码见 `src/render/kernel/diag.rs` 与 `cuda/render_megakernel.cu` 顶部的 `DIAG_*` 常量
//! （**没有**单独的 `render_diag.cu`）：f32 一律以
//! `u32` 位型（`to_bits`）输出，保证离线逐位重放；`face`/`mesh_id` 解码回 i32。

use mjwarp_rs::render::kernel::DiagRecord;
use mjwarp_rs::render::kernel::MegakernelRenderer;
use serde_json::{Value, json};

use std::collections::BTreeMap;

use crate::light::{PIXELS, WORLDS};
use crate::light_diag::{DumpInput, pixel_coords, segment_attribution, unpack};

fn bits(value: f32) -> u32 {
    value.to_bits()
}

fn bits2(values: [f32; 2]) -> Value {
    json!([bits(values[0]), bits(values[1])])
}

fn bits3(values: [f32; 3]) -> Value {
    json!([bits(values[0]), bits(values[1]), bits(values[2])])
}

fn light_json(light: &mjwarp_rs::render::kernel::DiagLight) -> Value {
    json!({
        "weight_bits": bits(light.weight),
        "visible_bits": bits(light.visible),
        "ndotl_bits": bits(light.ndotl),
        "ndoth_bits": bits(light.ndoth),
        "half_bits": bits3(light.half),
        "pow_bits": bits(light.spec_pow),
    })
}

fn record_json(ray: usize, record: &DiagRecord) -> Value {
    json!({
        "ray": ray,
        "stride": mjwarp_rs::render::kernel::diag::STRIDE,
        "encoding": "f32 values are u32 bit patterns (to_bits); face/mesh_id are decoded i32",
        "color_bits": bits3(record.color),
        "base_color_bits": bits3(record.base_color),
        "tex_uv_bits": bits2(record.tex_uv),
        "tex_color_bits": bits3(record.tex_color),
        "bary_bits": bits2([record.bary_u, record.bary_v]),
        "t_bits": bits(record.t),
        "face": record.face,
        "mesh_id": record.mesh_id,
        "hit_point_bits": bits3(record.hit_point),
        "normal_bits": bits3(record.normal),
        "ray_dir_bits": bits3(record.ray_dir),
        "view_dir_bits": bits3(record.view_dir),
        "view_dir_derivation": "view_dir = -ray_dir_world (exact sign-bit flip on host; kernel stores ray_dir_world)",
        "lights": record.lights.iter().map(light_json).collect::<Vec<_>>(),
        "light_slots": mjwarp_rs::render::kernel::diag::LIGHT_SLOTS,
    })
}

/// 对 `(world, cam, pixel)` 取该 ray 的诊断记录并编码；未启用/未命中/越界返回 `Null`。
///
/// ray 下标与内核一致：**per-world 单次发射**内的相机主序 `cam * PIXELS + pixel`（两相机同分辨率）；
/// 内核按 `rayid * STRIDE` 落盘，因此多 world 单次发射会互相覆盖（诊断采集要求 per-world 发射）。
pub fn pixel_diagnostics(
    renderer: Option<&MegakernelRenderer>,
    world: usize,
    cam: usize,
    pixel: usize,
) -> Value {
    let Some(renderer) = renderer else {
        return Value::Null;
    };
    let ray = cam.saturating_mul(PIXELS).saturating_add(pixel);
    match renderer.diag_record(world, ray) {
        Some(record) => record_json(ray, &record),
        None => Value::Null,
    }
}

/// near-miss：同字节像素的 0/255 饱和区比例 + 最接近失配的 5 个非失配像素的分 geom 归属。
pub(crate) fn near_miss(input: &DumpInput<'_>, model: Option<&Value>) -> Value {
    let mut by_frame = Vec::new();
    for world in 0..WORLDS {
        for cam in 0..input.product[world].len() {
            let frame = &input.product[world][cam];
            let reference = &input.reference.frames[world][cam];
            let mut equal = 0usize;
            let mut saturated_any = 0usize;
            let mut zero_counts = [0usize; 3];
            let mut full_counts = [0usize; 3];
            let mut mismatches = Vec::new();
            for pixel in 0..PIXELS {
                let product_bytes = unpack(frame.rgb[pixel]);
                if product_bytes == unpack(reference.rgb[pixel]) {
                    equal += 1;
                    let mut any = false;
                    for (channel, byte) in product_bytes.iter().enumerate() {
                        if *byte == 0 {
                            zero_counts[channel] += 1;
                        }
                        if *byte == 255 {
                            full_counts[channel] += 1;
                        }
                        if *byte == 0 || *byte == 255 {
                            any = true;
                        }
                    }
                    if any {
                        saturated_any += 1;
                    }
                } else {
                    mismatches.push(pixel);
                }
            }
            let ratio = if equal == 0 {
                0.0
            } else {
                saturated_any as f64 / equal as f64
            };
            let mut nearest = Vec::new();
            let mut histogram: BTreeMap<String, usize> = BTreeMap::new();
            if !mismatches.is_empty() {
                let mut candidates: Vec<(u64, usize, usize)> = Vec::new();
                for pixel in 0..PIXELS {
                    if mismatches.contains(&pixel) {
                        continue;
                    }
                    let (x, y) = pixel_coords(pixel);
                    let mut best = u64::MAX;
                    for target in &mismatches {
                        let (tx, ty) = pixel_coords(*target);
                        let (dx, dy) = (x as i64 - tx as i64, y as i64 - ty as i64);
                        let distance = (dx * dx + dy * dy) as u64;
                        best = best.min(distance);
                    }
                    candidates.push((best, pixel, distance_owner(&mismatches, pixel)));
                }
                candidates.sort_unstable();
                for (distance_sq, pixel, target) in candidates.into_iter().take(5) {
                    let (x, y) = pixel_coords(pixel);
                    let attribution = segment_attribution(model, frame.seg[pixel]);
                    let geom = frame.seg[pixel][0];
                    *histogram.entry(geom.to_string()).or_insert(0) += 1;
                    nearest.push(json!({
                        "pixel": pixel, "x": x, "y": y,
                        "distance_px": (distance_sq as f64).sqrt(),
                        "nearest_mismatch_pixel": target,
                        "seg": frame.seg[pixel],
                        "attribution": attribution,
                    }));
                }
            }
            by_frame.push(json!({
                "world": world,
                "cam": cam,
                "pixels": PIXELS,
                "equal_byte_pixels": equal,
                "mismatched_pixels": mismatches.len(),
                "saturated_any_pixels": saturated_any,
                "saturated_any_ratio_of_equal": ratio,
                "zero_pixels_by_channel": zero_counts,
                "full_pixels_by_channel": full_counts,
                "nearest_to_mismatch": nearest,
                "nearest_geom_histogram": histogram,
            }));
        }
    }
    json!({ "by_frame": by_frame })
}

fn distance_owner(mismatches: &[usize], pixel: usize) -> usize {
    let (x, y) = pixel_coords(pixel);
    let mut best = (u64::MAX, mismatches[0]);
    for target in mismatches {
        let (tx, ty) = pixel_coords(*target);
        let (dx, dy) = (x as i64 - tx as i64, y as i64 - ty as i64);
        let distance = (dx * dx + dy * dy) as u64;
        if distance < best.0 {
            best = (distance, *target);
        }
    }
    best.1
}
