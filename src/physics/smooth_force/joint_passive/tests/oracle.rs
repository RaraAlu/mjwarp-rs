//! 关节被动测试冻结参考工具。

#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
#[cfg(feature = "cuda-probe")]
pub(super) fn raw(b: &TransferBuffer<f32>) -> Vec<f32> {
    let mut v = vec![0.0; b.len()];
    b.read_range_into(0, &mut v).unwrap();
    v
}
#[cfg(feature = "cuda-probe")]
pub(super) fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|v| v.to_bits()).collect()
}
#[cfg(feature = "cuda-probe")]
pub(super) const ROTATION_ABS_TOLERANCE: f32 = 2.0e-6;
#[cfg(feature = "cuda-probe")]
pub(super) fn warp_len3(v: [f32; 3]) -> f32 {
    ((v[0] * v[0] + v[1] * v[1]) + v[2] * v[2]).sqrt()
}
#[cfg(feature = "cuda-probe")]
pub(super) fn warp_normalize_quat(q: [f32; 4]) -> [f32; 4] {
    let length = (((q[0] * q[0] + q[1] * q[1]) + q[2] * q[2]) + q[3] * q[3]).sqrt();
    if length > 0.0 {
        let inverse = 1.0 / length;
        [
            q[0] * inverse,
            q[1] * inverse,
            q[2] * inverse,
            q[3] * inverse,
        ]
    } else {
        [0.0, 0.0, 0.0, 1.0]
    }
}
#[cfg(feature = "cuda-probe")]
pub(super) fn warp_mul_quat(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [
        ((a[0] * b[0] - a[1] * b[1]) - a[2] * b[2]) - a[3] * b[3],
        ((a[0] * b[1] + a[1] * b[0]) + a[2] * b[3]) - a[3] * b[2],
        ((a[0] * b[2] - a[1] * b[3]) + a[2] * b[0]) + a[3] * b[1],
        ((a[0] * b[3] + a[1] * b[2]) - a[2] * b[1]) + a[3] * b[0],
    ]
}
#[cfg(feature = "cuda-probe")]
pub(super) fn warp_quat_to_vel(q: [f32; 4]) -> [f32; 3] {
    let axis = [q[1], q[2], q[3]];
    let sine = warp_len3(axis);
    if sine == 0.0 {
        return [0.0; 3];
    }
    let mut speed = 2.0 * sine.atan2(q[0]);
    if speed > std::f32::consts::PI {
        speed -= 2.0 * std::f32::consts::PI;
    }
    [
        (axis[0] * speed) / sine,
        (axis[1] * speed) / sine,
        (axis[2] * speed) / sine,
    ]
}
#[cfg(feature = "cuda-probe")]
pub(super) fn warp_quat_sub(current: [f32; 4], reference: [f32; 4]) -> [f32; 3] {
    let current = warp_normalize_quat(current);
    let conjugate = [reference[0], -reference[1], -reference[2], -reference[3]];
    warp_quat_to_vel(warp_mul_quat(conjugate, current))
}
#[cfg(feature = "cuda-probe")]
pub(super) fn warp_poly(linear: f32, a: f32, b: f32, x: f32) -> f32 {
    let mut result = linear;
    result += a * x;
    result += (b * x) * x;
    result
}
#[cfg(feature = "cuda-probe")]
pub(super) fn rotation_oracle(i: &Input, world: usize, qpos: &[f32]) -> Vec<f32> {
    let reference = i.params.qpos_spring.world(world);
    let stiffness = i.params.stiffness.world(world);
    let polynomial = i.params.stiffness_poly.world(world);
    let mut result = vec![0.0; i.model.nv()];
    let translation = [
        qpos[0] - reference[0],
        qpos[1] - reference[1],
        qpos[2] - reference[2],
    ];
    let length = warp_len3(translation);
    let gain = warp_poly(stiffness[0], polynomial[0], polynomial[1], length);
    for k in 0..3 {
        result[k] = -gain * translation[k];
    }
    for (joint, qadr, dadr) in [(0, 3, 3), (1, 7, 6)] {
        let difference = warp_quat_sub(
            qpos[qadr..qadr + 4].try_into().unwrap(),
            reference[qadr..qadr + 4].try_into().unwrap(),
        );
        let length = warp_len3(difference);
        let gain = warp_poly(
            stiffness[joint],
            polynomial[2 * joint],
            polynomial[2 * joint + 1],
            length,
        );
        for k in 0..3 {
            result[dadr + k] = -gain * difference[k];
        }
    }
    for (joint, qadr, dadr) in [(2, 11, 9), (3, 12, 10)] {
        let difference = qpos[qadr] - reference[qadr];
        let gain = warp_poly(
            stiffness[joint],
            polynomial[2 * joint],
            polynomial[2 * joint + 1],
            difference,
        );
        result[dadr] = -gain * difference;
    }
    result
}
#[cfg(feature = "cuda-probe")]
pub(super) fn oracle(
    i: &Input,
    w: usize,
    q: &[f32],
    v: &[f32],
    o: JointPassiveOptions,
) -> (Vec<f32>, Vec<f32>) {
    let m = &i.model;
    let ps = i.params.qpos_spring.world(w);
    let st = i.params.stiffness.world(w);
    let sp = i.params.stiffness_poly.world(w);
    let da = i.params.damping.world(w);
    let dp = i.params.damping_poly.world(w);
    let mut spring = vec![0.0f32; m.nv()];
    let mut damper = spring.clone();
    for j in 0..m.njnt() {
        let kind = JointType::try_from(m.fields().jnt_type[j]).unwrap();
        let d = m.fields().jnt_dofadr[j] as usize;
        for k in 0..kind.dof_width() {
            let a = d + k;
            if !o.disable_damper && (da[a] != 0.0 || dp[2 * a] != 0.0 || dp[2 * a + 1] != 0.0) {
                let vel = v[a] as f64;
                let x = vel.abs();
                let f = -vel * (da[a] as f64 + dp[2 * a] as f64 * x + dp[2 * a + 1] as f64 * x * x);
                assert_eq!(f as f32 as f64, f);
                damper[a] = f as f32;
            }
        }
        if !o.disable_spring
            && matches!(kind, JointType::Slide | JointType::Hinge)
            && (st[j] != 0.0 || sp[2 * j] != 0.0 || sp[2 * j + 1] != 0.0)
        {
            let a = m.fields().jnt_qposadr[j] as usize;
            let x = q[a] as f64 - ps[a] as f64;
            let f = -x * (st[j] as f64 + sp[2 * j] as f64 * x + sp[2 * j + 1] as f64 * x * x);
            assert_eq!(f as f32 as f64, f);
            spring[d] = f as f32;
        }
    }
    (spring, damper)
}
