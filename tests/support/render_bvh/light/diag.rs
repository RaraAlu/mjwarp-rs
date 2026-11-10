//! G-L2 判据 A 像素级只读诊断 dump（opt-in；默认关闭，不改任何断言/判据/阈值/参考）。
//!
//! 门控：`MJWARP_DIAG_PIXEL_DUMP=<输出 JSON 路径>`；未设置（或为空）时本模块所有函数立即返回，
//! 默认测试行为与之前完全一致（无打印、无文件、无额外 GPU 工作）。输出路径由调用方给出，
//! 产物落 `target/`，不入库。字段按 Chief 2026-11-11 修订：去掉无判别力的 `|ref_f32*255-round|`
//! 边界距离，改为前后字节 + packed u32 + 0/255 饱和标记、3×3 邻域字节矩阵、seg 的 geom/mat 归属
//! 与材质标量、每像素 ray/灯/相机位型；near-miss 改为「同字节像素的饱和区比例」+「最接近失配的
//! 5 个非失配像素的分 geom 归属」。
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use mjwarp_rs::render::RayBuffer;
use mjwarp_rs::render::context::ResolvedContext;
use mjwarp_rs::render::kernel::WorldLightPoses;
use mjwarp_rs::render::kernel::cast::CameraPose;
use serde_json::{Value, json};

use crate::light::{FrameStats, HEIGHT, MODEL, PIXELS, WIDTH, WORLDS};
use crate::{Frame, Reference};

pub const ENV: &str = "MJWARP_DIAG_PIXEL_DUMP";
pub const SCHEMA: u32 = 1;

/// dump 的全部输入（只读借用；`stats` 为已算好的判据 A 统计，dump 不重跑渲染）。
pub struct DumpInput<'a> {
    pub reference: &'a Reference,
    pub product: &'a [Vec<Frame>],
    pub stats: &'a [Vec<FrameStats>],
    pub rays: &'a RayBuffer,
    pub camera_poses: &'a [CameraPose],
    pub lights: &'a WorldLightPoses,
    pub resolved: &'a ResolvedContext,
}

/// 设置 `MJWARP_DIAG_PIXEL_DUMP=<路径>` 时写 dump 并返回路径；未设置返回 `None`（默认零副作用）。
pub fn dump_if_enabled(input: &DumpInput<'_>) -> Option<PathBuf> {
    let path = output_path()?;
    let document = build_document(input);
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        let _ = std::fs::create_dir_all(parent);
    }
    let bytes = match serde_json::to_vec_pretty(&document) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("diag pixel dump: serialise failed: {error}");
            return None;
        }
    };
    if let Err(error) = std::fs::write(&path, bytes) {
        eprintln!("diag pixel dump: write {} failed: {error}", path.display());
        return None;
    }
    let rgb = document["rgb_mismatches"].as_array().map_or(0, Vec::len);
    let depth = document["depth_mismatches"].as_array().map_or(0, Vec::len);
    let seg = document["seg_mismatches"].as_array().map_or(0, Vec::len);
    println!(
        "diag pixel dump: path={} rgb_mismatches={rgb} depth_mismatches={depth} seg_mismatches={seg}",
        path.display()
    );
    Some(path)
}

fn output_path() -> Option<PathBuf> {
    let raw = std::env::var(ENV).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

fn build_document(input: &DumpInput<'_>) -> Value {
    let model: Option<Value> = serde_json::from_str(MODEL).ok();
    let frames: Vec<Value> = (0..WORLDS)
        .flat_map(|world| (0..input.stats[world].len()).map(move |cam| (world, cam)))
        .map(|(world, cam)| {
            let stats = input.stats[world][cam];
            json!({
                "world": world,
                "cam": cam,
                "camera_pixels": PIXELS,
                "rgb_mismatched_pixels": stats.rgb_mismatched_pixels,
                "rgb_max_byte_diff": stats.rgb_max_byte_diff,
                "depth_mismatched_pixels": stats.depth_mismatched_pixels,
                "depth_max_ulp": stats.depth_max_ulp,
                "depth_beyond_ulp": stats.depth_beyond_ulp,
                "seg_mismatched_pixels": stats.seg_mismatched_pixels,
            })
        })
        .collect();
    let camera_poses: Vec<Value> = input
        .camera_poses
        .iter()
        .enumerate()
        .map(|(cam, pose)| {
            json!({
                "cam": cam,
                "xpos_bits": bits3(pose.xpos),
                "xmat_bits": bits9(pose.xmat),
            })
        })
        .collect();
    json!({
        "schema": SCHEMA,
        "kind": "g-l2-entity-pixel-diagnostic",
        "env": ENV,

        "generated_at_unix": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|elapsed| elapsed.as_secs()).unwrap_or(0),
        "model": "tests/support/render_bvh/light/model-315.json",
        "frames": frames,
        "camera_poses": camera_poses,
        "rgb_mismatches": rgb_mismatches(input, model.as_ref()),
        "depth_mismatches": depth_mismatches(input),
        "seg_mismatches": seg_mismatches(input),
        "near_miss": near_miss(input, model.as_ref()),

    })
}

