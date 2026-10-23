//! G21@P7 平滑速度导数测试：冻结上游解析输出、解析 vs 有限差分（上游 f32 与
//! 原生 f64）、以及输入校验面。

use super::*;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../fixtures/deriv/cases.json")).unwrap()
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

fn vec2_list(value: &Value) -> Vec<[f32; 2]> {
    let flat = f32_list(value);
    let (rows, _) = flat.as_chunks::<2>();
    rows.to_vec()
}

fn entries(value: &Value) -> Vec<(usize, usize)> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            let row = i32_list(item);
            (row[0] as usize, row[1] as usize)
        })
        .collect()
}

struct OwnedLayout {
    fullm_i: Vec<i32>,
    fullm_j: Vec<i32>,
    elemid: Vec<i32>,
    damping: Vec<f32>,
    damping_poly: Vec<[f32; 2]>,
    timestep: f32,
    nv: usize,
    nnz: usize,
    damping_batches: usize,
    damping_poly_batches: usize,
    damper_enabled: bool,
}

impl OwnedLayout {
    fn from_json(value: &Value) -> Self {
        Self {
            fullm_i: i32_list(&value["fullm_i"]),
            fullm_j: i32_list(&value["fullm_j"]),
            elemid: i32_list(&value["elemid"]),
            damping: f32_list(&value["damping"]),
            damping_poly: vec2_list(&value["damping_poly"]),
            timestep: value["timestep"].as_f64().unwrap() as f32,
            nv: value["nv"].as_u64().unwrap() as usize,
            nnz: value["nnz"].as_u64().unwrap() as usize,
            damping_batches: value["damping_batches"].as_u64().unwrap() as usize,
            damping_poly_batches: value["damping_poly_batches"].as_u64().unwrap() as usize,
            damper_enabled: value["damper_enabled"].as_bool().unwrap(),
        }
    }

    fn view(&self) -> DerivativeLayout<'_> {
        DerivativeLayout {
            timestep: self.timestep,
            nv: self.nv,
            nnz: self.nnz,
            fullm_i: &self.fullm_i,
            fullm_j: &self.fullm_j,
            elemid: &self.elemid,
            damping_batches: self.damping_batches,
            damping: &self.damping,
            damping_poly_batches: self.damping_poly_batches,
            damping_poly: &self.damping_poly,
            damper_enabled: self.damper_enabled,
        }
    }
}

fn assert_diagonal_fd(
    label: &str,
    mass: &[f32],
    product: &[f32],
    entries: &[(usize, usize)],
    differences: &[f32],
    timestep: f32,
    tolerance: f64,
) {
    let mut max_abs = 0.0_f64;
    for (madr, dof) in entries {
        let derived = (mass[*madr] as f64 - product[*madr] as f64)
            - timestep as f64 * differences[*dof] as f64;
        max_abs = max_abs.max(derived.abs());
    }
    assert!(
        max_abs <= tolerance,
        "{label}: analytic vs finite difference max abs {max_abs:.3e} > {tolerance:.3e}"
    );
}

#[test]
fn derivative_matches_frozen_upstream_reference() {
    let fixture = fixture();
    assert_eq!(
        fixture["task_id"].as_str().unwrap(),
        "2026-10-11-roadmap-g21-p7"
    );
    let upstream_tolerance = fixture["tolerance"]["upstream_abs"].as_f64().unwrap();
    let fd_tolerance = fixture["tolerance"]["fd_abs"].as_f64().unwrap();

    let poly_layout = OwnedLayout::from_json(&fixture["poly"]["layout"]);
    let poly_entries = entries(&fixture["poly"]["entries"]);
    let mut poly_max = 0.0_f64;
    let mut poly_bitwise = 0;
    let mut poly_total = 0;
    for case in fixture["poly"]["cases"].as_array().unwrap() {
        let mass = f32_list(&case["mass"]);
        let qvel = f32_list(&case["qvel"]);
        let expected = f32_list(&case["qfrc_deriv"]);
        let product = deriv_smooth_vel(&poly_layout.view(), 1, &mass, &qvel).unwrap();
        assert_eq!(product.len(), expected.len());
        for (left, right) in product.iter().zip(expected.iter()) {
            if left.to_bits() == right.to_bits() {
                poly_bitwise += 1;
            }
            poly_max = poly_max.max((*left as f64 - *right as f64).abs());
        }
        poly_total += product.len();
        assert!(
            poly_max <= upstream_tolerance,
            "poly case {} upstream delta {poly_max:.3e}",
            case["name"]
        );
        assert_diagonal_fd(
            &format!("poly {}", case["name"]),
            &mass,
            &product,
            &poly_entries,
            &f32_list(&case["passive_fd"]),
            poly_layout.timestep,
            fd_tolerance,
        );
    }

    let linear_layout = OwnedLayout::from_json(&fixture["linear"]["layout"]);
    let linear_entries = entries(&fixture["linear"]["entries"]);
    let mut native_max = 0.0_f64;
    for case in fixture["linear"]["cases"].as_array().unwrap() {
        let mass = f32_list(&case["mass"]);
        let qvel = f32_list(&case["qvel"]);
        let expected = f32_list(&case["qfrc_deriv"]);
        let product = deriv_smooth_vel(&linear_layout.view(), 1, &mass, &qvel).unwrap();
        for (left, right) in product.iter().zip(expected.iter()) {
            native_max = native_max.max((*left as f64 - *right as f64).abs());
        }
        assert!(
            native_max <= upstream_tolerance,
            "linear case {} upstream delta {native_max:.3e}",
            case["name"]
        );
        assert_diagonal_fd(
            &format!("linear {}", case["name"]),
            &mass,
            &product,
            &linear_entries,
            &f32_list(&case["native_passive_fd"]),
            linear_layout.timestep,
            fd_tolerance,
        );
    }
    println!(
        "deriv poly_bitwise={poly_bitwise}/{poly_total} poly_max={poly_max:.3e} linear_max={native_max:.3e}"
    );
}

#[test]
fn rejects_invalid_layout() {
    let fixture = fixture();
    let layout = OwnedLayout::from_json(&fixture["poly"]["layout"]);
    let case = &fixture["poly"]["cases"][0];
    let mass = f32_list(&case["mass"]);
    let qvel = f32_list(&case["qvel"]);
    let mut broken = OwnedLayout::from_json(&fixture["poly"]["layout"]);
    broken.elemid.pop();
    let error = deriv_smooth_vel(&broken.view(), 1, &mass, &qvel).unwrap_err();
    assert!(matches!(
        error,
        DerivativeError::InvalidLength {
            field: "elemid",
            ..
        }
    ));
    let error = deriv_smooth_vel(&layout.view(), 1, &mass, &qvel[1..]).unwrap_err();
    assert!(matches!(
        error,
        DerivativeError::InvalidLength { field: "qvel", .. }
    ));
    let mut broken = OwnedLayout::from_json(&fixture["poly"]["layout"]);
    broken.elemid[0] = 99;
    let error = deriv_smooth_vel(&broken.view(), 1, &mass, &qvel).unwrap_err();
    assert!(matches!(
        error,
        DerivativeError::InvalidIndex {
            field: "M_elemid",
            ..
        }
    ));
    let mut broken = OwnedLayout::from_json(&fixture["poly"]["layout"]);
    broken.damping_batches = 0;
    let error = deriv_smooth_vel(&broken.view(), 1, &mass, &qvel).unwrap_err();
    assert!(matches!(
        error,
        DerivativeError::InvalidBatch {
            field: "dof_damping",
            ..
        }
    ));
}
