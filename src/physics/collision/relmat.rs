//! G10@P5 G5：`relative_matrix` 的上游语义（`transpose(a) * b`，行优先存储）。
//!
//! 上游 `mujoco_warp/_src/collision_sdf.py`：
//! `params.rel_mat = wp.transpose(rot1) * rot2`，随后 `x1 = rel_mat * x2 + rel_pos`。
//! 在行优先 mat3 下即 `rel[row][col] = Σ_k a[k][row] * b[k][col]`（即 `aᵀb`）。
//!
//! 注意：G3/G4 host 里的同名 helper（`sdf/descent.rs`、`hfield/driver.rs`）按
//! 「逐列 `transpose_mul_mat3_vec(a, mul_mat3_vec(b, e_col))` 再写入 `column*3+i`」构造，
//! 在行优先约定下得到的是 `bᵀa`（上游结果的转置）。G3/G4 的数值 case 因 geom1 恒为单位旋转
//! （`a = I` 时 `aᵀb = bᵀa = b`）未被暴露；G5 设备侧按**上游语义**实现，不复制 host 约定。

/// 上游语义的 `transpose(a) * b`，行优先存储。
pub(crate) fn upstream_relative_matrix(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
    let mut out = [0.0_f32; 9];
    for row in 0..3 {
        for column in 0..3 {
            out[row * 3 + column] =
                a[row] * b[column] + a[3 + row] * b[3 + column] + a[6 + row] * b[6 + column];
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

    /// 独立参考：`aᵀb`，先用显式下标构造 `aᵀ`（行优先），再做普通矩阵乘 `aᵀ·b`。
    /// 期望值来自上游公式，而不是从本文件的 helper 反推。
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
            let column_b = [
                row_major_mat_vec(b, basis)[0],
                row_major_mat_vec(b, basis)[1],
                row_major_mat_vec(b, basis)[2],
            ];
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

    #[test]
    fn upstream_relative_matrix_matches_transpose_rot1_rot2() {
        let a = rot_z(0.7);
        let b = rot_x(0.4);
        let expected = upstream_reference(&a, &b);
        let actual = upstream_relative_matrix(&a, &b);
        assert_eq!(actual, expected, "设备/上游语义必须一致（行优先 aᵀb）");

        // QA 观察：旧 host 构造得到的是转置；两者差异不为 0。
        let previous = previous_host_semantics(&a, &b);
        assert!(
            max_abs_diff(&previous, &expected) > 1.0,
            "旧 host 约定与上游差异应显著（实测约 1.24）"
        );
        let mut expected_transposed = [0.0_f32; 9];
        for row in 0..3 {
            for column in 0..3 {
                expected_transposed[row * 3 + column] = expected[column * 3 + row];
            }
        }
        assert_eq!(
            previous, expected_transposed,
            "旧 host 构造应等于上游结果的转置"
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
