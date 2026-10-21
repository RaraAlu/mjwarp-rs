use super::*;

fn static_cylinder() -> SpatialTendonModelInput {
    let k = KinematicModelInput::new(
        0,
        KinematicFields {
            body_parentid: vec![0],
            body_jntadr: vec![-1],
            body_jntnum: vec![0],
            body_pos: vec![0.0; 3],
            body_quat: vec![1.0, 0.0, 0.0, 0.0],
            ..Default::default()
        },
    )
    .unwrap();
    let i = InertialModelInput::new(
        k,
        InertialFields {
            body_ipos: vec![0.0; 3],
            body_iquat: vec![1.0, 0.0, 0.0, 0.0],
            body_mass: vec![0.0],
            body_inertia: vec![0.0; 3],
            ..Default::default()
        },
    )
    .unwrap();
    let a = AttachedModelInput::new(
        i,
        AttachedFields {
            geom_bodyid: vec![0],
            geom_pos: vec![0.0; 3],
            geom_quat: vec![1.0, 0.0, 0.0, 0.0],
            site_bodyid: vec![0; 3],
            site_pos: vec![-1.0, 0.0, -0.5, 1.0, 0.0, 0.5, 0.0, 1.0, 0.0],
            site_quat: [1.0, 0.0, 0.0, 0.0].repeat(3),
        },
    )
    .unwrap();
    let c = CamLightModelInput::new(
        MocapModelInput::new(a, vec![-1]).unwrap(),
        CamLightFields::default(),
    )
    .unwrap();
    let b = FixedTendonModelInput::new(c, FixedTendonFields::default()).unwrap();
    SpatialTendonModelInput::with_geometry(
        b,
        SpatialTendonFields {
            tendon_adr: vec![0],
            tendon_num: vec![3],
            wrap_type: vec![3, 5, 3],
            wrap_objid: vec![0, 0, 1],
            wrap_prm: vec![0.0, 2.0, 0.0],
            ten_j_rowadr: vec![0],
            ten_j_rownnz: vec![0],
            ten_j_colind: vec![],
        },
        SpatialTendonGeometry {
            geom_type: vec![5],
            geom_size: ParameterBatch::new(
                3,
                3,
                vec![0.0, 0.8, 0.0, 0.25, 0.8, 0.0, 0.3, 0.8, 0.0],
            )
            .unwrap(),
        },
    )
    .unwrap()
}
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn zero_dof_cylinder_helix_matches_analytic_independent_parameter_periods() {
    let s = TransferSession::new(0).unwrap();
    let mut params = KinematicsParameters::default();
    params.set(
        F::GeomPos,
        ParameterBatch::new(2, 3, vec![0.0, 0.0, 0.125, 0.0, 0.0, -0.125]).unwrap(),
    );
    params.set(
        F::GeomQuat,
        ParameterBatch::new(
            5,
            4,
            (0..5)
                .flat_map(|i| {
                    let a = i as f32 * 0.12;
                    [a.cos(), 0.0, 0.0, a.sin()]
                })
                .collect(),
        )
        .unwrap(),
    );
    params.set(
        F::SitePos,
        ParameterBatch::new(
            7,
            9,
            (0..7)
                .flat_map(|i| {
                    let r = 1.0 + i as f32 * 0.1;
                    [-r, 0.0, -0.5, r, 0.0, 0.5, 0.0, 1.0, 0.0]
                })
                .collect(),
        )
        .unwrap(),
    );
    let p = KinematicsPlan::with_spatial_tendons(
        &s,
        static_cylinder(),
        params,
        CamLightParameters::default(),
    )
    .unwrap();
    for worlds in [1, 2, 5, 513] {
        let mut data = p.create_data(worlds).unwrap();
        for _ in 0..2 {
            p.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            for w in 0..worlds {
                let r = f64::from([0.0f32, 0.25, 0.3][w % 3]);
                let end = f64::from(1.0 + (w % 7) as f32 * 0.1);
                let a = out.spatial_tendon().world(w).unwrap();
                assert!(a.ten_jacobian.is_empty());
                let length = if r == 0.0 {
                    (4.0 * end * end + 1.0).sqrt()
                } else {
                    let flat = 2.0 * (end * end - r * r).sqrt()
                        + r * (std::f64::consts::PI - 2.0 * (r / end).acos());
                    (flat * flat + 1.0).sqrt()
                };
                let mut points = vec![0.0; 18];
                points[..3].copy_from_slice(&[-end, 0.0, -0.5]);
                if r == 0.0 {
                    points[3..6].copy_from_slice(&[end, 0.0, 0.5]);
                    assert_eq!(a.ten_wrapnum, &[2]);
                    assert_eq!(a.wrap_obj, &[-1, -1, 0, 0, 0, 0]);
                } else {
                    let l = (end * end - r * r).sqrt();
                    let flat = 2.0 * l + r * (std::f64::consts::PI - 2.0 * (r / end).acos());
                    let z = -0.5 + l / flat;
                    let x = r * r / end;
                    let y = r * (1.0 - (r / end).powi(2)).sqrt();
                    points[3..9].copy_from_slice(&[-x, y, z, x, y, -z]);
                    points[9..12].copy_from_slice(&[end, 0.0, 0.5]);
                    assert_eq!(a.ten_wrapnum, &[4]);
                    assert_eq!(a.wrap_obj, &[-1, 0, 0, -1, 0, 0]);
                }
                assert_eq!(a.ten_wrapadr, &[0]);
                let error = compare(
                    &[a.ten_length, a.wrap_xpos].concat(),
                    &[vec![length], points].concat(),
                );
                eprintln!("G01-geom-tendon kind=analytic world={w} max_abs_error={error:e}");
            }
        }
    }
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn sphere_parallel_tangent_contained_coincident_and_tiny_radius_keep_frozen_branches() {
    let m = static_cylinder();
    let mut f = m.fields().clone();
    f.wrap_type[1] = 4;
    let geometry = SpatialTendonGeometry {
        geom_type: vec![2],
        geom_size: ParameterBatch::new(
            7,
            3,
            (0..7)
                .flat_map(|i| [if i == 5 { 1e-16 } else { 0.25 }, 0.0, 0.0])
                .collect(),
        )
        .unwrap(),
    };
    let rows = [
        [-1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        [-1.0, 0.25, 0.0, 1.0, 0.25, 0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        [1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        [0.1, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        [-1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        [1.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    ];
    let mut params = KinematicsParameters::default();
    params.set(
        F::SitePos,
        ParameterBatch::new(7, 9, rows.concat()).unwrap(),
    );
    let s = TransferSession::new(0).unwrap();
    let p = KinematicsPlan::with_spatial_tendons(
        &s,
        SpatialTendonModelInput::with_geometry(m.fixed().clone(), f, geometry).unwrap(),
        params,
        CamLightParameters::default(),
    )
    .unwrap();
    let mut data = p.create_data(513).unwrap();
    p.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    let circle =
        2.0 * (1.0f64 - 0.25 * 0.25).sqrt() + 0.25 * (std::f64::consts::PI - 2.0 * 0.25f64.acos());
    for w in 0..513 {
        let i = w % 7;
        let row = out.spatial_tendon().world(w).unwrap();
        // 冻结f32叉积选中整圆。
        // 不用理想切线替换此分支。
        let tangent = 2.0 + 0.5 * std::f64::consts::PI;
        compare(
            row.ten_length,
            &[[circle, tangent, 1.0, 0.0, 0.9, 2.0, 2.5][i]],
        );
        let active = matches!(i, 0 | 1 | 6);
        assert_eq!(row.ten_wrapnum, &[if active { 4 } else { 2 }]);
        assert_eq!(
            row.wrap_obj,
            if active {
                &[-1, 0, 0, -1, 0, 0]
            } else {
                &[-1, -1, 0, 0, 0, 0]
            }
        );
        if active {
            for point in [&row.wrap_xpos[3..6], &row.wrap_xpos[6..9]] {
                let squared: f64 = point.iter().map(|&v| f64::from(v).powi(2)).sum();
                assert!((squared.sqrt() - 0.25).abs() < 2e-5);
            }
        }
        assert!(row.ten_jacobian.is_empty());
    }
}
