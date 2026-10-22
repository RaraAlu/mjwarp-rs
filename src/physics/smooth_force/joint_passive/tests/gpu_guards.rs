//! 关节被动GPU故障拒绝测试。

#[cfg(feature = "cuda-probe")]
use super::tests_oracle::*;
#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "joint producer guard/mapping/nonfinite/overflow rejection only"]
fn gpu_rejects_bad_joint_sources_without_publishing_partial_parts() {
    let s = TransferSession::new(0).unwrap();
    let mut plan =
        JointPassivePlan::new(&s, model(), params([1; 5]), JointPassiveOptions::default()).unwrap();
    let mut d = plan.create_data(2).unwrap();
    d.write_qvel(&[0.5; 22]).unwrap();
    plan.update_joint(&mut d).unwrap();
    let mut faults = 0;
    for field in 0..5 {
        for edge in 0..8 {
            let b = match field {
                0 => &mut d.state,
                1 => &mut d.current,
                2 => &mut d.staged,
                3 => &mut d.status,
                _ => &mut plan.parameters,
            };
            let index = if edge < 4 { edge } else { b.len() - 8 + edge };
            b.write_range(index, &[77.0]).unwrap();
            let before = bits(&raw(&d.current));
            assert!(plan.update_joint(&mut d).is_err());
            assert_eq!(before, bits(&raw(&d.current)));
            let b = match field {
                0 => &mut d.state,
                1 => &mut d.current,
                2 => &mut d.staged,
                3 => &mut d.status,
                _ => &mut plan.parameters,
            };
            b.write_range(index, &[-131072.0]).unwrap();
            plan.update_joint(&mut d).unwrap();
            faults += 1;
        }
    }
    for index in [
        0,
        3,
        plan.metadata.len() - 4,
        plan.metadata.len() - 1,
        4,
        4 + 11,
        4 + 22,
        4 + 33,
    ] {
        let mut old = [0];
        plan.metadata.read_range_into(index, &mut old).unwrap();
        plan.metadata
            .corrupt_device_for_validator_test(index, &[i32::MAX])
            .unwrap();
        let before = bits(&raw(&d.current));
        assert!(plan.update_joint(&mut d).is_err());
        assert_eq!(before, bits(&raw(&d.current)));
        plan.metadata
            .corrupt_device_for_validator_test(index, &old)
            .unwrap();
        plan.update_joint(&mut d).unwrap();
        faults += 1;
    }
    for index in [4, 4 + 26] {
        let old = raw(&d.state)[index];
        d.state.write_range(index, &[f32::NAN]).unwrap();
        let before = bits(&raw(&d.current));
        assert!(plan.update_joint(&mut d).is_err());
        assert_eq!(before, bits(&raw(&d.current)));
        d.state.write_range(index, &[old]).unwrap();
        plan.update_joint(&mut d).unwrap();
        faults += 1;
    }
    d.write_world_qvel(1, &[f32::MAX; 11]).unwrap();
    let before = bits(&raw(&d.current));
    assert!(plan.update_joint(&mut d).is_err());
    assert_eq!(before, bits(&raw(&d.current)));
    d.write_world_qvel(1, &[0.5; 11]).unwrap();
    plan.update_joint(&mut d).unwrap();
    faults += 1;
    let other =
        JointPassivePlan::new(&s, model(), params([1; 5]), JointPassiveOptions::default()).unwrap();
    let before = bits(&raw(&d.current));
    assert!(other.update_joint(&mut d).is_err());
    assert_eq!(before, bits(&raw(&d.current)));
    assert!(d.readback_joint().is_ok());
    assert_eq!(faults, 51);
    println!(
        "joint_passive_private faults={faults} local_mapping_rejected=true no_partial_publication=true recovered=true old_smooth_dependencies=0"
    );
}
