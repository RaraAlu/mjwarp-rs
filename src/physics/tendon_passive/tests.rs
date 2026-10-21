use super::*;

fn batch(period: usize, width: usize, value: f32) -> ParameterBatch {
    ParameterBatch::new(period, width, vec![value; period * width]).unwrap()
}
fn parameters(nt: usize, periods: [usize; 5]) -> TendonPassiveParameters {
    TendonPassiveParameters {
        stiffness: batch(periods[0], nt, 0.25),
        stiffness_poly: batch(periods[1], 2 * nt, 0.0),
        damping: batch(periods[2], nt, 0.125),
        damping_poly: batch(periods[3], 2 * nt, 0.0),
        length_spring: batch(periods[4], 2 * nt, 0.0),
    }
}
#[test]
fn host_preserves_independent_parameter_periods() {
    let spec = spec((7, 3, 4, 25, 2), parameters(3, [2, 3, 5, 7, 11])).unwrap();
    assert_eq!(spec.offsets, [0, 6, 24, 39, 81]);
    assert_eq!(spec.parameter_len, 147);
    assert_eq!(
        spec.parameters.fields().map(ParameterBatch::batches),
        [2, 3, 5, 7, 11]
    );
    assert_eq!(layout(257, 7).unwrap().total_elements(), 1799);
}
#[test]
fn host_rejects_bad_parameter_widths_and_work() {
    let mut parameters = parameters(3, [1; 5]);
    parameters.length_spring = batch(1, 5, 0.0);
    assert!(spec((7, 3, 4, 25, 2), parameters).is_err());
    assert!(layout(0, 7).is_err());
    assert!(layout(usize::MAX, 7).is_err());
}
