//! 混合因子单元测试。

use super::merge::{mixed_factor_merge_lengths, mixed_factor_merge_plan};
use super::*;
use crate as engine;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::TransferError;
#[cfg(feature = "cuda-probe")]
use crate::runtime::TransferSession;
use crate::runtime::launcher::{ElemType, LauncherError, SlotBinding};

mod input {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/support/small_mass_prefix.rs"
    ));
}
#[test]
fn mixed_routes_require_sparse_tail_and_keep_mathdx_rejected() {
    routes(&input::mixed_input()).unwrap();
    assert!(routes(&input::sparse(&input::base()["models"][3])).is_err());
    assert!(routes(&SparseMassInput::new(Default::default()).unwrap()).is_err());
    assert_eq!(input::mixed_matrix(0).len(), input::mixed_input().nc());
    assert_eq!(input::analytic()["cases"].as_array().unwrap().len(), 2);
}
#[test]
fn merged_world_layout_keeps_prefix_tail_and_inverse_distinct() {
    let input = Arc::new(input::mixed_input());
    let layout = layout(&input, 2).unwrap();
    assert_eq!(layout.elements_per_world(), 2377);
    assert!(super::layout(&input, 0).is_err());
    assert!(super::layout(&input, usize::MAX).is_err());
    let output = SparseMassMixedFactorOutput {
        input,
        layout,
        values: (0..layout.total_elements() + 8).map(|i| i as f32).collect(),
    };
    let world = output.world(1).unwrap();
    assert_eq!(
        (
            world.dense_upper.len(),
            world.sparse_factors.len(),
            world.diagonal_inverse.len()
        ),
        (90, 2201, 86)
    );
    assert_eq!(world.dense_upper[0], 2381.0);
    assert_eq!(world.sparse_factors[0], 2471.0);
    assert_eq!(world.diagonal_inverse[0], 4672.0);
    assert!(output.world(2).is_err());
}
#[test]
fn mixed_factor_merge_plan_checks_dims_spans_sessions_and_aliases() {
    use crate::runtime::launcher::DeviceSpan;

    let bind = |slot, context, addr, bytes, elem| SlotBinding {
        slot,
        span: DeviceSpan::new(context, addr, bytes).unwrap(),
        elem,
    };
    let dims = [2, 3, 4, 18];
    assert_eq!(mixed_factor_merge_lengths(&dims).unwrap(), [1, 12, 22, 26]);
    let bindings = [
        bind(0, 7, 0x1000, 4, ElemType::I32),
        bind(1, 7, 0x2000, 48, ElemType::F32),
        bind(2, 7, 0x3000, 88, ElemType::F32),
        bind(3, 7, 0x4000, 104, ElemType::F32),
    ];
    let launch = mixed_factor_merge_plan([2, 3, 4], &dims, &bindings).unwrap();
    assert_eq!(launch.grid(), (1, 1, 1));
    assert_eq!(launch.block(), (256, 1, 1));
    assert_eq!(launch.shared_bytes(), 0);
    assert!(mixed_factor_merge_plan([1, 3, 4], &dims, &bindings).is_err());

    let addresses = [0x1000, 0x2000, 0x3000, 0x4000];
    let bytes = [4, 48, 88, 104];
    for slot in 0..4 {
        let mut short = bindings;
        short[slot].span = DeviceSpan::new(
            7,
            addresses[slot],
            if bytes[slot] == 4 { 1 } else { bytes[slot] - 4 },
        )
        .unwrap();
        assert!(mixed_factor_merge_plan([2, 3, 4], &dims, &short).is_err());

        let mut long = bindings;
        long[slot].span = DeviceSpan::new(7, addresses[slot], bytes[slot] + 4).unwrap();
        assert!(mixed_factor_merge_plan([2, 3, 4], &dims, &long).is_err());
    }

    let mut foreign = bindings;
    foreign[1].span = DeviceSpan::new(8, 0x2000, 48).unwrap();
    assert!(matches!(
        mixed_factor_merge_plan([2, 3, 4], &dims, &foreign),
        Err(LauncherError::ContextMismatch { slot: 1 })
    ));

    let alias_dims = [0, 3, 4, 14];
    assert_eq!(
        mixed_factor_merge_lengths(&alias_dims).unwrap(),
        [1, 8, 22, 22]
    );
    let zero_prefix = [
        bind(0, 7, 0x1000, 4, ElemType::I32),
        bind(1, 7, 0x2000, 32, ElemType::F32),
        bind(2, 7, 0x3000, 88, ElemType::F32),
        bind(3, 7, 0x4000, 88, ElemType::F32),
    ];
    mixed_factor_merge_plan([0, 3, 4], &alias_dims, &zero_prefix).unwrap();
    let mut alias = zero_prefix;
    alias[3].span = alias[2].span;
    assert!(matches!(
        mixed_factor_merge_plan([0, 3, 4], &alias_dims, &alias),
        Err(LauncherError::Alias {
            first: 2,
            second: 3
        })
    ));
    assert!(mixed_factor_merge_lengths(&[2, 3, 4, 0]).is_err());
    assert!(mixed_factor_merge_lengths(&[2, 3, 4, 17]).is_err());
    assert!(mixed_factor_merge_lengths(&[2, 3, 4, u32::MAX - 254]).is_err());
    assert!(mixed_factor_merge_lengths(&[0, u32::MAX, 1, 1]).is_err());
    assert!(mixed_factor_merge_lengths(&[i32::MAX as u32, 1, 1, i32::MAX as u32 + 2]).is_err());
    assert!(mixed_factor_merge_lengths(&[u32::MAX, 3, 4, 1]).is_err());
}

