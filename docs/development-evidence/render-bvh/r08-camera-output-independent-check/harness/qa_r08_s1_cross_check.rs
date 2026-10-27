//! QA 独立交叉核对（R08-S1）：产品逐相机视图 vs QA 自己的 python/numpy 期望值。
//!
//! 只读仓库夹具与 `target/qa-r08/`（QA 自产期望二进制）；本文件为临时复核工具，
//! 复核完成后删除，不进入任何提交。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use mjwarp_rs::render::camera::CameraPlan;
use mjwarp_rs::render::context::OutputLayout;
use mjwarp_rs::render::{CameraOutputViews, CameraViewError, ImageOutput, WholeOutputData};

use serde_json::Value;

const REPO: &str = env!("CARGO_MANIFEST_DIR");

fn root() -> PathBuf {
    PathBuf::from(REPO)
}

fn expected_dir() -> PathBuf {
    root().join("target/qa-r08/expected")
}

fn out_dir() -> PathBuf {
    let dir = root().join("target/qa-r08");
    fs::create_dir_all(&dir).expect("out dir");
    dir
}

fn load_json(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("read json")).expect("parse json")
}

fn u32s(value: &Value) -> Vec<u32> {
    value
        .as_array()
        .expect("array")
        .iter()
        .map(|v| v.as_u64().expect("u64") as u32)
        .collect()
}

fn i32s(value: &Value) -> Vec<i32> {
    value
        .as_array()
        .expect("array")
        .iter()
        .map(|v| v.as_i64().expect("i64") as i32)
        .collect()
}

fn load_f32(name: &str) -> Vec<f32> {
    fs::read(expected_dir().join(name))
        .expect("expected f32")
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn load_i32(name: &str) -> Vec<i32> {
    fs::read(expected_dir().join(name))
        .expect("expected i32")
        .chunks_exact(4)
        .map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn pixels(resolution: [u32; 2]) -> usize {
    resolution[0] as usize * resolution[1] as usize
}

fn plan_for(resolutions: &[[u32; 2]]) -> CameraPlan {
    let total: usize = resolutions.iter().copied().map(pixels).sum();
    CameraPlan {
        cameras: (0..resolutions.len()).collect(),
        resolutions: resolutions.to_vec(),
        render_rgb: vec![true; resolutions.len()],
        render_depth: vec![true; resolutions.len()],
        render_seg: vec![true; resolutions.len()],
        total_rays: total,
        rgb_pixels: total,
        samples_per_pixel: 1,
        samples: 1,
        znear: 0.05,
        zfar: 100.0,
    }
}

fn views<'a>(
    plan: &'a CameraPlan,
    layout: &'a OutputLayout,
    data: WholeOutputData<'a>,
) -> CameraOutputViews<'a> {
    CameraOutputViews::new(plan, layout, 1, data).expect("views")
}

/// 上游 `wp.clamp(x, 0, 1)` = fminf(fmaxf(x, 0), 1)（CUDA：NaN 操作数被忽略）。
fn qa_clamp_unit(value: f32) -> f32 {
    let low = if value.is_nan() { 0.0 } else { value.max(0.0) };
    if low.is_nan() {
        return 1.0;
    }
    low.min(1.0)
}

fn qa_clamp_bits(value: f32) -> u32 {
    let clamped = qa_clamp_unit(value);
    if clamped == 0.0 {
        return 0.0f32.to_bits();
    }
    clamped.to_bits()
}

struct Report {
    checks: usize,
    counts: BTreeMap<String, u64>,
    failures: Vec<String>,
}

impl Report {
    fn new() -> Self {
        Self {
            checks: 0,
            counts: BTreeMap::new(),
            failures: Vec::new(),
        }
    }

    fn expect(&mut self, condition: bool, label: &str) {
        self.checks += 1;
        if !condition {
            self.failures.push(label.to_string());
        }
    }

    fn expect_eq<T: PartialEq + std::fmt::Debug>(&mut self, actual: T, expected: T, label: &str) {
        self.checks += 1;
        if actual != expected {
            self.failures
                .push(format!("{label}: actual {actual:?} expected {expected:?}"));
        }
    }

