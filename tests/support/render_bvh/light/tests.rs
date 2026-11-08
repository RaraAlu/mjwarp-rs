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
    Reference {
        frames,
        state: empty_state(),
        capture: serde_json::Value::Null,
    }
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
fn criterion_c_counts_c1_on_cam_persp_and_c2_on_cam_hinge() {
    let mut frames = vec![vec![base_frame(), base_frame()]; WORLDS];
    // cam-persp：相邻 world 对 rgb 各差 1 个字节（≥1/255），depth/seg 不变。
    frames[1][0].rgb[7] = 0xFF10_2031; // blue +1（0->1 变化）
    frames[2][0].rgb[7] = 0xFF10_2032; // blue +2（1->2 变化）
    // cam-hinge：相邻 world 对 depth 位型 + seg 变化；rgb 不变（诊断项应为 0）。
    frames[1][1].depth[11] = f32::from_bits(1.0f32.to_bits().wrapping_add(1));
    frames[2][1].depth[11] = f32::from_bits(1.0f32.to_bits().wrapping_add(2));
    frames[1][1].seg[13] = [3, 5];
    frames[2][1].seg[13] = [4, 5];
    let counts = criterion_c(&reference(frames));
    assert_eq!(
        counts.c1_rgb_changed,
        [1, 1],
        "C1 只统计 cam-persp 的 rgb 变化像素"
    );
    assert_eq!(counts.c2_depth_changed, [1, 1], "C2 depth 只统计 cam-hinge");
    assert_eq!(counts.c2_seg_changed, [1, 1], "C2 seg 只统计 cam-hinge");
    assert_eq!(
        counts.diag_campersp_depth,
        [0, 0],
        "cam-persp depth 诊断应为 0"
    );
    assert_eq!(counts.diag_campersp_seg, [0, 0], "cam-persp seg 诊断应为 0");
    assert_eq!(counts.diag_camhinge_rgb, [0, 0], "cam-hinge rgb 诊断应为 0");
}

#[test]
fn criterion_c_rejects_camera_swapped_reference() {
    // 反向构造：几何变化在 cam-persp、rgb 变化在 cam-hinge -> C1/C2 必须判 0（不得互换归属）。
    let mut frames = vec![vec![base_frame(), base_frame()]; WORLDS];
    frames[1][0].depth[3] = 2.0;
    frames[2][0].depth[3] = 3.0;
    frames[1][0].seg[3] = [1, 5];
    frames[2][0].seg[3] = [2, 5];
    frames[1][1].rgb[5] = 0xFF11_2030;
    frames[2][1].rgb[5] = 0xFF12_2030;
    let counts = criterion_c(&reference(frames));
    assert_eq!(
        counts.c1_rgb_changed,
        [0, 0],
        "cam-hinge 的 rgb 变化不得计入 C1"
    );
    assert_eq!(
        counts.c2_depth_changed,
        [0, 0],
        "cam-persp 的 depth 变化不得计入 C2"
    );
    assert_eq!(
        counts.c2_seg_changed,
        [0, 0],
        "cam-persp 的 seg 变化不得计入 C2"
    );
    assert_eq!(counts.diag_camhinge_rgb, [1, 1], "cam-hinge rgb 只进诊断项");
    assert_eq!(counts.diag_campersp_depth, [1, 1]);
    assert_eq!(
        verdict(&counts),
        Verdict::NonDiscriminating,
        "互换后必须拒绝通过"
    );
}

#[test]
fn criterion_c_rgb_threshold_is_one_byte_and_uses_max_channel() {
    let mut frames = vec![vec![base_frame(), base_frame()]; WORLDS];
    // world1: 像素 0 无变化（Δ=0），像素 1 蓝通道 +1（≥1/255 -> 计数 1）
    frames[1][0].rgb[0] = 0xFF10_2030;
    frames[1][0].rgb[1] = 0xFF10_2031;
    // world2 相对 world1：像素 2 红通道 -2（|Δ|=2 -> 计数 1）
    frames[2][0] = frames[1][0].clone();
    frames[2][0].rgb[2] = 0xFF0E_2031;
    let counts = criterion_c(&reference(frames));
    assert_eq!(
        counts.c1_rgb_changed,
        [1, 1],
        "1/255 阈值按字节整数比较且取通道最大差"
    );
}

#[test]
fn criterion_c_pair_order_is_world_major() {
    // 只有 pair(1->2) 有变化：pairs 顺序必须是 [0->1, 1->2]。
    let mut frames = vec![vec![base_frame(), base_frame()]; WORLDS];
    frames[2][0].rgb[9] = 0xFF10_2031;
    frames[2][1].depth[9] = 3.0;
    let counts = criterion_c(&reference(frames));
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
