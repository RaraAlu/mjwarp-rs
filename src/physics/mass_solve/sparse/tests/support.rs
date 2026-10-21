//! 稀疏质量测试共用断言与工具。

#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) use crate::model::SparseMassFields;

#[cfg(feature = "cuda-probe")]
pub(super) fn assert_copy_matches_matrix(data: &SparseMassFactorData) {
    let mut matrix = vec![0.0; data.matrix.len()];
    let mut output = vec![0.0; data.output.len()];
    data.matrix.read_range_into(0, &mut matrix).unwrap();
    data.output.read_range_into(0, &mut output).unwrap();
    for guard in output[..4].iter().chain(&output[output.len() - 4..]) {
        assert_eq!(guard.to_bits(), (-131072.0f32).to_bits());
    }
    let nc = data.input.nc();
    let stride = nc + data.input.nv();
    for world in 0..data.worlds() {
        let matrix_start = 4 + world * nc;
        let output_start = 4 + world * stride;
        for (actual, expected) in output[output_start..output_start + nc]
            .iter()
            .zip(&matrix[matrix_start..matrix_start + nc])
        {
            assert_eq!(actual.to_bits(), expected.to_bits());
        }
        for value in &output[output_start + nc..output_start + stride] {
            assert_eq!(value.to_bits(), 0.0f32.to_bits());
        }
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn assert_close(actual: &[f32], expected: &serde_json::Value) {
    let expected = expected.as_array().unwrap();
    assert_eq!(actual.len(), expected.len());
    for (&actual, expected) in actual.iter().zip(expected) {
        let expected = expected.as_f64().unwrap();
        let error = (f64::from(actual) - expected).abs();
        assert!(
            error <= 2e-4 + 2e-4 * expected.abs(),
            "actual={actual} expected={expected} error={error}"
        );
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn assert_reconstruction(input: &SparseMassInput, factor: &[f32], matrix: &[f32]) {
    let n = input.nv();
    let fields = input.fields();
    let mut lower = vec![0.0f64; n * n];
    let mut source = vec![0.0f64; n * n];
    let mut diagonal = vec![0.0f64; n];
    for row in 0..n {
        for item in 0..fields.m_rownnz[row] as usize {
            let address = fields.m_rowadr[row] as usize + item;
            let column = fields.m_colind[address] as usize;
            source[row * n + column] = f64::from(matrix[address]);
            source[column * n + row] = f64::from(matrix[address]);
            if column == row {
                diagonal[row] = f64::from(factor[address]);
                lower[row * n + row] = 1.0;
            } else {
                lower[row * n + column] = f64::from(factor[address]);
            }
        }
    }
    let norm = source
        .chunks(n)
        .map(|row| row.iter().map(|value| value.abs()).sum::<f64>())
        .fold(0.0, f64::max);
    let mut error = 0.0f64;
    for row in 0..n {
        let mut row_error = 0.0;
        for column in 0..n {
            let reconstructed = (0..n)
                .map(|k| lower[k * n + row] * diagonal[k] * lower[k * n + column])
                .sum::<f64>();
            row_error += (reconstructed - source[row * n + column]).abs();
        }
        error = error.max(row_error);
    }
    let relative = error / norm;
    assert!(relative <= 2e-6, "reconstruction={relative}");
}

#[cfg(feature = "cuda-probe")]
pub(super) fn assert_factor_matches_native(data: &SparseMassFactorData, model: &serde_json::Value) {
    let output = data.readback().unwrap();
    let cases = model["cases"].as_array().unwrap();
    for world in 0..output.worlds() {
        let case = &cases[world % cases.len()];
        let view = output.world(world).unwrap();
        assert_close(view.sparse_factors, &case["factor"]);
        assert_close(view.diagonal_inverse, &case["diagonal_inverse"]);
        let matrix = case["matrix"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_f64().unwrap() as f32)
            .collect::<Vec<_>>();
        assert_reconstruction(output.input(), view.sparse_factors, &matrix);
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn assert_diag_matches_factors_bitwise(
    data: &SparseMassFactorData,
    before: &[f32],
    after: &[f32],
) {
    assert_eq!(before.len(), after.len());
    for index in (0..4).chain(after.len() - 4..after.len()) {
        assert_eq!(before[index].to_bits(), after[index].to_bits());
    }
    let fields = data.input.fields();
    let nc = data.input.nc();
    let nv = data.input.nv();
    let stride = nc + nv;
    for world in 0..data.worlds() {
        let start = 4 + world * stride;
        for index in 0..nc {
            assert_eq!(
                before[start + index].to_bits(),
                after[start + index].to_bits()
            );
        }
        for row in 0..nv {
            let address = fields.m_rowadr[row] as usize;
            let count = fields.m_rownnz[row] as usize;
            let expected = 1.0f32 / before[start + address + count - 1];
            assert_eq!(expected.to_bits(), after[start + nc + row].to_bits());
        }
    }
}
