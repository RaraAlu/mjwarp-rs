//! 借用质量矩阵包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::BorrowedMassComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::BorrowedMassStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::BorrowedMassValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::BorrowedMassComputeMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::BorrowedMassValidateMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::validate_borrowed_mass_metadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::borrowed_mass_compute_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::borrowed_mass_compute_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::borrowed_mass_validate_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::borrowed_mass_validate_plan;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn static_dims() -> BorrowedMassStaticDims {
    BorrowedMassStaticDims {
        nbody: 4,
        n: 3,
        count: 2,
    }
}

#[cfg(feature = "cuda-probe")]
fn valid_metadata() -> Vec<i32> {
    // 四守卫、自由度/总数、行偏移 [0,1,2,2]、列 [0,2]、下三角地址 [0,7]、
    // 两个世界的跳过标志、尾部四守卫。
    vec![
        -131072, -131072, -131072, -131072, 3, 2, 0, 1, 2, 2, 0, 2, 0, 7, 0, 0, -131072, -131072,
        -131072, -131072,
    ]
}

#[test]
#[cfg(feature = "cuda-probe")]
fn lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let dims = BorrowedMassValidateDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    assert_eq!(
        borrowed_mass_validate_lengths(&dims).unwrap(),
        [20, 106, 14, 10]
    );
    let compute = BorrowedMassComputeDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    assert_eq!(
        borrowed_mass_compute_lengths(&compute).unwrap(),
        [20, 106, 14, 14]
    );
    assert_eq!(dims.abi().unwrap(), [4, 3, 14, 2]);
    assert_eq!(compute.abi().unwrap(), [4, 3, 14, 6]);

    for static_dims in [
        BorrowedMassStaticDims {
            nbody: 0,
            ..static_dims()
        },
        BorrowedMassStaticDims {
            n: 0,
            ..static_dims()
        },
    ] {
        assert!(
            borrowed_mass_validate_lengths(&BorrowedMassValidateDims {
                static_dims,
                worlds: 1,
            })
            .is_err()
        );
    }
    assert!(
        borrowed_mass_validate_lengths(&BorrowedMassValidateDims {
            static_dims: static_dims(),
            worlds: 0,
        })
        .is_err()
    );
    assert!(
        borrowed_mass_validate_lengths(&BorrowedMassValidateDims {
            static_dims: static_dims(),
            worlds: u32::MAX - 255,
        })
        .is_err()
    );
    // 工作量乘自由度溢出。
    assert!(matches!(
        borrowed_mass_compute_lengths(&BorrowedMassComputeDims {
            static_dims: static_dims(),
            worlds: u32::MAX - 255,
        }),
        Err(LauncherError::Overflow("borrowed_mass_work"))
    ));
    // 元数据长度超过i32表示上限。
    assert!(matches!(
        borrowed_mass_validate_lengths(&BorrowedMassValidateDims {
            static_dims: BorrowedMassStaticDims {
                count: i32::MAX as u32 / 2,
                ..static_dims()
            },
            worlds: 1,
        }),
        Err(LauncherError::Overflow("borrowed_mass_metadata"))
    ));
}

#[test]
#[cfg(feature = "cuda-probe")]
fn metadata_proof_rejects_bad_guards_prefix_rows_columns_addresses_and_skip() {
    let dims = static_dims();
    assert!(validate_borrowed_mass_metadata(&valid_metadata(), &dims, 2).is_ok());
    for (index, value, reason) in [
        (0, 0, "前守卫"),
        (16, 0, "后守卫"),
        (4, 2, "自由度不符"),
        (5, 1, "行总数不符"),
        (6, 1, "行起点非零"),
        (9, 3, "行终点不符"),
        (8, 0, "行偏移递减"),
        (10, 3, "列越界"),
        (10, -1, "列负值"),
        (13, 0, "下三角地址不符"),
        (14, 2, "跳过标志"),
    ] {
        let mut values = valid_metadata();
        values[index] = value;
        assert!(
            validate_borrowed_mass_metadata(&values, &dims, 2).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_borrowed_mass_metadata(&valid_metadata()[..19], &dims, 2).is_err());
    assert!(validate_borrowed_mass_metadata(&valid_metadata(), &dims, 1).is_err());
}

#[test]
#[cfg(feature = "cuda-probe")]
fn plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = BorrowedMassValidateDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    let lengths = borrowed_mass_validate_lengths(&dims).unwrap();
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
            span: span(7, 0x3000, lengths[2] * 4),
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
        Checked::<BorrowedMassValidateMetadata>::assume_uploaded(
            bindings[0].span,
            &values,
            dims,
            revision,
        )
    }
    .unwrap();
    assert!(borrowed_mass_validate_plan(&dims, &bindings, &[proof.binding(0)]).is_ok());

    // 容量：第二槽少一个元素。
    let mut short = bindings;
    short[1].span = span(7, 0x2000, (lengths[1] - 1) * 4);
    assert!(matches!(
        borrowed_mass_validate_plan(&dims, &short, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 1, .. })
    ));
    // 缺证明：元数据槽声明为地址类。
    assert!(matches!(
        borrowed_mass_validate_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    // 别名：状态槽与源矩阵槽重叠。
    let mut alias = bindings;
    alias[3].span = bindings[1].span;
    assert!(matches!(
        borrowed_mass_validate_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    // 错会话：向量槽属于另一上下文。
    let mut foreign = bindings;
    foreign[2].span = span(8, 0x3000, lengths[2] * 4);
    assert!(matches!(
        borrowed_mass_validate_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));

    // 计算计划同样核对证明、跨度、会话与别名。
    let compute = BorrowedMassComputeDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    let compute_lengths = borrowed_mass_compute_lengths(&compute).unwrap();
    let compute_bindings = [
        SlotBinding {
            slot: 0,
            span: span(7, 0x1000, compute_lengths[0] * 4),
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: span(7, 0x2000, compute_lengths[1] * 4),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: span(7, 0x3000, compute_lengths[2] * 4),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: span(7, 0x4000, compute_lengths[3] * 4),
            elem: ElemType::F32,
        },
    ];
    let compute_proof = unsafe {
        Checked::<BorrowedMassComputeMetadata>::assume_uploaded(
            compute_bindings[0].span,
            &values,
            compute,
            MetadataRevision::new(),
        )
    }
    .unwrap();
    assert!(
        borrowed_mass_compute_plan(&compute, &compute_bindings, &[compute_proof.binding(0)])
            .is_ok()
    );
    let mut short_staged = compute_bindings;
    short_staged[3].span = span(7, 0x4000, (compute_lengths[3] - 1) * 4);
    assert!(matches!(
        borrowed_mass_compute_plan(&compute, &short_staged, &[compute_proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 3, .. })
    ));
}
