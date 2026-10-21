//! 关节被动主机宽度与形状测试。

use super::tests_support::*;
use super::*;

#[test]
fn host_preserves_all_joint_widths_and_independent_parameter_periods() {
    let p = params([2, 3, 5, 7, 11]);
    let i = input(model(), p, JointPassiveOptions::default()).unwrap();
    assert_eq!(i.body, vec![1, 1, 1, 1, 1, 1, 2, 2, 2, 3, 4]);
    assert_eq!(&i.meta[4..15], &[0, 0, 0, 0, 0, 0, 1, 1, 1, 2, 3]);
    for (f, period) in i.params.fields().iter().zip([2, 3, 5, 7, 11]) {
        assert_eq!(f.batches(), period);
        assert_eq!(f.world(256), f.world(256 % period));
    }
    assert_eq!(i.param_len, 397);
    assert_eq!(i.offsets, [0, 26, 38, 78, 155]);
    let l = layout(13, 11, 257).unwrap();
    assert_eq!(l.state.total_elements(), 6168);
    assert_eq!(l.parts.total_elements(), 5654);
}
#[test]
fn host_accepts_active_rotation_springs_and_rejects_bad_shapes() {
    for (joint, poly) in [(0, false), (1, false), (0, true), (1, true)] {
        let mut p = params([1; 5]);
        if poly {
            let mut v = p.stiffness_poly.values().to_vec();
            v[2 * joint] = 0.125;
            p.stiffness_poly = ParameterBatch::new(1, 8, v).unwrap();
        } else {
            let mut v = p.stiffness.values().to_vec();
            v[joint] = 0.25;
            p.stiffness = ParameterBatch::new(1, 4, v).unwrap();
        }
        let i = input(model(), p.clone(), JointPassiveOptions::default()).unwrap();
        assert_eq!(&i.meta[37..48], &[0, 1, 2, 3, 4, 5, 0, 1, 2, 0, 0]);
        let disabled = input(
            model(),
            p,
            JointPassiveOptions {
                disable_spring: true,
                disable_damper: false,
            },
        )
        .unwrap();
        assert_eq!(disabled.meta, i.meta);
    }
    let mut p = params([1; 5]);
    p.damping = ParameterBatch::new(1, 10, vec![0.0; 10]).unwrap();
    assert!(input(model(), p, JointPassiveOptions::default()).is_err());
    assert!(layout(13, 11, 0).is_err());
    assert!(layout(13, 11, usize::MAX).is_err());
}
