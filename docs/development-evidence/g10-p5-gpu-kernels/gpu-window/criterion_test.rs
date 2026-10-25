//! G5 判据补全的**可执行判别力断言**（由 `gpu_compare.rs` 以
//! `#[path = "criterion_test.rs"] mod criterion_test;` 引入一次；不是独立测试目标）。
//!
//! 目的：证明「口径补全 ≠ 放宽容差」——把修前两组实测缺陷数字喂进**新**判据并断言 FAIL，
//! 同时断言修后残差通过。数据来源（`README` §11.6 / §11.7.1）：
//! - 修前 `ellipsoid-box` `pos` 绝对差 **8.3e-5**（修复后 3.02e-9）；
//! - 修前 `ellipsoid-box` `dist` 绝对差 **4.4e-8**（修复后 world1 位级）。
//!
//! 运行位置：集成测试目标 `tests/g10_p5_gpu_kernels.rs`
//! （`#![cfg(feature = "cuda-probe")]`）：`cargo test --locked --features cuda-probe
//! --test g10_p5_gpu_kernels` → 1 passed / 3 ignored；**不进默认 `cargo test` 的 unit 计数**。

// 字面量是证据 f32 位型的十进制抄录（保留原始有效位），不裁剪精度以免改测试数值。
#[allow(clippy::excessive_precision)]
#[test]
fn criterion_rejects_pre_fix_defects() {
    use crate::gpu_compare::criterion_scale::{
        dist_ok, vector_bound, vector_component_ok, vector_scale,
    };

    // ① 修前 pos：参考 world0 pos（`fixtures/g10-p5/references/ellipsoid-box.json`），
    //    设备 y 分量偏 8.3e-5（窗口 1 实测修前值）。
    let reference_pos = [0.039960995_f32, -3.2569260e-05, 0.055000056];
    let pre_fix_pos = [
        reference_pos[0],
        reference_pos[1] + 8.3e-5,
        reference_pos[2],
    ];
    let scale = vector_scale(&reference_pos);
    assert!(
        !vector_component_ok(pre_fix_pos[1], reference_pos[1], scale),
        "修前 pos 差 8.3e-5 必须被新判据拒绝（scale={scale}, bound={}）",
        vector_bound(scale)
    );
    // 修后残差 3.02e-9（实测）通过 —— 证明补全解的是「近零分量 ULP 无定义」，不是放宽缺陷。
    assert!(vector_component_ok(-0.00003256624, -0.00003256926, scale));

    // ② 修前 dist：参考 -0.00999989，设备差 4.4e-8（47 ULP@0.01）。
    let reference_dist = -0.00999989_f32;
    let pre_fix_dist = reference_dist - 4.4e-8;
    assert!(
        !dist_ok(pre_fix_dist, reference_dist),
        "修前 dist 差 4.4e-8 必须被新判据拒绝"
    );
    // 修后 world1 位级相等通过。
    assert!(dist_ok(-0.0099998210, -0.0099998210));

    // 拒绝阈值打印进证据日志（`--nocapture` 可见）。
    println!(
        "criterion_scale: pos scale={scale} bound={}；修前 8.3e-5 → {}；修后 3.02e-9 → pass；dist 修前 4.4e-8 → fail",
        vector_bound(scale),
        pre_fix_pos[1] - reference_pos[1]
    );
}