#[rustfmt::skip]
fn bits3(values: [f32; 3]) -> [u32; 3] { values.map(f32::to_bits) }

#[rustfmt::skip]
fn bits9(values: [f32; 9]) -> [u32; 9] { values.map(f32::to_bits) }

#[rustfmt::skip]
fn unpack(packed: u32) -> [u8; 3] {
    [((packed >> 16) & 0xFF) as u8, ((packed >> 8) & 0xFF) as u8, (packed & 0xFF) as u8]
}

/// 每通道 0/255 饱和标记（分通道）。
#[rustfmt::skip]
fn saturation(bytes: [u8; 3]) -> Value {
    json!(bytes.iter().map(|byte| json!({"zero": *byte == 0, "full": *byte == 255, "saturated": *byte == 0 || *byte == 255})).collect::<Vec<_>>())
}

fn ulp(actual: f32, expected: f32) -> u32 {
    let map = |value: f32| -> i64 {
        let bits = value.to_bits() as i32;
        if bits < 0 {
            0x8000_0000i64 - i64::from(bits)
        } else {
            i64::from(bits)
        }
    };
    (map(actual) - map(expected)).unsigned_abs() as u32
}

#[rustfmt::skip]
fn pixel_index(x: usize, y: usize) -> usize { y * WIDTH + x }

#[rustfmt::skip]
fn pixel_coords(pixel: usize) -> (usize, usize) { (pixel % WIDTH, pixel / WIDTH) }

fn array_f32(model: &Value, name: &str) -> Vec<f32> {
    model["arrays"][name]["values"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .map(|value| value.as_f64().unwrap_or(0.0) as f32)
                .collect()
        })
        .unwrap_or_default()
}

fn array_i32(model: &Value, name: &str) -> Vec<i32> {
    model["arrays"][name]["values"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .map(|value| value.as_i64().unwrap_or(0) as i32)
                .collect()
        })
        .unwrap_or_default()
}

/// 材质标量/数组：每材质 stride = len / nmat（nmat 由 mat_rgba/4 决定）。
fn material_values(model: &Value, name: &str, mat: i32) -> Value {
    let rgba = array_f32(model, "mat_rgba");
    let nmat = rgba.len() / 4;
    let values = array_f32(model, name);
    let stride = values.len().checked_div(nmat).unwrap_or(0);
    if mat < 0 || stride == 0 || mat as usize >= nmat {
        return Value::Null;
    }
    let start = mat as usize * stride;
    json!(
        values[start..start + stride]
            .iter()
            .map(|value| *value as f64)
            .collect::<Vec<_>>()
    )
}

/// seg `(id,type)` 对应的 geom / mat 归属 + 材质标量（模型数组只读；缺模型则给 null）。
fn segment_attribution(model: Option<&Value>, seg: [i32; 2]) -> Value {
    let (geom_id, geom_type) = (seg[0], seg[1]);
    let Some(model) = model else {
        return json!({ "geom_id": geom_id, "geom_type": geom_type, "mat_id": Value::Null });
    };
    let geom_matid = array_i32(model, "geom_matid");
    let mat = if geom_id >= 0 && (geom_id as usize) < geom_matid.len() {
        geom_matid[geom_id as usize]
    } else {
        -1
    };
    let texid_row = material_values(model, "mat_texid", mat);
    json!({
        "geom_id": geom_id,
        "geom_type": geom_type,
        "mat_id": mat,
        "mat_texid_row": texid_row,
        "mat_shininess": material_values(model, "mat_shininess", mat),
        "mat_specular": material_values(model, "mat_specular", mat),
        "mat_emission": material_values(model, "mat_emission", mat),
        "mat_reflectance": material_values(model, "mat_reflectance", mat),
        "mat_metallic": material_values(model, "mat_metallic", mat),
        "mat_roughness": material_values(model, "mat_roughness", mat),
        "mat_rgba": material_values(model, "mat_rgba", mat),
    })
}

/// 3×3 邻域打包字节矩阵（行优先 y-1..y+1 × x-1..x+1；越界为 null）。
fn neighborhood_3x3(frame: &Frame, x: usize, y: usize) -> Value {
    let mut rows = Vec::with_capacity(3);
    for dy in -1i32..=1 {
        let mut row = Vec::with_capacity(3);
        for dx in -1i32..=1 {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || ny < 0 || nx >= WIDTH as i32 || ny >= HEIGHT as i32 {
                row.push(Value::Null);
            } else {
                let pixel = pixel_index(nx as usize, ny as usize);
                row.push(json!(unpack(frame.rgb[pixel]).map(|byte| byte as u64)));
            }
        }
        rows.push(Value::Array(row));
    }
    json!(rows)
}

fn camera_ray_bits(input: &DumpInput<'_>, cam: usize, pixel: usize) -> Value {
    let (x, y) = pixel_coords(pixel);
    let index = input.resolved.plan.ray_index(0, cam, x as u32, y as u32);
    let directions = input.rays.directions();
    let offsets = input.rays.offsets();
    match directions.get(index) {
        Some(direction) => json!({
            "sample": 0,
            "ray_index": index,
            "direction_bits": bits3(*direction),
            "offset_bits": offsets.get(index).map(|offset| bits3(*offset)),
        }),
        None => {
            json!({ "sample": 0, "ray_index": index, "direction_bits": Value::Null, "offset_bits": Value::Null })
        }
    }
}

