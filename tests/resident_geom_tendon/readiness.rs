use super::*;

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn shrink_grow_readiness_identity_and_snapshot_lifetime_remain_checked() {
    let v = fixture();
    let s = TransferSession::new(0).unwrap();
    let p = plan(&s, &v);
    let other = plan(&s, &v);
    let mut data = p.create_data(513).unwrap();
    p.update(&mut data).unwrap();
    let saved = data.readback().unwrap();
    assert!(matches!(
        other.update_spatial_tendons(&mut data),
        Err(TransferError::ModelMismatch)
    ));
    assert!(data.write_world_qpos(512, &[f32::NAN; 6]).is_err());
    native(&data.readback().unwrap(), 512, &v["cases"][2]);
    for id in [5, 2, 11, 2] {
        write(&mut data, 512, &v["cases"][id]);
        assert!(data.readback().is_err());
        p.update(&mut data).unwrap();
        native(&data.readback().unwrap(), 512, &v["cases"][id]);
    }
    p.update_attached(&mut data).unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady {
            stage: "spatial_tendon"
        })
    ));
    p.update_spatial_tendons(&mut data).unwrap();
    p.update_com(&mut data).unwrap();
    assert!(data.readback().is_err());
    p.update_spatial_tendons(&mut data).unwrap();
    drop(p);
    drop(other);
    drop(s);
    native(&saved, 512, &v["cases"][2]);
    native(&data.readback().unwrap(), 512, &v["cases"][2]);
}
