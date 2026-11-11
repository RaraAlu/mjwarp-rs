//! z2vec 片 1：`flex_quat_z2vec` 宿主逐式镜像 CPU 单测（测试专用，不进入产品路径）。
//!
//! 镜像与 `src/render/kernel/cuda/render_flex.cu` 的 `flex_quat_z2vec` / `flex_quat_to_mat` 逐式对应
//! （对照见证据 `perworld-l-v2-flex-z2vec-pin/README.md` §mirror-map）。
//!
//! **不静默回退声明（硬约束）**：本文件的镜像仅用于 host 断言；产品运行时始终走 `render_flex.cu`
//! 的设备侧实现，**镜像不进入产品路径、不得作为运行时回退**；若 `.cu` 与镜像不一致（含文本锚点被改），
//! 本测试必须**失败**，不允许 `skip`/`fallback`/条件绕过。设备端行为仅由内核身份（megakernel
//! source/PTX 前后基线）+ 离线 PTX 证明；CPU 镜像断言不等价于设备端验收。
//!
//! 期望矩阵来自 Arch 规格 `target/render-bvh/gl2-ptx-diff/z2vec-probe-expectations.md` §2
//! （上游真值 `probe_quat.log`；#2–#7 容差 1e-6，其中 #3/#4/#6/#7 有 1 ulp CPU 三角函数库差）。

#![allow(clippy::excessive_precision)]
// 说明：期望矩阵按 Arch 规格 `probe_quat.log` 的十进制直接内联（f32 有效位之外的位数是规格转录），
// 故允许 excessive_precision（`-0.0` 是上游矩阵里的真实负零项，按规格直接内联）。

/// 上游 `types.MJ_MINVAL`（`mjtype.h:27`；与 `render_flex.cu:27` 的 `FLEX_MINVAL` 对应）。
const FLEX_MINVAL: f32 = 1.0e-15;

/// 镜像 `render_flex.cu::warp_length3`（`sqrt(dot(v,v))`，f32）。
fn warp_length3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// 镜像 `render_flex.cu::flex_quat_z2vec`（含 z2vec 修复后的近平行分支：显式四位 `(1,0,0,0)`）。
fn flex_quat_z2vec(vec: [f32; 3]) -> [f32; 4] {
    let mut quat = [0.0f32, 0.0, 0.0, 1.0];
    let norm = warp_length3(vec);
    if norm < FLEX_MINVAL {
        return quat;
    }
    let unit = [vec[0] / norm, vec[1] / norm, vec[2] / norm];
    let axis = [-unit[1], unit[0], 0.0];
    let a = warp_length3(axis);
    if a < FLEX_MINVAL {
        if unit[2] < 0.0 {
            quat = [1.0, 0.0, 0.0, 0.0];
        }
        return quat;
    }
    let axis = [axis[0] / a, axis[1] / a, axis[2] / a];
    let angle = a.atan2(unit[2]);
    let half = angle * 0.5;
    let s = half.sin();
    let c = half.cos();
    [c, axis[0] * s, axis[1] * s, axis[2] * s]
}

/// 镜像 `render_flex.cu::flex_quat_to_mat`（行主序 3x3）。
fn flex_quat_to_mat(q: [f32; 4]) -> [[f32; 3]; 3] {
    let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
    let (q00, q01, q02, q03) = (x * x, x * y, x * z, x * w);
    let (q11, q12, q13) = (y * y, y * z, y * w);
    let (q22, q23, q33) = (z * z, z * w, w * w);
    [
        [q00 + q11 - q22 - q33, 2.0 * (q12 - q03), 2.0 * (q13 + q02)],
        [2.0 * (q12 + q03), q00 - q11 + q22 - q33, 2.0 * (q23 - q01)],
        [2.0 * (q13 - q02), 2.0 * (q23 + q01), q00 - q11 - q22 + q33],
    ]
}

fn mat_for(vec: [f32; 3]) -> [[f32; 3]; 3] {
    flex_quat_to_mat(flex_quat_z2vec(vec))
}

fn assert_mat_close(actual: [[f32; 3]; 3], expected: [[f32; 3]; 3], label: &str) {
    for r in 0..3 {
        for c in 0..3 {
            let diff = (actual[r][c] - expected[r][c]).abs();
            assert!(
                diff <= 1e-6,
                "{label}: mat[{r}][{c}] actual={} expected={} diff={diff:e}",
                actual[r][c],
                expected[r][c]
            );
        }
    }
}

fn assert_quat_eq(actual: [f32; 4], expected: [f32; 4], label: &str) {
    for i in 0..4 {
        assert_eq!(actual[i], expected[i], "{label}: quat[{i}]");
    }
}

const DIAG_NEG_NEG_POS: [[f32; 3]; 3] = [[-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]];
const IDENTITY: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

