//! 关节被动GPU自由球弹簧测试。

#[cfg(feature = "cuda-probe")]
use super::tests_oracle::*;
#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires CUDA; targeted FREE/BALL spring parity only"]
fn gpu_produces_free_ball_springs_with_warp_quaternion_edges() {
    const WORLDS: usize = 13;
    const SQRT_HALF: f32 = 0.70710677;
    let session = TransferSession::new(0).unwrap();
    let source_model = model();
    let free_reference = [
        [1.0, 0.0, 0.0, 0.0],
        [2.0, 0.25, -0.5, 0.75],
        [1.0, 0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
        [0.5, 0.5, 0.5, 0.5],
        [1.0, 0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
        [SQRT_HALF, SQRT_HALF, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
        [1.0e-23, 0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
    ];
    let ball_reference = [
        [1.0, 0.0, 0.0, 0.0],
        [0.75, -0.5, 0.25, 0.125],
        [1.0, 0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
        [0.5, 0.5, 0.5, 0.5],
        [1.0, 0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
        [SQRT_HALF, 0.0, SQRT_HALF, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
        [1.0e-23, 0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
    ];
    let free_current = [
        [0.0, 0.0, 0.0, 0.0],
        [1.5, 0.375, -0.625, 0.5],
        [0.9238795, 0.38268343, 0.0, 0.0],
        [-0.9238795, -0.38268343, 0.0, 0.0],
        [0.5, 0.5, 0.5, 0.5],
        [0.0, 1.0, 0.0, 0.0],
        [0.0001, 1.0, 0.0, 0.0],
        [-0.0001, 1.0, 0.0, 0.0],
        [SQRT_HALF, 0.0, SQRT_HALF, 0.0],
        [1.0, 0.0, 0.0, 0.0],
        [1.0e-23, 1.0e-23, 1.0e-23, 1.0e-23],
        [SQRT_HALF, SQRT_HALF, 0.0, 0.0],
        [4.0e-23, 4.0e-23, 0.0, 0.0],
    ];
    let ball_current = [
        [0.0, 0.0, 0.0, 0.0],
        [1.25, -0.25, 0.5, 0.75],
        [0.9238795, 0.0, 0.38268343, 0.0],
        [-0.9238795, 0.0, -0.38268343, 0.0],
        [0.5, 0.5, 0.5, 0.5],
        [0.0, 0.0, 1.0, 0.0],
        [0.0001, 0.0, 0.0, 1.0],
        [-0.0001, 0.0, 0.0, 1.0],
        [SQRT_HALF, 0.0, 0.0, SQRT_HALF],
        [1.0, 0.0, 0.0, 0.0],
        [1.0e-23, 1.0e-23, 1.0e-23, 1.0e-23],
        [SQRT_HALF, 0.0, SQRT_HALF, 0.0],
        [4.0e-23, 0.0, 4.0e-23, 0.0],
    ];
    let translation = [
        [0.5, 0.0, 0.0],
        [0.0, -0.25, 0.0],
        [0.0, 0.0, 0.125],
        [-0.5, 0.0, 0.0],
        [0.0, 0.25, 0.0],
        [0.0, 0.0, -0.125],
        [0.25, 0.0, 0.0],
        [0.0, -0.5, 0.0],
        [0.0, 0.0, 0.25],
        [-0.125, 0.0, 0.0],
        [0.0, 0.125, 0.0],
        [0.0, 0.0, -0.25],
        [0.5, 0.0, 0.0],
    ];
    let slide = [
        0.25, -0.125, 0.5, -0.25, 0.0, 0.125, -0.5, 0.25, -0.125, 0.5, -0.25, 0.125, -0.5,
    ];
    let hinge = [
        -0.25, 0.5, -0.125, 0.25, 0.0, -0.5, 0.125, -0.25, 0.5, -0.125, 0.25, -0.5, 0.125,
    ];
    let mut references = Vec::with_capacity(WORLDS * 13);
    let mut qpos = Vec::with_capacity(WORLDS * 13);
    for world in 0..WORLDS {
        let mut reference = source_model.fields().qpos0.clone();
        reference[0] = world as f32 * 0.0625;
        reference[1] = -(world as f32) * 0.03125;
        reference[2] = world as f32 * 0.015625;
        reference[3..7].copy_from_slice(&free_reference[world]);
        reference[7..11].copy_from_slice(&ball_reference[world]);
        reference[11] = 0.125;
        reference[12] = -0.25;
        let mut state = reference.clone();
        for k in 0..3 {
            state[k] += translation[world][k];
        }
        state[3..7].copy_from_slice(&free_current[world]);
        state[7..11].copy_from_slice(&ball_current[world]);
        state[11] += slide[world];
        state[12] += hinge[world];
        references.extend(reference);
        qpos.extend(state);
    }
    let parameters = JointPassiveParameters {
        qpos_spring: ParameterBatch::new(WORLDS, 13, references).unwrap(),
        stiffness: ParameterBatch::new(1, 4, vec![1.25, 0.75, 0.5, 0.625]).unwrap(),
        stiffness_poly: ParameterBatch::new(
            1,
            8,
            vec![0.0, 0.0, 0.125, 0.0625, 0.125, -0.0625, -0.125, 0.0625],
        )
        .unwrap(),
        damping: ParameterBatch::new(1, 11, vec![0.0; 11]).unwrap(),
        damping_poly: ParameterBatch::new(1, 22, vec![0.0; 22]).unwrap(),
    };
    let mut plan = JointPassivePlan::new(
        &session,
        source_model.clone(),
        parameters.clone(),
        JointPassiveOptions::default(),
    )
    .unwrap();
    let mut data = plan.create_data(WORLDS).unwrap();
    let zeros = vec![0.0; WORLDS * 11];
    data.write_qpos(&qpos).unwrap();
    data.write_qvel(&zeros).unwrap();
    for field in [
        SmoothForceField::Bias,
        SmoothForceField::Actuator,
        SmoothForceField::Applied,
    ] {
        data.write_other_force(field, &zeros).unwrap();
    }
    plan.update_joint_only_smooth(&mut data).unwrap();
    let parts = data.readback_joint().unwrap();
    let smooth = data.readback_joint_only_smooth().unwrap();
    let mut expected = Vec::with_capacity(WORLDS);
    for world in 0..WORLDS {
        let oracle = rotation_oracle(&plan.input, world, &qpos[13 * world..13 * (world + 1)]);
        let spring = parts.spring(world).unwrap();
        let combined = smooth.world(world).unwrap();
        for index in [0, 1, 2, 9, 10] {
            assert_eq!(spring[index], oracle[index], "world={world} dof={index}");
            assert_eq!(combined[index], oracle[index], "world={world} dof={index}");
        }
        for index in 3..9 {
            let error = (spring[index] - oracle[index]).abs();
            assert!(
                error <= ROTATION_ABS_TOLERANCE,
                "world={world} dof={index} error={error}"
            );
            let smooth_error = (combined[index] - oracle[index]).abs();
            assert!(
                smooth_error <= ROTATION_ABS_TOLERANCE,
                "smooth world={world} dof={index} error={smooth_error}"
            );
        }
        assert_eq!(bits(parts.damper(world).unwrap()), vec![0; 11]);
        expected.push(oracle);
    }
    // 同一下标同时读取三组参考。
    #[allow(clippy::needless_range_loop)]
    for index in 3..9 {
        assert!(
            (expected[2][index] - expected[3][index]).abs() <= ROTATION_ABS_TOLERANCE,
            "q sign dof={index}"
        );
        assert_eq!(expected[4][index], 0.0, "zero axis dof={index}");
    }
    let zero_fallback = warp_quat_sub([0.0; 4], [1.0, 0.0, 0.0, 0.0]);
    assert_eq!(zero_fallback[0], 0.0);
    assert_eq!(zero_fallback[1], 0.0);
    assert_eq!(zero_fallback[2].to_bits(), std::f32::consts::PI.to_bits());
    let below = warp_normalize_quat(free_current[6]);
    let below_raw = 2.0 * warp_len3([below[1], below[2], below[3]]).atan2(below[0]);
    assert!(below[0] > 0.0 && below_raw < std::f32::consts::PI);
    let negative = warp_normalize_quat(free_current[7]);
    let negative_difference = warp_mul_quat([1.0, -0.0, -0.0, -0.0], negative);
    let negative_raw = 2.0
        * warp_len3([
            negative_difference[1],
            negative_difference[2],
            negative_difference[3],
        ])
        .atan2(negative_difference[0]);
    assert!(negative_difference[0] < 0.0 && negative_raw > std::f32::consts::PI);
    let current = warp_normalize_quat(free_current[8]);
    let reference = free_reference[8];
    let conjugate = [reference[0], -reference[1], -reference[2], -reference[3]];
    let forward = warp_quat_to_vel(warp_mul_quat(conjugate, current));
    let reversed = warp_quat_to_vel(warp_mul_quat(current, conjugate));
    let analytic = [-1.2091995, 1.2091995, -1.2091995];
    for index in 0..3 {
        assert!((forward[index] - analytic[index]).abs() <= ROTATION_ABS_TOLERANCE);
    }
    assert!(
        forward
            .iter()
            .zip(reversed)
            .any(|(&a, b)| (a - b).abs() > 0.1)
    );
    let right_angle = warp_quat_sub([SQRT_HALF, SQRT_HALF, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]);
    assert!((right_angle[0] - std::f32::consts::FRAC_PI_2).abs() <= ROTATION_ABS_TOLERANCE);
    assert_eq!((right_angle[1], right_angle[2]), (0.0, 0.0));
    let zero_reference = warp_quat_sub(free_current[9], free_reference[9]);
    assert_eq!(zero_reference, [0.0; 3]);
    let normalized_zero_reference =
        warp_quat_sub(free_current[9], warp_normalize_quat(free_reference[9]));
    assert!(normalized_zero_reference.iter().any(|v| v.abs() > 1.0));
    let underflow_current = warp_quat_sub(free_current[10], free_reference[10]);
    assert_eq!(underflow_current[0], 0.0);
    assert_eq!(underflow_current[1], 0.0);
    assert_eq!(
        underflow_current[2].to_bits(),
        std::f32::consts::PI.to_bits()
    );
    let underflow_reference = warp_quat_sub(free_current[11], free_reference[11]);
    assert_eq!(underflow_reference, [0.0; 3]);
    let normalized_underflow_reference =
        warp_quat_sub(free_current[11], warp_normalize_quat(free_reference[11]));
    assert!(normalized_underflow_reference.iter().any(|v| v.abs() > 1.0));
    let nonzero_square = warp_quat_sub(free_current[12], free_reference[12]);
    assert!(nonzero_square[0].abs() > 1.0);
    assert_eq!((nonzero_square[1], nonzero_square[2]), (0.0, 0.0));
    let local_index = 4 + 3 * 11;
    let mut old_local = [0];
    plan.metadata
        .read_range_into(local_index, &mut old_local)
        .unwrap();
    plan.metadata
        .corrupt_device_for_validator_test(local_index, &[i32::MAX])
        .unwrap();
    let before_rejection = bits(&raw(&data.current));
    assert!(plan.update_joint(&mut data).is_err());
    assert_eq!(before_rejection, bits(&raw(&data.current)));
    plan.metadata
        .corrupt_device_for_validator_test(local_index, &old_local)
        .unwrap();
    plan.update_joint_only_smooth(&mut data).unwrap();
    let disabled = JointPassivePlan::new(
        &session,
        source_model,
        parameters,
        JointPassiveOptions {
            disable_spring: true,
            disable_damper: false,
        },
    )
    .unwrap();
    let mut disabled_data = disabled.create_data(WORLDS).unwrap();
    disabled_data.write_qpos(&qpos).unwrap();
    disabled_data.write_qvel(&zeros).unwrap();
    for field in [
        SmoothForceField::Bias,
        SmoothForceField::Actuator,
        SmoothForceField::Applied,
    ] {
        disabled_data.write_other_force(field, &zeros).unwrap();
    }
    disabled
        .update_joint_only_smooth(&mut disabled_data)
        .unwrap();
    let disabled_parts = disabled_data.readback_joint().unwrap();
    let disabled_smooth = disabled_data.readback_joint_only_smooth().unwrap();
    for world in 0..WORLDS {
        assert!(
            disabled_parts
                .spring(world)
                .unwrap()
                .iter()
                .all(|&v| v == 0.0)
        );
        assert!(
            disabled_parts
                .damper(world)
                .unwrap()
                .iter()
                .all(|&v| v == 0.0)
        );
        assert!(
            disabled_smooth
                .world(world)
                .unwrap()
                .iter()
                .all(|&v| v == 0.0)
        );
    }
    println!(
        "joint_passive_rotation worlds={WORLDS} rotation_tolerance={ROTATION_ABS_TOLERANCE:e} q_sign=true zero_quaternion=true nonunit_current=true nonunit_reference=true disabled=true"
    );
}
