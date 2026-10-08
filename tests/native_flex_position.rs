#![cfg(all(feature = "native-model-probe", feature = "cuda-probe"))]
//! G01原生字段到GPU位置验收。
//! 原生加载不执行物理参考生成。
use mjwarp_rs::{diagnostics::NativeProbeError, io::NativeModelProbe};
const FLEX: &[u8] = include_bytes!("../fixtures/flex-position/flex-tree.mjb");
#[path = "support/flex_position_reference.rs"]
mod reference;
fn dll() -> std::path::PathBuf {
    std::env::var_os("MJWARP_MUJOCO_DLL").unwrap().into()
}
fn load(path: &std::path::Path, bytes: &[u8]) -> Result<NativeModelProbe, NativeProbeError> {
    // SAFETY: tests use a verified absolute DLL path and trusted frozen MJB only.
    unsafe { NativeModelProbe::load_trusted(path, bytes) }
}

#[test]
#[ignore = "需要可信MuJoCo DLL与NVIDIA GPU"]
fn feeds_native_flex_fields_into_resident_gpu_positions() {
    use mjwarp_rs::{
        diagnostics::InputError,
        model::{CamLightParameters, KinematicsParameters},
        physics::KinematicsPlan,
        runtime::TransferSession,
    };
    use reference::{fields, fixture, floats, tendons};
    let v = fixture();
    let p = load(&dll(), FLEX).unwrap();
    let s = p.flex_position_snapshot().unwrap();
    drop(p);
    assert_eq!(s.clone().into_fields().unwrap(), fields(&v));
    for field in 0..3 {
        let mut bad = s.clone();
        match field {
            0 => bad.info.nq += 1,
            1 => bad.info.nv += 1,
            _ => bad.info.nbody += 1,
        }
        assert!(matches!(
            bad.into_model(tendons(&v)),
            Err(NativeProbeError::Input(InputError::LengthMismatch { .. }))
        ));
    }
    let session = TransferSession::new(0).unwrap();
    let plan = KinematicsPlan::with_flex_positions(
        &session,
        s.into_model(tendons(&v)).unwrap(),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    drop(session);
    let mut count = 0;
    let mut max: f64 = 0.0;
    for nw in [1, 2, 5, 513] {
        let mut d = plan.create_data(nw).unwrap();
        for round in 0..4 {
            for w in 0..nw {
                let c = &v["cases"][(w + 3 * round) % 12];
                d.write_world_qpos(w, &floats(c, "qpos")).unwrap();
                d.write_world_mocap(w, &floats(c, "mocap_pos"), &floats(c, "mocap_quat"))
                    .unwrap();
            }
            plan.update(&mut d).unwrap();
            let out = d.readback().unwrap();
            for w in 0..nw {
                let c = &v["cases"][(w + 3 * round) % 12];
                let f = out.flex_positions().unwrap().world(w).unwrap();
                assert_eq!(f.flex_hessian_valid, &[false; 4]);
                for (actual, key) in [
                    (f.flexnode_xpos, "flexnode_xpos"),
                    (f.flexvert_xpos, "flexvert_xpos"),
                ] {
                    let expected = c[key].as_array().unwrap();
                    assert_eq!(actual.len(), expected.len());
                    for (&a, e) in actual.iter().zip(expected) {
                        let e = e.as_f64().unwrap();
                        let err = (f64::from(a) - e).abs();
                        max = max.max(err);
                        assert!(
                            a.is_finite() && err <= 2e-5 + 2e-5 * e.abs(),
                            "G01 native-flex actual={a} expected={e} error={err}"
                        );
                    }
                }
                count += 1;
            }
        }
    }
    assert_eq!(count, 2084);
    println!(
        "G01-native-flex states={count} scalars={} max_abs_error={max}",
        count * 300
    );
}
