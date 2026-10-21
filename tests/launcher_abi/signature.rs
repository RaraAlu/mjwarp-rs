//! 签名、绑定与别名校验测试。

use mjwarp_rs::runtime::launcher::{AbiSignature, LauncherError, validate_bindings};

use super::*;

#[test]
fn accepts_private_warp_and_in_place_signatures() {
    let mixed = AbiSignature::of::<MixedSolve128>().unwrap();
    assert_eq!(mixed.block().threads(), 128);
    assert_eq!(mixed.slots().len(), 8);
    let warp = AbiSignature::of::<WarpExample>().unwrap();
    assert_eq!(warp.block().threads(), 256);
    let in_place = AbiSignature::of::<InPlace>().unwrap();
    assert!(in_place.allows_overlap(1, 0));
    assert!(!mixed.allows_overlap(1, 3));
}

#[test]
fn rejects_invalid_signature_declarations() {
    const BAD_INDEX: &[SlotDesc] = &[ptr(1, ElemType::F32, Access::Read, SlotRole::Value)];
    const WRITE_SCALAR: &[SlotDesc] = &[SlotDesc {
        index: 0,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Write,
        role: SlotRole::Value,
    }];
    const ODD_SCALAR: &[SlotDesc] = &[SlotDesc {
        index: 0,
        kind: SlotKind::Scalar { bytes: 3 },
        access: Access::Read,
        role: SlotRole::Value,
    }];
    const WRITE_ADDRESS: &[SlotDesc] = &[ptr(0, ElemType::I32, Access::Write, META)];
    const BOUNDS_LATE: &[SlotDesc] = &[
        ptr(0, ElemType::F32, Access::Read, SlotRole::Value),
        SlotDesc {
            index: 1,
            kind: SlotKind::Bounds { dims: 2 },
            access: Access::Read,
            role: SlotRole::Value,
        },
    ];
    const WARP_NO_BOUNDS: &[SlotDesc] = &[SlotDesc {
        index: 0,
        kind: SlotKind::WarpArrayData {
            elem: ElemType::F32,
            ndim: 1,
        },
        access: Access::Read,
        role: SlotRole::Value,
    }];
    const MIXED_FAMILY: &[SlotDesc] = &[
        SlotDesc {
            index: 0,
            kind: SlotKind::Bounds { dims: 1 },
            access: Access::Read,
            role: SlotRole::Value,
        },
        SlotDesc {
            index: 1,
            kind: SlotKind::WarpArrayData {
                elem: ElemType::F32,
                ndim: 1,
            },
            access: Access::Read,
            role: SlotRole::Value,
        },
        ptr(2, ElemType::F32, Access::Write, SlotRole::Value),
    ];
    const DEEP_ARRAY: &[SlotDesc] = &[
        SlotDesc {
            index: 0,
            kind: SlotKind::Bounds { dims: 1 },
            access: Access::Read,
            role: SlotRole::Value,
        },
        SlotDesc {
            index: 1,
            kind: SlotKind::WarpArrayData {
                elem: ElemType::F32,
                ndim: 5,
            },
            access: Access::Read,
            role: SlotRole::Value,
        },
    ];
    const OK: &[SlotDesc] = MixedSolve128::SLOTS;
    let none: &[AliasException] = &[];
    let cases: [(&[SlotDesc], BlockRule); 10] = [
        (&[], BlockRule::WarpGrid),
        (BAD_INDEX, BlockRule::WarpGrid),
        (WRITE_SCALAR, BlockRule::WarpGrid),
        (ODD_SCALAR, BlockRule::WarpGrid),
        (WRITE_ADDRESS, BlockRule::WarpGrid),
        (BOUNDS_LATE, BlockRule::WarpGrid),
        (WARP_NO_BOUNDS, BlockRule::WarpGrid),
        (MIXED_FAMILY, BlockRule::WarpGrid),
        (DEEP_ARRAY, BlockRule::WarpGrid),
        (OK, BlockRule::Fixed(100)),
    ];
    for (slots, block) in cases {
        let error = AbiSignature::check("bad", block, slots, none).unwrap_err();
        assert!(matches!(
            error,
            LauncherError::InvalidSignature { abi: "bad", .. }
        ));
    }
    for block in [BlockRule::Fixed(0), BlockRule::Fixed(2048)] {
        assert!(AbiSignature::check("bad", block, OK, none).is_err());
    }
    let exceptions: [&[AliasException]; 4] = [
        // 地址类槽位不得放宽。
        &[AliasException {
            first: 0,
            second: 3,
        }],
        // 须升序。
        &[AliasException {
            first: 3,
            second: 1,
        }],
        // 不得引用标量。
        &[AliasException {
            first: 1,
            second: 4,
        }],
        &[
            AliasException {
                first: 1,
                second: 3,
            },
            AliasException {
                first: 1,
                second: 3,
            },
        ],
    ];
    for exception in exceptions {
        assert!(AbiSignature::check("bad", BlockRule::Fixed(128), OK, exception).is_err());
    }
}

