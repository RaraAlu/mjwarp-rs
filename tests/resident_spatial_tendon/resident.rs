//! 常驻一致性与陈旧状态测试。

use mjwarp_rs::diagnostics::TransferError;

use super::reference_cases::reference_hashes_and_site_pulley_paths_match;
use super::*;

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn resident_spatial_tendons_match_native_across_cuda_blocks() {
    reference_hashes_and_site_pulley_paths_match();
    let v = fixture();
    let s = TransferSession::new(0).unwrap();
    let a = plan(&s, &v);
    for worlds in [1, 2, 5, 513] {
        let mut data = a.create_data(worlds).unwrap();
        for round in 0..4 {
            if round > 0 {
                let cases: Vec<_> = (0..worlds).map(|w| &v["cases"][(w + round) % 8]).collect();
                data.write_qpos(
                    &cases
                        .iter()
                        .flat_map(|c| floats(c, "qpos"))
                        .collect::<Vec<_>>(),
                )
                .unwrap();
                data.write_mocap(
                    &cases
                        .iter()
                        .flat_map(|c| floats(c, "mocap_pos"))
                        .collect::<Vec<_>>(),
                    &cases
                        .iter()
                        .flat_map(|c| floats(c, "mocap_quat"))
                        .collect::<Vec<_>>(),
                )
                .unwrap();
            }
            a.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            assert_eq!(out.spatial_tendon().rows(), a.spatial_tendon_rows());
            for world in 0..worlds {
                let id = if round == 0 { 0 } else { (world + round) % 8 };
                eprintln!(
                    "G01-spatial-tendon kind=native world={world} max_abs_error={:e}",
                    compare_native(&out, world, &v["cases"][id])
                );
            }
        }
    }
}
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn staleness_identity_mocap_and_world_writes_preserve_snapshots() {
    let v = fixture();
    let s = TransferSession::new(0).unwrap();
    let a = plan(&s, &v);
    let b = plan(&s, &v);
    let mut data = a.create_data(513).unwrap();
    let mut second = a.create_data(2).unwrap();
    assert!(matches!(
        b.update_spatial_tendons(&mut data),
        Err(TransferError::ModelMismatch)
    ));
    assert!(matches!(
        a.update_spatial_tendons(&mut data),
        Err(TransferError::StageNotReady { stage: "attached" })
    ));
    a.update_rigid(&mut data).unwrap();
    a.update_attached(&mut data).unwrap();
    assert!(matches!(
        a.update_spatial_tendons(&mut data),
        Err(TransferError::StageNotReady { stage: "com" })
    ));
    a.update_com(&mut data).unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady {
            stage: "spatial_tendon"
        })
    ));
    a.update_spatial_tendons(&mut data).unwrap();
    a.update(&mut second).unwrap();
    let saved = data.readback().unwrap();
    assert!(
        data.write_world_mocap(513, &[0.0; 3], &[1.0, 0.0, 0.0, 0.0])
            .is_err()
    );
    assert!(
        data.write_world_mocap(512, &[f32::NAN; 3], &[1.0, 0.0, 0.0, 0.0])
            .is_err()
    );
    assert!(data.write_world_qpos(512, &[]).is_err());
    assert_eq!(
        flattened(&saved, 512),
        flattened(&data.readback().unwrap(), 512)
    );
    for (world, id) in [(0, 1), (256, 4), (512, 7)] {
        write_case(&mut data, world, &v["cases"][id]);
    }
    assert!(data.readback().is_err());
    a.update(&mut data).unwrap();
    let updated = data.readback().unwrap();
    for w in 0..513 {
        compare_native(
            &updated,
            w,
            &v["cases"][match w {
                0 => 1,
                256 => 4,
                512 => 7,
                _ => 0,
            }],
        );
    }
    for com in [false, true] {
        if com {
            a.update_com(&mut data).unwrap();
        } else {
            a.update_attached(&mut data).unwrap();
        }
        assert!(matches!(
            data.readback(),
            Err(TransferError::StageNotReady {
                stage: "spatial_tendon"
            })
        ));
        a.update_spatial_tendons(&mut data).unwrap();
    }
    a.update_fixed_tendons(&mut data).unwrap();
    a.update_camlight(&mut data).unwrap();
    assert_eq!(
        flattened(&updated, 512),
        flattened(&data.readback().unwrap(), 512)
    );
    drop(b);
    drop(a);
    drop(s);
    compare_native(&second.readback().unwrap(), 1, &v["cases"][0]);
    compare_native(&saved, 512, &v["cases"][0]);
    assert_eq!(
        flattened(&updated, 512),
        flattened(&data.readback().unwrap(), 512)
    );
}