/// 该 world 三灯位型：参考 `state.npz` 与产品 `WorldLightPoses` 两侧。
fn world_light_bits(input: &DumpInput<'_>, world: usize) -> Value {
    let mut lights = Vec::with_capacity(3);
    for light in 0..3 {
        let base = (world * 3 + light) * 3;
        let reference = json!({
            "xpos_bits": bits3([
                input.reference.state.light_xpos[base],
                input.reference.state.light_xpos[base + 1],
                input.reference.state.light_xpos[base + 2],
            ]),
            "xdir_bits": bits3([
                input.reference.state.light_xdir[base],
                input.reference.state.light_xdir[base + 1],
                input.reference.state.light_xdir[base + 2],
            ]),
        });
        let product = match input.lights.get(world, light) {
            Ok(pose) => json!({ "xpos_bits": bits3(pose.xpos), "xdir_bits": bits3(pose.xdir) }),
            Err(_) => Value::Null,
        };
        lights.push(json!({ "light": light, "reference": reference, "product": product }));
    }
    json!(lights)
}

fn rgb_mismatches(input: &DumpInput<'_>, model: Option<&Value>) -> Vec<Value> {
    let mut out = Vec::new();
    for (world, cam_row) in input.product.iter().enumerate() {
        for (cam, frame) in cam_row.iter().enumerate() {
            let reference = &input.reference.frames[world][cam];
            for pixel in 0..PIXELS {
                let (left, right) = (frame.rgb[pixel], reference.rgb[pixel]);
                if left == right {
                    continue;
                }
                let (x, y) = pixel_coords(pixel);
                let product_bytes = unpack(left);
                let reference_bytes = unpack(right);
                let pose = input.camera_poses.get(cam);
                out.push(json!({
                    "world": world,
                    "cam": cam,
                    "pixel": pixel,
                    "x": x,
                    "y": y,
                    "product": {
                        "packed_u32": left,
                        "bytes": product_bytes,
                        "saturation": saturation(product_bytes),
                    },
                    "reference": {
                        "packed_u32": right,
                        "bytes": reference_bytes,
                        "saturation": saturation(reference_bytes),
                    },
                    "neighborhood_3x3": {
                        "product": neighborhood_3x3(frame, x, y),
                        "reference": neighborhood_3x3(reference, x, y),
                    },
                    "seg": {
                        "product": frame.seg[pixel],
                        "reference": reference.seg[pixel],
                        "attribution": segment_attribution(model, frame.seg[pixel]),
                    },
                    "depth": {
                        "product_bits": frame.depth[pixel].to_bits(),
                        "reference_bits": reference.depth[pixel].to_bits(),
                        "ulp": ulp(frame.depth[pixel], reference.depth[pixel]),
                    },
                    "ray": camera_ray_bits(input, cam, pixel),
                    "camera": pose.map(|pose| json!({ "cam": cam, "xpos_bits": bits3(pose.xpos), "xmat_bits": bits9(pose.xmat) })),
                    "lights": world_light_bits(input, world),
                }));
            }
        }
    }
    out
}

fn depth_mismatches(input: &DumpInput<'_>) -> Vec<Value> {
    let mut out = Vec::new();
    for (world, cam_row) in input.product.iter().enumerate() {
        for (cam, frame) in cam_row.iter().enumerate() {
            let reference = &input.reference.frames[world][cam];
            for pixel in 0..PIXELS {
                let (left, right) = (frame.depth[pixel], reference.depth[pixel]);
                if left.to_bits() == right.to_bits() {
                    continue;
                }
                let (x, y) = pixel_coords(pixel);
                out.push(json!({
                    "world": world, "cam": cam, "pixel": pixel, "x": x, "y": y,
                    "product_bits": left.to_bits(), "reference_bits": right.to_bits(),
                    "ulp": ulp(left, right), "seg": frame.seg[pixel],
                }));
            }
        }
    }
    out
}

fn seg_mismatches(input: &DumpInput<'_>) -> Vec<Value> {
    let mut out = Vec::new();
    for (world, cam_row) in input.product.iter().enumerate() {
        for (cam, frame) in cam_row.iter().enumerate() {
            let reference = &input.reference.frames[world][cam];
            for pixel in 0..PIXELS {
                if frame.seg[pixel] == reference.seg[pixel] {
                    continue;
                }
                let (x, y) = pixel_coords(pixel);
                out.push(json!({
                    "world": world, "cam": cam, "pixel": pixel, "x": x, "y": y,
                    "product_seg": frame.seg[pixel], "reference_seg": reference.seg[pixel],
                }));
            }
        }
    }
    out
}

/// near-miss：同字节像素的 0/255 饱和区比例 + 最接近失配的 5 个非失配像素的分 geom 归属。
fn near_miss(input: &DumpInput<'_>, model: Option<&Value>) -> Value {
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
