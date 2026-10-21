use super::*;

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn preserves_qvel_readiness_identity_snapshot_ownership_and_lifetimes() {
    let s = TransferSession::new(0).unwrap();
    let b = base();
    let e = fixture();
    let p = plan(&s, &b, &e);
    let other = plan(&s, &b, &e);
    let mut d = p.create_data(513).unwrap();
    let mut independent = p.create_data(1).unwrap();
    assert_eq!(d.flex_edge_fields(), p.flex_edge_fields());
    assert!(matches!(
        p.update_flex_edges(&mut d),
        Err(TransferError::StageNotReady { stage: "com" })
    ));
    p.update_rigid(&mut d).unwrap();
    p.update_com(&mut d).unwrap();
    assert!(matches!(
        p.update_flex_edges(&mut d),
        Err(TransferError::StageNotReady {
            stage: "flex_positions"
        })
    ));
    for w in 0..513 {
        write(&mut d, w, &e["cases"][w % 12]);
    }
    p.update(&mut d).unwrap();
    let held = d.readback().unwrap();
    assert!(matches!(
        other.update_flex_edges(&mut d),
        Err(TransferError::ModelMismatch)
    ));
    for bad in [
        vec![],
        vec![0.0; 64],
        vec![f32::NAN; 65],
        vec![f32::INFINITY; 65],
    ] {
        assert!(d.write_world_qvel(0, &bad).is_err());
    }
    assert!(d.write_world_qvel(513, &[0.0; 65]).is_err());
    assert!(d.write_qvel(&[0.0; 65]).is_err());
    assert_eq!(bits(&held, 0), bits(&d.readback().unwrap(), 0));
    d.write_world_qvel(0, &[0.0; 65]).unwrap();
    assert!(matches!(
        d.readback(),
        Err(TransferError::StageNotReady {
            stage: "flex_edges"
        })
    ));
    p.update_flex_edges(&mut d).unwrap();
    let after = d.readback().unwrap();
    let f = after.flex_edges().unwrap().world(0).unwrap();
    assert!(f.flexedge_velocity.iter().all(|&v| v == 0.0));
    assert_eq!(
        f.flexedge_length,
        held.flex_edges().unwrap().world(0).unwrap().flexedge_length
    );
    assert_eq!(
        f.flexedge_jacobian,
        held.flex_edges()
            .unwrap()
            .world(0)
            .unwrap()
            .flexedge_jacobian
    );
    assert_ne!(bits(&after, 0), bits(&held, 0));
    assert_eq!(bits(&after, 512), bits(&held, 512));
    p.update_com(&mut d).unwrap();
    assert!(matches!(
        d.readback(),
        Err(TransferError::StageNotReady {
            stage: "spatial_tendon"
        })
    ));
    p.update_tendons(&mut d).unwrap();
    assert!(matches!(
        d.readback(),
        Err(TransferError::StageNotReady {
            stage: "flex_edges"
        })
    ));
    p.update_flex_edges(&mut d).unwrap();
    p.update_flex_positions(&mut d).unwrap();
    assert!(matches!(
        d.readback(),
        Err(TransferError::StageNotReady {
            stage: "flex_edges"
        })
    ));
    p.update_flex_edges(&mut d).unwrap();
    d.write_qvel(&vec![0.0; 513 * 65]).unwrap();
    p.update_flex_edges(&mut d).unwrap();
    p.update(&mut independent).unwrap();
    let saved = independent.readback().unwrap();
    drop(p);
    drop(other);
    drop(s);
    assert_eq!(held.flex_edges().unwrap().fields(), &edges(&e));
    assert_eq!(bits(&saved, 0), bits(&independent.readback().unwrap(), 0));
    assert_ne!(bits(&held, 0), bits(&d.readback().unwrap(), 0));
}
