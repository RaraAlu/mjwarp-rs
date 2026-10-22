//! 重力补偿包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityClearDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravitySumDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::GravityValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::GravityClearMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::GravitySumMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::gravity_launch_error;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::validate_gravity_metadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::gravity_clear_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::gravity_clear_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::gravity_compute_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::gravity_sum_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::gravity_sum_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::gravity_validate_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::gravity_validate_plan;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn static_dims() -> GravityStaticDims {
    GravityStaticDims {
        nb: 3,
        nj: 1,
        nv: 2,
        parameter_len: 9,
    }
}

#[cfg(feature = "cuda-probe")]
fn valid_metadata() -> Vec<i32> {
    // 四守卫、根表 [0,1,1]、祖先表、自由度关节 [0,0]、驱动器 [1]、四守卫。
    vec![
        -131072, -131072, -131072, -131072, 0, 1, 1, 1, 0, 1, 0, 0, 1, 0, 0, 1, -131072, -131072,
        -131072, -131072,
    ]
}

#[test]
#[cfg(feature = "cuda-probe")]
fn lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let clear = GravityClearDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    assert_eq!(gravity_clear_lengths(&clear).unwrap(), [20, 17, 17, 12]);
    assert_eq!(clear.abi().unwrap(), [3, 2, 2, 4]);
    let validate = GravityValidateDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    assert_eq!(
        gravity_validate_lengths(&validate).unwrap(),
        [20, 17, 188, 116, 10]
    );
    assert_eq!(validate.abi(), [3, 2, 2, 2]);
    let compute = GravityComputeDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    assert_eq!(
        gravity_compute_lengths(&compute).unwrap(),
        [20, 17, 188, 116, 12]
    );
    assert_eq!(compute.abi().unwrap(), [3, 2, 2, 8]);
    let sum = GravitySumDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    assert_eq!(gravity_sum_lengths(&sum).unwrap(), [20, 16, 12, 12, 12]);
    assert_eq!(sum.abi().unwrap(), [3, 2, 2, 4]);

    for static_dims in [
        GravityStaticDims {
            nb: 1,
            ..static_dims()
        },
        GravityStaticDims {
            nv: 0,
            ..static_dims()
        },
    ] {
        assert!(
            gravity_clear_lengths(&GravityClearDims {
                static_dims,
                worlds: 1,
            })
            .is_err()
        );
    }
    assert!(
        gravity_clear_lengths(&GravityClearDims {
            static_dims: static_dims(),
            worlds: 0,
        })
        .is_err()
    );
    assert!(matches!(
        gravity_clear_lengths(&GravityClearDims {
            static_dims: GravityStaticDims {
                nb: i32::MAX as u32 / 2,
                nv: 2,
                ..static_dims()
            },
            worlds: 1,
        }),
        Err(LauncherError::Overflow("gravity_compensation_metadata"))
    ));
}

