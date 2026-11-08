//! 判据实体 CPU 合成自证（零 GPU）：边界 383/384/575/576、两对 world、两台相机、C1/C2 不互换、
//! 某套 0/0 时 C2 必失败、A 判据位级统计方向、B 判据位型、隔离/两跑 sha、npz 解析。
//! 由 `tests/render_per_world_light.rs` 的 `#[cfg(test)]` 挂载；与 `#[ignore]` 的 GPU 用例分离。

use super::*;
use crate::{Frame, Reference, ReferenceState};

fn flat_depth(value: f32) -> Vec<f32> {
    vec![value; PIXELS]
}

fn flat_seg(pair: [i32; 2]) -> Vec<[i32; 2]> {
    vec![pair; PIXELS]
}

fn base_frame() -> Frame {
    Frame {
        rgb: vec![0xFF10_2030; PIXELS],
        depth: flat_depth(1.0),
        seg: flat_seg([-1, -1]),
    }
}

fn empty_state() -> ReferenceState {
    ReferenceState {
        geom_xpos: vec![0.0; WORLDS * LEAVES * 3],
        geom_xmat: vec![0.0; WORLDS * LEAVES * 9],
        light_xpos: vec![0.0; WORLDS * 3 * 3],
        light_xdir: vec![0.0; WORLDS * 3 * 3],
    }
}

fn reference(frames: Vec<Vec<Frame>>) -> Reference {
    let rgb_f32 = frames
        .iter()
        .map(|row| row.iter().map(|_| vec![[0.0f32; 3]; PIXELS]).collect())
        .collect();
    Reference {
        frames,
        rgb_f32,
        state: empty_state(),
        capture: serde_json::Value::Null,
    }
}

/// 设置参考侧原始 f32 rgb（判据 C 的 C1 只消费它，不消费量化字节）。
fn set_rgb(reference: &mut Reference, world: usize, cam: usize, pixel: usize, value: [f32; 3]) {
    reference.rgb_f32[world][cam][pixel] = value;
}

fn counts(values: [usize; 6]) -> CCounts {
    CCounts {
        c1_rgb_changed: [values[0], values[1]],
        c2_depth_changed: [values[2], values[3]],
        c2_seg_changed: [values[4], values[5]],
        ..CCounts::default()
    }
}

// ---- 判据 C 判定边界（383/384/575/576）----

#[test]
fn verdict_boundaries_383_384_575_576() {
    assert_eq!(
        verdict(&counts([MARGIN; 6])),
        Verdict::PassMargin,
        "恰 576 = pass-margin"
    );
    assert_eq!(verdict(&counts([MARGIN + 1; 6])), Verdict::PassMargin);
    assert_eq!(
        verdict(&counts([MARGIN - 1; 6])),
        Verdict::Insufficient,
        "575 = 余量不足"
    );
    assert_eq!(
        verdict(&counts([ACCEPT; 6])),
        Verdict::Insufficient,
        "384 = 余量不足（<576）"
    );
    assert_eq!(
        verdict(&counts([ACCEPT - 1; 6])),
        Verdict::NonDiscriminating,
        "383 = 无判别力"
    );
    // 任一集合/任一 world 对不达标即整轮失败
    for index in 0..6 {
        let mut values = [MARGIN; 6];
        values[index] = ACCEPT - 1;
        assert_eq!(
            verdict(&counts(values)),
            Verdict::NonDiscriminating,
            "集合 {index} <384"
        );
        let mut values = [MARGIN; 6];
        values[index] = ACCEPT;
        assert_eq!(
            verdict(&counts(values)),
            Verdict::Insufficient,
            "集合 {index} = 384"
        );
    }
    assert_eq!(
        verdict(&CCounts::default()),
        Verdict::NonDiscriminating,
        "全 0 = 无判别力"
    );
}

#[test]
fn zero_c2_set_fails_even_when_c1_is_large() {
    let mut values = [MARGIN * 4; 6];
    values[2] = 0;
    values[3] = 0; // C2 depth 两对 0/0
    assert_eq!(
        verdict(&counts(values)),
        Verdict::NonDiscriminating,
        "某套 0/0 时该套必须失败，C1 再大也不能救"
    );
}

// ---- 判据 C 统计方向与相机归属 ----

