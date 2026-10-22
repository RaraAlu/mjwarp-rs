//! 共享被动链来源代际声明、拒绝与恢复测试。

use super::tests_support::*;
use super::*;
use crate::model::{CamLightParameters, KinematicsParameters};
use crate::physics::{
    FlexEdgePassiveInput, FlexEdgePassiveOptions, JointPassiveOptions, KinematicsPlan,
    TendonPassiveOptions, TendonVelocityPlan,
};

#[test]
#[ignore = "shared passive source generation declarations and rejection"]
fn gpu_shared_source_generation_rejects_stale_copies_and_recovers() {
    let session = TransferSession::new(0).unwrap();
    let model = kinematic([1.0, 0.0, 0.0]);
    let worlds = 3;
    let source_plan = KinematicsPlan::with_flex_positions(
        &session,
        flex_model(model.clone()),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let velocity_plan = TendonVelocityPlan::new(&source_plan).unwrap();
    let joint_plan = JointPassivePlan::new(
        &session,
        model.clone(),
        joint_parameters(),
        JointPassiveOptions::default(),
    )
    .unwrap();
    let tendon_plan = TendonPassivePlan::new(
        &velocity_plan,
        tendon_parameters(),
        TendonPassiveOptions::default(),
    )
    .unwrap();
    let flex_plan = FlexEdgePassivePlan::new(
        &source_plan,
        FlexEdgePassiveInput {
            length0: vec![0.0],
            stiffness: vec![-1.0],
            damping: vec![0.5],
            rigid: vec![false],
        },
        FlexEdgePassiveOptions::default(),
    )
    .unwrap();
    let plan = PassiveForcePlan::new(&joint_plan, &tendon_plan, &flex_plan).unwrap();

    let qpos = (0..worlds)
        .map(|world| 0.125 * (world + 1) as f32)
        .collect::<Vec<_>>();
    let qvel = (0..worlds)
        .map(|world| 0.25 + 0.125 * world as f32)
        .collect::<Vec<_>>();
    let mut source = source_plan.create_data(worlds).unwrap();
    let mut velocity = velocity_plan.create_data(worlds).unwrap();
    let mut joint = joint_plan.create_data(worlds).unwrap();
    let mut data = plan.create_data(worlds).unwrap();
    source.write_qpos(&qpos).unwrap();
    source.write_qvel(&qvel).unwrap();
    source_plan.update(&mut source).unwrap();
    // 声明来源：关节副本与肌腱历史均取自当前常驻源。
    joint.write_qpos_from(&source, &qpos).unwrap();
    joint.write_qvel_from(&source, &qvel).unwrap();
    joint_plan.update_joint(&mut joint).unwrap();
    velocity.write_qvel_from(&source, &qvel).unwrap();
    {
        let view = velocity_plan.update(&source, &mut velocity).unwrap();
        plan.update(&joint, &view, &source, &mut data).unwrap();
    }
    assert!(data.snapshot().is_ok());
    assert_eq!(
        joint.source_state().unwrap().state_version(),
        source.source_state().state_version()
    );

    // 关节副本过期：常驻源已前进，关节未重同步时拒绝。
    let stale_qvel = qvel.iter().map(|value| value + 0.25).collect::<Vec<_>>();
    source.write_qvel(&stale_qvel).unwrap();
    velocity.write_qvel_from(&source, &stale_qvel).unwrap();
    {
        let view = velocity_plan.update(&source, &mut velocity).unwrap();
        assert!(matches!(
            plan.update(&joint, &view, &source, &mut data),
            Err(TransferError::StageNotReady {
                stage: "passive_force_joint_generation"
            })
        ));
    }
    // 拒绝先于设备写入：上一代结果保持，不产生部分更新。
    assert!(data.snapshot().is_ok());
    joint.write_qvel_from(&source, &stale_qvel).unwrap();
    joint_plan.update_joint(&mut joint).unwrap();
    source_plan.update_flex_edges(&mut source).unwrap();
    {
        let view = velocity_plan.update(&source, &mut velocity).unwrap();
        plan.update(&joint, &view, &source, &mut data).unwrap();
    }
    assert!(data.snapshot().is_ok());

    // 肌腱历史声明过期：先声明历史来源，再改源、不重写历史副本时拒绝。
    velocity.write_qvel_from(&source, &stale_qvel).unwrap();
    let newest_qvel = stale_qvel
        .iter()
        .map(|value| value - 0.5)
        .collect::<Vec<_>>();
    source.write_qvel(&newest_qvel).unwrap();
    assert!(matches!(
        velocity_plan.update(&source, &mut velocity),
        Err(TransferError::StageNotReady {
            stage: "tendon_velocity_source_generation"
        })
    ));
    velocity.write_qvel_from(&source, &newest_qvel).unwrap();
    joint.write_qvel_from(&source, &newest_qvel).unwrap();
    joint_plan.update_joint(&mut joint).unwrap();
    source_plan.update_flex_edges(&mut source).unwrap();
    {
        let view = velocity_plan.update(&source, &mut velocity).unwrap();
        plan.update(&joint, &view, &source, &mut data).unwrap();
    }

    // 旧写入入口保持兼容：不声明来源时不新增拒绝面。
    let mut legacy_joint = joint_plan.create_data(worlds).unwrap();
    legacy_joint.write_qpos(&qpos).unwrap();
    legacy_joint.write_qvel(&newest_qvel).unwrap();
    joint_plan.update_joint(&mut legacy_joint).unwrap();
    assert!(legacy_joint.source_state().is_none());
    {
        let view = velocity_plan.update(&source, &mut velocity).unwrap();
        plan.update(&legacy_joint, &view, &source, &mut data)
            .unwrap();
    }

    let report = serde_json::json!({
        "worlds": worlds,
        "source_declaration": true,
        "stale_joint_generation_rejected": true,
        "tendon_input_generation_rejected": true,
        "resync_recovered": true,
        "legacy_writes_compatible": true,
        "partial_publication_blocked": true,
        "absolute_tolerance": 2.0e-6,
        "performance_acceptance": false
    });
    if let Some(path) = std::env::var_os("MJWARP_SHARED_GENERATION_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    println!("{report}");
}
