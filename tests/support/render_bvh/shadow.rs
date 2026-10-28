#![allow(dead_code)]
//! R09-S3b 阴影参考读取：把 R09-S3a authoring 捕获的 npz 参考转成可比较结构。
//!
//! - 参考为 authoring-only 产物（pinned MJWarp 3.15.0 + warp 1.15.0，**fast-math 采集配方**），
//!   本模块**只读**（`include_bytes!` + sha256 冻结断言），不重新采集、不修改任何夹具。
//! - npz 由 `scripts/capture_render_shadow.py::save_npz_deterministic` 写出：
//!   `zipfile.ZIP_STORED`（无压缩）容器 + 标准 npy v1.0 数组成员
//!   （`rgb_camN.npy` / `depth_camN.npy` / `seg_camN.npy`）。仓库不为测试引入 npz 依赖，
//!   故内置最小解析器（只支持 stored + 小端定长类型 + C 顺序；超出即 panic）。
//! - 口径：
//!   - RGB 参考是 `get_rgb` 解包后的 float32（= `byte/255` 的 f32），按 `round(v*255)`
//!     还原 packed u32（`A<<24|R<<16|G<<8|B`，与 `s7b2::unpack_rgb`/`packed_byte` 同格式），
//!     复用 `s7b2::diff_rgb` 的「每通道 ≤1 LSB（byte 差 ≤1）」判据；
//!   - depth 参考是 `rc.depth_data` 的 raw f32（0 = 无命中），按 f32 位型比较（≤4 ULP）；
//!   - seg 参考是 `(object_id, object_type)` int32 对，逐像素全等。

use sha2::{Digest, Sha256};

use mjwarp_rs::render::kernel::output::RefCamera;

use super::s7b2::RgbCamera;

/// shadow-on 参考 sha256（S3a 窗口产物冻结值：rgb / depth / seg）。
pub const SHADOW_ON_SHA256: [&str; 3] = [
    "65596761f77a099a6c47ed885143ba3a8b50c0ca128606ae52b373142533ebc7",
    "0fa5582170a2b2a117e46868930fa2b4270dd993b0f5d86dc603981e0a2fd7d3",
    "2153fa728d634c032cf38847761803c4faa806aa554e108a2e0bd7b8ac8e6a97",
];

/// shadow-off 参考 sha256（S3a 窗口产物冻结值：rgb / depth / seg）。
///
/// 注：shadow-off 的 depth/seg 与 shadow-on、以及 S0b `scene-a/reference` 的 depth/seg
/// 逐字节相同（sha 相同），这是 S3a 记录的管线交叉校验信号之一。
pub const SHADOW_OFF_SHA256: [&str; 3] = [
    "9379f5d3b39e2d5d0ce8ef21a28b365e3608d20f6a992c39dff2d5df3ef90c0c",
    "0fa5582170a2b2a117e46868930fa2b4270dd993b0f5d86dc603981e0a2fd7d3",
    "2153fa728d634c032cf38847761803c4faa806aa554e108a2e0bd7b8ac8e6a97",
];

const SHADOW_ON_RGB: &[u8] =
    include_bytes!("../../../fixtures/render-bvh/scene-shadow/capture/shadow-on/rgb.npz");
const SHADOW_ON_DEPTH: &[u8] =
    include_bytes!("../../../fixtures/render-bvh/scene-shadow/capture/shadow-on/depth.npz");
const SHADOW_ON_SEG: &[u8] =
    include_bytes!("../../../fixtures/render-bvh/scene-shadow/capture/shadow-on/seg.npz");
const SHADOW_OFF_RGB: &[u8] =
    include_bytes!("../../../fixtures/render-bvh/scene-shadow/capture/shadow-off/rgb.npz");
const SHADOW_OFF_DEPTH: &[u8] =
    include_bytes!("../../../fixtures/render-bvh/scene-shadow/capture/shadow-off/depth.npz");
const SHADOW_OFF_SEG: &[u8] =
    include_bytes!("../../../fixtures/render-bvh/scene-shadow/capture/shadow-off/seg.npz");

/// 一组阴影参考（shadow-on 或 shadow-off）：两相机 RGB packed + depth/seg。
#[derive(Clone, Debug)]
pub struct ShadowReference {
    pub group: &'static str,
    pub rgb: Vec<RgbCamera>,
    pub depth_seg: Vec<RefCamera>,
    pub total_pixels: usize,
}

impl ShadowReference {
    pub fn camera_rgb(&self, index: usize) -> &RgbCamera {
        &self.rgb[index]
    }