    fn add(&mut self, key: &str, value: usize) {
        *self.counts.entry(key.to_string()).or_insert(0) += value as u64;
    }
}

fn references() -> (
    Vec<[u32; 2]>,
    Vec<Vec<u32>>,
    Vec<Vec<u32>>,
    Vec<Vec<[i32; 2]>>,
    Vec<u32>,
    Vec<f32>,
    Vec<[i32; 2]>,
) {
    let rgb_ref = load_json(&root().join("fixtures/render-bvh/scene-a/s7b/rgb_fast_math_false.json"));
    let depth_seg_ref =
        load_json(&root().join("fixtures/render-bvh/scene-a/s7b/depth_seg_fast_math_false.json"));
    let resolutions: Vec<[u32; 2]> = rgb_ref["cameras"]
        .as_array()
        .expect("cameras")
        .iter()
        .map(|camera| {
            [
                camera["width"].as_u64().expect("w") as u32,
                camera["height"].as_u64().expect("h") as u32,
            ]
        })
        .collect();
    let mut packed_per_camera = Vec::new();
    for camera in rgb_ref["cameras"].as_array().expect("cameras") {
        packed_per_camera.push(u32s(&camera["rgb_bits_packed"]));
    }
    let mut depth_per_camera = Vec::new();
    let mut seg_per_camera = Vec::new();
    for camera in depth_seg_ref["cameras"].as_array().expect("cameras") {
        depth_per_camera.push(u32s(&camera["depth_bits"]));
        seg_per_camera.push(
            i32s(&camera["seg_pairs"])
                .chunks_exact(2)
                .map(|pair| [pair[0], pair[1]])
                .collect::<Vec<_>>(),
        );
    }
    let whole_packed: Vec<u32> = packed_per_camera.iter().flatten().copied().collect();
    let whole_depth: Vec<f32> = depth_per_camera
        .iter()
        .flatten()
        .map(|bits| f32::from_bits(*bits))
        .collect();
    let whole_seg: Vec<[i32; 2]> = seg_per_camera.iter().flatten().copied().collect();
    (
        resolutions,
        packed_per_camera,
        depth_per_camera,
        seg_per_camera,
        whole_packed,
        whole_depth,
        whole_seg,
    )
}

