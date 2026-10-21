use super::*;

#[cfg(windows)]
pub(super) fn stats(actual: &[f32], expected: &[f32]) -> Value {
    assert_eq!(actual.len(), expected.len());
    let mut changed = 0usize;
    let mut max = 0.0f64;
    let mut over = 0usize;
    for (&a, &e) in actual.iter().zip(expected) {
        assert!(a.is_finite() && e.is_finite());
        changed += usize::from(a.to_bits() != e.to_bits());
        let error = (f64::from(a) - f64::from(e)).abs();
        max = max.max(error);
        over += usize::from(error > 2e-5 + 2e-5 * f64::from(e).abs());
    }
    json!({"comparisons":actual.len(),"bit_differences":changed,"max_abs_error":max,"outside_original_tolerance":over})
}
