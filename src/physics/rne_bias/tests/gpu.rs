//! 偏置力GPU包装与守卫测试。

#[cfg(feature = "cuda-probe")]
use super::metadata::rne_bias_lengths;
#[cfg(feature = "cuda-probe")]
use super::spec::rne_bias_plan;
#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn runs_rne_bias_wrapper_and_recovers_after_real_metadata_write() {
    let session = TransferSession::new(0).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("fixtures/rne-bias/mixed-joints.json")).unwrap(),
    )
    .unwrap();
    let model = reference_model(&fixture);
    assert_eq!(
        model.rigid().kinematics().fields().body_parentid,
        [0, 0, 1, 2, 3, 0]
    );
    let mut plan = RneBiasPlan::new(&session, model, [0.0, 0.0, -9.81]).unwrap();
    assert_eq!((plan.nbody, plan.nv), (6, 11));
    const WORLDS: usize = 513;
    let mut data = plan.create_data(WORLDS).unwrap();
    let qvel: Vec<_> = (0..WORLDS * plan.nv)
        .map(|index| (index as f32 - 2821.0) * 0.000_125)
        .collect();
    data.write_qvel(&qvel).unwrap();
    plan.update_velocity(&mut data).unwrap();
    let width = 12 * plan.nbody + plan.nv;
    let sentinel = -777.25f32;
    let mut initial = vec![sentinel; data.output.len()];
    initial[..4].fill(-131072.0);
    let end = initial.len() - 4;
    initial[end..].fill(-131072.0);

    let (raw, typed) = {
        let velocity_inputs = data.velocity.rne_inputs().unwrap();
        let inputs = RneBiasInputs {
            metadata: &plan.metadata,
            qvel: velocity_inputs.qvel,
            com: velocity_inputs.com,
            velocity: velocity_inputs.velocity,
        };
        data.output.write_range(0, &initial).unwrap();
        // SAFETY: 复用产品计划的同一实际缓冲和冻结私有ABI。
        unsafe {
            plan.kernel.kernel.launch_flex_edges(
                inputs.metadata.as_buffer(),
                inputs.qvel,
                inputs.com,
                inputs.velocity,
                &mut data.output,
                WORLDS as u32,
            )
        }
        .unwrap();
        let mut raw = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut raw).unwrap();

        data.output.write_range(0, &initial).unwrap();
        let launch = plan
            .kernel
            .preflight(&plan.session, &inputs, &data.output, WORLDS as u32)
            .unwrap();
        plan.kernel
            .launch(&launch, &inputs, &mut data.output)
            .unwrap();
        let mut typed = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut typed).unwrap();
        (raw, typed)
    };
    assert_eq!(
        raw.iter().map(|value| value.to_bits()).collect::<Vec<_>>(),
        typed
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    );
    assert_eq!(&raw[..4], &initial[..4]);
    assert_eq!(&raw[raw.len() - 4..], &initial[initial.len() - 4..]);
    for world in 0..WORLDS {
        let start = 4 + world * width;
        assert!(
            raw[start..start + width]
                .iter()
                .all(|value| value.to_bits() != sentinel.to_bits())
        );
    }

    {
        let velocity_inputs = data.velocity.rne_inputs().unwrap();
        let short_qvel = session
            .upload(&vec![0.0; velocity_inputs.qvel.len() - 1])
            .unwrap();
        let short_inputs = RneBiasInputs {
            metadata: &plan.metadata,
            qvel: &short_qvel,
            com: velocity_inputs.com,
            velocity: velocity_inputs.velocity,
        };
        assert!(
            plan.kernel
                .preflight(&plan.session, &short_inputs, &data.output, WORLDS as u32)
                .is_err()
        );
        let proof = plan.metadata.proof(WORLDS as u32).unwrap();
        let inputs = RneBiasInputs {
            metadata: &plan.metadata,
            qvel: velocity_inputs.qvel,
            com: velocity_inputs.com,
            velocity: velocity_inputs.velocity,
        };
        let mut alias = RneBiasKernel::bindings(&inputs, &data.output).unwrap();
        let output_bytes = rne_bias_lengths(proof.dims()).unwrap()[4] * 4;
        alias[4].span = alias[2].span.view(0, output_bytes).unwrap();
        assert!(matches!(
            rne_bias_plan(proof.dims(), &alias, &[proof.binding(0)]),
            Err(LauncherError::Alias {
                first: 2,
                second: 4
            })
        ));
    }

    let sentinel_workspace = |data: &mut RneBiasData, seed: f32| {
        let mut values = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut values).unwrap();
        let end = values.len() - 4;
        for (index, value) in values[4..end].iter_mut().enumerate() {
            *value = seed + (index % 31) as f32 * 0.125;
        }
        data.output.write_range(0, &values).unwrap();
        values
    };
    plan.update_bias(&mut data).unwrap();
    data.readback().unwrap();
    let before = sentinel_workspace(&mut data, -4096.0);
    plan.metadata.overwrite_for_test(0, 1).unwrap();
    assert!(plan.update_bias(&mut data).is_err());
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady { stage: "rne_bias" })
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
    plan.update_bias(&mut data).unwrap();
    data.readback().unwrap();

    let before_session = sentinel_workspace(&mut data, -8192.0);
    let foreign_session = TransferSession::new(0).unwrap();
    let capture = std::env::var_os("MJWARP_PHYSICS_PTX_DIR");
    if let Some(root) = capture.as_ref() {
        let foreign = std::path::PathBuf::from(root).join("foreign-session");
        assert!(!foreign.exists());
        // SAFETY: 本测试单线程运行，并在编译后立即恢复环境。
        unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &foreign) };
    }
    let foreign_kernel = RneBiasKernel::compile(&foreign_session).unwrap();
    if let Some(root) = capture.as_ref() {
        // SAFETY: 本测试单线程运行，并恢复调用方的原始环境。
        unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
    } else {
        // SAFETY: 本测试单线程运行，调用前也没有该变量。
        unsafe { std::env::remove_var("MJWARP_PHYSICS_PTX_DIR") };
    }
    let correct_kernel = std::mem::replace(&mut plan.kernel, foreign_kernel);
    assert!(matches!(
        plan.update_bias(&mut data),
        Err(TransferError::SessionMismatch)
    ));
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady { stage: "rne_bias" })
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
    plan.kernel = correct_kernel;
    plan.update_bias(&mut data).unwrap();
    data.readback().unwrap();

    let zero = super::super::tests::test_model(None);
    let zero = AttachedModelInput::new(zero, crate::model::AttachedFields::default()).unwrap();
    if let Some(root) = capture.as_ref() {
        let zero_capture = std::path::PathBuf::from(root).join("zero-dof");
        assert!(!zero_capture.exists());
        // SAFETY: 本测试单线程运行，并在编译后立即恢复环境。
        unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &zero_capture) };
    }
    let zero_plan = RneBiasPlan::new(&session, zero, [0.0; 3]).unwrap();
    if let Some(root) = capture.as_ref() {
        // SAFETY: 本测试单线程运行，并恢复调用方的原始环境。
        unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
    }
    let mut zero_data = zero_plan.create_data(2).unwrap();
    zero_plan.update(&mut zero_data).unwrap();
    let zero_output = zero_data.readback().unwrap();
    for world in 0..2 {
        assert!(zero_output.world(world).unwrap().qfrc_bias.is_empty());
    }
}

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn bias_readback_rejects_guards_nonfinite_workspaces_and_results() {
    let m = super::super::tests::test_model(Some(3));
    let m = AttachedModelInput::new(m, crate::model::AttachedFields::default()).unwrap();
    let s = TransferSession::new(0).unwrap();
    let p = RneBiasPlan::new(&s, m, [0.0; 3]).unwrap();
    let mut d = p.create_data(1).unwrap();
    p.update(&mut d).unwrap();
    d.readback().unwrap();
    for i in [0, d.layout.guarded - 1] {
        d.output.write_range(i, &[0.0]).unwrap();
        assert!(d.readback().is_err());
        d.output.write_range(i, &[-131072.0]).unwrap();
    }
    for i in [4, 4 + 6 * d.layout.nbody, 4 + 12 * d.layout.nbody] {
        d.output.write_range(i, &[f32::NAN]).unwrap();
        assert!(d.readback().is_err());
        p.update_bias(&mut d).unwrap();
        d.readback().unwrap();
    }
    d.output
        .write_range(4 + 12 * d.layout.nbody, &[f32::INFINITY])
        .unwrap();
    assert!(d.readback().is_err());
    p.update_bias(&mut d).unwrap();
    d.readback().unwrap();
}
