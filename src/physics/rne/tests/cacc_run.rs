//! RNE加速度运行测试。

#[cfg(feature = "cuda-probe")]
use super::kernels::RneCaccKernel;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::rne_cacc_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::rne_cacc_plan;
#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn runs_rne_cacc_wrapper_and_recovers_after_real_metadata_write() {
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
    let mut plan = RnePlan::new(&session, model, [0.0, 0.0, -9.81]).unwrap();
    assert_eq!((plan.nbody, plan.nv), (6, 11));
    const WORLDS: usize = 513;
    let mut data = plan.create_data(WORLDS).unwrap();
    let qvel: Vec<_> = (0..WORLDS * plan.nv)
        .map(|index| (index as f32 - 2821.0) * 0.000_125)
        .collect();
    let qacc: Vec<_> = (0..WORLDS * plan.nv)
        .map(|index| (2821.0 - index as f32) * 0.000_25)
        .collect();
    data.write_qvel(&qvel).unwrap();
    data.write_qacc(&qacc).unwrap();
    plan.update_velocity(&mut data).unwrap();
    let width = 12 * plan.nbody + plan.nv;
    let cacc_width = 6 * plan.nbody;
    let sentinel = -777.25f32;
    let mut initial = vec![sentinel; data.output.len()];
    initial[..4].fill(-131072.0);
    let end = initial.len() - 4;
    initial[end..].fill(-131072.0);

    for flg_acc in [false, true] {
        let (raw, typed) = {
            let inputs = data.velocity.rne_inputs().unwrap();
            data.output.write_range(0, &initial).unwrap();
            // SAFETY: 复用产品计划的同一实际缓冲和冻结私有ABI。
            unsafe {
                plan.cacc_kernel.kernel.launch_rne_cacc(
                    plan.metadata.as_buffer(),
                    (inputs.qvel, &data.qacc, inputs.com, inputs.velocity),
                    &mut data.output,
                    flg_acc,
                    WORLDS as u32,
                )
            }
            .unwrap();
            let mut raw = vec![0.0; data.output.len()];
            data.output.read_range_into(0, &mut raw).unwrap();

            data.output.write_range(0, &initial).unwrap();
            let launch = plan
                .cacc_kernel
                .preflight(
                    &plan.session,
                    &plan.metadata,
                    inputs.qvel,
                    &data.qacc,
                    inputs.com,
                    inputs.velocity,
                    &data.output,
                    flg_acc,
                    WORLDS as u32,
                )
                .unwrap();
            plan.cacc_kernel
                .launch(
                    &launch,
                    &plan.metadata,
                    inputs.qvel,
                    &data.qacc,
                    inputs.com,
                    inputs.velocity,
                    &mut data.output,
                )
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
                raw[start..start + cacc_width]
                    .iter()
                    .all(|value| value.to_bits() != sentinel.to_bits())
            );
            assert!(
                raw[start + cacc_width..start + width]
                    .iter()
                    .all(|value| value.to_bits() == sentinel.to_bits())
            );
        }
    }

    {
        let inputs = data.velocity.rne_inputs().unwrap();
        let short_qacc = session.upload(&vec![0.0; data.qacc.len() - 1]).unwrap();
        assert!(
            plan.cacc_kernel
                .preflight(
                    &plan.session,
                    &plan.metadata,
                    inputs.qvel,
                    &short_qacc,
                    inputs.com,
                    inputs.velocity,
                    &data.output,
                    true,
                    WORLDS as u32,
                )
                .is_err()
        );
        let proof = plan.metadata.cacc_proof(true, WORLDS as u32).unwrap();
        let mut alias = RneCaccKernel::bindings(
            &plan.metadata,
            inputs.qvel,
            &data.qacc,
            inputs.com,
            inputs.velocity,
            &data.output,
        )
        .unwrap();
        let output_bytes = rne_cacc_lengths(proof.dims()).unwrap()[5] * 4;
        alias[5].span = alias[3].span.view(0, output_bytes).unwrap();
        assert!(matches!(
            rne_cacc_plan(proof.dims(), &alias, &[proof.binding(0)]),
            Err(LauncherError::Alias {
                first: 3,
                second: 5
            })
        ));
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
    if let Some(root) = capture.as_ref() {
        let foreign = std::path::PathBuf::from(root).join("foreign-session");
        assert!(!foreign.exists());
        // SAFETY: 本测试单线程运行，并在编译后立即恢复环境。
        unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &foreign) };
    }
    let foreign_kernel = RneCaccKernel::compile(&foreign_session).unwrap();
    if let Some(root) = capture.as_ref() {
        // SAFETY: 本测试单线程运行，并恢复调用方的原始环境。
        unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
    } else {
        // SAFETY: 本测试单线程运行，调用前也没有该变量。
        unsafe { std::env::remove_var("MJWARP_PHYSICS_PTX_DIR") };
    }
    let correct_kernel = std::mem::replace(&mut plan.cacc_kernel, foreign_kernel);
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
    plan.cacc_kernel = correct_kernel;
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