    pub fn camera_depth_seg(&self, index: usize) -> &RefCamera {
        &self.depth_seg[index]
    }
}

/// 读取 shadow-on 参考（`fixtures/render-bvh/scene-shadow/capture/shadow-on/`）。
pub fn shadow_on_reference() -> ShadowReference {
    build(
        "shadow-on",
        [SHADOW_ON_RGB, SHADOW_ON_DEPTH, SHADOW_ON_SEG],
        SHADOW_ON_SHA256,
    )
}

/// 读取 shadow-off 参考（`fixtures/render-bvh/scene-shadow/capture/shadow-off/`）。
pub fn shadow_off_reference() -> ShadowReference {
    build(
        "shadow-off",
        [SHADOW_OFF_RGB, SHADOW_OFF_DEPTH, SHADOW_OFF_SEG],
        SHADOW_OFF_SHA256,
    )
}

fn build(
    group: &'static str,
    parts: [&'static [u8]; 3],
    expected_sha: [&str; 3],
) -> ShadowReference {
    let names = ["rgb.npz", "depth.npz", "seg.npz"];
    for (index, (bytes, expected)) in parts.iter().zip(expected_sha).enumerate() {
        assert_eq!(
            sha256_hex(bytes),
            expected,
            "{group}/{} 参考 sha256 不符（参考被动过或夹具替换）",
            names[index]
        );
    }
    let rgb_members = read_npz(parts[0]);
    let depth_members = read_npz(parts[1]);
    let seg_members = read_npz(parts[2]);
    let cameras = rgb_members.len();
    assert!(cameras > 0, "{group} rgb.npz 成员为空");
    assert_eq!(depth_members.len(), cameras, "{group} depth 相机数");
    assert_eq!(seg_members.len(), cameras, "{group} seg 相机数");

    let mut rgb = Vec::with_capacity(cameras);
    let mut depth_seg = Vec::with_capacity(cameras);
    let mut total_pixels = 0usize;
    let mut depth_adr = 0i32;
    let mut seg_adr = 0i32;
    for camera in 0..cameras {
        let ((h, w), values) = f32_member_rgb(&rgb_members, &format!("rgb_cam{camera}.npy"));
        let pixels = w * h;
        let mut packed = Vec::with_capacity(pixels);
        for index in 0..pixels {
            let base = 3 * index;
            let r = quantize_channel(values[base], "rgb.r", group, camera);
            let g = quantize_channel(values[base + 1], "rgb.g", group, camera);
            let b = quantize_channel(values[base + 2], "rgb.b", group, camera);
            packed
                .push((0xFFu32 << 24) | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b));
        }
        rgb.push(RgbCamera {
            index: camera,
            width: w,
            height: h,
            pixels,
            packed,
        });

        let ((depth_h, depth_w), depth_values) =
            f32_member(&depth_members, &format!("depth_cam{camera}.npy"));
        assert_eq!((depth_h, depth_w), (h, w), "{group} depth 形状");
        let depth_bits: Vec<u32> = depth_values.iter().map(|value| value.to_bits()).collect();
        let depth_zero_pixels = depth_bits.iter().filter(|bits| **bits == 0).count();
        let depth_max_hex = depth_values
            .iter()
            .max_by(|left, right| left.partial_cmp(right).expect("depth 参考不得含 NaN"))
            .map(|value| format!("0x{:08x}", value.to_bits()))
            .unwrap_or_else(|| "0x00000000".to_string());

        let ((seg_h, seg_w, seg_components), seg_values) =
            i32_member(&seg_members, &format!("seg_cam{camera}.npy"));
        assert_eq!(
            (seg_h, seg_w, seg_components),
            (h, w, 2),
            "{group} seg 形状"
        );
        let (seg_chunks, seg_remainder) = seg_values.as_chunks::<2>();
        assert!(seg_remainder.is_empty(), "{group} seg 数据长度");
        let seg_pairs: Vec<[i32; 2]> = seg_chunks.to_vec();
        assert_eq!(seg_pairs.len(), pixels, "{group} seg 像素数");
        let mut seg_unique: Vec<[i32; 2]> = seg_pairs.clone();
        seg_unique.sort_unstable();
        seg_unique.dedup();

        depth_seg.push(RefCamera {
            index: camera,
            width: w,
            height: h,
            pixels,
            depth_adr,
            seg_adr,
            depth_zero_pixels,
            depth_max_hex,
            depth_bits,
            seg_pairs,
            seg_unique,
        });
        depth_adr += pixels as i32;
        seg_adr += pixels as i32;
        total_pixels += pixels;
    }
    ShadowReference {
        group,
        rgb,
        depth_seg,
        total_pixels,
    }
}