#[test]
fn qa_cross_check_product_views_against_independent_expectations() {
    let mut report = Report::new();
    let py = load_json(&out_dir().join("python_expected.json"));
    let (resolutions, packed_per_camera, depth_per_camera, _, whole_packed, whole_depth, whole_seg) =
        references();
    report.expect_eq(resolutions.clone(), vec![[320, 240], [240, 180]], "fixture resolutions");
    let total: usize = resolutions.iter().copied().map(pixels).sum();
    report.expect_eq(total, 120_000, "fixture total pixels");
    for (index, packed) in packed_per_camera.iter().enumerate() {
        report.expect_eq(packed.len(), pixels(resolutions[index]), "packed len");
    }
    for (index, bits) in depth_per_camera.iter().enumerate() {
        report.expect_eq(bits.len(), pixels(resolutions[index]), "depth bits len");
    }
    report.expect_eq(whole_packed.len(), 120_000, "whole packed");
    report.expect_eq(whole_depth.len(), 120_000, "whole depth");
    report.expect_eq(whole_seg.len(), 120_000, "whole seg");

    let plan = plan_for(&resolutions);
    let layout = OutputLayout::resolve(&plan).expect("layout");
    report.expect_eq(layout.rgb_adr.clone(), vec![0, 76_800], "rgb adr");
    report.expect_eq(layout.depth_adr.clone(), vec![0, 76_800], "depth adr");
    report.expect_eq(layout.seg_adr.clone(), vec![0, 76_800], "seg adr");
    let data = WholeOutputData {
        rgb: Some(&whole_packed),
        depth: Some(&whole_depth),
        segmentation: Some(&whole_seg),
    };
    let output_views = views(&plan, &layout, data);
    report.expect_eq(output_views.camera_count(), 2, "camera count");
    report.expect_eq(output_views.camera_id(0).expect("id"), 0, "camera_id 0");
    report.expect_eq(output_views.camera_id(1).expect("id"), 1, "camera_id 1");

    // 1) packed RGB bit equal
    for (index, packed) in packed_per_camera.iter().enumerate() {
        let region = output_views.rgb_packed(index).expect("packed");
        report.expect(region == packed.as_slice(), "packed region bit equal");
        let equal = region.iter().zip(packed).filter(|(a, b)| a == b).count();
        report.add("packed_equal", equal);
    }
    // 2) unpacked RGB bit equal to the QA numpy expectations
    for index in 0..resolutions.len() {
        let expected = load_f32(&format!("rgb_c{index}.f32.bin"));
        report.expect_eq(expected.len(), pixels(resolutions[index]) * 3, "rgb expected len");
        let image = output_views.rgb(index).expect("rgb");
        let actual: Vec<u32> = image
            .pixels()
            .iter()
            .flat_map(|pixel| pixel.map(f32::to_bits))
            .collect();
        let expected_bits: Vec<u32> = expected.iter().map(|value| value.to_bits()).collect();
        let equal = actual.iter().zip(&expected_bits).filter(|(a, b)| a == b).count();
        report.add("unpacked_equal", equal);
        report.expect(equal == expected_bits.len(), "unpacked channels bit equal");
        report.expect(
            image.pixels().first().map(|pixel| pixel.map(f32::to_bits))
                == Some([
                    expected_bits.first().copied().unwrap_or(0),
                    expected_bits.get(1).copied().unwrap_or(0),
                    expected_bits.get(2).copied().unwrap_or(0),
                ]),
            "rgb first pixel r,g,b order",
        );
    }
    // 3) depth raw bit equal
    for (index, bits) in depth_per_camera.iter().enumerate() {
        let raw = output_views.depth_raw(index).expect("depth raw");
        let actual: Vec<u32> = raw.iter().map(|value| value.to_bits()).collect();
        let equal = actual.iter().zip(bits).filter(|(a, b)| a == b).count();
        report.add("depth_raw_equal", equal);
        report.expect(equal == bits.len(), "depth raw bit equal");
    }
    // 4) depth scaled bit equal (3 scales)
    for (tag, scale) in [("1_0", 1.0f32), ("0_1", 0.1), ("100_0", 100.0)] {
        for index in 0..resolutions.len() {
            let expected = load_f32(&format!("depth_s{tag}_c{index}.f32.bin"));
            let image = output_views.depth(index, scale).expect("depth scaled");
            let actual: Vec<u32> = image.pixels().iter().map(|value| value.to_bits()).collect();
            let expected_bits: Vec<u32> = expected.iter().map(|value| value.to_bits()).collect();
            let equal = actual.iter().zip(&expected_bits).filter(|(a, b)| a == b).count();
            report.add("depth_scaled_equal", equal);
            report.expect(equal == expected_bits.len(), "depth scaled bit equal");
        }
    }
    // 5) seg equal
    for index in 0..resolutions.len() {
        let expected = load_i32(&format!("seg_c{index}.i32x2.bin"));
        let expected_pairs: Vec<[i32; 2]> =
            expected.chunks_exact(2).map(|pair| [pair[0], pair[1]]).collect();
        let image = output_views.segmentation(index).expect("seg");
        let equal = image
            .pixels()
            .iter()
            .zip(&expected_pairs)
            .filter(|(a, b)| a == b)
            .count();
        report.add("seg_equal", equal);
        report.expect(equal == expected_pairs.len(), "seg pairs equal");
        report.expect(
            output_views.segmentation_raw(index).expect("seg raw") == expected_pairs.as_slice(),
            "seg raw equal",
        );
    }
    // 6) independent counts vs the QA python recompute
    for index in 0..2 {
        let seg = output_views.segmentation(index).expect("seg counts");
        let background = seg.pixels().iter().filter(|pair| **pair == [-1, -1]).count();
        let raw = output_views.depth_raw(index).expect("depth counts");
        let zero_depth = raw.iter().filter(|value| value.to_bits() == 0).count();
        let raw_max = raw.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let scaled = output_views.depth(index, 1.0).expect("depth 1.0");
        let clamp_high = scaled
            .pixels()
            .iter()
            .filter(|value| value.to_bits() == 1.0f32.to_bits())
            .count();
        report.expect_eq(
            background as u64,
            py["seg_background"][index.to_string()].as_u64().expect("py background"),
            "background count matches QA recompute",
        );
        report.expect_eq(
            zero_depth as u64,
            py["depth_zero_plus0"][index.to_string()].as_u64().expect("py zero"),
            "depth zero count matches QA recompute",
        );
        report.expect_eq(
            clamp_high as u64,
            py["clamp_high_at_scale_1"][index.to_string()].as_u64().expect("py clamp"),
            "clamp-to-1 count matches QA recompute",
        );
        report.expect_eq(
            raw_max as f64,
            py["depth_raw_max"][index.to_string()].as_f64().expect("py max"),
            "raw depth max matches QA recompute",
        );
    }
    report.expect_eq(
        mjwarp_rs::render::texture::INV_255.to_bits(),
        0x3B80_8081u32,
        "INV_255 bits",
    );
    report.expect_eq(
        ((1.0f64 / 255.0) as f32).to_bits(),
        0x3B80_8081u32,
        "float32(1/255) bits",
    );

    for failure in &report.failures {
        eprintln!("FAILURE: {failure}");
    }
    assert!(
        report.failures.is_empty(),
        "{} failures (first: {:?})",
        report.failures.len(),
        report.failures.first()
    );
    let summary = serde_json::json!({
        "checks": report.checks,
        "counts": report.counts,
        "failures": report.failures,
    });
    fs::write(
        out_dir().join("harness_cross_check.json"),
        serde_json::to_vec_pretty(&summary).expect("json"),
    )
    .expect("write summary");
    println!(
        "cross-check ok: checks={} counts={:?}",
        report.checks, report.counts
    );
}

