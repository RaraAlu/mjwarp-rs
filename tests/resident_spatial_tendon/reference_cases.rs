//! 静态参考哈希与模型边界测试。

use mjwarp_rs::diagnostics::InputError;
use mjwarp_rs::model::SpatialTendonModelInput;
use sha2::{Digest, Sha256};

use super::*;

#[test]
pub(super) fn reference_hashes_and_site_pulley_paths_match() {
    let manifest: Value =
        serde_json::from_str(include_str!("../../fixtures/spatial-tendon/manifest.json")).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/spatial-tendon");
    for file in manifest["files"].as_array().unwrap() {
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(std::fs::read(root.join(file["path"].as_str().unwrap())).unwrap())
            ),
            file["sha256"].as_str().unwrap()
        );
    }
    assert_eq!(
        manifest["upstream_revision"],
        "71da24d956378a87a703b6e1442b13aec0c4ac29"
    );
    let v = fixture();
    assert_eq!(v["native_version"], 3012000);
    assert_eq!(v["seed"], 1789);
    assert_eq!(v["absolute_tolerance"].as_f64(), Some(2e-5));
    assert_eq!(v["relative_tolerance"].as_f64(), Some(2e-5));
    let m = model(&v);
    assert_eq!(
        (m.rows().ntendon(), m.rows().nnz(), m.rows().nwrap()),
        (5, 45, 19)
    );
    assert!(m.rows().row(4).unwrap().is_empty());
    assert!(m.rows().row(5).is_err());
    assert_eq!(m.fields().wrap_type.iter().filter(|&&t| t == 2).count(), 2);
    assert_eq!(v["cases"].as_array().unwrap().len(), 8);
    for c in v["cases"].as_array().unwrap() {
        for key in ["qpos", "mocap_pos", "mocap_quat"] {
            for value in c[key].as_array().unwrap() {
                let n = value.as_f64().unwrap();
                assert!((n - f64::from(n as f32)).abs() <= f64::EPSILON * n.abs());
            }
        }
    }
}
#[test]
fn rejects_invalid_lengths_wraps_branches_and_pulley_divisors() {
    let m = model(&fixture());
    for which in 0..8 {
        let mut f = m.fields().clone();
        match which {
            0 => {
                f.tendon_adr.pop();
            }
            1 => {
                f.tendon_num.pop();
            }
            2 => {
                f.wrap_type.pop();
            }
            3 => {
                f.wrap_objid.pop();
            }
            4 => {
                f.wrap_prm.pop();
            }
            5 => {
                f.ten_j_rowadr.pop();
            }
            6 => {
                f.ten_j_rownnz.pop();
            }
            _ => {
                f.ten_j_colind.pop();
            }
        }
        assert!(SpatialTendonModelInput::new(m.fixed().clone(), f).is_err());
    }
    for kind in [0, 1, 4, 5, -1, i32::MAX] {
        let mut f = m.fields().clone();
        f.wrap_type[0] = kind;
        assert!(matches!(
            SpatialTendonModelInput::new(m.fixed().clone(), f),
            Err(InputError::InvalidTopology {
                field: "wrap_type",
                ..
            })
        ));
    }
    for id in [-1, 9, i32::MAX] {
        let mut f = m.fields().clone();
        f.wrap_objid[0] = id;
        assert!(matches!(
            SpatialTendonModelInput::new(m.fixed().clone(), f),
            Err(InputError::InvalidTopology {
                field: "wrap_objid",
                ..
            })
        ));
    }
    for value in [0.0, -0.0, -1.0, f32::NAN, f32::INFINITY, f32::from_bits(1)] {
        let mut f = m.fields().clone();
        f.wrap_prm[7] = value;
        assert!(SpatialTendonModelInput::new(m.fixed().clone(), f).is_err());
    }
    for slot in [1, 4, 8] {
        let mut f = m.fields().clone();
        f.wrap_type[slot] = 2;
        f.wrap_prm[slot] = 2.0;
        assert!(SpatialTendonModelInput::new(m.fixed().clone(), f).is_err());
    }
    // Pulley object IDs do not refer to a site and never form an address.
    let mut f = m.fields().clone();
    f.wrap_objid[7] = i32::MAX;
    SpatialTendonModelInput::new(m.fixed().clone(), f).unwrap();
}
#[test]
fn rejects_malformed_sparse_rows_and_missing_nonshared_dofs() {
    let m = model(&fixture());
    for column in [-1, 1, 12, i32::MAX] {
        let mut f = m.fields().clone();
        f.ten_j_colind[0] = column;
        assert!(SpatialTendonModelInput::new(m.fixed().clone(), f).is_err());
    }
    for (csr, value) in [(true, -1), (true, 1), (false, -1), (false, 1)] {
        let mut f = m.fields().clone();
        if csr {
            f.ten_j_rowadr[0] = value;
        } else {
            f.tendon_adr[0] = value;
        }
        assert!(SpatialTendonModelInput::new(m.fixed().clone(), f).is_err());
    }
    let mut f = m.fields().clone();
    f.ten_j_colind.remove(11);
    f.ten_j_rownnz[0] -= 1;
    for start in &mut f.ten_j_rowadr[1..] {
        *start -= 1;
    }
    assert!(matches!(
        SpatialTendonModelInput::new(m.fixed().clone(), f),
        Err(InputError::InvalidTopology {
            reason: "missing_spatial_dof_column",
            ..
        })
    ));
    // Shared free-body motion cancels from the shared-root path.
    let mut f = m.fields().clone();
    f.ten_j_colind.drain(34..40);
    f.ten_j_rownnz[3] -= 6;
    f.ten_j_rowadr[4] -= 6;
    SpatialTendonModelInput::new(m.fixed().clone(), f).unwrap();
}
