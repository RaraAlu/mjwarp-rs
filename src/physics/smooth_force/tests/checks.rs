//! 平滑力布局与边界测试。

use super::data::check_force;
use super::tests_support::*;
use super::*;

#[test]
fn fixed_scalar_certificate_distinguishes_step_rounding_and_signed_zeros() {
    let v = reference();
    assert_eq!(v["rows"].as_array().unwrap().len(), 18);
    for i in 0..18 {
        let (x, bits) = row(&v, i);
        let first = x[0] - x[1];
        let second = first + x[2];
        let third = second + x[3];
        assert_eq!(third.to_bits(), bits, "case{i}");
    }
    let (x, bits) = row(&v, 2);
    assert_eq!(
        f64::from(x[0]) - f64::from(x[1]) + f64::from(x[2]) + f64::from(x[3]),
        0.0
    );
    assert_eq!(bits, 1.0f32.to_bits());
    assert_eq!(row(&v, 7).1, (-0.0f32).to_bits());
}
#[test]
fn bounds_layout_and_force_lengths_without_gpu_allocation() {
    for (w, nb, n, nt) in [
        (0, 1, 1, 0),
        (1, 0, 1, 0),
        (1, 1, 0, 0),
        (usize::MAX, 1, 1, 1),
        (1, 1, 1, usize::MAX),
        (1, usize::MAX, 1, 0),
    ] {
        assert!(layout(w, nb, n, nt).is_err());
    }
    let l = layout(257, 273, 513, 129).unwrap();
    assert_eq!(l.force_len, 4 * 257 * 513);
    assert_eq!(l.meta_len, 273 + 513 + 257 * 129 + 8);
    assert!(layout(2, 2, 2, 0).is_ok());
    for f in [
        SmoothForceField::Passive,
        SmoothForceField::Bias,
        SmoothForceField::Actuator,
        SmoothForceField::Applied,
    ] {
        assert!(check_force(&[], 1, f).is_err());
        assert!(check_force(&[f32::NAN], 1, f).is_err());
    }
    for (sleep, island, enabled) in [
        (false, false, false),
        (false, true, false),
        (true, false, true),
        (true, true, false),
    ] {
        assert_eq!(
            SmoothForceOptions {
                sleep_enabled: sleep,
                island_disabled: island
            }
            .enable_sleep(),
            enabled
        );
    }
}
