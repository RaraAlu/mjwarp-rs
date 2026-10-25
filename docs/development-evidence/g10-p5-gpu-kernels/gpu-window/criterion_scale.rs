//! G5 判据补全（Arch 2026-10-25 裁决 2，向量尺度 ULP）：位置向量判据的**实现（单一来源）**。
//!
//! 只由 `gpu_compare.rs`（接触数组）以 `#[path = "criterion_scale.rs"]` 引入一次；
//! `witness_compare.rs`（ccd witness）通过
//! `use crate::gpu_compare::criterion_scale::{...}` 复用同一模块，不加载第二份副本。
//! 判别力 `#[test]` 放在 `criterion_test.rs`（由 `gpu_compare.rs` 以 `#[path]` 引入一次）。
//!
//! 判据全文（Arch 裁决，禁止再放宽容差）：
//! - `pos`/`x1`/`x2`（位置向量）：`scale = max_i |ref_i|`，逐分量 `|dev-ref| ≤ 4 · ULP(scale)`，
//!   位级优先；**整向量全零**时才回落到 `scale = 1.0`。
//! - `frame`/`normal`：不变（单位向量，`scale = 1.0`，≤4 ULP 绝对 ≈ 4.77e-7，不设近零带）。
//! - `dist`（标量）：按自身量级 ULP（≤4 ULP）；`ref == 0` 用 4 ULP@1.0 绝对差。
//! - 参数/整数字段：位级相等。
//!
//! 为什么要向量尺度而不是「近零带」：`pos` 由 `0.5*(w1+w2)+origin` 合成，舍入尺度是坐标
//! 量级（≈0.055），不是某个接近 0 的分量自身（其 ULP 随 |v|→0 无限缩小）。向量尺度在
//! `ellipsoid-box` 只放行 `4·ULP(0.055)=1.49e-8`，而 4 ULP@1.0 带放行到 4.77e-7，
//! 松约 **32×**（4.768e-7 ÷ 1.490e-8 = 32.0；Arch 裁决文字记 31×，按精确值应为 32×）。
//! 修前缺陷 `pos` 差 8.3e-5、`dist` 差 4.4e-8 在新规则下**仍然拒绝**（见 `criterion_test.rs`）。

#![allow(dead_code)]

/// 4 ULP @1.0（f32）：`frame`/`normal` 的绝对差上限，也是 `dist` 零参考分量的兜底。
pub(crate) const FOUR_ULP_AT_ONE: f32 = 4.0 * (1.0 / 8_388_608.0);

/// f32 在 |v| 处的 ULP（相邻可表示数间距）。要求 `value > 0` 且有限。
pub(crate) fn ulp_at(value: f32) -> f32 {
    f32::from_bits(value.to_bits() + 1) - value
}

/// 位置向量的判据尺度：`max_i |ref_i|`；整向量全零时回落到 1.0。
pub(crate) fn vector_scale(reference: &[f32]) -> f32 {
    let mut scale = 0.0_f32;
    for value in reference {
        scale = scale.max(value.abs());
    }
    if scale == 0.0 { 1.0 } else { scale }
}

/// 位置向量的允许绝对差：`4 · ULP(scale)`。
pub(crate) fn vector_bound(scale: f32) -> f32 {
    4.0 * ulp_at(scale)
}

/// 位置向量单分量是否通过（向量尺度绝对差）。
pub(crate) fn vector_component_ok(device: f32, reference: f32, scale: f32) -> bool {
    (device - reference).abs() <= vector_bound(scale)
}

/// `dist` 标量是否通过：非零参考按 ULP ≤4；零参考按 ≤4 ULP@1.0 绝对差。
pub(crate) fn dist_ok(device: f32, reference: f32) -> bool {
    if reference == 0.0 {
        (device - reference).abs() <= FOUR_ULP_AT_ONE
    } else {
        fn key(value: f32) -> i64 {
            let bits = value.to_bits() as i32;
            if bits < 0 {
                0x8000_0000_i64 - i64::from(bits)
            } else {
                i64::from(bits)
            }
        }
        (key(device) - key(reference)).unsigned_abs() <= 4
    }
}
