use super::*;

#[test]
pub(super) fn reference_hashes_cover_dynamic_sphere_cylinder_and_inside_paths() {
    let manifest: Value =
        serde_json::from_str(include_str!("../../fixtures/geom-tendon/manifest.json")).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/geom-tendon");
    for f in manifest["files"].as_array().unwrap() {
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(std::fs::read(root.join(f["path"].as_str().unwrap())).unwrap())
            ),
            f["sha256"].as_str().unwrap()
        );
    }
    assert_eq!(
        manifest["upstream_revision"],
        "71da24d956378a87a703b6e1442b13aec0c4ac29"
    );
    let v = fixture();
    assert_eq!(v["native_version"], 3012000);
    assert_eq!(v["seed"], 2099);
    let m = model(&v);
    assert_eq!(
        (m.rows().ntendon(), m.rows().nnz(), m.rows().nwrap()),
        (10, 10, 37)
    );
    assert_eq!(m.geometry().unwrap().geom_size.batches(), 3);
    assert_eq!(v["cases"].as_array().unwrap().len(), 12);
    assert_ne!(v["cases"][0]["ten_wrapnum"], v["cases"][3]["ten_wrapnum"]);
    assert_eq!(v["absolute_tolerance"].as_f64(), Some(2e-5));
    assert_eq!(v["relative_tolerance"].as_f64(), Some(2e-5));
    let motion = motion_fixture();
    assert_eq!(motion["native_version"], 3012000);
    assert_eq!(motion["absolute_tolerance"].as_f64(), Some(2e-5));
    assert_eq!(motion["relative_tolerance"].as_f64(), Some(2e-5));
    assert_eq!(motion["source_world"], 2);
    assert_eq!(
        motion["cases"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["source_frame"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [310, 344, 393]
    );
}
#[test]
fn rejects_geometry_side_path_dimensions_and_missing_geom_dofs() {
    let v = fixture();
    let (b, f, g) = parts(&v);
    assert!(SpatialTendonModelInput::new(b.clone(), f.clone()).is_err());
    for id in [-1, 4, i32::MAX] {
        let mut bad = f.clone();
        bad.wrap_objid[1] = id;
        assert!(SpatialTendonModelInput::with_geometry(b.clone(), bad, g.clone()).is_err());
    }
    for side in [20.0, f32::MAX, f32::NAN] {
        let mut bad = f.clone();
        bad.wrap_prm[1] = side;
        assert!(SpatialTendonModelInput::with_geometry(b.clone(), bad, g.clone()).is_err());
    }
    for side in [-f32::MAX, -1.0, -0.49, 0.49] {
        let mut good = f.clone();
        good.wrap_prm[1] = side;
        SpatialTendonModelInput::with_geometry(b.clone(), good, g.clone()).unwrap();
    }
    for kind in [-1, 0, 5, 9] {
        let mut bad = g.clone();
        bad.geom_type[0] = kind;
        assert!(SpatialTendonModelInput::with_geometry(b.clone(), f.clone(), bad).is_err());
    }
    let mut bad = g.clone();
    bad.geom_type.pop();
    assert!(SpatialTendonModelInput::with_geometry(b.clone(), f.clone(), bad).is_err());
    let mut bad = g.clone();
    bad.geom_size = ParameterBatch::new(1, 11, vec![0.1; 11]).unwrap();
    assert!(SpatialTendonModelInput::with_geometry(b.clone(), f.clone(), bad).is_err());
    let mut bad = g.clone();
    bad.geom_size = ParameterBatch::new(1, 12, vec![-1.0; 12]).unwrap();
    assert!(SpatialTendonModelInput::with_geometry(b.clone(), f.clone(), bad).is_err());
    for slot in [0, 2] {
        let mut bad = f.clone();
        bad.wrap_type[slot] = 4;
        bad.wrap_objid[slot] = 0;
        bad.wrap_prm[slot] = -1.0;
        assert!(SpatialTendonModelInput::with_geometry(b.clone(), bad, g.clone()).is_err());
    }
    let mut bad = f.clone();
    bad.ten_j_colind.remove(2);
    bad.ten_j_rownnz[3] -= 1;
    for a in &mut bad.ten_j_rowadr[4..] {
        *a -= 1;
    }
    assert!(matches!(
        SpatialTendonModelInput::with_geometry(b, bad, g),
        Err(InputError::InvalidTopology {
            reason: "missing_spatial_dof_column",
            ..
        })
    ));
}