#[test]
fn probe_1_negative_z_is_exactly_identity_after_fix() {
    // 规格 §2 #1：旧版红（矩阵差 2.0），修复后 = **精确单位阵**（Arch 补充要求：assert_eq!）。
    assert_quat_eq(
        flex_quat_z2vec([0.0, 0.0, -1.0]),
        [1.0, 0.0, 0.0, 0.0],
        "probe#1 quat",
    );
    assert_eq!(
        mat_for([0.0, 0.0, -1.0]),
        IDENTITY,
        "probe#1 矩阵必须精确等于单位阵"
    );
}

#[test]
fn probe_2_positive_z_keeps_unit_quaternion() {
    assert_quat_eq(
        flex_quat_z2vec([0.0, 0.0, 1.0]),
        [0.0, 0.0, 0.0, 1.0],
        "probe#2 quat",
    );
    assert_mat_close(mat_for([0.0, 0.0, 1.0]), DIAG_NEG_NEG_POS, "probe#2");
}

#[test]
fn probe_3_y_axis() {
    let expected = [[1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, -1.0, 0.0]];
    assert_mat_close(mat_for([0.0, 0.5, 0.0]), expected, "probe#3");
}

#[test]
fn probe_4_x_axis() {
    let expected = [[0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]];
    assert_mat_close(mat_for([0.5, 0.0, 0.0]), expected, "probe#4");
}

#[test]
fn probe_5_zero_vector_uses_upstream_quat_to_mat_convention() {
    assert_mat_close(mat_for([0.0, 0.0, 0.0]), DIAG_NEG_NEG_POS, "probe#5");
}

#[test]
fn probe_6_and_7_diagonal_planar_vectors() {
    let expected6 = [
        [0.63999999, -0.47999999, 0.60000002],
        [-0.47999999, 0.36000001, 0.80000001],
        [-0.60000002, -0.80000001, -0.0],
    ];
    let expected7 = [
        [0.63999999, -0.47999999, -0.60000002],
        [-0.47999999, 0.36000001, -0.80000001],
        [0.60000002, 0.80000001, -0.0],
    ];
    assert_mat_close(mat_for([0.6, 0.8, 0.0]), expected6, "probe#6");
    assert_mat_close(mat_for([-0.6, -0.8, 0.0]), expected7, "probe#7");
}

#[test]
fn threshold_and_near_parallel_branches() {
    assert_mat_close(mat_for([1.0e-16, 0.0, 0.0]), DIAG_NEG_NEG_POS, "norm 1e-16");
    assert_mat_close(
        mat_for([1.0e-20, 0.0, -1.0e-20]),
        DIAG_NEG_NEG_POS,
        "norm 1e-20",
    );
    assert_mat_close(
        mat_for([1.0e-30, 0.0, -1.0]),
        IDENTITY,
        "parallel -z (fixed)",
    );
    assert_mat_close(
        mat_for([1.0e-30, 0.0, 1.0]),
        DIAG_NEG_NEG_POS,
        "parallel +z",
    );
}

#[test]
fn negative_zero_vector() {
    assert_mat_close(mat_for([-0.0, -0.0, -0.0]), DIAG_NEG_NEG_POS, "-0.0");
}

/// 文本锚点自检：源改而镜像不跟改时本测试必红（Arch 补充要求）。
#[test]
fn cu_source_anchors_match_mirror_contract() {
    let src = include_str!("../src/render/kernel/cuda/render_flex.cu");
    for line in [
        "quat[0] = 1.0f;",
        "quat[1] = 0.0f;",
        "quat[2] = 0.0f;",
        "quat[3] = 0.0f;",
    ] {
        assert!(src.contains(line), "产品 .cu 缺少四位赋值：{line}");
    }
    assert!(
        !src.contains("// 近平行：反向时绕 x 轴 180°"),
        "产品 .cu 仍含旧注释形态（只置 quat[0] 的旧版）"
    );
    let branch_start = src
        .find("if (unit.z < 0.0f) {")
        .expect("近平行反向分支存在");
    let branch_end = src[branch_start..]
        .find("return;")
        .expect("分支 return 存在")
        + branch_start;
    let branch = &src[branch_start..branch_end];
    assert_eq!(
        branch.matches("quat[").count(),
        4,
        "近平行反向分支必须恰有 4 个 quat 赋值"
    );
    for anchor in [
        "float norm = warp_length3(vec);",
        "float a = warp_length3(axis);",
        "float angle = atan2f(a, unit.z);",
        "float half = angle * 0.5f;",
        "quat[0] = c;",
        "quat[1] = axis.x * s;",
        "quat[2] = axis.y * s;",
        "quat[3] = axis.z * s;",
        "float w = quat[3];",
    ] {
        assert!(src.contains(anchor), "产品 .cu 缺少镜像锚点：{anchor}");
    }
}
