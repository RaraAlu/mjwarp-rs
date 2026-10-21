//! 混合求解融合元数据与类型化计划测试。

#[cfg(any(feature = "cuda-probe", test))]
use super::fused_meta::MixedSolveFusedDims;
#[cfg(any(feature = "cuda-probe", test))]
use super::fused_meta::MixedSolveFusedLaunchMetadata;
#[cfg(any(feature = "cuda-probe", test))]
use super::fused_meta::StaticMixedSolveFusedMetadata;
#[cfg(any(feature = "cuda-probe", test))]
use super::fused_spec::mixed_solve_fused_groups;
#[cfg(any(feature = "cuda-probe", test))]
use super::fused_spec::mixed_solve_fused_lengths;
#[cfg(any(feature = "cuda-probe", test))]
use super::fused_spec::mixed_solve_fused_plan;
#[cfg(any(feature = "cuda-probe", test))]
use super::layout::metadata;
use super::tests_support::*;
use super::*;

#[test]
fn mixed_solve_fused_metadata_proves_routes_levels_and_small_regions() {
    let values = fused_metadata();
    assert!(StaticMixedSolveFusedMetadata::validate(&values, &[4, 10]).is_ok());
    assert!(
        StaticMixedSolveFusedMetadata::validate(&values[..values.len() - 1], &[4, 10]).is_err()
    );
    let mut long = values.clone();
    long.push(0);
    assert!(StaticMixedSolveFusedMetadata::validate(&long, &[4, 10]).is_err());

    let crossed = vec![
        1, 0, 2, 0, -1, -1, -1, -1, 0, 0, 0, 0, 0, 2, 0, 1, 0, 1, 2, 1,
    ];
    // 两条更新各自满足顺序。
    // 它们在索引1发生交叉。
    assert!(StaticMixedSolveFusedMetadata::validate(&crossed, &[4, 6]).is_err());
    let mut bad_factor = values.clone();
    bad_factor[16] = 2;
    assert!(StaticMixedSolveFusedMetadata::validate(&bad_factor, &[4, 10]).is_err());
    let mut bad_small = values.clone();
    bad_small[18] = 3;
    assert!(StaticMixedSolveFusedMetadata::validate(&bad_small, &[4, 10]).is_err());
    let mut bad_route = values.clone();
    bad_route[5] = 0;
    assert!(StaticMixedSolveFusedMetadata::validate(&bad_route, &[4, 10]).is_err());
    let mut reversed = values.clone();
    reversed[14] = 1;
    reversed[15] = 0;
    assert!(StaticMixedSolveFusedMetadata::validate(&reversed, &[4, 10]).is_err());
    let mut compact_endpoint = values.clone();
    compact_endpoint[5] = -2;
    assert!(StaticMixedSolveFusedMetadata::validate(&compact_endpoint, &[4, 10]).is_err());

    let same_target = vec![
        1, 4, 2, 1, -1, -1, 0, 0, 6, 7, 2, 3, 0, 2, 0, 1, 0, 0, 1, 1, 2, 2, 0,
    ];
    assert!(StaticMixedSolveFusedMetadata::validate(&same_target, &[4, 10]).is_ok());

    let placeholder_only = vec![0, 0, 2, 0, -1, -2, 2, 3, 0, 0, 0, 0];
    assert!(StaticMixedSolveFusedMetadata::validate(&placeholder_only, &[2, 4]).is_ok());
    let placeholder_before_small = vec![0, 4, 2, 1, -1, -2, 0, 0, 6, 7, 2, 3, 0, 0, 0, 0, 2, 2, 0];
    assert!(StaticMixedSolveFusedMetadata::validate(&placeholder_before_small, &[4, 10]).is_err());

    let input = input();
    let actual = metadata(&input).unwrap();
    let stride = input.qld_block_total() + input.nc() + input.nv();
    assert!(
        StaticMixedSolveFusedMetadata::validate(&actual, &[input.nv() as u32, stride as u32],)
            .is_ok()
    );
}
#[test]
fn typed_mixed_solve_fused_plan_freezes_exact_contract() {
    use crate::runtime::launcher::DeviceSpan;

    let values = fused_metadata();
    let dims = MixedSolveFusedDims {
        abi: [4, 10, 129, 2],
        metadata_elements: values.len() as u64,
    };
    let (metadata_len, rhs, factor, output) = mixed_solve_fused_lengths(&dims).unwrap();
    assert_eq!((metadata_len, rhs, factor, output), (20, 1040, 28, 1040));
    let metadata_span = DeviceSpan::new(7, 0x1000, metadata_len * 4).unwrap();
    let revision = MetadataRevision::new();
    // SAFETY: 测试跨度代表同一份完整宿主快照。
    let proof = unsafe {
        Checked::<MixedSolveFusedLaunchMetadata>::assume_uploaded(
            metadata_span,
            &values,
            dims,
            revision.clone(),
        )
    }
    .unwrap();
    let bindings = [
        SlotBinding {
            slot: 0,
            span: metadata_span,
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: DeviceSpan::new(7, 0x2000, rhs * 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: DeviceSpan::new(7, 0x4000, factor * 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: DeviceSpan::new(7, 0x6000, output * 4).unwrap(),
            elem: ElemType::F32,
        },
    ];
    let launch = mixed_solve_fused_plan([4, 10], &dims, &bindings, proof.binding(0)).unwrap();
    assert_eq!(launch.grid(), (258, 1, 1));
    assert_eq!(launch.block(), (128, 1, 1));
    assert_eq!(launch.shared_bytes(), 0);

    for (slot, elements) in [metadata_len, rhs, factor, output].into_iter().enumerate() {
        for bytes in [elements * 4 - 4, elements * 4 + 4] {
            let mut changed = bindings;
            changed[slot].span = DeviceSpan::new(7, changed[slot].span.addr(), bytes).unwrap();
            assert!(mixed_solve_fused_plan([4, 10], &dims, &changed, proof.binding(0)).is_err());
        }
    }
    let mut alias = bindings;
    alias[3].span = alias[1].span;
    assert!(matches!(
        mixed_solve_fused_plan([4, 10], &dims, &alias, proof.binding(0)),
        Err(LauncherError::Alias {
            first: 1,
            second: 3
        })
    ));
    let mut foreign = bindings;
    foreign[2].span = DeviceSpan::new(8, 0x4000, factor * 4).unwrap();
    assert!(matches!(
        mixed_solve_fused_plan([4, 10], &dims, &foreign, proof.binding(0)),
        Err(LauncherError::ContextMismatch { slot: 2 })
    ));
    assert!(
        mixed_solve_fused_groups(&MixedSolveFusedDims {
            abi: [4, 10, 65536, 65536],
            metadata_elements: metadata_len,
        })
        .is_err()
    );
    assert!(
        mixed_solve_fused_groups(&MixedSolveFusedDims {
            abi: [4, 10, 1, i32::MAX as u32 + 1],
            metadata_elements: metadata_len,
        })
        .is_err()
    );
    revision.invalidate();
    assert!(matches!(
        mixed_solve_fused_plan([4, 10], &dims, &bindings, proof.binding(0)),
        Err(LauncherError::StaleMetadata)
    ));
}
