//! RNE广义力运行、守卫与冻结对照测试。

#[cfg(feature = "cuda-probe")]
use super::kernels::RneQfrcKernel;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::rne_qfrc_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::rne_qfrc_plan;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn runs_rne_qfrc_wrapper_and_recovers_after_real_metadata_write() {
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
    plan.update(&mut data, true).unwrap();
    let mut initial = vec![0.0; data.output.len()];
    data.output.read_range_into(0, &mut initial).unwrap();
    let width = 12 * plan.nbody + plan.nv;
    let qfrc_offset = 12 * plan.nbody;
    let sentinel = -777.25f32;
    for world in 0..WORLDS {
        initial[4 + world * width + qfrc_offset] = sentinel;
    }

    let (raw, typed) = {
        let inputs = data.velocity.rne_inputs().unwrap();
        data.output.write_range(0, &initial).unwrap();
        // SAFETY: 复用产品计划的同一实际缓冲和冻结私有ABI。
        unsafe {
            plan.qfrc_kernel.kernel.launch(
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

        data.output.write_range(0, &initial).unwrap();
        let launch = plan
            .qfrc_kernel
            .preflight(
                &plan.session,
                &plan.metadata,
                inputs.com,
                inputs.velocity,
                &data.output,
                WORLDS as u32,
            )
            .unwrap();
        plan.qfrc_kernel
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

        let short_com = session.upload(&vec![0.0; inputs.com.len() - 1]).unwrap();
        assert!(
            plan.qfrc_kernel
                .preflight(
                    &plan.session,
                    &plan.metadata,
                    &short_com,
                    inputs.velocity,
                    &data.output,
                    WORLDS as u32,
                )
                .is_err()
        );
        let proof = plan.metadata.qfrc_proof(WORLDS as u32).unwrap();
        let mut alias =
            RneQfrcKernel::bindings(&plan.metadata, inputs.com, inputs.velocity, &data.output)
                .unwrap();
        let output_bytes = rne_qfrc_lengths(proof.dims()).unwrap()[3] * 4;
        alias[3].span = alias[1].span.view(0, output_bytes).unwrap();
        assert!(matches!(
            rne_qfrc_plan(proof.dims(), &alias, &[proof.binding(0)]),
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
        assert_ne!(
            raw[4 + world * width + qfrc_offset].to_bits(),
            sentinel.to_bits()
        );
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
        before.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        rejected.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
    );
    plan.metadata.restore_for_test().unwrap();
    plan.update_rne(&mut data, true).unwrap();
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
    let foreign_kernel = RneQfrcKernel::compile(&foreign_session).unwrap();
    if let Some(root) = capture.as_ref() {
        // SAFETY: 本测试单线程运行，并恢复调用方的原始环境。
        unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
    } else {
        // SAFETY: 本测试单线程运行，调用前也没有该变量。
        unsafe { std::env::remove_var("MJWARP_PHYSICS_PTX_DIR") };
    }
    let correct_kernel = std::mem::replace(&mut plan.qfrc_kernel, foreign_kernel);
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
    plan.qfrc_kernel = correct_kernel;
    plan.update_rne(&mut data, true).unwrap();
    data.readback().unwrap();

    let zero = super::super::tests::test_model(None);
    let zero = AttachedModelInput::new(zero, crate::model::AttachedFields::default()).unwrap();
    if let Some(root) = capture.as_ref() {
        let zero_capture = std::path::PathBuf::from(root).join("zero-dof");
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

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn checks_acceleration_guards_and_unpublished_nonfinite_workspaces() {
    let model = super::super::tests::test_model(Some(3));
    let model = AttachedModelInput::new(model, crate::model::AttachedFields::default()).unwrap();
    let session = TransferSession::new(0).unwrap();
    let plan = RnePlan::new(&session, model, [0.0; 3]).unwrap();
    let mut data = plan.create_data(2).unwrap();
    data.write_qacc(&[0.25, -0.5]).unwrap();
    plan.update(&mut data, true).unwrap();
    for index in [0, data.layout.guarded - 1] {
        data.output.write_range(index, &[0.0]).unwrap();
        assert!(data.readback().is_err());
        data.output.write_range(index, &[-131072.0]).unwrap();
    }
    for index in [4, 4 + 6 * data.layout.nbody, 4 + 12 * data.layout.nbody] {
        data.output.write_range(index, &[f32::NAN]).unwrap();
        assert!(data.readback().is_err());
        plan.update_rne(&mut data, true).unwrap();
    }
    for index in [0, data.layout.qacc.total_elements() + 7] {
        data.qacc.write_range(index, &[0.0]).unwrap();
        assert!(data.readback().is_err());
        data.qacc.write_range(index, &[-131072.0]).unwrap();
    }
    data.qacc.write_range(4, &[f32::NAN]).unwrap();
    assert!(data.readback().is_err());
    data.write_world_qacc(0, &[0.25]).unwrap();
    plan.update_rne(&mut data, true).unwrap();
    data.readback().unwrap();
}

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires NVIDIA driver and pinned frozen smooth PTX"]
fn diagnoses_new_qacc_stage_against_frozen_ptx_on_identical_device_inputs() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let base: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("fixtures/rne-bias/mixed-joints.json")).unwrap(),
    )
    .unwrap();
    let v: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("fixtures/rne-acceleration/mixed-joints.json")).unwrap(),
    )
    .unwrap();
    let model = crate::physics::rne::tests::reference_model(&base);
    let session = TransferSession::new(0).unwrap();
    let mut rows = Vec::new();
    for (state, g, gravity) in [(3, 0, [0.0, 0.0, -9.81]), (7, 2, [1.25, -2.0, 3.5])] {
        let plan = RnePlan::new(&session, model.clone(), gravity).unwrap();
        let mut data = plan.create_data(1).unwrap();
        let values = |key: &str| {
            v["cases"][state][key]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| n.as_f64().unwrap() as f32)
                .collect::<Vec<_>>()
        };
        data.write_qpos(&values("qpos")).unwrap();
        data.write_qvel(&values("qvel")).unwrap();
        data.write_qacc(&values("qacc")).unwrap();
        plan.update_velocity(&mut data).unwrap();
        for flag in [false, true] {
            plan.update_rne(&mut data, flag).unwrap();
            let candidate = data.readback().unwrap();
            let inputs = data.velocity.rne_inputs().unwrap();
            let mut m = vec![0; 5 + 4 * plan.nbody + plan.nv];
            plan.metadata
                .as_buffer()
                .read_range_into(0, &mut m)
                .unwrap();
            let mut qvel = vec![0.0; plan.nv];
            let mut qacc = vec![0.0; plan.nv];
            let mut com = vec![0.0; 14 * plan.nbody + 6 * plan.nv];
            let mut velocity = vec![0.0; 6 * plan.nbody + 6 * plan.nv];
            inputs.qvel.read_range_into(4, &mut qvel).unwrap();
            data.qacc.read_range_into(4, &mut qacc).unwrap();
            inputs.com.read_range_into(4, &mut com).unwrap();
            inputs.velocity.read_range_into(4, &mut velocity).unwrap();
            let frozen = frozen::run(&m, &qvel, &qacc, &com, &velocity, flag);
            let actual = &candidate.values[4..candidate.values.len() - 4];
            let differences: Vec<_> = actual.iter().zip(&frozen).enumerate().filter(|(_, (a, b))| a.to_bits() != b.to_bits())
                .map(|(i, (&a, &b))| serde_json::json!({"index":i,"candidate":a,"frozen_stage":b,"candidate_bits":a.to_bits(),"frozen_bits":b.to_bits()})).collect();
            let native = &v["cases"][state]["qfrc"][usize::from(flag)][g];
            rows.push(serde_json::json!({"state":state,"gravity":g,"flag":flag,"differences":differences,"candidate_qfrc":candidate.world(0).unwrap().qfrc_bias,
                "frozen_stage_qfrc":&frozen[12 * plan.nbody..],"native_qfrc":native}));
        }
    }
    let result = serde_json::json!({"scope":"new qacc first-difference; frozen RNE consumes identical product-prefix f32 inputs; not full upstream pipeline or admission", "model":"mixed-joints","rows":rows});
    std::fs::write(
        root.join("target/rne-acceleration-validation/frozen-first-difference.json"),
        serde_json::to_string_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("new qacc frozen stage diagnostic: {}", result);
    assert!(
        rows.iter()
            .all(|row| row["differences"].as_array().unwrap().is_empty()),
        "local RNE stage bit differences remain"
    );
}
