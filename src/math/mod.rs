// Copyright 2025 The Newton Developers.
// Licensed under the Apache License, Version 2.0.
// Adapted from mujoco_warp at 71da24d956378a87a703b6e1442b13aec0c4ac29.
// See THIRD_PARTY_NOTICES.md and LICENSES/Apache-2.0.txt.

//! 冻结上游的八项纯数学公式。
//! 四元数顺序为`[w,x,y,z]`。
//! 空间向量先转动，后平动。
//! 不隐式归一化或检查有限值。

pub type Vec3 = [f32; 3];
pub type Quat = [f32; 4];
pub type Mat3 = [[f32; 3]; 3];
pub type SpatialVector = [f32; 6];

pub fn mul_quat(u: Quat, v: Quat) -> Quat {
    [
        u[0] * v[0] - u[1] * v[1] - u[2] * v[2] - u[3] * v[3],
        u[0] * v[1] + u[1] * v[0] + u[2] * v[3] - u[3] * v[2],
        u[0] * v[2] - u[1] * v[3] + u[2] * v[0] + u[3] * v[1],
        u[0] * v[3] + u[1] * v[2] - u[2] * v[1] + u[3] * v[0],
    ]
}

pub fn quat_mul_axis(q: Quat, axis: Vec3) -> Quat {
    [
        -q[1] * axis[0] - q[2] * axis[1] - q[3] * axis[2],
        q[0] * axis[0] + q[2] * axis[2] - q[3] * axis[1],
        q[0] * axis[1] + q[3] * axis[0] - q[1] * axis[2],
        q[0] * axis[2] + q[1] * axis[1] - q[2] * axis[0],
    ]
}

pub fn rot_vec_quat(vector: Vec3, quaternion: Quat) -> Vec3 {
    let scalar = quaternion[0];
    let axis = [quaternion[1], quaternion[2], quaternion[3]];
    let projection = dot(axis, vector);
    let scale = scalar * scalar - dot(axis, axis);
    let cross = cross(axis, vector);
    std::array::from_fn(|i| {
        2.0 * (projection * axis[i]) + scale * vector[i] + 2.0 * scalar * cross[i]
    })
}

pub fn axis_angle_to_quat(axis: Vec3, angle: f32) -> Quat {
    let half_angle = angle * 0.5;
    let sin = half_angle.sin();
    [
        half_angle.cos(),
        axis[0] * sin,
        axis[1] * sin,
        axis[2] * sin,
    ]
}

/// 返回行主序矩阵。
pub fn quat_to_mat(q: Quat) -> Mat3 {
    let q00 = q[0] * q[0];
    let q11 = q[1] * q[1];
    let q22 = q[2] * q[2];
    let q33 = q[3] * q[3];
    let q01 = q[0] * q[1];
    let q02 = q[0] * q[2];
    let q03 = q[0] * q[3];
    let q12 = q[1] * q[2];
    let q13 = q[1] * q[3];
    let q23 = q[2] * q[3];
    [
        [q00 + q11 - q22 - q33, 2.0 * (q12 - q03), 2.0 * (q13 + q02)],
        [2.0 * (q12 + q03), q00 - q11 + q22 - q33, 2.0 * (q23 - q01)],
        [2.0 * (q13 - q02), 2.0 * (q23 + q01), q00 - q11 - q22 + q33],
    ]
}

/// 沿用上游的共轭语义。
/// 不除以四元数模长平方。
pub fn quat_inv(q: Quat) -> Quat {
    [q[0], -q[1], -q[2], -q[3]]
}

pub fn motion_cross(u: SpatialVector, v: SpatialVector) -> SpatialVector {
    let angular = cross([u[0], u[1], u[2]], [v[0], v[1], v[2]]);
    let linear_a = cross([u[3], u[4], u[5]], [v[0], v[1], v[2]]);
    let linear_b = cross([u[0], u[1], u[2]], [v[3], v[4], v[5]]);
    [
        angular[0],
        angular[1],
        angular[2],
        linear_a[0] + linear_b[0],
        linear_a[1] + linear_b[1],
        linear_a[2] + linear_b[2],
    ]
}

pub fn motion_cross_force(v: SpatialVector, force: SpatialVector) -> SpatialVector {
    let angular_a = cross([v[0], v[1], v[2]], [force[0], force[1], force[2]]);
    let angular_b = cross([v[3], v[4], v[5]], [force[3], force[4], force[5]]);
    let linear = cross([v[0], v[1], v[2]], [force[3], force[4], force[5]]);
    [
        angular_a[0] + angular_b[0],
        angular_a[1] + angular_b[1],
        angular_a[2] + angular_b[2],
        linear[0],
        linear[1],
        linear[2],
    ]
}

