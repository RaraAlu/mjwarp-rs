//! G01等价速度位置测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn g01_velocity_position_matches_equivalent_rigid_for_degenerate_quaternions() {
    fn raw(buffer: &TransferBuffer<f32>) -> Vec<u32> {
        let mut values = vec![0.0_f32; buffer.len()];
        buffer.read_range_into(0, &mut values).unwrap();
        values.iter().map(|value| value.to_bits()).collect()
    }
    fn workspace(session: &TransferSession, length: usize) -> TransferBuffer<f32> {
        let mut values = vec![f32::NAN; length];
        values[..4].fill(-131072.0);
        values[length - 4..].fill(-131072.0);
        session.upload(&values).unwrap()
    }
    let session = TransferSession::new(0).unwrap();
    // 非等价计划只接受单位四元数；两条路径须逐位一致。
    let strict = degenerate_quaternion_plan(&session, false);
    let mut strict_data = strict.create_data(4).unwrap();
    let unit_qpos = (0..4)
        .flat_map(|world| {
            let mut values = vec![0.25 * world as f32, -0.5, 0.75];
            values.extend([0.5, 0.5, -0.5, 0.5, 0.0, 0.6, 0.0, 0.8]);
            values
        })
        .collect::<Vec<_>>();
    strict_data.write_qpos(&unit_qpos).unwrap();
    strict_data
        .write_mocap(&[1.0, -2.0, 0.5].repeat(4), &[0.0, 0.0, 1.0, 0.0].repeat(4))
        .unwrap();
    strict.update_rigid(&mut strict_data).unwrap();
    let strict_kernels = strict.compile_velocity_position().unwrap();
    let mut strict_rigid = workspace(
        &session,
        strict_data.layout.rigid.output.total_elements() + 8,
    );
    let mut strict_com = workspace(&session, strict_data.layout.com.output.total_elements() + 8);
    strict
        .update_velocity_position(
            &strict_data,
            &strict_kernels,
            &mut strict_rigid,
            &mut strict_com,
        )
        .unwrap();
    assert_eq!(raw(&strict_data.rigid), raw(&strict_rigid));

    let (qpos, mocap_pos, mocap_quat) = degenerate_quaternion_state();
    let plan = degenerate_quaternion_plan(&session, true);
    let mut data = plan.create_data(4).unwrap();
    data.write_qpos(&qpos).unwrap();
    data.write_mocap(&mocap_pos, &mocap_quat).unwrap();
    plan.update_rigid(&mut data).unwrap();
    let expected = raw(&data.rigid);
    let kernels = plan.compile_velocity_position().unwrap();
    let mut rigid = workspace(&session, data.layout.rigid.output.total_elements() + 8);
    let mut com = workspace(&session, data.layout.com.output.total_elements() + 8);
    plan.update_velocity_position(&data, &kernels, &mut rigid, &mut com)
        .unwrap();
    let actual = raw(&rigid);
    let mismatched = expected
        .iter()
        .zip(&actual)
        .filter(|(expected, actual)| expected != actual)
        .count();
    println!(
        "g01-velocity-position worlds=4 slots={} mismatched={mismatched}",
        expected.len()
    );
    assert_eq!(expected, actual);

    // 模拟G01导入的非单位世界位姿；私有FK须跟随。
    let start = 4 + 2 * data.layout.rigid.output.elements_per_world();
    data.rigid.write_range(start, &[0.5, -1.0, 2.0]).unwrap();
    data.rigid
        .write_range(start + 3 * data.layout.rigid.nbody, &[0.0, 0.6, 0.0, 0.8])
        .unwrap();
    plan.update_rigid(&mut data).unwrap();
    let posed = raw(&data.rigid);
    assert_ne!(posed, expected);
    plan.update_velocity_position(&data, &kernels, &mut rigid, &mut com)
        .unwrap();
    assert_eq!(posed, raw(&rigid));
    println!("g01-velocity-position imported-world-pose=matched");
}
