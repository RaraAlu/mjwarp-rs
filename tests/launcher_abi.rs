//! M1发射器基础：配方、签名、布局与绑定校验。
//! 全部为宿主测试，不需要GPU或NVRTC。

use mjwarp_rs::runtime::launcher::{
    AbiSignature, Access, AliasException, BlockRule, CompileRecipe, DeviceSpan, ElemType,
    KernelAbi, LaunchBounds2, LaunchBounds3, LauncherError, ProofKind, RecipePurpose, SlotBinding,
    SlotDesc, SlotKind, SlotRequirement, SlotRole, WarpArray, rebind_values, validate_bindings,
};

const META: SlotRole = SlotRole::AddressBearing {
    proof: ProofKind("mixed-solve-route"),
};

const fn ptr(index: usize, elem: ElemType, access: Access, role: SlotRole) -> SlotDesc {
    SlotDesc {
        index,
        kind: SlotKind::DevicePtr { elem },
        access,
        role,
    }
}

const fn scalar(index: usize) -> SlotDesc {
    SlotDesc {
        index,
        kind: SlotKind::Scalar { bytes: 4 },
        access: Access::Read,
        role: SlotRole::Value,
    }
}

/// 镜像`launch_mixed_solve`的私有ABI：
/// 元数据、右端项、因子、输出与四项u32维度。
struct MixedSolve128;

// SAFETY: 测试声明，不用于发射。
unsafe impl KernelAbi for MixedSolve128 {
    const NAME: &'static str = "mixed_solve_128";
    const BLOCK: BlockRule = BlockRule::Fixed(128);
    const SLOTS: &'static [SlotDesc] = &[
        ptr(0, ElemType::I32, Access::Read, META),
        ptr(1, ElemType::F32, Access::Read, SlotRole::Value),
        ptr(2, ElemType::F32, Access::Read, SlotRole::Value),
        ptr(3, ElemType::F32, Access::Write, SlotRole::Value),
        scalar(4),
        scalar(5),
        scalar(6),
        scalar(7),
    ];
}

/// 原位更新示例：输入与输出显式允许重叠。
struct InPlace;

// SAFETY: 测试声明，不用于发射。
unsafe impl KernelAbi for InPlace {
    const NAME: &'static str = "in_place";
    const BLOCK: BlockRule = BlockRule::WarpGrid;
    const SLOTS: &'static [SlotDesc] = &[
        ptr(0, ElemType::F32, Access::Read, SlotRole::Value),
        ptr(1, ElemType::F32, Access::ReadWrite, SlotRole::Value),
        scalar(2),
    ];
    const ALIAS_EXCEPTIONS: &'static [AliasException] = &[AliasException {
        first: 0,
        second: 1,
    }];
}

/// Warp ABI示例：三维边界加三个数组。
struct WarpExample;

// SAFETY: 测试声明，不用于发射。
unsafe impl KernelAbi for WarpExample {
    const NAME: &'static str = "warp_example";
    const BLOCK: BlockRule = BlockRule::WarpGrid;
    const SLOTS: &'static [SlotDesc] = &[
        SlotDesc {
            index: 0,
            kind: SlotKind::Bounds { dims: 3 },
            access: Access::Read,
            role: SlotRole::Value,
        },
        SlotDesc {
            index: 1,
            kind: SlotKind::WarpArrayData {
                elem: ElemType::I32,
                ndim: 1,
            },
            access: Access::Read,
            role: SlotRole::AddressBearing {
                proof: ProofKind("body-parent"),
            },
        },
        SlotDesc {
            index: 2,
            kind: SlotKind::WarpArrayData {
                elem: ElemType::F32,
                ndim: 2,
            },
            access: Access::Read,
            role: SlotRole::Value,
        },
        SlotDesc {
            index: 3,
            kind: SlotKind::WarpArrayData {
                elem: ElemType::F32,
                ndim: 2,
            },
            access: Access::Write,
            role: SlotRole::Value,
        },
    ];
}

fn span(context: u64, addr: u64, bytes: u64) -> DeviceSpan {
    DeviceSpan::new(context, addr, bytes).unwrap()
}

