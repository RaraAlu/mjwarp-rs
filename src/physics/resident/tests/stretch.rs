//! 拉伸Hessian与柔体位置测试。

#[cfg(feature = "cuda-probe")]
use super::output::read_flex_validity;
#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn stretch_hessian_checks_analytic_compression_and_partial_cache() {
    let session = TransferSession::new(0).unwrap();
    let plan = stretch_plan(&session);
    let mut data = plan.create_data(513).unwrap();
    plan.update(&mut data).unwrap();
    let vertices = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0].repeat(2);
    let lengths = [2.0f32.sqrt(), 1.0, 1.0].repeat(2);
    let expected_diag = [
        2.0, 0.0, 0.0, 2.0, 0.0, 0.0, 4.0, -2.0, 0.0, 2.0, 0.0, 0.0, 2.0, -2.0, 0.0, 4.0, 0.0, 0.0,
    ];
    let expected_edge = [
        -2.0, 2.0, 0.0, 2.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -2.0, 0.0, 0.0, 0.0,
        0.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ];
    for scale in [1.0, 0.5] {
        for w in 0..513 {
            data.flex
                .as_mut()
                .unwrap()
                .write_range(
                    4 + 18 * w,
                    &vertices.iter().map(|v| v * scale).collect::<Vec<_>>(),
                )
                .unwrap();
            data.edges
                .as_mut()
                .unwrap()
                .values
                .write_range(
                    4 + 12 * w,
                    &lengths.iter().map(|v| v * scale).collect::<Vec<_>>(),
                )
                .unwrap();
        }
        data.flex_validity
            .as_mut()
            .unwrap()
            .write_range(4, &vec![0; 1026])
            .unwrap();
        plan.update_flex_hessian(&mut data).unwrap();
        let out = data.readback().unwrap();
        for w in [0, 256, 512] {
            let h = out.flex_hessian().unwrap().world(w).unwrap();
            for (actual, expected) in h.flexvert_hessian.iter().zip(expected_diag.repeat(2)) {
                assert!((actual - expected * scale * scale).abs() < 2e-6);
            }
            for (actual, expected) in h.flexedge_hessian.iter().zip(expected_edge.repeat(2)) {
                assert!((actual - expected * scale * scale).abs() < 2e-6);
            }
        }
    }
    let width = data.layout.hessian.unwrap().output.elements_per_world();
    let payload = vec![37.0; 513 * width];
    data.hessian
        .as_mut()
        .unwrap()
        .write_range(4, &payload)
        .unwrap();
    let flags: Vec<i32> = (0..1026).map(|i| i % 2).collect();
    data.flex_validity
        .as_mut()
        .unwrap()
        .write_range(4, &flags)
        .unwrap();
    plan.update_flex_hessian(&mut data).unwrap();
    let out = data.readback().unwrap();
    for w in 0..513 {
        let h = out.flex_hessian().unwrap().world(w).unwrap();
        assert!(
            h.flexvert_hessian[18..]
                .iter()
                .chain(&h.flexedge_hessian[27..])
                .all(|&v| v == 37.0)
        );
        assert!(
            h.flexvert_hessian[..18]
                .iter()
                .chain(&h.flexedge_hessian[..27])
                .all(|&v| v != 37.0)
        );
    }
    // A valid cache ignores even private geometry changes; G01 clears it.
    data.flex
        .as_mut()
        .unwrap()
        .write_range(4, &[7.0; 18])
        .unwrap();
    plan.update_flex_hessian(&mut data).unwrap();
    assert_eq!(
        out.flex_hessian().unwrap().values,
        data.readback().unwrap().flex_hessian().unwrap().values
    );
    plan.update(&mut data).unwrap();
    assert!(data.readback().unwrap().flex_hessian().is_none());
    plan.update_flex_hessian(&mut data).unwrap();
    assert!(!data.readback().unwrap().flex_hessian().unwrap().values[4..4 + width].contains(&37.0));
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn stretch_hessian_rejects_guards_nonfinite_and_late_publish_failure() {
    let session = TransferSession::new(0).unwrap();
    let foreign = TransferSession::new(0).unwrap();
    let mut plan = stretch_plan(&session);
    let mut data = plan.create_data(513).unwrap();
    plan.update(&mut data).unwrap();
    plan.update_flex_hessian(&mut data).unwrap();
    let len = data.layout.hessian.unwrap().guarded;
    for index in (0..4).chain(len - 4..len) {
        data.hessian
            .as_mut()
            .unwrap()
            .write_range(index, &[0.0])
            .unwrap();
        assert!(data.readback().is_err());
        data.hessian
            .as_mut()
            .unwrap()
            .write_range(index, &[-131072.0])
            .unwrap();
    }
    for index in [4, 4 + 90 * 256, len - 5] {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            data.hessian
                .as_mut()
                .unwrap()
                .write_range(index, &[value])
                .unwrap();
            assert!(matches!(
                data.readback(),
                Err(TransferError::Input(InputError::NonFinite {
                    field: "resident_flex_hessian_output",
                    ..
                }))
            ));
            plan.update(&mut data).unwrap();
            plan.update_flex_hessian(&mut data).unwrap();
            data.readback().unwrap();
        }
    }
    for publish in [false, true] {
        plan.update(&mut data).unwrap();
        let replacement = SynchronousKernel::compile(
            &foreign,
            super::super::flex_hessian::FLEX_HESSIAN_CUDA,
            if publish {
                "flex_hessian_validate"
            } else {
                "flex_hessian"
            },
        )
        .unwrap();
        let device = plan.hessian.as_mut().unwrap();
        let slot = if publish {
            &mut device.validate_kernel
        } else {
            &mut device.kernel
        };
        let original = std::mem::replace(slot, replacement);
        assert_eq!(
            plan.update_flex_hessian(&mut data),
            Err(TransferError::SessionMismatch)
        );
        assert!(data.readback().unwrap().flex_hessian().is_none());
        assert!(
            read_flex_validity(data.flex_validity.as_ref().unwrap(), 1034)
                .unwrap()
                .iter()
                .all(|&v| !v)
        );
        let device = plan.hessian.as_mut().unwrap();
        *if publish {
            &mut device.validate_kernel
        } else {
            &mut device.kernel
        } = original;
        plan.update_flex_hessian(&mut data).unwrap();
        data.readback().unwrap();
    }
    // Finite positions/edge lengths can still overflow material arithmetic.
    data.flex
        .as_mut()
        .unwrap()
        .write_range(4, &[f32::MAX, -f32::MAX, 0.0, 0.0, f32::MAX, 0.0].repeat(3))
        .unwrap();
    data.edges
        .as_mut()
        .unwrap()
        .values
        .write_range(4, &[f32::MAX; 6])
        .unwrap();
    data.flex_validity
        .as_mut()
        .unwrap()
        .write_range(4, &[0; 2])
        .unwrap();
    plan.update_flex_hessian(&mut data).unwrap();
    assert!(data.readback().is_err());
    let mixed = flex_hessian_plan(&session);
    let mut state = mixed.create_data(2).unwrap();
    mixed.update(&mut state).unwrap();
    mixed.update_flex_hessian(&mut state).unwrap();
    assert!(state.readback().unwrap().flex_faces().is_some());
    assert!(state.readback().unwrap().sleep_trees().is_some());
    assert_eq!(
        state
            .readback()
            .unwrap()
            .flex_positions()
            .unwrap()
            .world(1)
            .unwrap()
            .flex_hessian_valid,
        &[true; 3]
    );
}
