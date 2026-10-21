//! 参考哈希、五种模式与原生结果对照。

use super::{compare, fixture, flattened, model, native_expected, write_cases};
use mjwarp_rs::{
    model::{CamLightParameters, KinematicsParameters},
    physics::KinematicsPlan,
    runtime::TransferSession,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[test]
fn reference_hashes_and_all_five_modes_match() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/camlight");
    let manifest: Value =
        serde_json::from_str(include_str!("../../fixtures/camlight/manifest.json")).unwrap();
    for file in manifest["files"].as_array().unwrap() {
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(std::fs::read(root.join(file["path"].as_str().unwrap())).unwrap())
            ),
            file["sha256"].as_str().unwrap()
        );
    }
    let v = fixture();
    let model = model(&v);
    assert_eq!(v["native_version"], 3012000);
    assert_eq!(v["seed"], 1789);
    assert_eq!(v["absolute_tolerance"].as_f64(), Some(2e-5));
    assert_eq!(v["relative_tolerance"].as_f64(), Some(2e-5));
    assert_eq!(
        (model.ncam(), model.nlight(), model.mocap().nmocap()),
        (8, 8, 1)
    );
    for modes in [&model.fields().cam_mode, &model.fields().light_mode] {
        assert_eq!(modes, &[0, 0, 1, 2, 3, 4, 3, 4]);
    }
    assert_eq!(model.fields().cam_targetbodyid[6..], [-2, -1]);
    assert_eq!(model.fields().light_targetbodyid[6..], [-2, -1]);
    assert_eq!(v["cases"].as_array().unwrap().len(), 8);
    for case in v["cases"].as_array().unwrap() {
        for f in ["qpos", "mocap_pos", "mocap_quat"] {
            for e in case[f].as_array().unwrap() {
                let e = e.as_f64().unwrap();
                assert!((e - f64::from(e as f32)).abs() <= f64::EPSILON * e.abs());
            }
        }
    }
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn resident_camlight_matches_native_across_modes_and_cuda_blocks() {
    reference_hashes_and_all_five_modes_match();
    let v = fixture();
    let session = TransferSession::new(0).unwrap();
    let plan = KinematicsPlan::with_camlight(
        &session,
        model(&v),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    assert_eq!((plan.ncam(), plan.nlight()), (8, 8));
    for worlds in [1, 2, 5, 513] {
        let mut data = plan.create_data(worlds).unwrap();
        plan.update(&mut data).unwrap();
        let out = data.readback().unwrap();
        for world in 0..worlds {
            eprintln!(
                "G01-camlight kind=native world={world} max_abs_error={:e}",
                compare(&flattened(&out, world), &native_expected(&v["cases"][0]))
            );
        }
        for round in 1..=3 {
            write_cases(&mut data, &v, round);
            plan.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            for world in 0..worlds {
                eprintln!(
                    "G01-camlight kind=native world={world} max_abs_error={:e}",
                    compare(
                        &flattened(&out, world),
                        &native_expected(&v["cases"][(world + round) % 8])
                    )
                );
            }
        }
    }
}
