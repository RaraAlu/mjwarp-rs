//! 动能主机形状、标签与位测试。

use super::checks::check_pair;
use super::tests_support::*;
use super::*;

#[test]
fn host_rejects_shapes_tags_and_both_invalid_vectors_before_pairing() {
    assert!(spec(0, 1).is_err());
    assert!(spec(47, 0).is_err());
    assert!(spec(47, 1025).is_err());
    assert!(layout(0, 1).is_err());
    assert!(layout(i32::MAX as usize + 1, 1).is_err());
    for n in [1, 7, 31, 32, 33, 65, 129, 513, 1024] {
        let l = layout(257, n).unwrap();
        assert_eq!(l.pair.total_elements(), 257 * 2 * n);
        assert_eq!(l.energy.total_elements(), 514);
        assert!(l.pair.world_elements(257).is_err());
    }
    let s = spec(47, 2).unwrap();
    let q = [-0.0, 1.0];
    let m = [0.0, 2.0];
    check_pair(&s, 2, tagged(&q, 1), tagged(&m, 1)).unwrap();
    let wrong = KineticEnergyVector {
        model_id: 48,
        ..tagged(&m, 1)
    };
    assert!(check_pair(&s, 2, wrong, tagged(&m, 1)).is_err());
    assert!(check_pair(&s, 2, tagged(&q, 1), wrong).is_err());
    assert!(check_pair(&s, 2, tagged(&q, 1), tagged(&m, 2)).is_err());
    for bad in [
        &[][..],
        &[1.0][..],
        &[1.0, 2.0, 3.0][..],
        &[f32::NAN, 1.0][..],
        &[1.0, f32::INFINITY][..],
    ] {
        assert!(check_pair(&s, 2, tagged(bad, 1), tagged(&m, 1)).is_err());
        assert!(check_pair(&s, 2, tagged(&q, 1), tagged(bad, 1)).is_err());
    }
}
#[test]
fn host_certifies_register_tail_shuffle_tree_and_signed_subnormal_bits() {
    let v = fixture();
    assert_eq!(v["block_dim"], 32);
    for c in v["cases"].as_array().unwrap() {
        let (q, m, expected) = stress(c);
        assert_eq!(warp_certificate(&q, &m).to_bits(), expected);
        if let Some(bits) = c["serial_bits"].as_str() {
            let serial = 0.5 * q.iter().zip(&m).fold(0.0f32, |s, (&q, &m)| s + q * m);
            assert_eq!(serial.to_bits(), u32::from_str_radix(bits, 16).unwrap());
            assert_ne!(serial.to_bits(), expected);
            let diagnostic = 0.5
                * q.iter()
                    .zip(&m)
                    .map(|(&q, &m)| q as f64 * m as f64)
                    .sum::<f64>();
            assert_eq!(diagnostic, c["f64_energy"].as_f64().unwrap());
        }
    }
}
