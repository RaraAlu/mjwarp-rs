//! 混合求解校验元数据与类型化计划测试。

use super::tests_support::*;
#[cfg(any(feature = "cuda-probe", test))]
use super::validation_meta::MixedSolveValidationLaunchMetadata;
#[cfg(any(feature = "cuda-probe", test))]
use super::validation_meta::mixed_solve_validation_lengths;
#[cfg(any(feature = "cuda-probe", test))]
use super::validation_meta::mixed_solve_validation_plan;
use super::*;

#[test]
fn typed_mixed_solve_validation_plan_freezes_exact_contract() {
    use crate::runtime::launcher::DeviceSpan;

    let dims = [3, 10, 2, 257];
    let values = validation_metadata();
    let metadata_elements = values.len() as u64;
    let (metadata_min, rhs, factor, status) = mixed_solve_validation_lengths(&dims).unwrap();
    assert_eq!((metadata_min, rhs, factor, status), (10, 1550, 2578, 265));
    let metadata_span = DeviceSpan::new(7, 0x1000, metadata_elements * 4).unwrap();
    let revision = MetadataRevision::new();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let proof = unsafe {
        Checked::<MixedSolveValidationLaunchMetadata>::assume_uploaded(
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
            span: DeviceSpan::new(7, 0x5000, factor * 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: DeviceSpan::new(7, 0x9000, status * 4).unwrap(),
            elem: ElemType::F32,
        },
    ];
    let launch = mixed_solve_validation_plan(
        [3, 10],
        metadata_elements,
        &dims,
        &bindings,
        proof.binding(0),
    )
    .unwrap();
    assert_eq!(launch.grid(), (2, 1, 1));
    assert_eq!(launch.block(), (256, 1, 1));

    for (slot, elements) in [(0, metadata_elements), (1, rhs), (2, factor), (3, status)] {
        for bytes in [elements * 4 - 4, elements * 4 + 4] {
            let mut changed = bindings;
            changed[slot].span = DeviceSpan::new(7, changed[slot].span.addr(), bytes).unwrap();
            assert!(
                mixed_solve_validation_plan(
                    [3, 10],
                    metadata_elements,
                    &dims,
                    &changed,
                    proof.binding(0),
                )
                .is_err()
            );
        }
    }

    let mut alias = bindings;
    alias[3].span = alias[1].span.view(0, status * 4).unwrap();
    assert!(matches!(
        mixed_solve_validation_plan([3, 10], metadata_elements, &dims, &alias, proof.binding(0),),
        Err(LauncherError::Alias {
            first: 1,
            second: 3,
        })
    ));
    let mut foreign = bindings;
    foreign[2].span = DeviceSpan::new(8, 0x5000, factor * 4).unwrap();
    assert!(matches!(
        mixed_solve_validation_plan(
            [3, 10],
            metadata_elements,
            &dims,
            &foreign,
            proof.binding(0),
        ),
        Err(LauncherError::ContextMismatch { slot: 2 })
    ));
    let mut replaced = bindings;
    replaced[0].span = DeviceSpan::new(7, 0xa000, metadata_elements * 4).unwrap();
    assert!(matches!(
        mixed_solve_validation_plan(
            [3, 10],
            metadata_elements,
            &dims,
            &replaced,
            proof.binding(0),
        ),
        Err(LauncherError::MetadataMismatch(_))
    ));
    assert!(
        mixed_solve_validation_plan([3, 10], u64::MAX, &dims, &bindings, proof.binding(0),)
            .is_err()
    );
    assert!(mixed_solve_validation_lengths(&[1, 1, 1, u32::MAX - 254]).is_err());
    assert!(matches!(
        mixed_solve_validation_lengths(&[u32::MAX, u32::MAX, u32::MAX, u32::MAX - 255,]),
        Err(LauncherError::Overflow(_))
    ));
    assert!(
        mixed_solve_validation_plan(
            [4, 10],
            metadata_elements,
            &dims,
            &bindings,
            proof.binding(0),
        )
        .is_err()
    );
    revision.invalidate();
    assert!(matches!(
        mixed_solve_validation_plan(
            [3, 10],
            metadata_elements,
            &dims,
            &bindings,
            proof.binding(0),
        ),
        Err(LauncherError::StaleMetadata)
    ));
}