#[test]
#[rustfmt::skip]
fn criterion_c_counts_c1_on_cam_persp_and_c2_on_cam_hinge() {
    // cam-persp：相邻 world 对 rgb_f32 各差 1/255 步进（f32 下 ≥ 1/255），depth/seg 不变。
    let mut reference = reference(vec![vec![base_frame(), base_frame()]; WORLDS]);
    set_rgb(&mut reference, 1, 0, 7, [0.5, 0.5, 0.5 + 1.0 / 255.0]);
    set_rgb(&mut reference, 2, 0, 7, [0.5, 0.5, 0.5 + 3.0 / 255.0]);
    // cam-hinge：相邻 world 对 depth 位型 + seg 变化；rgb 不变（诊断项应为 0）。
    reference.frames[1][1].depth[11] = f32::from_bits(1.0f32.to_bits().wrapping_add(1));
    reference.frames[2][1].depth[11] = f32::from_bits(1.0f32.to_bits().wrapping_add(2));
    reference.frames[1][1].seg[13] = [3, 5];
    reference.frames[2][1].seg[13] = [4, 5];
    let counts = criterion_c(&reference);
    assert_eq!(counts.c1_rgb_changed, [1, 1], "C1 只统计 cam-persp 的 rgb 变化像素");
    assert_eq!(counts.c2_depth_changed, [1, 1], "C2 depth 只统计 cam-hinge");
    assert_eq!(counts.c2_seg_changed, [1, 1], "C2 seg 只统计 cam-hinge");
    assert_eq!(counts.diag_campersp_depth, [0, 0], "cam-persp depth 诊断应为 0");
    assert_eq!(counts.diag_campersp_seg, [0, 0], "cam-persp seg 诊断应为 0");
    assert_eq!(counts.diag_camhinge_rgb, [0, 0], "cam-hinge rgb 诊断应为 0");
}

#[test]
#[rustfmt::skip]
fn criterion_c_rejects_camera_swapped_reference() {
    // 反向构造：几何变化在 cam-persp、rgb 变化在 cam-hinge -> C1/C2 必须判 0（不得互换归属）。
    let mut frames = vec![vec![base_frame(), base_frame()]; WORLDS];
    frames[1][0].depth[3] = 2.0;
    frames[2][0].depth[3] = 3.0;
    frames[1][0].seg[3] = [1, 5];
    frames[2][0].seg[3] = [2, 5];
    let mut reference = reference(frames);
    set_rgb(&mut reference, 1, 1, 5, [0.6, 0.5, 0.5]);
    set_rgb(&mut reference, 2, 1, 5, [0.7, 0.5, 0.5]);
    let counts = criterion_c(&reference);
    assert_eq!(counts.c1_rgb_changed, [0, 0], "cam-hinge 的 rgb 变化不得计入 C1");
    assert_eq!(counts.c2_depth_changed, [0, 0], "cam-persp 的 depth 变化不得计入 C2");
    assert_eq!(counts.c2_seg_changed, [0, 0], "cam-persp 的 seg 变化不得计入 C2");
    assert_eq!(counts.diag_camhinge_rgb, [1, 1], "cam-hinge rgb 只进诊断项");
    assert_eq!(counts.diag_campersp_depth, [1, 1]);
    assert_eq!(verdict(&counts), Verdict::NonDiscriminating, "互换后必须拒绝通过");
}

#[test]
#[rustfmt::skip]
fn criterion_c_c1_counts_f32_threshold_1_over_255_not_byte_steps() {
    // F1 回归：C1 必须逐位复刻 spec v3 §6 / G-L0 readout 的 f32 `max_channel|Δrgb| ≥ 1/255`，
    // 不得把 rgb 量化回 8-bit 再比字节差（254↔255 饱和步进会被字节法多算）。
    let step = 1.0f32 / 255.0f32; // 0.003921568859_3685627（f32 舍入后 > 1/255）
    assert!(rgb_above_threshold([0.0, 0.0, 0.0], [0.0, 0.0, step]), "1/255 步进必须计入");
    assert!(rgb_above_threshold([0.0, 0.0, 0.0], [0.0, 0.0, 2.0 * step]), "2/255 步进必须计入");
    assert!(!rgb_above_threshold([0.0, 0.0, 0.0], [0.0, 0.0, step * 0.5]), "半步不得计入");
    assert!(!rgb_above_threshold([1.0, 1.0, 1.0], [254.0f32 / 255.0f32, 1.0, 1.0]),
        "254/255 -> 1.0 的 f32 差 0.00392150879 < 1/255，饱和步进不计");
    let mut reference = reference(vec![vec![base_frame(), base_frame()]; WORLDS]);
    // 对 0->1：像素 1 蓝 +1/255 计；像素 2 红 +2/255 计（取通道最大差）；像素 3 饱和步进不计。
    set_rgb(&mut reference, 1, 0, 1, [0.0, 0.0, step]);
    set_rgb(&mut reference, 2, 0, 1, [0.0, 0.0, 2.0 * step]);
    set_rgb(&mut reference, 1, 0, 2, [2.0 * step, 0.0, 0.0]);
    set_rgb(&mut reference, 0, 0, 3, [254.0f32 / 255.0f32, 1.0, 1.0]);
    set_rgb(&mut reference, 1, 0, 3, [1.0, 1.0, 1.0]);
    for cam in 0..ACTIVE_CAMERA_COUNT { for pixel in 0..PIXELS {
        let value = reference.rgb_f32[1][cam][pixel];
        set_rgb(&mut reference, 2, cam, pixel, value);
    } }
    assert_eq!(criterion_c(&reference).c1_rgb_changed, [2, 0], "只有 f32 语义下的 2 个像素计入");
}