/// 参考层 RGB 差异像素数（两组 packed 逐像素任意通道不同）。
pub fn rgb_difference_pixels(left: &ShadowReference, right: &ShadowReference) -> usize {
    assert_eq!(left.rgb.len(), right.rgb.len(), "RGB 相机数");
    let mut count = 0usize;
    for (a, b) in left.rgb.iter().zip(&right.rgb) {
        assert_eq!(a.packed.len(), b.packed.len(), "RGB 像素数");
        count += a
            .packed
            .iter()
            .zip(&b.packed)
            .filter(|(left_pixel, right_pixel)| left_pixel != right_pixel)
            .count();
    }
    count
}

/// 参考层 depth 差异像素数（f32 位型不等）。
pub fn depth_difference_pixels(left: &ShadowReference, right: &ShadowReference) -> usize {
    assert_eq!(left.depth_seg.len(), right.depth_seg.len(), "depth 相机数");
    let mut count = 0usize;
    for (a, b) in left.depth_seg.iter().zip(&right.depth_seg) {
        assert_eq!(a.depth_bits.len(), b.depth_bits.len(), "depth 像素数");
        count += a
            .depth_bits
            .iter()
            .zip(&b.depth_bits)
            .filter(|(left_bits, right_bits)| left_bits != right_bits)
            .count();
    }
    count
}

/// 参考层 seg 差异像素数（任一分量不等）。
pub fn seg_difference_pixels(left: &ShadowReference, right: &ShadowReference) -> usize {
    assert_eq!(left.depth_seg.len(), right.depth_seg.len(), "seg 相机数");
    let mut count = 0usize;
    for (a, b) in left.depth_seg.iter().zip(&right.depth_seg) {
        assert_eq!(a.seg_pairs.len(), b.seg_pairs.len(), "seg 像素数");
        count += a
            .seg_pairs
            .iter()
            .zip(&b.seg_pairs)
            .filter(|(left_pair, right_pair)| left_pair != right_pair)
            .count();
    }
    count
}

/// 参考 float32 通道值 → packed byte（`get_rgb` 输出是 `byte/255` 的 f32）。
fn quantize_channel(value: f32, what: &str, group: &str, camera: usize) -> u8 {
    let scaled = value * 255.0;
    let rounded = scaled.round();
    assert!(
        (scaled - rounded).abs() <= 0.01,
        "{group}/cam{camera} {what} 参考值 {value} 不在 byte/255 网格上（scaled={scaled}）"
    );
    assert!(
        (0.0..=255.0).contains(&rounded),
        "{group}/cam{camera} {what} 量化越界：{value}"
    );
    rounded as u8
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

// ---- 最小 npz 解析（ZIP_STORED + npy v1.0；本仓库夹具专用） ----

fn read_npz(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut members = Vec::new();
    let mut offset = 0usize;
    while offset + 30 <= bytes.len() {
        let signature = u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("zip sig"));
        if signature != 0x0403_4b50 {
            // 中央目录 / 结束标记：成员遍历结束。
            break;
        }
        let flags = u16::from_le_bytes(bytes[offset + 6..offset + 8].try_into().expect("flags"));
        let method = u16::from_le_bytes(bytes[offset + 8..offset + 10].try_into().expect("method"));
        let compressed = u32::from_le_bytes(
            bytes[offset + 18..offset + 22]
                .try_into()
                .expect("compressed size"),
        ) as usize;
        let name_len = u16::from_le_bytes(
            bytes[offset + 26..offset + 28]
                .try_into()
                .expect("name len"),
        ) as usize;
        let extra_len = u16::from_le_bytes(
            bytes[offset + 28..offset + 30]
                .try_into()
                .expect("extra len"),
        ) as usize;
        let name = String::from_utf8_lossy(&bytes[offset + 30..offset + 30 + name_len]).to_string();
        assert_eq!(
            flags & 0x8,
            0,
            "npz 成员 {name} 使用 data descriptor（未预期）"
        );
        assert_eq!(method, 0, "npz 成员 {name} 非 ZIP_STORED（未预期）");
        let data_start = offset + 30 + name_len + extra_len;
        let data_end = data_start + compressed;
        assert!(data_end <= bytes.len(), "npz 成员 {name} 越界");
        members.push((name, bytes[data_start..data_end].to_vec()));
        offset = data_end;
    }
    assert!(!members.is_empty(), "npz 无成员");
    members
}

