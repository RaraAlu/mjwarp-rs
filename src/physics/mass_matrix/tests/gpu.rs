//! 刚体质量矩阵设备路径测试。

#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn device_adds_armature_after_the_completed_spatial_dot() {
    let session = TransferSession::new(0).unwrap();
    let metadata = session.upload(&[0_i32, 0, -1]).unwrap();
    let parameters = session.upload(&[1.0_f32]).unwrap();
    // 用二进制幂隔离舍入，不构造物理惯量。
    let mut values = vec![-131072.0_f32; 28];
    values[4..24].fill(0.0);
    values[8..11].copy_from_slice(&[134217728.0, -134217728.0, 8.0]);
    values[18..21].fill(1.0);
    let state = session.upload(&values).unwrap();
    let mut values = vec![-131072.0_f32; 19];
    values[4..15].fill(f32::NAN);
    let mut output = session.upload(&values).unwrap();
    let kernel =
        crate::runtime::SynchronousKernel::compile(&session, MASS_MATRIX_CUDA, "rigid_mass_matrix")
            .unwrap();
    // SAFETY: One world uses 20 COM scalars and 11 result scalars.
    // All buffers share this session; guards bound both layouts.
    unsafe {
        kernel
            .launch(&metadata, &parameters, &state, &mut output, [1, 1, 0, 1])
            .unwrap();
    }
    output.read_range_into(0, &mut values).unwrap();
    super::super::check_device_values(&values, "mass_matrix_output").unwrap();
    assert_eq!(values[14], 9.0);
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn runs_rigid_mass_matrix_wrapper_paths_and_recovers() {
    fn captured<T>(label: &str, run: impl FnOnce() -> T) -> T {
        let capture = std::env::var_os("MJWARP_PHYSICS_PTX_DIR");
        if let Some(child) = crate::runtime::capture_child(module_path!(), label) {
            assert!(!child.exists(), "捕获目录必须为空：{}", child.display());
            // SAFETY: 本测试单线程运行，并在本次编译后恢复环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &child) };
        }
        let result = run();
        if let Some(root) = capture.as_ref() {
            // SAFETY: 本测试单线程运行，并恢复调用方环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
        } else {
            // SAFETY: 本测试单线程运行，调用前也没有该变量。
            unsafe { std::env::remove_var("MJWARP_PHYSICS_PTX_DIR") };
        }
        result
    }
    fn bits(values: &[f32]) -> Vec<u32> {
        values.iter().map(|value| value.to_bits()).collect()
    }
    fn read(buffer: &TransferBuffer<f32>) -> Vec<f32> {
        let mut values = vec![0.0; buffer.len()];
        buffer.read_range_into(0, &mut values).unwrap();
        values
    }
    fn guarded(len: usize) -> Vec<f32> {
        let mut values = vec![f32::NAN; len];
        values[..4].fill(-131072.0);
        values[len - 4..].fill(-131072.0);
        values
    }

    let session = TransferSession::new(0).unwrap();
    // 合成拓扑覆盖分叉、静态祖先与多自由度刚体。
    let worlds = 257usize;
    let (nbody, nv) = (5usize, 6usize);
    let topology = vec![0, 0, 1, 1, 3, 1, 1, 2, 3, 4, 4, -1, 0, 1, 0, 3, 4];
    let abi = [nbody as u32, nv as u32, 0, worlds as u32];
    let mut metadata =
        RigidMassMatrixMetadataBuffer::upload(&session, topology.clone(), abi).unwrap();
    let armature = (0..nv)
        .map(|dof| 0.125 * (dof + 1) as f32)
        .collect::<Vec<_>>();
    let armature_buffer = session.upload(&armature).unwrap();
    let state_width = 14 * nbody + 6 * nv;
    let mut state = vec![-131072.0_f32; 8 + worlds * state_width];
    for (index, value) in state[4..4 + worlds * state_width].iter_mut().enumerate() {
        *value = ((index * 37 + 11) % 101) as f32 / 64.0 - 0.75;
    }
    let state_buffer = session.upload(&state).unwrap();
    let initial = guarded(8 + worlds * (10 * nbody + nv * nv));
    let mut output = session.upload(&initial).unwrap();
    let raw_kernel = captured("raw-baseline", || {
        SynchronousKernel::compile(&session, MASS_MATRIX_CUDA, "rigid_mass_matrix")
    })
    .unwrap();
    // SAFETY: 迁移前原始调用；拓扑已验证，缓冲精确匹配冻结ABI。
    unsafe {
        raw_kernel
            .launch(
                metadata.as_buffer(),
                &armature_buffer,
                &state_buffer,
                &mut output,
                abi,
            )
            .unwrap();
    }
    let raw = read(&output);
    super::super::check_device_values(&raw, "mass_matrix_output").unwrap();
    output.write_range(0, &initial).unwrap();
    let kernel = captured("typed-main", || RigidMassMatrixKernel::compile(&session)).unwrap();
    let launch = kernel
        .preflight(
            &session,
            &metadata,
            &armature_buffer,
            &state_buffer,
            &output,
        )
        .unwrap();
    kernel
        .launch(
            &launch,
            &metadata,
            &armature_buffer,
            &state_buffer,
            &mut output,
        )
        .unwrap();
    let typed = read(&output);
    assert_eq!(bits(&raw), bits(&typed));

    // 拒绝发生在写入之前。
    let short = session.upload(&armature[..nv - 1]).unwrap();
    assert!(
        kernel
            .preflight(&session, &metadata, &short, &state_buffer, &output)
            .is_err()
    );
    let short_state = session.upload(&state[..state.len() - 1]).unwrap();
    assert!(
        kernel
            .preflight(&session, &metadata, &armature_buffer, &short_state, &output)
            .is_err()
    );
    let mut long_output = session.upload(&guarded(initial.len() + 1)).unwrap();
    assert!(
        kernel
            .launch(
                &launch,
                &metadata,
                &armature_buffer,
                &state_buffer,
                &mut long_output,
            )
            .is_err()
    );
    assert_eq!(bits(&read(&long_output)), bits(&guarded(initial.len() + 1)));
    assert!(matches!(
        RigidMassMatrixMetadataBuffer::upload(
            &session,
            vec![0, 0, 2, 1, 3, 1, 1, 2, 3, 4, 4, -1, 0, 1, 0, 3, 4],
            abi
        ),
        Err(TransferError::Input(InputError::InvalidTopology { .. }))
    ));
    assert!(matches!(
        RigidMassMatrixMetadataBuffer::upload(
            &session,
            topology.clone(),
            [nbody as u32, nv as u32, 1, worlds as u32]
        ),
        Err(TransferError::Input(InputError::InvalidDimension { .. }))
    ));
    assert_eq!(bits(&read(&output)), bits(&typed));

    let stale = kernel
        .preflight(
            &session,
            &metadata,
            &armature_buffer,
            &state_buffer,
            &output,
        )
        .unwrap();
    metadata.overwrite_for_test(1, 7).unwrap();
    assert!(
        kernel
            .launch(
                &stale,
                &metadata,
                &armature_buffer,
                &state_buffer,
                &mut output,
            )
            .is_err()
    );
    assert!(
        kernel
            .preflight(
                &session,
                &metadata,
                &armature_buffer,
                &state_buffer,
                &output
            )
            .is_err()
    );
    assert_eq!(bits(&read(&output)), bits(&typed));
    metadata.restore_for_test().unwrap();
    output.write_range(0, &initial).unwrap();
    let recovered = kernel
        .preflight(
            &session,
            &metadata,
            &armature_buffer,
            &state_buffer,
            &output,
        )
        .unwrap();
    kernel
        .launch(
            &recovered,
            &metadata,
            &armature_buffer,
            &state_buffer,
            &mut output,
        )
        .unwrap();
    assert_eq!(bits(&read(&output)), bits(&typed));

    let foreign_session = TransferSession::new(0).unwrap();
    let foreign = captured("foreign-session", || {
        RigidMassMatrixKernel::compile(&foreign_session)
    })
    .unwrap();
    assert!(matches!(
        foreign.preflight(
            &session,
            &metadata,
            &armature_buffer,
            &state_buffer,
            &output
        ),
        Err(TransferError::SessionMismatch)
    ));
    let foreign_state = foreign_session.upload(&state).unwrap();
    assert!(matches!(
        kernel.preflight(
            &session,
            &metadata,
            &armature_buffer,
            &foreign_state,
            &output
        ),
        Err(TransferError::SessionMismatch)
    ));
    assert_eq!(bits(&read(&output)), bits(&typed));

    // 零自由度保留真实armature占位。
    let zero_abi = [2, 0, 0, 2];
    let zero_metadata =
        RigidMassMatrixMetadataBuffer::upload(&session, vec![0, 0], zero_abi).unwrap();
    let zero_armature = session.upload(&[0.0_f32]).unwrap();
    let mut zero_state = vec![-131072.0_f32; 64];
    for (index, value) in zero_state[4..60].iter_mut().enumerate() {
        *value = index as f32 / 8.0;
    }
    let zero_state = session.upload(&zero_state).unwrap();
    let zero_initial = guarded(48);
    let mut zero_output = session.upload(&zero_initial).unwrap();
    // SAFETY: 零自由度原始调用；缓冲精确匹配冻结ABI。
    unsafe {
        raw_kernel
            .launch(
                zero_metadata.as_buffer(),
                &zero_armature,
                &zero_state,
                &mut zero_output,
                zero_abi,
            )
            .unwrap();
    }
    let zero_raw = read(&zero_output);
    zero_output.write_range(0, &zero_initial).unwrap();
    let zero_launch = kernel
        .preflight(
            &session,
            &zero_metadata,
            &zero_armature,
            &zero_state,
            &zero_output,
        )
        .unwrap();
    kernel
        .launch(
            &zero_launch,
            &zero_metadata,
            &zero_armature,
            &zero_state,
            &mut zero_output,
        )
        .unwrap();
    assert_eq!(bits(&zero_raw), bits(&read(&zero_output)));
    super::super::check_device_values(&zero_raw, "mass_matrix_output").unwrap();

    // 产品入口与同输入原始调用逐位一致。
    let model = super::super::tests::test_model(Some(0));
    let qpos = [0.25, -0.5, 0.75, 0.5, 0.5, -0.5, 0.5].repeat(3);
    let (_, product) = captured("product-main", || {
        mass_matrix_device(&session, &model, 3, &qpos)
    })
    .unwrap();
    let (fk, com) = super::super::check_com_position(&model, 3, &qpos).unwrap();
    let (com_state, _) =
        super::super::com_position_device(&session, &model, fk, com, &qpos).unwrap();
    let k = model.kinematics();
    let mut packed = k.fields().body_parentid.clone();
    packed.extend(&model.fields().dof_bodyid);
    packed.extend(&model.fields().dof_parentid);
    let product_metadata = session.upload(&packed).unwrap();
    let product_armature = session.upload(&model.fields().dof_armature).unwrap();
    let product_initial = guarded(product.values.len());
    let mut product_output = session.upload(&product_initial).unwrap();
    // SAFETY: 产品模型已通过拓扑校验；缓冲精确匹配冻结ABI。
    unsafe {
        raw_kernel
            .launch(
                &product_metadata,
                &product_armature,
                &com_state,
                &mut product_output,
                [k.nbody() as u32, k.nv() as u32, 0, 3],
            )
            .unwrap();
    }
    assert_eq!(bits(&read(&product_output)), bits(&product.values));
}
