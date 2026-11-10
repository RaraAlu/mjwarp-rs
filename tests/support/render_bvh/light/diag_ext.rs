//! P10：扩展像素 dump 的 JSON 编码（仅 opt-in 诊断轮；`MJWARP_DIAG_PIXEL_DUMP` 未设时
//! 调用方不会进入本模块，默认路径零输出）。
//!
//! 记录布局与编码见 `src/render/kernel/diag.rs` 与 `cuda/render_diag.cu`：f32 一律以
//! `u32` 位型（`to_bits`）输出，保证离线逐位重放；`face`/`mesh_id` 解码回 i32。

use mjwarp_rs::render::kernel::DiagRecord;
use mjwarp_rs::render::kernel::MegakernelRenderer;
use serde_json::{Value, json};

use crate::light::PIXELS;

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
/// ray 下标与内核一致：每次 world 发射内为相机主序 `cam * PIXELS + pixel`（两相机同分辨率）。
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
