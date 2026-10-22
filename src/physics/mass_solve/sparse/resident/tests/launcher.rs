//! 常驻稀疏质量包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseResidentGatherDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseResidentStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SparseResidentValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::SparseResidentValidateMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::validate_sparse_resident_metadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::sparse_resident_gather_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::sparse_resident_gather_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::sparse_resident_validate_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::sparse_resident_validate_plan;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn static_dims() -> SparseResidentStaticDims {
    SparseResidentStaticDims {
        nbody: 3,
        n: 2,
        nc: 3,
    }
}

#[cfg(feature = "cuda-probe")]
fn valid_metadata() -> Vec<i32> {
    // 行计数 [1,2]、行地址 [0,1]、列索引 [0,1,0]。
    vec![1, 2, 0, 1, 0, 1, 0]
}

#[test]
#[cfg(feature = "cuda-probe")]
fn lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let validate = SparseResidentValidateDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    assert_eq!(
        sparse_resident_validate_lengths(&validate).unwrap(),
        [7, 76, 76, 10]
    );
    assert_eq!(validate.abi(), [3, 2, 3, 2]);
    let gather = SparseResidentGatherDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    assert_eq!(
        sparse_resident_gather_lengths(&gather).unwrap(),
        [7, 76, 76, 14]
    );
    assert_eq!(gather.abi().unwrap(), [3, 2, 3, 4]);

    // 零自由度：元数据只剩占位，采集不发射。
    let zero = SparseResidentStaticDims {
        nbody: 3,
        n: 0,
        nc: 0,
    };
    assert_eq!(
        sparse_resident_validate_lengths(&SparseResidentValidateDims {
            static_dims: zero,
            worlds: 2,
        })
        .unwrap(),
        [1, 68, 68, 10]
    );
    assert_eq!(
        sparse_resident_gather_lengths(&SparseResidentGatherDims {
            static_dims: zero,
            worlds: 2,
        })
        .unwrap(),
        [1, 68, 68, 8]
    );
    assert!(
        sparse_resident_validate_lengths(&SparseResidentValidateDims {
            static_dims: SparseResidentStaticDims {
                nbody: 0,
                ..static_dims()
            },
            worlds: 1,
        })
        .is_err()
    );
    assert!(
        sparse_resident_validate_lengths(&SparseResidentValidateDims {
            static_dims: static_dims(),
            worlds: 0,
        })
        .is_err()
    );
    assert!(
        sparse_resident_validate_lengths(&SparseResidentValidateDims {
            static_dims: static_dims(),
            worlds: u32::MAX - 254,
        })
        .is_err()
    );
    assert!(matches!(
        sparse_resident_validate_lengths(&SparseResidentValidateDims {
            static_dims: SparseResidentStaticDims {
                n: i32::MAX as u32 / 2 + 1,
                nc: 0,
                ..static_dims()
            },
            worlds: 1,
        }),
        Err(LauncherError::Overflow("resident_sparse_metadata"))
    ));
}

#[test]
#[cfg(feature = "cuda-probe")]
fn metadata_proof_rejects_bad_rows_addresses_and_columns() {
    let dims = static_dims();
    assert!(validate_sparse_resident_metadata(&valid_metadata(), &dims).is_ok());
    for (index, value, reason) in [
        (0, -1, "行计数为负"),
        (1, 3, "行区间越界"),
        (2, -1, "行地址为负"),
        (3, 2, "行地址越界"),
        (4, 2, "列索引越界"),
        (5, -1, "列索引为负"),
    ] {
        let mut values = valid_metadata();
        values[index] = value;
        assert!(
            validate_sparse_resident_metadata(&values, &dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_sparse_resident_metadata(&valid_metadata()[..6], &dims).is_err());
    let zero = SparseResidentStaticDims {
        nbody: 3,
        n: 0,
        nc: 0,
    };
    assert!(validate_sparse_resident_metadata(&[0], &zero).is_ok());
    assert!(validate_sparse_resident_metadata(&[1], &zero).is_err());
    assert!(validate_sparse_resident_metadata(&[0, 0], &zero).is_err());
}

#[test]
#[cfg(feature = "cuda-probe")]
fn plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = SparseResidentValidateDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    let lengths = sparse_resident_validate_lengths(&dims).unwrap();
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
        Checked::<SparseResidentValidateMetadata>::assume_uploaded(
            bindings[0].span,
            &values,
            dims,
            revision,
        )
    }
    .unwrap();
    assert!(sparse_resident_validate_plan(&dims, &bindings, &[proof.binding(0)]).is_ok());

    // 容量：状态槽少一个元素。
    let mut short = bindings;
    short[3].span = span(7, 0x4000, (lengths[3] - 1) * 4);
    assert!(matches!(
        sparse_resident_validate_plan(&dims, &short, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 3, .. })
    ));
    // 缺证明：元数据槽声明为地址类。
    assert!(matches!(
        sparse_resident_validate_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    // 别名：状态槽与源缓冲重叠（源为只读，状态可写）。
    let mut alias = bindings;
    alias[3].span = bindings[1].span;
    assert!(matches!(
        sparse_resident_validate_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    // 错会话：第二源槽属于另一上下文。
    let mut foreign = bindings;
    foreign[2].span = span(8, 0x2000, lengths[2] * 4);
    assert!(matches!(
        sparse_resident_validate_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));

    // 采集计划：同一只读源两槽重叠允许，结果槽不得与源重叠。
    let gather = SparseResidentGatherDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    let gather_lengths = sparse_resident_gather_lengths(&gather).unwrap();
    let gather_bindings = [
        SlotBinding {
            slot: 0,
            span: span(7, 0x1000, gather_lengths[0] * 4),
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: span(7, 0x2000, gather_lengths[1] * 4),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: span(7, 0x2000, gather_lengths[2] * 4),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: span(7, 0x4000, gather_lengths[3] * 4),
            elem: ElemType::F32,
        },
    ];
    let gather_proof = unsafe {
        Checked::<super::launcher_meta::SparseResidentGatherMetadata>::assume_uploaded(
            gather_bindings[0].span,
            &values,
            gather,
            MetadataRevision::new(),
        )
    }
    .unwrap();
    assert!(
        sparse_resident_gather_plan(&gather, &gather_bindings, &[gather_proof.binding(0)]).is_ok()
    );
    let mut overlap = gather_bindings;
    overlap[3].span = gather_bindings[1].span;
    assert!(matches!(
        sparse_resident_gather_plan(&gather, &overlap, &[gather_proof.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
}