#[test]
fn qa_error_surface_isolation_and_clamp_edges() {
    let (resolutions, _packed_per_camera, _depth_per_camera, _, whole_packed, whole_depth, whole_seg) =
        references();
    let plan = plan_for(&resolutions);
    let layout = OutputLayout::resolve(&plan).expect("layout");
    let data = WholeOutputData {
        rgb: Some(&whole_packed),
        depth: Some(&whole_depth),
        segmentation: Some(&whole_seg),
    };
    let output_views = views(&plan, &layout, data);
    let mut checks = 0usize;
    let mut failures: Vec<String> = Vec::new();
    macro_rules! expect {
        ($condition:expr, $label:expr) => {{
            checks += 1;
            if !$condition {
                failures.push($label.to_string());
            }
        }};
    }
    macro_rules! expect_eq {
        ($actual:expr, $expected:expr, $label:expr) => {{
            let actual = $actual;
            let expected = $expected;
            checks += 1;
            if actual != expected {
                failures.push(format!("{}: {:?} != {:?}", $label, actual, expected));
            }
        }};
    }

    // 1) camera_index out of range (incl. usize::MAX)
    for index in [2usize, 9, usize::MAX] {
        expect_eq!(
            output_views.rgb(index).unwrap_err(),
            CameraViewError::CameraIndex { index, cameras: 2 },
            "camera_index out of range"
        );
    }
    // 2) nworld 0 / 2 / 3 rejected
    for nworld in [0usize, 2, 3] {
        expect_eq!(
            CameraOutputViews::new(&plan, &layout, nworld, data).unwrap_err(),
            CameraViewError::NworldUnsupported { nworld },
            "nworld unsupported"
        );
    }
    // 3) channel disabled per channel (camera 0 off -> adr -1), and camera 1 still reads its region
    let camera0_pixels = pixels(resolutions[0]);
    for channel in ["rgb", "depth", "seg"] {
        let mut partial = plan.clone();
        match channel {
            "rgb" => partial.render_rgb = vec![false, true],
            "depth" => partial.render_depth = vec![false, true],
            _ => partial.render_seg = vec![false, true],
        }
        let partial_layout = OutputLayout::resolve(&partial).expect("layout");
        let (rgb, depth, seg) = match channel {
            "rgb" => (
                Some(&whole_packed[camera0_pixels..]),
                Some(whole_depth.as_slice()),
                Some(whole_seg.as_slice()),
            ),
            "depth" => (
                Some(whole_packed.as_slice()),
                Some(&whole_depth[camera0_pixels..]),
                Some(whole_seg.as_slice()),
            ),
            _ => (
                Some(whole_packed.as_slice()),
                Some(whole_depth.as_slice()),
                Some(&whole_seg[camera0_pixels..]),
            ),
        };
        let partial_views = views(
            &partial,
            &partial_layout,
            WholeOutputData {
                rgb,
                depth,
                segmentation: seg,
            },
        );
        let expected_channel = match channel {
            "rgb" => ImageOutput::Rgb,
            "depth" => ImageOutput::Depth,
            _ => ImageOutput::Segmentation,
        };
        let error = match channel {
            "rgb" => partial_views.rgb(0).unwrap_err(),
            "depth" => partial_views.depth(0, 1.0).unwrap_err(),
            _ => partial_views.segmentation(0).unwrap_err(),
        };
        expect_eq!(
            error,
            CameraViewError::ChannelDisabled {
                channel: expected_channel,
                camera_index: 0
            },
            "channel disabled"
        );
        // camera 1 view must still be the correct region of the shrunken buffer
        match channel {
            "rgb" => expect!(
                partial_views.rgb_packed(1).expect("camera1 rgb")
                    == &whole_packed[camera0_pixels..],
                "disabled-camera buffer keeps camera 1 region"
            ),
            "depth" => expect!(
                partial_views.depth_raw(1).expect("camera1 depth")
                    == &whole_depth[camera0_pixels..],
                "disabled-camera buffer keeps camera 1 depth region"
            ),
            _ => expect!(
                partial_views.segmentation_raw(1).expect("camera1 seg")
                    == &whole_seg[camera0_pixels..],
                "disabled-camera buffer keeps camera 1 seg region"
            ),
        }
    }
    // 4) buffer missing per channel
    let empty = views(&plan, &layout, WholeOutputData::default());
    expect_eq!(
        empty.rgb(0).unwrap_err(),
        CameraViewError::BufferMissing { channel: ImageOutput::Rgb },
        "buffer missing rgb"
    );
    expect_eq!(
        empty.depth_raw(0).unwrap_err(),
        CameraViewError::BufferMissing { channel: ImageOutput::Depth },
        "buffer missing depth"
    );
    expect_eq!(
        empty.segmentation_raw(0).unwrap_err(),
        CameraViewError::BufferMissing { channel: ImageOutput::Segmentation },
        "buffer missing seg"
    );
    // 5) buffer length short / long
    let short = WholeOutputData { rgb: Some(&whole_packed[..whole_packed.len() - 1]), ..data };
    expect_eq!(
        CameraOutputViews::new(&plan, &layout, 1, short).unwrap_err(),
        CameraViewError::BufferLength {
            channel: ImageOutput::Rgb,
            expected: 120_000,
            actual: 119_999
        },
        "buffer short"
    );
    let mut long_rgb = whole_packed.clone();
    long_rgb.push(0);
    expect_eq!(
        CameraOutputViews::new(&plan, &layout, 1, WholeOutputData { rgb: Some(&long_rgb), ..data })
            .unwrap_err(),
        CameraViewError::BufferLength {
            channel: ImageOutput::Rgb,
            expected: 120_000,
            actual: 120_001
        },
        "buffer long"
    );
    // 6) region range overflow
    let mut broken_layout = layout.clone();
    broken_layout.rgb_adr = vec![0, 100_000];
    let broken_views = views(&plan, &broken_layout, data);
    expect_eq!(
        broken_views.rgb(1).unwrap_err(),
        CameraViewError::BufferRange {
            channel: ImageOutput::Rgb,
            camera_index: 1,
            end: 143_200,
            actual: 120_000
        },
        "buffer range"
    );
    // 7) depth_scale invalid; subnormal / f32::MAX accepted
    for scale in [0.0f32, -0.0, -1.0, f32::INFINITY, f32::NEG_INFINITY] {
        expect_eq!(
            output_views.depth(0, scale).unwrap_err(),
            CameraViewError::DepthScale { depth_scale: scale },
            "depth scale invalid"
        );
    }
    expect!(
        matches!(output_views.depth(0, f32::NAN).unwrap_err(), CameraViewError::DepthScale { .. }),
        "depth scale NaN"
    );
    expect!(output_views.depth(0, f32::MIN_POSITIVE).is_ok(), "subnormal scale accepted");
    expect!(output_views.depth(0, f32::MAX).is_ok(), "f32::MAX scale accepted");
    // 7b) error precedence: invalid scale on a disabled/missing channel surfaces the buffer error first
    let mut depth_off = plan.clone();
    depth_off.render_depth = vec![false, true];
    let depth_off_layout = OutputLayout::resolve(&depth_off).expect("layout");
    let depth_off_views = views(
        &depth_off,
        &depth_off_layout,
        WholeOutputData {
            depth: Some(&whole_depth[camera0_pixels..]),
            ..WholeOutputData::default()
        },
    );
    expect_eq!(
        depth_off_views.depth(0, 0.0).unwrap_err(),
        CameraViewError::ChannelDisabled { channel: ImageOutput::Depth, camera_index: 0 },
        "error precedence: disabled before depth_scale"
    );
    // 8) plan/layout not from the same resolve
    let other_layout = OutputLayout::resolve(&plan_for(&[[2, 2], [2, 2], [2, 2]])).expect("layout");
    expect!(
        matches!(
            CameraOutputViews::new(&plan, &other_layout, 1, WholeOutputData::default()).unwrap_err(),
            CameraViewError::CameraCount { expected: 2, actual: 3 }
        ),
        "camera count mismatch"
    );
    let mut flags = plan.clone();
    flags.render_seg = vec![true];
    expect!(
        matches!(
            CameraOutputViews::new(&flags, &layout, 1, WholeOutputData::default()).unwrap_err(),
            CameraViewError::PlanFlags { channel: ImageOutput::Segmentation, cameras: 2, actual: 1 }
        ),
        "plan flags length"
    );
    let mut flipped = plan.clone();
    flipped.render_depth = vec![true, false];
    expect!(
        matches!(
            CameraOutputViews::new(
                &flipped,
                &layout,
                1,
                WholeOutputData { rgb: Some(&whole_packed), ..WholeOutputData::default() }
            )
            .unwrap_err(),
            CameraViewError::LayoutPixels { channel: ImageOutput::Depth, expected: 76_800, actual: 120_000 }
        ),
        "layout pixels mismatch"
    );
    // 9) empty camera list and single camera list
    let zero_plan = plan_for(&[]);
    let zero_layout = OutputLayout::resolve(&zero_plan).expect("zero layout");
    let zero_views = views(&zero_plan, &zero_layout, WholeOutputData::default());
    expect_eq!(zero_views.camera_count(), 0, "zero cameras count");
    expect_eq!(
        zero_views.camera_id(0).unwrap_err(),
        CameraViewError::CameraIndex { index: 0, cameras: 0 },
        "zero cameras index"
    );
    expect_eq!(
        zero_views.rgb(0).unwrap_err(),
        CameraViewError::CameraIndex { index: 0, cameras: 0 },
        "zero cameras rgb"
    );
    let one_plan = plan_for(&[[3, 2]]);
    let one_layout = OutputLayout::resolve(&one_plan).expect("one layout");
    let one_rgb: Vec<u32> = (0..6u32).map(|value| value | (value << 8) | (value << 16)).collect();
    let one_views = views(
        &one_plan,
        &one_layout,
        WholeOutputData { rgb: Some(&one_rgb), ..WholeOutputData::default() },
    );
    expect_eq!(one_views.camera_count(), 1, "single camera");
    expect!(one_views.rgb_packed(0).expect("single").as_ref() == one_rgb.as_slice(), "single camera packed");
    // 10) depth clamp edge semantics (NaN / +-0 / +-inf / out-of-range)
    let edge_raw: Vec<f32> = vec![
        0.0,
        -0.0,
        0.5,
        1.0,
        1.5,
        -2.0,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NAN,
        2.0,
        1e-30,
        -1e-30,
    ];
    let edge_plan = plan_for(&[[4, 3]]);
    let edge_layout = OutputLayout::resolve(&edge_plan).expect("edge layout");
    let edge_views = views(
        &edge_plan,
        &edge_layout,
        WholeOutputData { depth: Some(&edge_raw), ..WholeOutputData::default() },
    );
    for scale in [1.0f32, 0.5, 1e30] {
        let image = edge_views.depth(0, scale).expect("edge depth");
        for (value, raw) in image.pixels().iter().zip(&edge_raw) {
            expect!(
                value.to_bits() == qa_clamp_bits(*raw / scale),
                "depth clamp edge semantics"
            );
        }
    }
    // 11) isolation sentinel: 2 cameras 4x3 + 2x2, cameras=[1,0], adr=[0,12]
    let mut sentinel_plan = plan_for(&[[4, 3], [2, 2]]);
    sentinel_plan.cameras = vec![1, 0];
    let sentinel_layout = OutputLayout::resolve(&sentinel_plan).expect("sentinel layout");
    expect_eq!(sentinel_layout.rgb_adr.clone(), vec![0, 12], "sentinel rgb adr");
    expect_eq!(sentinel_layout.depth_adr.clone(), vec![0, 12], "sentinel depth adr");
    expect_eq!(sentinel_layout.seg_adr.clone(), vec![0, 12], "sentinel seg adr");
    let sentinel_rgb: Vec<u32> = (0..16u32)
        .map(|value| value | (value << 8) | (value << 16))
        .collect();
    let sentinel_depth: Vec<f32> = (0..16u32).map(|value| value as f32 + 0.25).collect();
    let sentinel_seg: Vec<[i32; 2]> = (0..16i32).map(|value| [value, value / 12]).collect();
    let sentinel_views = views(
        &sentinel_plan,
        &sentinel_layout,
        WholeOutputData {
            rgb: Some(&sentinel_rgb),
            depth: Some(&sentinel_depth),
            segmentation: Some(&sentinel_seg),
        },
    );
    expect_eq!(sentinel_views.camera_id(0).expect("sentinel id"), 1, "sentinel id 0 -> 1");
    expect_eq!(sentinel_views.camera_id(1).expect("sentinel id"), 0, "sentinel id 1 -> 0");
    expect!(sentinel_views.rgb_packed(0).expect("A") == &sentinel_rgb[..12], "sentinel A slice");
    expect!(sentinel_views.rgb_packed(1).expect("B") == &sentinel_rgb[12..], "sentinel B slice");
    let camera_a = sentinel_views.rgb(0).expect("A rgb");
    let camera_b = sentinel_views.rgb(1).expect("B rgb");
    expect_eq!((camera_a.width(), camera_a.height()), (4, 3), "sentinel A resolution");
    expect_eq!((camera_b.width(), camera_b.height()), (2, 2), "sentinel B resolution");
    for local in 0..12usize {
        expect!(
            camera_a
                .get(local as u32 % 4, local as u32 / 4)
                .map(|pixel| pixel.map(f32::to_bits))
                == camera_a.pixels().get(local).map(|pixel| pixel.map(f32::to_bits)),
            "sentinel A row-major mapping"
        );
    }
    expect!(camera_a.row_major_index(4, 0).is_none(), "sentinel A x boundary");
    expect!(camera_a.row_major_index(0, 3).is_none(), "sentinel A y boundary");
    expect!(camera_b.row_major_index(2, 0).is_none(), "sentinel B x boundary");
    expect!(
        sentinel_views.segmentation_raw(1).expect("B seg").first().copied() == Some([12, 1]),
        "sentinel B seg start"
    );
    expect_eq!(
        sentinel_views.depth(1, 16.0).expect("B depth").pixels()[0].to_bits(),
        (12.25f32 / 16.0).to_bits(),
        "sentinel B depth scaled"
    );
    expect!(
        sentinel_views.depth(0, 16.0).expect("A depth").pixels().last().copied().map(f32::to_bits)
            == Some((11.25f32 / 16.0).to_bits()),
        "sentinel A depth last"
    );

    for failure in &failures {
        eprintln!("FAILURE: {failure}");
    }
    assert!(
        failures.is_empty(),
        "{} failures (first: {:?})",
        failures.len(),
        failures.first()
    );
    let summary = serde_json::json!({"checks": checks, "failures": failures});
    fs::write(
        out_dir().join("harness_edges.json"),
        serde_json::to_vec_pretty(&summary).expect("json"),
    )
    .expect("write summary");
    println!("edge/isolation ok: checks={checks}");
}
