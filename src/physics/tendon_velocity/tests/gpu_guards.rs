//! 肌腱速度GPU故障拒绝测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new consumer guards/mapping/nonfinite/overflow and publication isolation"]
fn gpu_rejects_corrupted_connection_without_partial_publication() {
    let session = TransferSession::new(0).unwrap();
    let source_plan = producer(&session, 3);
    let mut source = source_plan.create_data(2).unwrap();
    source_plan.update(&mut source).unwrap();
    let mut plan = TendonVelocityPlan::new(&source_plan).unwrap();
    let mut data = plan.create_data(2).unwrap();
    data.write_qvel(&[0.25; 6]).unwrap();
    plan.update(&source, &mut data).unwrap();
    let mut faults = 0;
    for kind in 0..5 {
        let n = match kind {
            0 => data.qvel.len(),
            1 => data.current.len(),
            2 => data.staged.len(),
            3 => data.status.len(),
            _ => source_plan.tendon_velocity_source(&source).unwrap().len(),
        };
        for index in [0, 1, 2, 3, n - 4, n - 3, n - 2, n - 1] {
            let before = bits(&raw(&data.current));
            let b = match kind {
                0 => &mut data.qvel,
                1 => &mut data.current,
                2 => &mut data.staged,
                3 => &mut data.status,
                _ => source.tendon_velocity_source_for_test(),
            };
            let value = raw(b)[index];
            b.write_range(index, &[99.0]).unwrap();
            assert!(plan.update(&source, &mut data).is_err());
            assert!(!data.ready);
            let b = match kind {
                0 => &mut data.qvel,
                1 => &mut data.current,
                2 => &mut data.staged,
                3 => &mut data.status,
                _ => source.tendon_velocity_source_for_test(),
            };
            b.write_range(index, &[value]).unwrap();
            assert_eq!(before, bits(&raw(&data.current)));
            plan.update(&source, &mut data).unwrap();
            faults += 1;
        }
    }
    let m = plan.metadata.len_for_test();
    for index in [0, 1, 2, 3, m - 4, m - 3, m - 2, m - 1, 4, 9, 14] {
        let before = bits(&raw(&data.current));
        let mut old = [0];
        plan.metadata.read_range_for_test(index, &mut old).unwrap();
        plan.metadata
            .write_range_for_test(index, &[i32::MAX])
            .unwrap();
        assert!(plan.update(&source, &mut data).is_err());
        assert_eq!(before, bits(&raw(&data.current)));
        plan.metadata.write_range_for_test(index, &old).unwrap();
        plan.update(&source, &mut data).unwrap();
        faults += 1;
    }
    for kind in 0..2 {
        let index = if kind == 0 { 4 } else { 9 };
        let before = bits(&raw(&data.current));
        let b = if kind == 0 {
            &mut data.qvel
        } else {
            source.tendon_velocity_source_for_test()
        };
        let old = raw(b)[index];
        b.write_range(index, &[f32::NAN]).unwrap();
        assert!(plan.update(&source, &mut data).is_err());
        assert_eq!(before, bits(&raw(&data.current)));
        let b = if kind == 0 {
            &mut data.qvel
        } else {
            source.tendon_velocity_source_for_test()
        };
        b.write_range(index, &[old]).unwrap();
        plan.update(&source, &mut data).unwrap();
        faults += 1;
    }
    let before = bits(&raw(&data.current));
    let old_j = raw(source_plan.tendon_velocity_source(&source).unwrap())[9];
    source
        .tendon_velocity_source_for_test()
        .write_range(9, &[f32::MAX])
        .unwrap();
    data.qvel.write_range(4, &[2.0]).unwrap();
    assert!(plan.update(&source, &mut data).is_err());
    assert_eq!(before, bits(&raw(&data.current)));
    source
        .tendon_velocity_source_for_test()
        .write_range(9, &[old_j])
        .unwrap();
    data.qvel.write_range(4, &[0.25]).unwrap();
    plan.update(&source, &mut data).unwrap();
    faults += 1;
    assert_eq!(faults, 54);
    // 元数据证明失效：设备写入先于发射被拒绝；恢复镜像后再次成功。
    plan.metadata.overwrite_for_test(0, 1).unwrap();
    assert!(plan.update(&source, &mut data).is_err());
    plan.metadata.restore_for_test().unwrap();
    plan.update(&source, &mut data).unwrap();
    println!(
        "tendon_velocity_private faults={faults} source_updates=1 no_partial_publication=true recovered=true"
    );
}