#[test]
#[rustfmt::skip]
fn criterion_c_pair_order_is_world_major() {
    // 只有 pair(1->2) 有变化：pairs 顺序必须是 [0->1, 1->2]。
    let mut reference = reference(vec![vec![base_frame(), base_frame()]; WORLDS]);
    set_rgb(&mut reference, 2, 0, 9, [0.7, 0.5, 0.5]);
    reference.frames[2][1].depth[9] = 3.0;
    let counts = criterion_c(&reference);
    assert_eq!(counts.c1_rgb_changed, [0, 1], "第一对不变、第二对变化");
    assert_eq!(counts.c2_depth_changed, [0, 1]);
}

// ---- 判据 A：位级严格比较 ----

#[test]
fn criterion_a_detects_rgb_depth_seg_bit_differences() {
    let expected = base_frame();
    assert_eq!(compare_frame(&expected, &expected), FrameStats::default());

    let mut rgb = expected.clone();
    rgb.rgb[3] = 0xFF10_2130; // green +1
    let stats = compare_frame(&rgb, &expected);
    assert_eq!(
        (stats.rgb_mismatched_pixels, stats.rgb_max_byte_diff),
        (1, 1),
        "rgb 单字节差必须被检出"
    );

    let mut wide = expected.clone();
    wide.rgb[4] = 0xFF10_2032; // blue +2
    assert_eq!(
        compare_frame(&wide, &expected).rgb_max_byte_diff,
        2,
        "max_byte_diff 取通道最大差"
    );

    let mut depth = expected.clone();
    let bits = depth.depth[5].to_bits().wrapping_add(1);
    depth.depth[5] = f32::from_bits(bits);
    let stats = compare_frame(&depth, &expected);
    assert_eq!(
        (
            stats.depth_mismatched_pixels,
            stats.depth_beyond_ulp,
            stats.depth_max_ulp
        ),
        (1, 1, 1),
        "depth 位差必须被检出（0 ULP 口径）"
    );

    let mut seg = expected.clone();
    seg.seg[6] = [7, 5];
    assert_eq!(
        compare_frame(&seg, &expected).seg_mismatched_pixels,
        1,
        "seg pair 差必须被检出"
    );

    let mut length = expected.clone();
    length.depth.pop();
    assert!(
        std::panic::catch_unwind(|| compare_frame(&length, &expected)).is_err(),
        "长度不同必须显式失败"
    );
}

#[test]
fn criterion_a_aggregates_worlds_and_cameras() {
    let frames = vec![vec![base_frame(), base_frame()]; WORLDS];
    let reference = reference(frames.clone());
    let stats = criterion_a(&reference, &frames);
    assert_eq!(stats.len(), WORLDS);
    assert!(stats.iter().all(|row| row.len() == ACTIVE_CAMERA_COUNT));
    assert!(
        stats
            .iter()
            .flatten()
            .all(|item| *item == FrameStats::default())
    );
    let mut moved = frames;
    moved[2][1].seg[0] = [1, 5];
    let stats = criterion_a(&reference, &moved);
    assert_eq!(
        stats[2][1].seg_mismatched_pixels, 1,
        "统计必须定位到 world/cam"
    );
    assert_eq!(stats[0][0], FrameStats::default());
}

// ---- 判据 B：灯字段位型 ----