#[test]
fn accepts_complete_disjoint_bindings() {
    let signature = AbiSignature::of::<MixedSolve128>().unwrap();
    validate_bindings(&signature, &mixed_bindings(), &requirements()).unwrap();
}

#[test]
fn rejects_missing_duplicate_and_unexpected_bindings() {
    let signature = AbiSignature::of::<MixedSolve128>().unwrap();
    let mut missing = mixed_bindings();
    missing.remove(2);
    assert_eq!(
        validate_bindings(&signature, &missing, &[]),
        Err(LauncherError::MissingBinding { slot: 2 })
    );
    let mut duplicate = mixed_bindings();
    duplicate.push(duplicate[1]);
    assert_eq!(
        validate_bindings(&signature, &duplicate, &[]),
        Err(LauncherError::DuplicateBinding { slot: 1 })
    );
    let mut scalar = mixed_bindings();
    scalar.push(bind(4, span(7, 0x5_0000, 4), ElemType::U32));
    assert_eq!(
        validate_bindings(&signature, &scalar, &[]),
        Err(LauncherError::UnexpectedBinding { slot: 4 })
    );
    let mut beyond = mixed_bindings();
    beyond.push(bind(9, span(7, 0x5_0000, 4), ElemType::F32));
    assert_eq!(
        validate_bindings(&signature, &beyond, &[]),
        Err(LauncherError::UnexpectedBinding { slot: 9 })
    );
}

#[test]
fn rejects_element_context_and_alignment_mismatches() {
    let signature = AbiSignature::of::<MixedSolve128>().unwrap();
    let mut element = mixed_bindings();
    element[1].elem = ElemType::F64;
    assert_eq!(
        validate_bindings(&signature, &element, &[]),
        Err(LauncherError::ElementMismatch { slot: 1 })
    );
    let mut context = mixed_bindings();
    context[2].span = span(8, 0x3_0000, 4096);
    assert_eq!(
        validate_bindings(&signature, &context, &[]),
        Err(LauncherError::ContextMismatch { slot: 2 })
    );
    let mut aligned = mixed_bindings();
    aligned[3].span = span(7, 0x4_0002, 4096);
    assert_eq!(
        validate_bindings(&signature, &aligned, &[]),
        Err(LauncherError::Misaligned { slot: 3 })
    );
}

#[test]
fn rejects_short_capacity_and_overflowing_requirements() {
    let signature = AbiSignature::of::<MixedSolve128>().unwrap();
    let short = [SlotRequirement {
        slot: 3,
        min_elements: 1025,
    }];
    assert_eq!(
        validate_bindings(&signature, &mixed_bindings(), &short),
        Err(LauncherError::Capacity {
            slot: 3,
            required: 4100,
            actual: 4096,
        })
    );
    let overflow = [SlotRequirement {
        slot: 1,
        min_elements: u64::MAX / 2,
    }];
    assert_eq!(
        validate_bindings(&signature, &mixed_bindings(), &overflow),
        Err(LauncherError::Overflow("slot_requirement_bytes"))
    );
    let scalar = [SlotRequirement {
        slot: 4,
        min_elements: 1,
    }];
    assert_eq!(
        validate_bindings(&signature, &mixed_bindings(), &scalar),
        Err(LauncherError::UnexpectedBinding { slot: 4 })
    );
}

#[test]
fn writable_slots_are_exclusive_but_reads_may_share() {
    let signature = AbiSignature::of::<MixedSolve128>().unwrap();
    // 两个只读槽共享同一区间。
    let mut shared = mixed_bindings();
    shared[2].span = shared[1].span;
    validate_bindings(&signature, &shared, &[]).unwrap();
    // 输出与右端项部分重叠。
    let mut overlap = mixed_bindings();
    overlap[3].span = span(7, 0x2_0800, 4096);
    assert_eq!(
        validate_bindings(&signature, &overlap, &[]),
        Err(LauncherError::Alias {
            first: 1,
            second: 3,
        })
    );
    // 同一分配的两个视图也须比较。
    let base = span(7, 0x6_0000, 8192);
    let mut views = mixed_bindings();
    views[2].span = base.view(0, 4096).unwrap();
    views[3].span = base.view(4092, 4096).unwrap();
    assert_eq!(
        validate_bindings(&signature, &views, &[]),
        Err(LauncherError::Alias {
            first: 2,
            second: 3,
        })
    );
    // 首尾相接不算重叠。
    views[3].span = base.view(4096, 4096).unwrap();
    validate_bindings(&signature, &views, &[]).unwrap();
}

#[test]
fn declared_exception_permits_in_place_overlap() {
    let signature = AbiSignature::of::<InPlace>().unwrap();
    let same = span(3, 0x9000, 1024);
    validate_bindings(
        &signature,
        &[bind(0, same, ElemType::F32), bind(1, same, ElemType::F32)],
        &[],
    )
    .unwrap();
}
