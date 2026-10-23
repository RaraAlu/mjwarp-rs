//! G20@P7 逆向动力学测试：冻结上游 qfrc_inverse、原生 f64 一级判定、正逆向
//! 力平衡与校验面。

use super::*;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../fixtures/inverse/cases.json")).unwrap()
}

fn f32_list(value: &Value) -> Vec<f32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_f64().unwrap() as f32)
        .collect()
}

fn i32_list(value: &Value) -> Vec<i32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_i64().unwrap() as i32)
        .collect()
}

struct OwnedLayout {
    mulm_rowadr: Vec<i32>,
    mulm_col: Vec<i32>,
    mulm_madr: Vec<i32>,
}

impl OwnedLayout {
    fn new(fixture: &Value) -> Self {
        Self {
            mulm_rowadr: i32_list(&fixture["mass_layout"]["mulm_rowadr"]),
            mulm_col: i32_list(&fixture["mass_layout"]["mulm_col"]),
            mulm_madr: i32_list(&fixture["mass_layout"]["mulm_madr"]),
        }
    }

    fn view<'a>(&'a self, nv: usize, nnz: usize, values: &'a [f32]) -> SparseMass<'a> {
        SparseMass {
            nv,
            nnz,
            mulm_nnz: self.mulm_col.len(),
            mulm_rowadr: &self.mulm_rowadr,
            mulm_col: &self.mulm_col,
            mulm_madr: &self.mulm_madr,
            values,
        }
    }
}

fn assert_within(label: &str, product: &[f32], expected: &[f32], tolerance: f64) -> f64 {
    assert_eq!(product.len(), expected.len(), "{label} length");
    let mut max_abs = 0.0_f64;
    let mut bitwise = 0;
    for (left, right) in product.iter().zip(expected.iter()) {
        if left.to_bits() == right.to_bits() {
            bitwise += 1;
        }
        max_abs = max_abs.max((*left as f64 - *right as f64).abs());
    }
    assert!(
        max_abs <= tolerance,
        "{label} max abs {max_abs:.3e} > {tolerance:.3e} (bitwise {bitwise}/{})",
        product.len()
    );
    max_abs
}

#[test]
fn inverse_matches_frozen_upstream_and_native_reference() {
    let fixture = fixture();
    assert_eq!(
        fixture["task_id"].as_str().unwrap(),
        "2026-10-11-roadmap-g20-p7"
    );
    let nv = fixture["mass_layout"]["nv"].as_u64().unwrap() as usize;
    let nnz = fixture["mass_layout"]["nnz"].as_u64().unwrap() as usize;
    let layout = OwnedLayout::new(&fixture);
    let upstream_tolerance = fixture["tolerance"]["upstream_abs"].as_f64().unwrap();
    let native_tolerance = fixture["tolerance"]["native_abs"].as_f64().unwrap();
    let balance_tolerance = fixture["tolerance"]["force_balance_abs"].as_f64().unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 3);
    let mut upstream_max = 0.0_f64;
    let mut native_max = 0.0_f64;
    let mut balance_max = 0.0_f64;
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let values = f32_list(&case["mass"]);
        let mass_view = layout.view(nv, nnz, &values);
        let qacc = f32_list(&case["qacc"]);
        let bias = f32_list(&case["qfrc_bias"]);
        let passive = f32_list(&case["qfrc_passive"]);
        let constraint = f32_list(&case["qfrc_constraint"]);
        let product = inverse_dynamics(&mass_view, 1, &qacc, &bias, &passive, &constraint).unwrap();
        upstream_max = upstream_max.max(assert_within(
            &format!("{name} vs upstream"),
            &product,
            &f32_list(&case["qfrc_inverse"]),
            upstream_tolerance,
        ));
        native_max = native_max.max(assert_within(
            &format!("{name} vs native"),
            &product,
            &f32_list(&case["native_qfrc_inverse"]),
            native_tolerance,
        ));
        let balance = product
            .iter()
            .fold(0.0_f64, |acc, value| acc.max((*value as f64).abs()));
        balance_max = balance_max.max(balance);
        assert!(
            balance <= balance_tolerance,
            "{name} forward-inverse balance {balance:.3e} > {balance_tolerance:.3e}"
        );
        let ma = mul_m(&mass_view, 1, &qacc).unwrap();
        assert_eq!(ma.len(), nv);
    }
    println!(
        "inverse upstream_max={upstream_max:.3e} native_max={native_max:.3e} balance_max={balance_max:.3e}"
    );
}

#[test]
fn rejects_invalid_mass_layout() {
    let fixture = fixture();
    let nv = fixture["mass_layout"]["nv"].as_u64().unwrap() as usize;
    let nnz = fixture["mass_layout"]["nnz"].as_u64().unwrap() as usize;
    let layout = OwnedLayout::new(&fixture);
    let values = vec![0.0_f32; nnz];
    let view = layout.view(nv, nnz, &values);
    let qacc = vec![0.0_f32; nv];
    let error = mul_m(&view, 1, &qacc[1..]).unwrap_err();
    assert!(matches!(
        error,
        InverseError::InvalidLength { field: "qacc", .. }
    ));
    let error = mul_m(&view, 2, &qacc).unwrap_err();
    assert!(matches!(
        error,
        InverseError::InvalidLength { field: "M", .. }
    ));
    let error = inverse_dynamics(&view, 1, &qacc, &qacc, &qacc[1..], &qacc).unwrap_err();
    assert!(matches!(
        error,
        InverseError::InvalidLength {
            field: "qfrc_passive",
            ..
        }
    ));
    let mut broken = OwnedLayout::new(&fixture);
    broken.mulm_col[0] = 99;
    let broken_view = broken.view(nv, nnz, &values);
    let error = mul_m(&broken_view, 1, &qacc).unwrap_err();
    assert!(matches!(
        error,
        InverseError::InvalidIndex {
            field: "M_mulm_col",
            ..
        }
    ));
}
