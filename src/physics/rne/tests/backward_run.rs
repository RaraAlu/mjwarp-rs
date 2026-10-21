//! RNE反向包装运行与恢复测试。

#[cfg(feature = "cuda-probe")]
use super::kernels::RneBackwardKernel;
#[cfg(feature = "cuda-probe")]
#[cfg(feature = "cuda-probe")]
use super::kernels::RneBackwardRequest;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::rne_backward_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_plans::rne_backward_plan;
#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn runs_rne_backward_wrapper_in_reverse_levels_and_recovers() {
    let session = TransferSession::new(0).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("fixtures/rne-bias/mixed-joints.json")).unwrap(),
    )
    .unwrap();
    let model = reference_model(&fixture);
    let mut plan = RnePlan::new(&session, model, [0.0, 0.0, -9.81]).unwrap();
    assert_eq!((plan.nbody, plan.nv, plan.depth), (6, 11, 4));
    let parents = [0usize, 0, 1, 2, 3, 0];
    let depths = [0usize, 1, 2, 3, 4, 1];
    assert_eq!(
        plan.metadata.values[5..5 + plan.nbody],
        parents.map(|value| value as i32)
    );
    const WORLDS: usize = 43;
    let mut data = plan.create_data(WORLDS).unwrap();
    data.write_qvel(&vec![0.0; WORLDS * plan.nv]).unwrap();
    plan.update_velocity(&mut data).unwrap();

    let width = 12 * plan.nbody + plan.nv;
    let mut initial = vec![0.0; data.output.len()];
    initial[..4].fill(-131072.0);
    let guard_end = initial.len() - 4;
    initial[guard_end..].fill(-131072.0);
    for world in 0..WORLDS {
        let force = 4 + world * width + 6 * plan.nbody;
        for body in 0..plan.nbody {
            for lane in 0..6 {
                initial[force + 6 * body + lane] = (world * 1_000 + body * 16 + lane + 1) as f32;
            }
        }
    }
    let mut expected = initial.clone();
    for level in (1..=plan.depth).rev() {
        for body in 1..plan.nbody {
            if depths[body] != level {
                continue;
            }
            for world in 0..WORLDS {
                let force = 4 + world * width + 6 * plan.nbody;
                for lane in 0..6 {
                    let child = expected[force + 6 * body + lane];
                    expected[force + 6 * parents[body] + lane] += child;
                }
            }
        }
    }

    let bits = |values: &[f32]| {
        values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    };
    let (raw, typed) = {
        let inputs = data.velocity.rne_inputs().unwrap();
        data.output.write_range(0, &initial).unwrap();
        // SAFETY: 复用产品计划的真实缓冲和冻结私有ABI。
        unsafe {
            for level in (1..=plan.depth).rev() {
                plan.backward_kernel
                    .kernel
                    .launch(
                        plan.metadata.as_buffer(),
                        inputs.com,
                        inputs.velocity,
                        &mut data.output,
                        [
                            plan.nbody as u32,
                            level as u32,
                            0,
                            (WORLDS * plan.nbody) as u32,
                        ],
                    )
                    .unwrap();
            }
        }
        let mut raw = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut raw).unwrap();

        data.output.write_range(0, &initial).unwrap();
        let mut launches = Vec::with_capacity(plan.depth);
        for level in (1..=plan.depth).rev() {
            launches.push(
                plan.backward_kernel
                    .preflight(
                        &plan.session,
                        &plan.metadata,
                        inputs.com,
                        inputs.velocity,
                        &data.output,
                        RneBackwardRequest {
                            worlds: WORLDS as u32,
                            level: level as u32,
                        },
                    )
                    .unwrap(),
            );
        }
        for launch in &launches {
            plan.backward_kernel
                .launch(
                    launch,
                    &plan.metadata,
                    inputs.com,
                    inputs.velocity,
                    &mut data.output,
                )
                .unwrap();
        }
        let mut typed = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut typed).unwrap();

        let short_com = session.upload(&vec![0.0; inputs.com.len() - 1]).unwrap();
        assert!(
            plan.backward_kernel
                .preflight(
                    &plan.session,
                    &plan.metadata,
                    &short_com,
                    inputs.velocity,
                    &data.output,
                    RneBackwardRequest {
                        worlds: WORLDS as u32,
                        level: plan.depth as u32,
                    },
                )
                .is_err()
        );
        let proof = plan
            .metadata
            .backward_proof(WORLDS as u32, plan.depth as u32)
            .unwrap();
        let mut alias =
            RneBackwardKernel::bindings(&plan.metadata, inputs.com, inputs.velocity, &data.output)
                .unwrap();
        let output_bytes = rne_backward_lengths(proof.dims()).unwrap()[3] * 4;
        alias[3].span = alias[1].span.view(0, output_bytes).unwrap();
        assert!(matches!(
            rne_backward_plan(proof.dims(), &alias, &[proof.binding(0)]),
            Err(LauncherError::Alias {
                first: 1,
                second: 3
            })
        ));

        let stale_launch = plan
            .backward_kernel
            .preflight(
                &plan.session,
                &plan.metadata,
                inputs.com,
                inputs.velocity,
                &data.output,
                RneBackwardRequest {
                    worlds: WORLDS as u32,
                    level: plan.depth as u32,
                },
            )
            .unwrap();
        let mut before_stale = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut before_stale).unwrap();
        plan.metadata.overwrite_for_test(0, 1).unwrap();
        assert!(
            plan.backward_kernel
                .launch(
                    &stale_launch,
                    &plan.metadata,
                    inputs.com,
                    inputs.velocity,
                    &mut data.output,
                )
                .is_err()
        );
        let mut rejected_stale = vec![0.0; data.output.len()];
        data.output.read_range_into(0, &mut rejected_stale).unwrap();
        assert_eq!(bits(&before_stale), bits(&rejected_stale));
        plan.metadata.restore_for_test().unwrap();
        let recovered_launch = plan
            .backward_kernel
            .preflight(
                &plan.session,
                &plan.metadata,
                inputs.com,
                inputs.velocity,
                &data.output,
                RneBackwardRequest {
                    worlds: WORLDS as u32,
                    level: plan.depth as u32,
                },
            )
            .unwrap();
        data.output.write_range(0, &initial).unwrap();
        plan.backward_kernel
            .launch(
                &recovered_launch,
                &plan.metadata,
                inputs.com,
                inputs.velocity,
                &mut data.output,
            )
            .unwrap();
        (raw, typed)
    };
    assert_eq!(bits(&raw), bits(&typed));
    assert_eq!(bits(&raw), bits(&expected));

    plan.update_rne(&mut data, true).unwrap();
    data.readback().unwrap();
    let mut before = vec![0.0; data.output.len()];
    before[..4].fill(-131072.0);
    let end = before.len() - 4;
    before[end..].fill(-131072.0);
    for (index, value) in before[4..end].iter_mut().enumerate() {
        *value = -4096.0 + (index % 31) as f32 * 0.125;
    }
    data.output.write_range(0, &before).unwrap();

    let foreign_session = TransferSession::new(0).unwrap();
    let capture = std::env::var_os("MJWARP_PHYSICS_PTX_DIR");
    if let Some(root) = capture.as_ref() {
        let foreign = std::path::PathBuf::from(root).join("foreign-session");
        assert!(!foreign.exists());
        // SAFETY: 本测试单线程运行，并在编译后立即恢复环境。
        unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &foreign) };
    }
    let foreign_kernel = RneBackwardKernel::compile(&foreign_session).unwrap();
    if let Some(root) = capture.as_ref() {
        // SAFETY: 本测试单线程运行，并恢复调用方的原始环境。
        unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
    } else {
        // SAFETY: 本测试单线程运行，调用前也没有该变量。
        unsafe { std::env::remove_var("MJWARP_PHYSICS_PTX_DIR") };
    }
    let correct_kernel = std::mem::replace(&mut plan.backward_kernel, foreign_kernel);
    assert!(matches!(
        plan.update_rne(&mut data, true),
        Err(TransferError::SessionMismatch)
    ));
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady { stage: "rne" })
    ));
    let mut rejected = vec![0.0; data.output.len()];
    data.output.read_range_into(0, &mut rejected).unwrap();
    assert_eq!(bits(&before), bits(&rejected));
    let foreign_kernel = std::mem::replace(&mut plan.backward_kernel, correct_kernel);
    plan.update_rne(&mut data, true).unwrap();
    data.readback().unwrap();

    use crate::model::{InertialFields, InertialModelInput, KinematicFields, KinematicModelInput};
    let root_model = KinematicModelInput::new(
        0,
        KinematicFields {
            body_parentid: vec![0],
            body_jntadr: vec![-1],
            body_jntnum: vec![0],
            body_pos: vec![0.0; 3],
            body_quat: vec![1.0, 0.0, 0.0, 0.0],
            ..Default::default()
        },
    )
    .unwrap();
    let root_model = InertialModelInput::new(
        root_model,
        InertialFields {
            body_ipos: vec![0.0; 3],
            body_iquat: vec![1.0, 0.0, 0.0, 0.0],
            body_mass: vec![0.0],
            body_inertia: vec![0.0; 3],
            ..Default::default()
        },
    )
    .unwrap();
    let root_model =
        AttachedModelInput::new(root_model, crate::model::AttachedFields::default()).unwrap();
    if let Some(root) = capture.as_ref() {
        let root_only = std::path::PathBuf::from(root).join("root-only");
        assert!(!root_only.exists());
        // SAFETY: 本测试单线程运行，并在编译后立即恢复环境。
        unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &root_only) };
    }
    let mut root_plan = RnePlan::new(&session, root_model, [0.0; 3]).unwrap();
    if let Some(root) = capture.as_ref() {
        // SAFETY: 本测试单线程运行，并恢复调用方的原始环境。
        unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
    } else {
        // SAFETY: 本测试单线程运行，调用前也没有该变量。
        unsafe { std::env::remove_var("MJWARP_PHYSICS_PTX_DIR") };
    }
    assert_eq!(root_plan.depth, 0);
    root_plan.backward_kernel = foreign_kernel;
    let mut root_data = root_plan.create_data(2).unwrap();
    root_plan.update(&mut root_data, false).unwrap();
    root_data.readback().unwrap();
}
