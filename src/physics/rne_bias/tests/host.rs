//! 偏置力布局与空视图测试。

use super::plan::check_gravity;
use super::*;

#[test]
fn rejects_bias_layout_dimensions_and_overflow() {
    for (w, b, v) in [
        (0, 1, 0),
        (1, 0, 0),
        (u32::MAX as usize, 1, 0),
        (1, usize::MAX, 0),
        (1, 1, usize::MAX),
    ] {
        assert!(RneBiasLayout::new(w, b, v).is_err());
    }
    let layout = RneBiasLayout::new(513, 1, 0).unwrap();
    assert_eq!(layout.output.total_bytes(), 513 * 12 * size_of::<f32>());
}

#[test]
fn rejects_nonfinite_gravity_and_checks_empty_bias_views() {
    for v in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for i in 0..3 {
            let mut g = [0.0; 3];
            g[i] = v;
            assert!(check_gravity(&g).is_err());
        }
    }
    assert!(check_gravity(&[1.0, -2.0, 3.0]).is_ok());
    let out = RneBiasSnapshot {
        layout: RneBiasLayout::new(2, 1, 0).unwrap(),
        values: vec![],
    };
    assert_eq!(out.worlds(), 2);
    assert!(out.world(1).unwrap().qfrc_bias.is_empty());
    assert!(out.world(2).is_err());
}