fn bind(slot: usize, span: DeviceSpan, elem: ElemType) -> SlotBinding {
    SlotBinding { slot, span, elem }
}

/// 四个互不重叠的合法绑定。
fn mixed_bindings() -> Vec<SlotBinding> {
    vec![
        bind(0, span(7, 0x1_0000, 256), ElemType::I32),
        bind(1, span(7, 0x2_0000, 4096), ElemType::F32),
        bind(2, span(7, 0x3_0000, 4096), ElemType::F32),
        bind(3, span(7, 0x4_0000, 4096), ElemType::F32),
    ]
}

fn requirements() -> Vec<SlotRequirement> {
    vec![
        SlotRequirement {
            slot: 0,
            min_elements: 64,
        },
        SlotRequirement {
            slot: 3,
            min_elements: 1024,
        },
    ]
}

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

#[test]
fn recipes_admit_only_their_own_purpose() {
    let recipes = [
        (CompileRecipe::WARP_1_15_PHYSICS, RecipePurpose::Physics),
        (
            CompileRecipe::WARP_1_15_DIAGNOSTIC,
            RecipePurpose::Diagnostic,
        ),
        (CompileRecipe::P1_PROBE, RecipePurpose::Probe),
    ];
    for (recipe, own) in &recipes {
        for (_, required) in &recipes {
            let result = recipe.admit(*required);
            if own == required {
                assert!(result.is_ok());
            } else {
                assert_eq!(
                    result,
                    Err(LauncherError::RecipePurpose {
                        expected: *required,
                        actual: *own,
                    })
                );
            }
        }
    }
    let fingerprints: Vec<String> = recipes.iter().map(|(r, _)| r.fingerprint()).collect();
    assert_ne!(fingerprints[0], fingerprints[1]);
    assert_ne!(fingerprints[0], fingerprints[2]);
    assert_ne!(fingerprints[1], fingerprints[2]);
    assert_eq!(
        CompileRecipe::WARP_1_15_PHYSICS.fingerprint(),
        "recipe-v1;name=warp-1.15-physics;purpose=physics;fmad=1;fast-math=0;std=--std=c++17;arch=--gpu-architecture=compute_70;gate=f32-physics;defines="
    );
}

#[test]
fn warp_array_rejects_null_empty_deep_and_overflowing_shapes() {
    assert!(WarpArray::<f32>::contiguous(0, &[4]).is_err());
    assert!(WarpArray::<f32>::contiguous(0x1000, &[]).is_err());
    assert!(WarpArray::<f32>::contiguous(0x1000, &[4, 0]).is_err());
    assert!(WarpArray::<f32>::contiguous(0x1000, &[1, 1, 1, 1, 1]).is_err());
    assert_eq!(
        WarpArray::<f32>::contiguous(0x1000, &[1usize << 31]).unwrap_err(),
        LauncherError::Overflow("warp_array_shape")
    );
    // 内层字节跨度超出i32。
    assert_eq!(
        WarpArray::<f64>::contiguous(0x1000, &[2, 1 << 28]).unwrap_err(),
        LauncherError::Overflow("warp_array_stride")
    );
    let array = WarpArray::<f32>::contiguous(0x1000, &[1 << 20, 64]).unwrap();
    assert_eq!(array.strides(), [256, 4]);
    assert_eq!(array.byte_len(), 1 << 28);
}

#[test]
fn launch_bounds_reject_zero_and_overflowing_extents() {
    let bounds = LaunchBounds3::new([4, 5, 6]).unwrap();
    assert_eq!(bounds.size(), 120);
    assert_eq!(bounds.shape(), [4, 5, 6]);
    assert!(LaunchBounds2::new([0, 8]).is_err());
    assert_eq!(
        LaunchBounds2::new([1 << 31, 1]).unwrap_err(),
        LauncherError::Overflow("launch_bounds_shape")
    );
    assert_eq!(
        LaunchBounds2::new([1 << 16, 1 << 16]).unwrap_err(),
        LauncherError::Overflow("launch_bounds_size")
    );
    assert_eq!(
        LaunchBounds2::new([1 << 16, (1 << 16) - 1]).unwrap().size(),
        u64::from(u32::MAX) - 65535
    );
}