#[test]
fn criterion_b_compares_light_bits_and_reports_mutations() {
    let mut state = empty_state();
    for (world, light) in [(0, 0), (1, 1), (2, 2)] {
        let base = (world * 3 + light) * 3;
        state.light_xpos[base] = 1.5;
        state.light_xdir[base + 2] = -1.0;
    }
    let lights = world_light_poses(&state);
    let stats = criterion_b(&state, &lights);
    assert_eq!(
        (stats.xpos_mismatched, stats.xdir_mismatched),
        (0, 0),
        "逐位透传应 0 例外"
    );

    let mut poses = Vec::new();
    for world in 0..WORLDS {
        for light in 0..3 {
            let base = (world * 3 + light) * 3;
            let mutate = world == 0 && light == 0;
            poses.push(mjwarp_rs::render::kernel::LightDataPose {
                xpos: [
                    if mutate {
                        f32::from_bits(state.light_xpos[base].to_bits().wrapping_add(1))
                    } else {
                        state.light_xpos[base]
                    },
                    state.light_xpos[base + 1],
                    state.light_xpos[base + 2],
                ],
                xdir: [
                    state.light_xdir[base],
                    state.light_xdir[base + 1],
                    state.light_xdir[base + 2],
                ],
            });
        }
    }
    let mutated = mjwarp_rs::render::kernel::WorldLightPoses::new(WORLDS, 3, poses).unwrap();
    let stats = criterion_b(&state, &mutated);
    assert_eq!(stats.xpos_mismatched, 1, "xpos 位型差必须被检出");
    assert_eq!(stats.xdir_mismatched, 0);
}

// ---- 两跑 sha 与 world 隔离 ----

#[test]
fn sha_and_isolation_helpers_detect_only_world1_change() {
    let frames = vec![vec![base_frame(), base_frame()]; WORLDS];
    assert_eq!(sha256_frames(&frames), sha256_frames(&frames.clone()));
    let mut moved = frames.clone();
    moved[1][1].depth[0] = 2.0;
    let (before, after) = (sha256_frames(&frames), sha256_frames(&moved));
    assert_ne!(before, after, "任一 world 的位变化必须改变整块 sha");

    let changed = isolation_changed(&frames, &moved);
    assert_eq!(
        changed[0],
        vec![false; ACTIVE_CAMERA_COUNT],
        "world0 不得变化"
    );
    assert_eq!(
        changed[2],
        vec![false; ACTIVE_CAMERA_COUNT],
        "world2 不得变化"
    );
    assert!(
        changed[1].iter().any(|value| *value),
        "world1 至少一台相机输出必须变化（隔离反例成立性）"
    );
}

// ---- npz 最小解析（G-L2 参考读取路径；合成 ZIP_STORED 用例）----

