//! G10@P5 G5：`relative_matrix` 的上游语义（`transpose(a) * b`，行优先存储）。
//!
//! 上游 `mujoco_warp/_src/collision_sdf.py`：
//! `params.rel_mat = wp.transpose(rot1) * rot2`，随后 `x1 = rel_mat * x2 + rel_pos`。
//! 在行优先 mat3 下即 `rel[row][col] = Σ_k a[k][row] * b[k][col]`（即 `aᵀb`）。
//!
//! **累加方式也必须是上游位型**：上游 `warp/native/mat.h` 的 `mul(mat,mat)` 用 `muladd`
//! （`fmaf`）逐项累加：`sum = 0; for k: sum = fma(a[i][k], b[k][j], sum)`（k 升序）；
//! 而 `mul(mat,vec)` 用普通乘加（故 `transpose_mul_mat3_vec` 保持普通乘加）。
//! QA 实测：只改转置仍会在「两侧都旋转」场景留残差，补上 fma 累加后 rel_mat 位型与 warp
//! 10/10 world 一致、SDF 接触 40/40 位级 0 ULP（见 `qa-sdf-rotation-report.json`）。
//!
//! 注意：G3/G4 host 里的同名 helper（`sdf/descent.rs`、`hfield/driver.rs`）按
//! 「逐列 `transpose_mul_mat3_vec(a, mul_mat3_vec(b, e_col))` 再写入 `column*3+i`」构造，
//! 在行优先约定下得到的是 `bᵀa`（上游结果的转置），且用普通乘加。G3/G4 的数值 case 因
//! geom1 恒为单位旋转（`a = I` 时 `aᵀb = bᵀa = b`）未被暴露；G5 设备侧按**上游语义**实现，
//! 不复制 host 约定（产品 host 路径由 `2026-10-25-g10-p5-sdf-relmat-fix` 修正）。

/// 上游 `warp/native/mat.h::muladd` 的 f32 语义：`fma(a, b, c)`（一次正确舍入）。
///
/// 刻意**不用** `f32::mul_add`：本机 dev 档（opt-level 0）实测未融合
/// （QA：`0x3f17e6af` vs 正确 `0x3f17e6b0`，见 `qa-sdf-rotation-report.json`）。
/// 这里用 f64 精确承载 f32 乘积（`f32 × f32` 在 f64 中精确可表示），再做一次和，
/// 最后单次舍入回 f32：与正确舍入的 fma 在实测用例上位级一致（QA rel_mat 10/10 world 位等）。
#[inline]
fn fma_f32(a: f32, b: f32, c: f32) -> f32 {
    (f64::from(a) * f64::from(b) + f64::from(c)) as f32
}

/// 上游语义的 `transpose(a) * b`，行优先存储；按上游 `muladd` 的 fma 链累加（k 升序）。
pub(crate) fn upstream_relative_matrix(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
    let mut out = [0.0_f32; 9];
    for row in 0..3 {
        for column in 0..3 {
            let sum = fma_f32(a[row], b[column], 0.0);
            let sum = fma_f32(a[3 + row], b[3 + column], sum);
            out[row * 3 + column] = fma_f32(a[6 + row], b[6 + column], sum);
        }
    }
    out
}