fn dot(u: Vec3, v: Vec3) -> f32 {
    u[0] * v[0] + u[1] * v[1] + u[2] * v[2]
}

fn cross(u: Vec3, v: Vec3) -> Vec3 {
    [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: f32 = 1.0e-6;

    fn assert_close<const N: usize>(actual: [f32; N], expected: [f32; N]) {
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!((actual - expected).abs() <= TOLERANCE);
        }
    }

    #[test]
    fn preserves_hamilton_product_order() {
        let x = [0.0, 1.0, 0.0, 0.0];
        let y = [0.0, 0.0, 1.0, 0.0];
        assert_eq!(mul_quat(x, y), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(mul_quat(y, x), [0.0, 0.0, 0.0, -1.0]);
        assert_eq!(mul_quat([1.0, 0.0, 0.0, 0.0], x), x);
        assert_eq!(
            quat_mul_axis([1.0, 2.0, 3.0, 4.0], [5.0, 6.0, 7.0]),
            [-56.0, 2.0, 12.0, 4.0]
        );
    }

    #[test]
    fn preserves_conjugate_without_normalization() {
        let q = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(quat_inv(q), [1.0, -2.0, -3.0, -4.0]);
        assert_eq!(mul_quat(q, quat_inv(q)), [30.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn matches_analytic_quarter_turn() {
        let q = axis_angle_to_quat([0.0, 0.0, 1.0], std::f32::consts::FRAC_PI_2);
        assert_close(
            q,
            [
                std::f32::consts::FRAC_1_SQRT_2,
                0.0,
                0.0,
                std::f32::consts::FRAC_1_SQRT_2,
            ],
        );
        assert_close(rot_vec_quat([2.0, 0.0, 3.0], q), [0.0, 2.0, 3.0]);
        assert_close(
            rot_vec_quat([2.0, 0.0, 3.0], q.map(|v| -v)),
            [0.0, 2.0, 3.0],
        );
        let matrix = quat_to_mat(q);
        assert_close(matrix[0], [0.0, -1.0, 0.0]);
        assert_close(matrix[1], [1.0, 0.0, 0.0]);
        assert_close(matrix[2], [0.0, 0.0, 1.0]);
    }

    #[test]
    fn preserves_non_unit_and_zero_quaternions() {
        assert_eq!(
            rot_vec_quat([1.0, 2.0, 3.0], [2.0, 0.0, 0.0, 0.0]),
            [4.0, 8.0, 12.0]
        );
        assert_eq!(
            quat_to_mat([2.0, 0.0, 0.0, 0.0]),
            [[4.0, 0.0, 0.0], [0.0, 4.0, 0.0], [0.0, 0.0, 4.0]]
        );
        assert_eq!(rot_vec_quat([1.0, 2.0, 3.0], [0.0; 4]), [0.0; 3]);
        assert_eq!(quat_to_mat([0.0; 4]), [[0.0; 3]; 3]);
        let q = axis_angle_to_quat([0.0, 0.0, 2.0], std::f32::consts::PI);
        assert_close(q, [0.0, 0.0, 0.0, 2.0]);
    }

    #[test]
    fn matrix_matches_rotation_for_non_unit_input() {
        let q = [0.5, -1.0, 2.0, 0.25];
        let vector = [3.0, -2.0, 1.0];
        let product = quat_to_mat(q).map(|row| dot(row, vector));
        assert_eq!(product, rot_vec_quat(vector, q));
    }

    #[test]
    fn matches_analytic_spatial_cross_products() {
        let u = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let v = [7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        assert_eq!(motion_cross(u, v), [-6.0, 12.0, -6.0, -12.0, 24.0, -12.0]);
        assert_eq!(
            motion_cross_force(u, v),
            [-12.0, 24.0, -12.0, -9.0, 18.0, -9.0]
        );
    }

    #[test]
    fn preserves_motion_force_duality() {
        let u = [1.0, -2.0, 3.0, 4.0, -5.0, 6.0];
        let v = [-7.0, 8.0, 2.0, -1.0, 4.0, 3.0];
        let force = [3.0, 1.0, -4.0, 2.0, -6.0, 5.0];
        let dot6 = |a: SpatialVector, b: SpatialVector| {
            a.into_iter().zip(b).map(|(a, b)| a * b).sum::<f32>()
        };
        assert_eq!(
            dot6(motion_cross(u, v), force) + dot6(v, motion_cross_force(u, force)),
            0.0
        );
    }

    #[test]
    fn propagates_non_finite_values() {
        assert!(
            rot_vec_quat([1.0; 3], [f32::NAN, 0.0, 0.0, 0.0])
                .iter()
                .all(|v| v.is_nan())
        );
        assert!(axis_angle_to_quat([0.0, 0.0, 1.0], f32::INFINITY)[0].is_nan());
    }
}
