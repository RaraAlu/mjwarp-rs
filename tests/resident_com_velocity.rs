#![cfg(feature = "cuda-probe")]
//! G02空间速度子集的静态参考与状态边界。
//! 产品测试不运行参考生成器或原生物理。

use mjwarp_rs::{
    diagnostics::{InputError, TransferError},
    model::{AttachedFields, AttachedModelInput},
    physics::{ComVelocityPlan, ComVelocitySnapshot},
    runtime::TransferSession,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[path = "support/rigid_reference.rs"]
mod reference;
use reference::floats;

const FIXTURES: [&str; 5] = [
    include_str!("../fixtures/com-velocity/mixed-joints.json"),
    include_str!("../fixtures/com-velocity/inertial-tree.json"),
    include_str!("../fixtures/com-velocity/rotated-tree.json"),
    include_str!("../fixtures/com-velocity/zero-dof.json"),
    include_str!("../fixtures/com-velocity/massless-tree.json"),
];
const ABS: f64 = 2e-5;
const REL: f64 = 2e-5;

fn fixture(index: usize) -> Value {
    serde_json::from_str(FIXTURES[index]).unwrap()
}

fn model(value: &Value) -> AttachedModelInput {
    AttachedModelInput::new(reference::model(value), AttachedFields::default()).unwrap()
}

fn compare(snapshot: &ComVelocitySnapshot, world: usize, case: &Value) -> (usize, f64) {
    let values = snapshot.world(world).unwrap();
    let mut maximum = 0.0f64;
    let mut count = 0;
    for (name, actual) in [("cvel", values.cvel), ("cdof_dot", values.cdof_dot)] {
        let expected = case[name].as_array().unwrap();
        assert_eq!(actual.len(), expected.len());
        for (index, (&a, e)) in actual.iter().zip(expected).enumerate() {
            let e = e.as_f64().unwrap();
            let error = (f64::from(a) - e).abs();
            maximum = maximum.max(error);
            count += 1;
            assert!(
                a.is_finite() && error <= ABS + REL * e.abs(),
                "G02-velocity {name} world={world} index={index} actual={a} expected={e} error={error}"
            );
        }
    }
    (count, maximum)
}

fn assert_same(actual: &ComVelocitySnapshot, expected: &ComVelocitySnapshot) {
    assert_eq!(actual.worlds(), expected.worlds());
    for world in 0..actual.worlds() {
        let a = actual.world(world).unwrap();
        let e = expected.world(world).unwrap();
        assert_eq!(a.cvel, e.cvel);
        assert_eq!(a.cdof_dot, e.cdof_dot);
    }
}

#[test]
fn reference_hashes_fix_velocity_inputs_joint_coverage_and_tolerances() {
    let manifest: Value =
        serde_json::from_str(include_str!("../fixtures/com-velocity/manifest.json")).unwrap();
    assert_eq!(manifest["files"].as_array().unwrap().len(), 11);
    assert_eq!(manifest["candidate_version"], "3.12.0");
    assert_eq!(
        manifest["frozen_warp_revision"],
        "71da24d956378a87a703b6e1442b13aec0c4ac29"
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for entry in manifest["files"].as_array().unwrap() {
        let bytes = std::fs::read(root.join(entry["path"].as_str().unwrap())).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            entry["sha256"].as_str().unwrap()
        );
    }
    let mut types = [false; 4];
    for text in FIXTURES {
        let value: Value = serde_json::from_str(text).unwrap();
        let m = model(&value);
        assert_eq!(value["native_version"], 3012000);
        assert_eq!(value["seed"], 1789);
        assert_eq!(value["absolute_tolerance"].as_f64(), Some(ABS));
        assert_eq!(value["relative_tolerance"].as_f64(), Some(REL));
        for &kind in &m.rigid().kinematics().fields().jnt_type {
            types[kind as usize] = true;
        }
        assert_eq!(value["cases"].as_array().unwrap().len(), 8);
        for case in value["cases"].as_array().unwrap() {
            let k = m.rigid().kinematics();
            for (field, count) in [
                ("qpos", k.nq()),
                ("qvel", k.nv()),
                ("cvel", 6 * k.nbody()),
                ("cdof_dot", 6 * k.nv()),
            ] {
                let values = case[field].as_array().unwrap();
                assert_eq!(values.len(), count);
                assert!(values.iter().all(|v| v.as_f64().unwrap().is_finite()));
            }
        }
    }
    assert_eq!(types, [true; 4]);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn spatial_velocities_match_native_references_for_all_joint_types() {
    let session = TransferSession::new(0).unwrap();
    let mut count = 0;
    let mut maximum = 0.0f64;
    for text in FIXTURES {
        let value: Value = serde_json::from_str(text).unwrap();
        let plan = ComVelocityPlan::new(&session, model(&value)).unwrap();
        let cases = value["cases"].as_array().unwrap();
        let mut data = plan.create_data(cases.len()).unwrap();
        assert!(matches!(
            data.readback(),
            Err(TransferError::StageNotReady { .. })
        ));
        let qpos: Vec<_> = cases.iter().flat_map(|c| floats(c, "qpos")).collect();
        let qvel: Vec<_> = cases.iter().flat_map(|c| floats(c, "qvel")).collect();
        data.write_qpos(&qpos).unwrap();
        data.write_qvel(&qvel).unwrap();
        plan.update(&mut data).unwrap();
        let snapshot = data.readback().unwrap();
        for (world, case) in cases.iter().enumerate() {
            let (n, error) = compare(&snapshot, world, case);
            count += n;
            maximum = maximum.max(error);
        }
        assert_eq!(
            qpos,
            cases
                .iter()
                .flat_map(|c| floats(c, "qpos"))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            qvel,
            cases
                .iter()
                .flat_map(|c| floats(c, "qvel"))
                .collect::<Vec<_>>()
        );
    }
    eprintln!("G02-velocity states=40 scalars={count} max_abs_error={maximum:e}");
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn velocity_world_writes_cross_blocks_and_keep_snapshots_owned() {
    let value = fixture(0);
    let cases = value["cases"].as_array().unwrap();
    let session = TransferSession::new(0).unwrap();
    let plan = ComVelocityPlan::new(&session, model(&value)).unwrap();
    let mut data = plan.create_data(513).unwrap();
    data.write_qpos(
        &(0..513)
            .flat_map(|w| floats(&cases[w % 8], "qpos"))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    data.write_qvel(
        &(0..513)
            .flat_map(|w| floats(&cases[w % 8], "qvel"))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    plan.update(&mut data).unwrap();
    let before = data.readback().unwrap();
    let mut count = 0;
    let mut maximum = 0.0f64;
    for world in 0..513 {
        let (n, error) = compare(&before, world, &cases[world % 8]);
        count += n;
        maximum = maximum.max(error);
    }
    data.write_world_qpos(256, &floats(&cases[7], "qpos"))
        .unwrap();
    data.write_world_qvel(256, &floats(&cases[7], "qvel"))
        .unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady { .. })
    ));
    plan.update(&mut data).unwrap();
    let after = data.readback().unwrap();
    for world in 0..513 {
        compare(
            &after,
            world,
            &cases[if world == 256 { 7 } else { world % 8 }],
        );
    }
    compare(&before, 256, &cases[0]);
    eprintln!("G02-velocity cross_block_states=513 scalars={count} max_abs_error={maximum:e}");
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn invalid_velocity_inputs_and_model_identity_preserve_ready_results() {
    let value = fixture(0);
    let case = &value["cases"][5];
    let session = TransferSession::new(0).unwrap();
    let plan = ComVelocityPlan::new(&session, model(&value)).unwrap();
    let wrong = ComVelocityPlan::new(&session, model(&value)).unwrap();
    let mut data = plan.create_data(1).unwrap();
    data.write_qpos(&floats(case, "qpos")).unwrap();
    data.write_qvel(&floats(case, "qvel")).unwrap();
    plan.update(&mut data).unwrap();
    let before = data.readback().unwrap();
    assert!(matches!(
        wrong.update(&mut data),
        Err(TransferError::ModelMismatch)
    ));
    assert_same(&data.readback().unwrap(), &before);
    assert!(data.write_qvel(&[]).is_err());
    assert!(data.write_qpos(&[]).is_err());
    assert!(data.write_world_qpos(1, &floats(case, "qpos")).is_err());
    assert!(data.write_world_qvel(1, &floats(case, "qvel")).is_err());
    for x in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut qvel = floats(case, "qvel");
        qvel[0] = x;
        assert!(matches!(
            data.write_qvel(&qvel),
            Err(TransferError::Input(InputError::NonFinite { .. }))
        ));
        let mut qpos = floats(case, "qpos");
        qpos[0] = x;
        assert!(data.write_qpos(&qpos).is_err());
        assert_same(&data.readback().unwrap(), &before);
    }
    assert!(before.world(1).is_err());
    assert!(plan.create_data(0).is_err());
    assert!(plan.create_data(usize::MAX).is_err());
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn independent_velocity_states_and_snapshots_outlive_the_plan() {
    let value = fixture(1);
    let mut first;
    let second;
    {
        let session = TransferSession::new(0).unwrap();
        let plan = ComVelocityPlan::new(&session, model(&value)).unwrap();
        first = plan.create_data(1).unwrap();
        let mut data = plan.create_data(1).unwrap();
        first
            .write_qpos(&floats(&value["cases"][3], "qpos"))
            .unwrap();
        first
            .write_qvel(&floats(&value["cases"][3], "qvel"))
            .unwrap();
        data.write_qpos(&floats(&value["cases"][6], "qpos"))
            .unwrap();
        data.write_qvel(&floats(&value["cases"][6], "qvel"))
            .unwrap();
        plan.update(&mut first).unwrap();
        plan.update(&mut data).unwrap();
        second = data.readback().unwrap();
        compare(&first.readback().unwrap(), 0, &value["cases"][3]);
    }
    compare(&first.readback().unwrap(), 0, &value["cases"][3]);
    compare(&second, 0, &value["cases"][6]);
    first.write_qvel(&vec![0.0; second.nv()]).unwrap();
    assert!(matches!(
        first.readback(),
        Err(TransferError::StageNotReady { .. })
    ));
    compare(&second, 0, &value["cases"][6]);
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn velocity_overflow_rejects_results_without_poisoning_the_session() {
    let value = fixture(1);
    let session = TransferSession::new(0).unwrap();
    let plan = ComVelocityPlan::new(&session, model(&value)).unwrap();
    let mut data = plan.create_data(1).unwrap();
    data.write_qpos(&floats(&value["cases"][5], "qpos"))
        .unwrap();
    data.write_qvel(&vec![f32::MAX; value["nv"].as_u64().unwrap() as usize])
        .unwrap();
    plan.update(&mut data).unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::Input(InputError::NonFinite { .. }))
    ));
    data.write_qvel(&vec![0.0; value["nv"].as_u64().unwrap() as usize])
        .unwrap();
    plan.update(&mut data).unwrap();
    let healthy = data.readback().unwrap();
    assert!(healthy.world(0).unwrap().cvel.iter().all(|&v| v == 0.0));
    assert!(healthy.world(0).unwrap().cdof_dot.iter().all(|&v| v == 0.0));
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn zero_dof_velocity_worlds_keep_empty_derivatives_and_zero_body_velocities() {
    let session = TransferSession::new(0).unwrap();
    for index in [3, 4] {
        let value = fixture(index);
        let plan = ComVelocityPlan::new(&session, model(&value)).unwrap();
        let mut data = plan.create_data(257).unwrap();
        data.write_qvel(&[]).unwrap();
        data.write_world_qvel(256, &[]).unwrap();
        assert!(data.write_world_qvel(257, &[]).is_err());
        plan.update(&mut data).unwrap();
        let snapshot = data.readback().unwrap();
        for world in 0..257 {
            let view = snapshot.world(world).unwrap();
            assert!(view.cvel.iter().all(|&v| v == 0.0));
            assert!(view.cdof_dot.is_empty());
        }
    }
}