fn build_npz(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut central = Vec::new();
    for (name, data) in entries {
        let crc = {
            let mut hash = 0xFFFF_FFFFu32;
            for byte in *data {
                hash ^= u32::from(*byte);
                for _ in 0..8 {
                    hash = if hash & 1 != 0 {
                        (hash >> 1) ^ 0xEDB8_8320
                    } else {
                        hash >> 1
                    };
                }
            }
            !hash
        };
        let offset = bytes.len() as u32;
        let name_bytes = name.as_bytes();
        bytes.extend_from_slice(&[0x50, 0x4B, 3, 4, 20, 0, 0, 0, 0, 0]);
        bytes.extend_from_slice(&0x1234_5678u32.to_le_bytes());
        bytes.extend_from_slice(&crc.to_le_bytes());
        bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(name_bytes);
        bytes.extend_from_slice(data);
        central.push((name_bytes.to_vec(), crc, data.len() as u32, offset));
    }
    let central_start = bytes.len() as u32;
    for (name, crc, size, offset) in &central {
        bytes.extend_from_slice(&[0x50, 0x4B, 1, 2, 20, 0, 20, 0, 0, 0, 0, 0]);
        bytes.extend_from_slice(&0x1234_5678u32.to_le_bytes());
        bytes.extend_from_slice(&crc.to_le_bytes());
        bytes.extend_from_slice(&size.to_le_bytes());
        bytes.extend_from_slice(&size.to_le_bytes());
        bytes.extend_from_slice(&(name.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&offset.to_le_bytes());
        bytes.extend_from_slice(name);
    }
    let central_size = bytes.len() as u32 - central_start;
    bytes.extend_from_slice(&[0x50, 0x4B, 5, 6, 0, 0, 0, 0]);
    bytes.extend_from_slice(&(central.len() as u16).to_le_bytes());
    bytes.extend_from_slice(&(central.len() as u16).to_le_bytes());
    bytes.extend_from_slice(&central_size.to_le_bytes());
    bytes.extend_from_slice(&central_start.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes
}

fn npy_f32(values: &[f32]) -> Vec<u8> {
    let mut header = format!(
        "{{'descr': '<f4', 'fortran_order': False, 'shape': ({}), }}",
        values.len()
    );
    while (10 + header.len() + 1) % 64 != 0 {
        header.push(' ');
    }
    header.push('\n');
    let mut bytes = b"\x93NUMPY\x01\x00".to_vec();
    bytes.extend_from_slice(&(header.len() as u16).to_le_bytes());
    bytes.extend_from_slice(header.as_bytes());
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

#[test]
fn npz_parser_reads_members_and_npy_header() {
    let a = npy_f32(&[1.0, 2.5]);
    let b = npy_f32(&[3.0]);
    let npz = build_npz(&[("a.npy", &a), ("b.npy", &b)]);
    let members = crate::npz::members(&npz);
    assert_eq!(members.len(), 2);
    let (descr, shape, data) = crate::npz::member(&members, "a.npy");
    assert_eq!(descr, "<f4");
    assert_eq!(shape, vec![2]);
    assert_eq!(data.len(), 8);
    assert!(std::panic::catch_unwind(|| crate::npz::member(&members, "missing.npy")).is_err());
    assert!(std::panic::catch_unwind(|| crate::npz::members(b"not-a-zip")).is_err());
}

// ---- O2：capture provenance 值与 C1 交叉核对 ----

#[test]
#[rustfmt::skip]
fn capture_c1_cross_check_compares_spec_2_4_dict_counts() {
    let counts = CCounts { c1_rgb_changed: [38_285, 40_644], c1_rgb_bitwise_changed: [43_604, 41_815], ..CCounts::default() };
    let capture = serde_json::json!({"c1_rgb_changed_px": {
        "0->1": {"magnitude": 38_285, "bitwise": 43_604},
        "1->2": {"magnitude": 40_644, "bitwise": 41_815}}});
    assert_eq!(check_capture_c1(&capture, &counts), CaptureC1Check::Compared, "字典结构两对数一致必须比对通过");
    // capture 只记录 magnitude 时：仍比对 magnitude 两对（bitwise 缺省不比对）。
    let magnitude_only = serde_json::json!({"c1_rgb_changed_px": {
        "0->1": {"magnitude": 38_285}, "1->2": {"magnitude": 40_644}}});
    assert_eq!(check_capture_c1(&magnitude_only, &counts), CaptureC1Check::Compared);
    let mut bad_magnitude = capture.clone();
    bad_magnitude["c1_rgb_changed_px"]["1->2"]["magnitude"] = serde_json::json!(40_645);
    assert!(std::panic::catch_unwind(|| check_capture_c1(&bad_magnitude, &counts)).is_err(), "magnitude 不一致必须失败");
    let mut bad_bitwise = capture.clone();
    bad_bitwise["c1_rgb_changed_px"]["0->1"]["bitwise"] = serde_json::json!(0);
    assert!(std::panic::catch_unwind(|| check_capture_c1(&bad_bitwise, &counts)).is_err(), "bitwise 不一致必须失败");
}

#[test]
#[rustfmt::skip]
fn capture_c1_cross_check_reports_explicit_skip_reasons() {
    let counts = CCounts::default();
    assert!(matches!(check_capture_c1(&serde_json::json!({}), &counts), CaptureC1Check::Skipped(_)), "缺字段必须显式跳过");
    assert!(matches!(check_capture_c1(&serde_json::json!({"c1_rgb_changed_px": [38_285, 40_644]}), &counts),
                     CaptureC1Check::Skipped(_)), "旧数组结构必须显式跳过并说明");
    assert!(matches!(check_capture_c1(&serde_json::json!({"c1_rgb_changed_px": {"0->1": {"magnitude": 1}}}), &counts),
                     CaptureC1Check::Skipped(_)), "缺 1->2 magnitude 必须显式跳过");
    assert!(!matches!(check_capture_c1(&serde_json::json!({"c1_rgb_changed_px": {"0->1": {"magnitude": 0}, "1->2": {"magnitude": 0}}}), &counts),
                      CaptureC1Check::Skipped(_)), "完整结构不得被当作跳过");
}

#[test]
#[rustfmt::skip]
fn capture_scope_text_handles_string_and_list() {
    assert_eq!(scope_text(&serde_json::json!("per-world light poses")), "per-world light poses");
    assert_eq!(scope_text(&serde_json::json!(["per-world light poses", "AND geom"])), "per-world light poses AND geom");
    assert_eq!(scope_text(&serde_json::json!(3)), "");
}
