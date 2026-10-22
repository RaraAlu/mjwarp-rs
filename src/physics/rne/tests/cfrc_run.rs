//! RNE复合力运行测试。

#[cfg(feature = "cuda-probe")]
use super::kernels::RneCfrcKernel;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::rne_cfrc_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::rne_cfrc_plan;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn runs_rne_cfrc_wrapper_and_recovers_after_real_metadata_write() {
    let session = TransferSession::new(0).unwrap();
    let model = super::super::tests::test_model(Some(3));
    let model = AttachedModelInput::new(model, crate::model::AttachedFields::default()).unwrap();
    let mut plan = RnePlan::new(&session, model, [0.0, 0.0, -9.81]).unwrap();
    const WORLDS: usize = 513;
    let mut data = plan.create_data(WORLDS).unwrap();
    let qvel: Vec<_> = (0..WORLDS)
        .map(|world| (world as f32 - 256.0) * 0.000_125)
        .collect();
    let qacc: Vec<_> = (0..WORLDS)
        .map(|world| (256.0 - world as f32) * 0.000_25)
        .collect();
    data.write_qvel(&qvel).unwrap();
    data.write_qacc(&qacc).unwrap();
    plan.update_velocity(&mut data).unwrap();
    let width = 12 * plan.nbody + plan.nv;
    let cfrc_offset = 6 * plan.nbody;
    let sentinel = -777.25f32;
    let mut initial = vec![sentinel; data.output.len()];
    initial[..4].fill(-131072.0);
    let end = initial.len() - 4;
    initial[end..].fill(-131072.0);

    let (raw, typed) = {
        let inputs = data.velocity.rne_inputs().unwrap();
        data.output.write_range(0, &initial).unwrap();
        // SAFETY: 先用冻结旧ABI生成相同cacc前缀。
        unsafe {
            plan.cacc_kernel.kernel.launch_rne_cacc(
                plan.metadata.as_buffer(),
                (inputs.qvel, &data.qacc, inputs.com, inputs.velocity),
                &mut data.output,
                true,
                WORLDS as u32,
            )
        }
        .unwrap();
        let mut cacc_ready = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut cacc_ready).unwrap();

        data.output.write_range(0, &cacc_ready).unwrap();
        // SAFETY: 复用产品计划的同一实际缓冲和冻结私有ABI。
        unsafe {
            plan.cfrc_kernel.kernel.launch(
                plan.metadata.as_buffer(),
                inputs.com,
                inputs.velocity,
                &mut data.output,
                [0, 0, 0, WORLDS as u32],
            )
        }
        .unwrap();
        let mut raw = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut raw).unwrap();

        data.output.write_range(0, &cacc_ready).unwrap();
        let launch = plan
            .cfrc_kernel
            .preflight(
                &plan.session,
                &plan.metadata,
                inputs.com,
                inputs.velocity,
                &data.output,
                WORLDS as u32,
            )
            .unwrap();
        plan.cfrc_kernel
            .launch(
                &launch,
                &plan.metadata,
                inputs.com,
                inputs.velocity,
                &mut data.output,
            )
            .unwrap();
        let mut typed = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut typed).unwrap();

        let short_velocity = session
            .upload(&vec![0.0; inputs.velocity.len() - 1])
            .unwrap();
        assert!(
            plan.cfrc_kernel
                .preflight(
                    &plan.session,
                    &plan.metadata,
                    inputs.com,
                    &short_velocity,
                    &data.output,
                    WORLDS as u32,
                )
                .is_err()
        );
        let proof = plan.metadata.cfrc_proof(WORLDS as u32).unwrap();
        let mut alias =
            RneCfrcKernel::bindings(&plan.metadata, inputs.com, inputs.velocity, &data.output)
                .unwrap();
        let output_bytes = rne_cfrc_lengths(proof.dims()).unwrap()[3] * 4;
        alias[3].span = alias[1].span.view(0, output_bytes).unwrap();
        assert!(matches!(
            rne_cfrc_plan(proof.dims(), &alias, &[proof.binding(0)]),
            Err(LauncherError::Alias {
                first: 1,
                second: 3
            })
        ));
        (raw, typed)
    };
    assert_eq!(
        raw.iter().map(|value| value.to_bits()).collect::<Vec<_>>(),
        typed
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    );
    for world in 0..WORLDS {
        for lane in 0..6 * plan.nbody {
            assert_ne!(
                raw[4 + world * width + cfrc_offset + lane].to_bits(),
                sentinel.to_bits()
            );
        }
    }

    let sentinel_workspace = |data: &mut RneData, seed: f32| {
        let mut values = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut values).unwrap();
        let end = values.len() - 4;
        for (index, value) in values[4..end].iter_mut().enumerate() {
            *value = seed + (index % 31) as f32 * 0.125;
        }
        data.output.write_range(0, &values).unwrap();
        values
    };
    plan.update_rne(&mut data, true).unwrap();
    data.readback().unwrap();
    let before = sentinel_workspace(&mut data, -4096.0);
    plan.metadata.overwrite_for_test(0, 1).unwrap();
    assert!(plan.update_rne(&mut data, true).is_err());
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady { stage: "rne" })
    ));
    let mut rejected = vec![0.0; data.output.len()];
    data.output.read_range_into(0, &mut rejected).unwrap();
    assert_eq!(
        before
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        rejected
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    );
    plan.metadata.restore_for_test().unwrap();
    plan.update_rne(&mut data, true).unwrap();
    data.readback().unwrap();

    let before_session = sentinel_workspace(&mut data, -8192.0);
    let foreign_session = TransferSession::new(0).unwrap();
    let capture = std::env::var_os("MJWARP_PHYSICS_PTX_DIR");
    if let Some(foreign) = crate::runtime::capture_child(module_path!(), "foreign-session") {
        assert!(!foreign.exists());
        // SAFETY: 本测试单线程运行，并在编译后立即恢复环境。
        unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &foreign) };
    }
    let foreign_kernel = RneCfrcKernel::compile(&foreign_session).unwrap();
    if let Some(root) = capture.as_ref() {
        // SAFETY: 本测试单线程运行，并恢复调用方的原始环境。
        unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
    } else {
        // SAFETY: 本测试单线程运行，调用前也没有该变量。
        unsafe { std::env::remove_var("MJWARP_PHYSICS_PTX_DIR") };
    }
    let correct_kernel = std::mem::replace(&mut plan.cfrc_kernel, foreign_kernel);
    assert!(matches!(
        plan.update_rne(&mut data, true),
        Err(TransferError::SessionMismatch)
    ));
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady { stage: "rne" })
    ));
    let mut rejected_session = vec![0.0; data.output.len()];
    data.output
        .read_range_into(0, &mut rejected_session)
        .unwrap();
    assert_eq!(
        before_session
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        rejected_session
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    );
    plan.cfrc_kernel = correct_kernel;
    plan.update_rne(&mut data, true).unwrap();
    data.readback().unwrap();

    let zero = super::super::tests::test_model(None);
    let zero = AttachedModelInput::new(zero, crate::model::AttachedFields::default()).unwrap();
    if let Some(zero_capture) = crate::runtime::capture_child(module_path!(), "zero-dof") {
        assert!(!zero_capture.exists());
        // SAFETY: 本测试单线程运行，并在编译后立即恢复环境。
        unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &zero_capture) };
    }
    let zero_plan = RnePlan::new(&session, zero, [0.0; 3]).unwrap();
    if let Some(root) = capture.as_ref() {
        // SAFETY: 本测试单线程运行，并恢复调用方的原始环境。
        unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
    }
    let mut zero_data = zero_plan.create_data(2).unwrap();
    zero_plan.update(&mut zero_data, false).unwrap();
    let zero_output = zero_data.readback().unwrap();
    for world in 0..2 {
        assert!(zero_output.world(world).unwrap().qfrc_bias.is_empty());
    }
}