#[cfg(feature = "cuda-probe")]
fn assert_merge_matches_producers(data: &SparseMassMixedFactorData) {
    let prefix = data.sparse.input.qld_block_total();
    let tail = data.sparse.input.nc() + data.sparse.input.nv();
    let mut small = vec![0.0; data.small.factor_buffer().unwrap().len()];
    let mut sparse = vec![0.0; data.sparse.output.len()];
    let mut merged = vec![0.0; data.output.len()];
    data.small
        .factor_buffer()
        .unwrap()
        .read_range_into(0, &mut small)
        .unwrap();
    data.sparse.output.read_range_into(0, &mut sparse).unwrap();
    data.output.read_range_into(0, &mut merged).unwrap();
    for guard in merged[..4].iter().chain(&merged[merged.len() - 4..]) {
        assert_eq!(guard.to_bits(), (-131072.0f32).to_bits());
    }
    for world in 0..data.worlds() {
        let merged_start = 4 + world * (prefix + tail);
        let small_start = 4 + world * prefix;
        let sparse_start = 4 + world * tail;
        for (actual, expected) in merged[merged_start..merged_start + prefix]
            .iter()
            .zip(&small[small_start..small_start + prefix])
        {
            assert_eq!(actual.to_bits(), expected.to_bits());
        }
        for (actual, expected) in merged[merged_start + prefix..merged_start + prefix + tail]
            .iter()
            .zip(&sparse[sparse_start..sparse_start + tail])
        {
            assert_eq!(actual.to_bits(), expected.to_bits());
        }
    }
}

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new mixed main-CSR/merged-output guarded cache failure purpose"]
fn mixed_device_guards_and_cached_stage_failures_reject_and_recover() {
    let session = TransferSession::new(0).unwrap();
    let plan = SparseMassMixedFactorPlan::new(&session, input::mixed_input()).unwrap();
    let mut data = plan.create_data(2).unwrap();
    data.write_matrix(&input::mixed_matrix(0).repeat(2))
        .unwrap();
    plan.update(&mut data).unwrap();
    assert_merge_matches_producers(&data);
    let foreign = TransferSession::new(0).unwrap();
    let foreign_small = foreign
        .upload(&vec![0.0f32; data.small.factor_buffer().unwrap().len()])
        .unwrap();
    let worlds = data.worlds();
    assert!(matches!(
        plan.merge.launch(
            &plan.metadata,
            &foreign_small,
            &data.sparse.output,
            &mut data.output,
            worlds,
        ),
        Err(TransferError::SessionMismatch)
    ));
    assert_merge_matches_producers(&data);
    // Poisoning old main M cannot affect either captured producer. A fresh
    // gather must detect it, instead of publishing a cross-generation mix.
    let before = data.readback().unwrap();
    let mut saved = [0.0];
    data.matrix.read_range_into(4, &mut saved).unwrap();
    data.matrix.write_range(4, &[f32::NAN]).unwrap();
    plan.update_factor(&mut data).unwrap();
    let cached = data.readback().unwrap();
    for w in 0..2 {
        assert_eq!(
            cached.world(w).unwrap().dense_upper,
            before.world(w).unwrap().dense_upper
        );
        for (&a, &b) in cached
            .world(w)
            .unwrap()
            .sparse_factors
            .iter()
            .zip(before.world(w).unwrap().sparse_factors)
        {
            assert!((f64::from(a) - f64::from(b)).abs() <= 2e-4 + 2e-4 * f64::from(b).abs());
        }
    }
    assert!(plan.update_gather(&mut data).is_err());
    assert!(!data.ready);
    assert!(data.readback().is_err());
    data.matrix.write_range(4, &saved).unwrap();
    plan.update(&mut data).unwrap();
    assert_merge_matches_producers(&data);
    let end = 4 + data.sparse.matrix_layout.total_elements();
    for offset in [0, end, 4] {
        let mut saved = [0.0];
        data.matrix.read_range_into(offset, &mut saved).unwrap();
        data.matrix.write_range(offset, &[f32::NAN]).unwrap();
        assert!(plan.update_gather(&mut data).is_err());
        assert!(plan.update_factor(&mut data).is_err());
        assert!(!data.ready);
        assert!(data.readback().is_err());
        data.matrix.write_range(offset, &saved).unwrap();
        plan.update(&mut data).unwrap();
        assert_merge_matches_producers(&data);
    }
    let end = 4 + data.layout.total_elements();
    for offset in [0, end] {
        data.output.write_range(offset, &[0.0]).unwrap();
        assert!(data.readback().is_err());
        assert!(plan.update_factor(&mut data).is_err());
        assert!(data.readback().is_err());
        data.output.write_range(offset, &[-131072.0]).unwrap();
        plan.update_factor(&mut data).unwrap();
        assert_merge_matches_producers(&data);
    }
    for (offset, value) in [(4, f32::NAN), (4, -1.0), (4 + 90 + 2201, -1.0)] {
        data.output.write_range(offset, &[value]).unwrap();
        assert!(data.readback().is_err());
        plan.update_factor(&mut data).unwrap();
        assert!(data.readback().is_ok());
    }
    // Cached-factor publication must still validate both actual producer
    // buffers; it cannot trust a good-looking merged snapshot alone.
    data.sparse.output.write_range(0, &[0.0]).unwrap();
    assert!(data.readback().is_err());
    data.sparse.output.write_range(0, &[-131072.0]).unwrap();
    assert!(data.readback().is_ok());
    for offset in [0, 4 + data.small.factor_buffer().unwrap().len() - 8] {
        data.small
            .test_factor_buffer_mut()
            .write_range(offset, &[0.0])
            .unwrap();
        assert!(data.readback().is_err());
        data.small
            .test_factor_buffer_mut()
            .write_range(offset, &[-131072.0])
            .unwrap();
        assert!(data.readback().is_ok());
    }
    // Poison captured Sparse M, not the primary old source. Publication
    // must reject this producer's corrupted snapshot independently.
    data.sparse.matrix.write_range(4, &[f32::NAN]).unwrap();
    assert!(plan.update_factor(&mut data).is_err());
    assert!(!data.ready);
    assert!(data.readback().is_err());
    plan.update(&mut data).unwrap();
    assert_merge_matches_producers(&data);
    let mut raw = vec![0.0; data.matrix.len()];
    data.matrix.read_range_into(0, &mut raw).unwrap();
    data.write_world_matrix(1, &input::mixed_matrix(1)).unwrap();
    let mut next = vec![0.0; raw.len()];
    data.matrix.read_range_into(0, &mut next).unwrap();
    for i in (0..4 + 2201).chain(4 + 2 * 2201..raw.len()) {
        assert_eq!(raw[i].to_bits(), next[i].to_bits());
    }
    plan.update(&mut data).unwrap();
    assert_merge_matches_producers(&data);
    assert!(data.readback().is_ok());
}