#[test]
#[cfg(feature = "cuda-probe")]
fn metadata_proof_rejects_bad_guards_roots_ancestry_and_flags() {
    let dims = static_dims();
    assert!(validate_gravity_metadata(&valid_metadata(), &dims).is_ok());
    for (index, value, reason) in [
        (0, 0, "前守卫"),
        (16, 0, "后守卫"),
        (4, 3, "根表越界"),
        (5, -1, "根表为负"),
        (7, 2, "祖先表非布尔"),
        (13, 1, "自由度关节越界"),
        (15, 2, "驱动器标志非布尔"),
    ] {
        let mut values = valid_metadata();
        values[index] = value;
        assert!(
            validate_gravity_metadata(&values, &dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_gravity_metadata(&valid_metadata()[..19], &dims).is_err());
    let _ = gravity_launch_error;
}

#[test]
#[cfg(feature = "cuda-probe")]
fn plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = GravityClearDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    let lengths = gravity_clear_lengths(&dims).unwrap();
    let span = |context: u64, addr: u64, bytes: u64| DeviceSpan::new(context, addr, bytes).unwrap();
    let bindings = [
        SlotBinding {
            slot: 0,
            span: span(7, 0x1000, lengths[0] * 4),
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: span(7, 0x2000, lengths[1] * 4),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: span(7, 0x2000, lengths[2] * 4),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: span(7, 0x4000, lengths[3] * 4),
            elem: ElemType::F32,
        },
    ];
    let values = valid_metadata();
    let revision = MetadataRevision::new();
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    let proof = unsafe {
        Checked::<GravityClearMetadata>::assume_uploaded(bindings[0].span, &values, dims, revision)
    }
    .unwrap();
    assert!(gravity_clear_plan(&dims, &bindings, &[proof.binding(0)]).is_ok());

    // 容量：输出槽少一个元素。
    let mut short = bindings;
    short[3].span = span(7, 0x4000, (lengths[3] - 1) * 4);
    assert!(matches!(
        gravity_clear_plan(&dims, &short, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 3, .. })
    ));
    // 缺证明：元数据槽声明为地址类。
    assert!(matches!(
        gravity_clear_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    // 别名：输出槽与参数槽重叠。
    let mut alias = bindings;
    alias[3].span = bindings[1].span;
    assert!(matches!(
        gravity_clear_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    // 错会话：参数槽属于另一上下文。
    let mut foreign = bindings;
    foreign[1].span = span(8, 0x2000, lengths[1] * 4);
    assert!(matches!(
        gravity_clear_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));

    // 验证计划：五缓冲、源缓冲可读重叠、状态槽独占写入。
    let validate = GravityValidateDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    let validate_lengths = gravity_validate_lengths(&validate).unwrap();
    let validate_bindings = [
        SlotBinding {
            slot: 0,
            span: span(7, 0x1000, validate_lengths[0] * 4),
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: span(7, 0x2000, validate_lengths[1] * 4),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: span(7, 0x3000, validate_lengths[2] * 4),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: span(7, 0x5000, validate_lengths[3] * 4),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 4,
            span: span(7, 0x8000, validate_lengths[4] * 4),
            elem: ElemType::F32,
        },
    ];
    let validate_proof = unsafe {
        Checked::<super::launcher_meta::GravityValidateMetadata>::assume_uploaded(
            validate_bindings[0].span,
            &values,
            validate,
            MetadataRevision::new(),
        )
    }
    .unwrap();
    assert!(
        gravity_validate_plan(&validate, &validate_bindings, &[validate_proof.binding(0)]).is_ok()
    );
    let mut overlap = validate_bindings;
    overlap[4].span = validate_bindings[2].span;
    assert!(matches!(
        gravity_validate_plan(&validate, &overlap, &[validate_proof.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
}

#[test]
#[cfg(feature = "cuda-probe")]
fn sum_plan_reads_two_field_parts_and_accepts_wide_shared_output() {
    let dims = GravitySumDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    let lengths = gravity_sum_lengths(&dims).unwrap();
    let span = |context: u64, addr: u64, bytes: u64| DeviceSpan::new(context, addr, bytes).unwrap();
    let binding = |slot: usize, addr: u64, length: u64, elem: ElemType| SlotBinding {
        slot,
        span: span(7, addr, length * 4),
        elem,
    };
    let bindings = [
        binding(0, 0x1000, lengths[0], ElemType::I32),
        binding(1, 0x2000, lengths[1], ElemType::F32),
        binding(2, 0x3000, lengths[2], ElemType::F32),
        binding(3, 0x3000, lengths[3], ElemType::F32),
        binding(4, 0x4000, lengths[4], ElemType::F32),
    ];
    let values = valid_metadata();
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    let proof = unsafe {
        Checked::<GravitySumMetadata>::assume_uploaded(
            bindings[0].span,
            &values,
            dims,
            MetadataRevision::new(),
        )
    }
    .unwrap();
    assert!(gravity_sum_plan(&dims, &bindings, &[proof.binding(0)]).is_ok());
    // 共享链输出是四字段平滑力缓冲：内核只写首个字段，按最小容量接受。
    let mut wide = bindings;
    wide[4].span = span(7, 0x4000, (8 + 4 * 2 * 2) * 4);
    assert!(gravity_sum_plan(&dims, &wide, &[proof.binding(0)]).is_ok());
    // 分片缓冲必须容纳弹簧与阻尼两个字段。
    let mut narrow = bindings;
    narrow[1].span = span(7, 0x2000, 12 * 4);
    assert!(matches!(
        gravity_sum_plan(&dims, &narrow, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 1, .. })
    ));
    // 输出仍须容纳最小单字段。
    let mut short = bindings;
    short[4].span = span(7, 0x4000, 11 * 4);
    assert!(matches!(
        gravity_sum_plan(&dims, &short, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 4, .. })
    ));
}