/// 解析 npy：返回 `(descr, shape, raw data)`（只支持小端 f4/i4 + C 顺序）。
fn parse_npy(bytes: &[u8]) -> (String, Vec<usize>, Vec<u8>) {
    assert_eq!(&bytes[..6], b"\x93NUMPY", "npy magic");
    let major = bytes[6];
    let (header_len, header_start) = match major {
        1 => (u16::from_le_bytes([bytes[8], bytes[9]]) as usize, 10usize),
        2 | 3 => (
            u32::from_le_bytes(bytes[8..12].try_into().expect("npy header len")) as usize,
            12usize,
        ),
        other => panic!("npy 版本 {other} 未支持"),
    };
    let header_end = header_start + header_len;
    let header = std::str::from_utf8(&bytes[header_start..header_end]).expect("npy header utf8");
    assert!(
        header.contains("fortran_order': False"),
        "npy 必须 C 顺序：{header}"
    );
    let descr = extract_descr(header);
    let shape = extract_shape(header);
    let data = bytes[header_end..].to_vec();
    (descr, shape, data)
}

fn extract_descr(header: &str) -> String {
    let marker = "descr':";
    let start = header.find(marker).expect("npy descr") + marker.len();
    let rest = &header[start..];
    let quote = rest.find('\'').expect("npy descr 引号") + 1;
    let tail = &rest[quote..];
    let end = tail.find('\'').expect("npy descr 结束引号");
    tail[..end].to_string()
}

fn extract_shape(header: &str) -> Vec<usize> {
    let marker = "shape':";
    let start = header.find(marker).expect("npy shape") + marker.len();
    let rest = &header[start..];
    let open = rest.find('(').expect("npy shape 左括号");
    let close = rest.find(')').expect("npy shape 右括号");
    let inner = &rest[open + 1..close];
    inner
        .split(',')
        .filter_map(|item| {
            let trimmed = item.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.parse::<usize>().expect("npy shape 维度"))
            }
        })
        .collect()
}

fn f32_member(members: &[(String, Vec<u8>)], name: &str) -> ((usize, usize), Vec<f32>) {
    let (descr, shape, data) = member(members, name);
    assert_eq!(descr, "<f4", "{name} descr");
    assert_eq!(shape.len(), 2, "{name} shape 维度");
    let (chunks, remainder) = data.as_chunks::<4>();
    assert!(remainder.is_empty(), "{name} 数据长度");
    let values: Vec<f32> = chunks
        .iter()
        .map(|chunk| f32::from_le_bytes(*chunk))
        .collect();
    assert_eq!(values.len(), shape[0] * shape[1], "{name} 数据长度");
    ((shape[0], shape[1]), values)
}

fn i32_member(members: &[(String, Vec<u8>)], name: &str) -> ((usize, usize, usize), Vec<i32>) {
    let (descr, shape, data) = member(members, name);
    assert_eq!(descr, "<i4", "{name} descr");
    assert_eq!(shape.len(), 3, "{name} shape 维度");
    let (chunks, remainder) = data.as_chunks::<4>();
    assert!(remainder.is_empty(), "{name} 数据长度");
    let values: Vec<i32> = chunks
        .iter()
        .map(|chunk| i32::from_le_bytes(*chunk))
        .collect();
    assert_eq!(
        values.len(),
        shape[0] * shape[1] * shape[2],
        "{name} 数据长度"
    );
    ((shape[0], shape[1], shape[2]), values)
}

/// RGB 成员按 `[h, w, 3]` 读取（与 depth 的形状断言配合）。
fn f32_member_rgb(members: &[(String, Vec<u8>)], name: &str) -> ((usize, usize), Vec<f32>) {
    let (descr, shape, data) = member(members, name);
    assert_eq!(descr, "<f4", "{name} descr");
    assert_eq!(shape.len(), 3, "{name} shape 维度");
    assert_eq!(shape[2], 3, "{name} 通道");
    let (chunks, remainder) = data.as_chunks::<4>();
    assert!(remainder.is_empty(), "{name} 数据长度");
    let values: Vec<f32> = chunks
        .iter()
        .map(|chunk| f32::from_le_bytes(*chunk))
        .collect();
    assert_eq!(
        values.len(),
        shape[0] * shape[1] * shape[2],
        "{name} 数据长度"
    );
    ((shape[0], shape[1]), values)
}

fn member(members: &[(String, Vec<u8>)], name: &str) -> (String, Vec<usize>, Vec<u8>) {
    let (_, bytes) = members
        .iter()
        .find(|(member_name, _)| member_name == name)
        .unwrap_or_else(|| panic!("npz 缺少成员 {name}"));
    parse_npy(bytes)
}
