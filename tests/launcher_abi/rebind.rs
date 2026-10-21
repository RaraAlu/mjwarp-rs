//! 值槽重绑测试。

use mjwarp_rs::runtime::launcher::{AbiSignature, LauncherError, rebind_values, validate_bindings};

use super::*;

#[test]
fn rebinds_value_slots_and_returns_full_binding_set() {
    let signature = AbiSignature::of::<MixedSolve128>().unwrap();
    let current = mixed_bindings();
    let update = bind(3, span(7, 0x8_0000, 8192), ElemType::F32);
    let next = rebind_values(&signature, &current, &[update], &requirements()).unwrap();
    assert_eq!(next.len(), current.len());
    assert_eq!(next[3], update);
    assert_eq!(next[..3], current[..3]);
}

#[test]
fn rebind_rejects_address_slots_shrink_type_context_and_duplicates() {
    let signature = AbiSignature::of::<MixedSolve128>().unwrap();
    let current = mixed_bindings();
    let cases = [
        (
            vec![bind(0, span(7, 0x8_0000, 256), ElemType::I32)],
            LauncherError::AddressSlotRebind { slot: 0 },
        ),
        (
            vec![bind(3, span(7, 0x8_0000, 2048), ElemType::F32)],
            LauncherError::RebindShrink { slot: 3 },
        ),
        (
            vec![bind(3, span(7, 0x8_0000, 8192), ElemType::U32)],
            LauncherError::ElementMismatch { slot: 3 },
        ),
        (
            vec![bind(3, span(9, 0x8_0000, 8192), ElemType::F32)],
            LauncherError::ContextMismatch { slot: 3 },
        ),
        (
            vec![bind(4, span(7, 0x8_0000, 4), ElemType::U32)],
            LauncherError::UnexpectedBinding { slot: 4 },
        ),
        (
            vec![
                bind(2, span(7, 0x8_0000, 4096), ElemType::F32),
                bind(2, span(7, 0x9_0000, 4096), ElemType::F32),
            ],
            LauncherError::DuplicateBinding { slot: 2 },
        ),
    ];
    for (updates, expected) in cases {
        assert_eq!(
            rebind_values(&signature, &current, &updates, &[]),
            Err(expected)
        );
    }
}

#[test]
fn rebind_checks_untouched_slots_jointly_and_keeps_current() {
    let signature = AbiSignature::of::<MixedSolve128>().unwrap();
    let current = mixed_bindings();
    let snapshot = current.clone();
    // 新输出与未更新的因子重叠。
    let update = bind(3, current[2].span, ElemType::F32);
    assert_eq!(
        rebind_values(&signature, &current, &[update], &[]),
        Err(LauncherError::Alias {
            first: 2,
            second: 3,
        })
    );
    // 两个各自合法的更新组合后重叠。
    let updates = [
        bind(1, span(7, 0xA_0000, 4096), ElemType::F32),
        bind(3, span(7, 0xA_0800, 4096), ElemType::F32),
    ];
    assert_eq!(
        rebind_values(&signature, &current, &updates, &[]),
        Err(LauncherError::Alias {
            first: 1,
            second: 3,
        })
    );
    // 容量要求覆盖重绑结果。
    let short = [SlotRequirement {
        slot: 1,
        min_elements: 2048,
    }];
    let update = bind(1, span(7, 0xB_0000, 4096), ElemType::F32);
    assert!(matches!(
        rebind_values(&signature, &current, &[update], &short),
        Err(LauncherError::Capacity { slot: 1, .. })
    ));
    assert_eq!(current, snapshot);
}

#[test]
fn warp_address_slot_is_also_frozen_against_rebind() {
    let signature = AbiSignature::of::<WarpExample>().unwrap();
    let current = vec![
        bind(1, span(1, 0x1000, 64), ElemType::I32),
        bind(2, span(1, 0x2000, 1024), ElemType::F32),
        bind(3, span(1, 0x3000, 1024), ElemType::F32),
    ];
    validate_bindings(&signature, &current, &[]).unwrap();
    let update = bind(1, span(1, 0x4000, 64), ElemType::I32);
    assert_eq!(
        rebind_values(&signature, &current, &[update], &[]),
        Err(LauncherError::AddressSlotRebind { slot: 1 })
    );
    let update = bind(2, span(1, 0x5000, 1024), ElemType::F32);
    rebind_values(&signature, &current, &[update], &[]).unwrap();
}
