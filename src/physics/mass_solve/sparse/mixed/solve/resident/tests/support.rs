//! 常驻混合测试共用样例与数值工具。

#[cfg(feature = "cuda-probe")]
use super::*;
use crate as engine;
pub(super) mod input {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/support/resident_mixed_mass.rs"
    ));
}

#[cfg(feature = "cuda-probe")]
pub(super) fn close(a: f32, b: f64, tolerance: f64) -> f64 {
    let error = (f64::from(a) - b).abs();
    assert!(a.is_finite() && error <= tolerance + tolerance * b.abs());
    error
}
#[cfg(feature = "cuda-probe")]
pub(super) fn compare_solution(
    out: &SparseMassMixedSolveOutput,
    w: usize,
    v: &serde_json::Value,
    sample: usize,
) -> (usize, f64, f64) {
    let x = out.world(w).unwrap();
    let nr = out.rhs_count();
    let rhs = input::rhs(v, sample, nr);
    let mut max = 0.0f64;
    for (i, &a) in x.solution.iter().enumerate() {
        let b = v["cases"][sample]["solution"][(i / 68 % 3) * 68 + i % 68]
            .as_f64()
            .unwrap();
        max = max.max(close(a, b, 2e-4));
        assert_eq!(x.rhs[i].to_bits(), rhs[i].to_bits());
    }
    let mut worst = 0.0f64;
    for r in 0..nr {
        for block in out.input().blocks() {
            let mut an = 0.0f64;
            let mut xn = 0.0f64;
            let mut bn = 0.0f64;
            let mut rn = 0.0f64;
            for i in block.start..block.start + block.size {
                let mut ax = 0.0f64;
                let mut row = 0.0f64;
                for j in block.start..block.start + block.size {
                    let m = v["cases"][sample]["matrix"][68 * i + j].as_f64().unwrap() as f32;
                    ax += f64::from(m) * f64::from(x.solution[68 * r + j]);
                    row += f64::from(m).abs();
                }
                an = an.max(row);
                xn = xn.max(f64::from(x.solution[68 * r + i]).abs());
                bn = bn.max(f64::from(rhs[68 * r + i]).abs());
                rn = rn.max((ax - f64::from(rhs[68 * r + i])).abs());
            }
            let error = rn / (an * xn + bn).max(1e-30);
            assert!(error <= 2e-5);
            worst = worst.max(error);
        }
    }
    (nr * 68, max, worst)
}