/// 行优先 `mat * vec`（仅用于测试参考，避免依赖其他模块）。
pub(crate) fn row_major_mat_vec(matrix: &[f32; 9], vector: [f32; 3]) -> [f32; 3] {
    [
        matrix[0] * vector[0] + matrix[1] * vector[1] + matrix[2] * vector[2],
        matrix[3] * vector[0] + matrix[4] * vector[1] + matrix[5] * vector[2],
        matrix[6] * vector[0] + matrix[7] * vector[1] + matrix[8] * vector[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// QA 实测位型：`docs/development-evidence/g10-p5-hfield-host-independent-check/`
    /// `qa-sdf-rotation-cases.json`（mujoco_warp 3.15 f32 CPU，两次捕获位等）。
    ///
    /// 场景 `sdf-rot-both`：box `euler="25 10 30"`、SDF `Rz(0.7)`；捕获的 `geom_xmat`
    /// 顺序为 [box, SDF]（QA 探针按该顺序算 `transpose(rot1)*rot2`），
    /// 期望值取自同文件 `rel_mat_warp`（warp 实际位型）。
    const QA_BOX_M: [f32; 9] = [
        f32::from_bits(0x3F5A5598),
        f32::from_bits(0xBEFC1C5C),
        f32::from_bits(0x3E31D0D4),
        f32::from_bits(0x3F044707),
        f32::from_bits(0x3F3F8985),
        f32::from_bits(0xBED517DF),
        f32::from_bits(0x3D99A18F),
        f32::from_bits(0x3EE3AE14),
        f32::from_bits(0x3F647D6F),
    ];
    const QA_SDF_RZ: [f32; 9] = [
        f32::from_bits(0x3F43CCB2),
        f32::from_bits(0xBF24EB73),
        f32::from_bits(0x00000000),
        f32::from_bits(0x3F24EB73),
        f32::from_bits(0x3F43CCB2),
        f32::from_bits(0x00000000),
        f32::from_bits(0x00000000),
        f32::from_bits(0x00000000),
        f32::from_bits(0x3F800000),
    ];
    const QA_REL_MAT_BOTH: [f32; 9] = [
        f32::from_bits(0x3F7C34F0),
        f32::from_bits(0xBE1DEF14),
        f32::from_bits(0x3D99A18F),
        f32::from_bits(0x3DD7D554),
        f32::from_bits(0x3F63B3EC),
        f32::from_bits(0x3EE3AE14),
        f32::from_bits(0xBE0A8E4F),
        f32::from_bits(0xBEDC423B),
        f32::from_bits(0x3F647D6F),
    ];

    /// 场景 `sdf-rot-rz-rx`：box `euler="22.918312 0 0"`（`Rx(0.4)`）、SDF `Rz(0.7)`；
    /// 该 cell（`rel[1][0]`）正是 QA 指出的 `f32::mul_add` 未融合敏感位（正确 `0x3F17E6B0`）。
    const QA_BOX_RX: [f32; 9] = [
        f32::from_bits(0x3F800000),
        f32::from_bits(0x00000000),
        f32::from_bits(0x00000000),
        f32::from_bits(0x00000000),
        f32::from_bits(0x3F6BCAA7),
        f32::from_bits(0xBEC761D7),
        f32::from_bits(0x00000000),
        f32::from_bits(0x3EC761D7),
        f32::from_bits(0x3F6BCAA7),
    ];
    const QA_REL_MAT_RZ_RX: [f32; 9] = [
        f32::from_bits(0x3F43CCB2),
        f32::from_bits(0xBF24EB73),
        f32::from_bits(0x00000000),
        f32::from_bits(0x3F17E6B0),
        f32::from_bits(0x3F3457E7),
        f32::from_bits(0x3EC761D7),
        f32::from_bits(0xBE80720E),
        f32::from_bits(0xBE987EF3),
        f32::from_bits(0x3F6BCAA7),
    ];

    /// 行优先绕 z 轴旋转 `angle`。
    fn rot_z(angle: f32) -> [f32; 9] {
        let (s, c) = (angle.sin(), angle.cos());
        [c, -s, 0.0, s, c, 0.0, 0.0, 0.0, 1.0]
    }

    /// 行优先绕 x 轴旋转 `angle`。
    fn rot_x(angle: f32) -> [f32; 9] {
        let (s, c) = (angle.sin(), angle.cos());
        [1.0, 0.0, 0.0, 0.0, c, -s, 0.0, s, c]
    }

    /// 独立参考（**普通乘加**，非 fma）：`aᵀb`，先用显式下标构造 `aᵀ`，再做普通矩阵乘。
    /// 用于(i)复现旧 host 约定、(ii)对比 fma 链的位差。
    fn upstream_reference(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
        let mut at = [0.0_f32; 9];
        for row in 0..3 {
            for column in 0..3 {
                at[row * 3 + column] = a[column * 3 + row];
            }
        }
        let mut out = [0.0_f32; 9];
        for row in 0..3 {
            for column in 0..3 {
                out[row * 3 + column] = (0..3).map(|k| at[row * 3 + k] * b[k * 3 + column]).sum();
            }
        }
        out
    }

    /// 旧 host 构造语义（`bᵀa` 的行优先存储），仅用于复现 QA 的差异观察。
    fn previous_host_semantics(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
        let mut out = [0.0_f32; 9];
        for column in 0..3 {
            let mut basis = [0.0_f32; 3];
            basis[column] = 1.0;
            // mul_mat3_vec(b, e_col) = b 的第 column 列
            let column_b = row_major_mat_vec(b, basis);
            // transpose_mul_mat3_vec(a, w) = aᵀ w
            let mut at = [0.0_f32; 9];
            for row in 0..3 {
                for col in 0..3 {
                    at[row * 3 + col] = a[col * 3 + row];
                }
            }
            let vector = row_major_mat_vec(&at, column_b);
            out[column * 3] = vector[0];
            out[column * 3 + 1] = vector[1];
            out[column * 3 + 2] = vector[2];
        }
        out
    }

    fn max_abs_diff(left: &[f32; 9], right: &[f32; 9]) -> f32 {
        left.iter()
            .zip(right)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f32, f32::max)
    }

    /// 同号/异号 f32 的有序位距离（ULP）。
    fn ulp_diff(left: f32, right: f32) -> u32 {
        fn key(value: f32) -> i64 {
            let bits = value.to_bits() as i32;
            if bits < 0 {
                0x8000_0000_i64 - i64::from(bits)
            } else {
                i64::from(bits)
            }
        }
        (key(left) - key(right)).unsigned_abs() as u32
    }

    /// 上游 `warp/native/mat.h` 的 `mul(mat,mat)` 用 fma 累加；本实现必须复现 warp 位型。
    #[test]
    fn upstream_relative_matrix_matches_warp_bits_sdf_rot_both() {
        let actual = upstream_relative_matrix(&QA_BOX_M, &QA_SDF_RZ);
        for index in 0..9 {
            assert_eq!(
                actual[index].to_bits(),
                QA_REL_MAT_BOTH[index].to_bits(),
                "cell {index}: fma 链位型必须等于 warp rel_mat"
            );
        }
    }

    /// 单侧旋转场景同样位级一致；cell 3（`0x3F17E6B0`）是 QA 指出的 `f32::mul_add` 敏感位。
    #[test]
    fn upstream_relative_matrix_matches_warp_bits_sdf_rot_rz_rx() {
        let actual = upstream_relative_matrix(&QA_BOX_RX, &QA_SDF_RZ);
        for index in 0..9 {
            assert_eq!(
                actual[index].to_bits(),
                QA_REL_MAT_RZ_RX[index].to_bits(),
                "cell {index}: fma 链位型必须等于 warp rel_mat"
            );
        }
        assert_eq!(QA_REL_MAT_RZ_RX[3].to_bits(), 0x3F17_E6B0);
    }

    /// 普通乘加（不含 fma 链）在「两侧都旋转」场景与 warp 位型差 1 cell / 1 ULP，
    /// 与 QA `rel_mat_cells_diff_computed_vs_warp = 1` 对应；这就是必须用 fma 链的原因。
    #[test]
    fn plain_accumulation_differs_from_warp_bits() {
        let plain = upstream_reference(&QA_BOX_M, &QA_SDF_RZ);
        let differing: Vec<usize> = (0..9)
            .filter(|&index| plain[index].to_bits() != QA_REL_MAT_BOTH[index].to_bits())
            .collect();
        assert_eq!(differing, vec![6], "普通乘加只应在 cell 6 与 warp 位型不同");
        assert_eq!(ulp_diff(plain[6], QA_REL_MAT_BOTH[6]), 1);
    }

    #[test]
    fn upstream_relative_matrix_matches_transpose_rot1_rot2() {
        let a = rot_z(0.7);
        let b = rot_x(0.4);
        let plain = upstream_reference(&a, &b);
        let actual = upstream_relative_matrix(&a, &b);
        for index in 0..9 {
            assert!(
                ulp_diff(actual[index], plain[index]) <= 1,
                "cell {index}: fma={} plain={}",
                actual[index],
                plain[index]
            );
        }

        // QA 观察：旧 host 构造得到的是转置（同一普通乘加运算族，精确等于转置）。
        let previous = previous_host_semantics(&a, &b);
        let mut plain_transposed = [0.0_f32; 9];
        for row in 0..3 {
            for column in 0..3 {
                plain_transposed[row * 3 + column] = plain[column * 3 + row];
            }
        }
        assert_eq!(
            previous, plain_transposed,
            "旧 host 构造应等于上游结果的转置"
        );
        assert!(
            max_abs_diff(&previous, &actual) > 1.0,
            "旧 host 约定与上游差异应显著（实测约 1.24）"
        );

        // 语义检查：rel * x2 等于 aᵀ (b x2)。
        let x2 = [0.3_f32, -0.7, 1.1];
        let mut at = [0.0_f32; 9];
        for row in 0..3 {
            for column in 0..3 {
                at[row * 3 + column] = a[column * 3 + row];
            }
        }
        let expected_vector = row_major_mat_vec(&at, row_major_mat_vec(&b, x2));
        let actual_vector = row_major_mat_vec(&actual, x2);
        for axis in 0..3 {
            assert!(
                (actual_vector[axis] - expected_vector[axis]).abs() < 1.0e-6,
                "axis {axis}"
            );
        }
    }
}
