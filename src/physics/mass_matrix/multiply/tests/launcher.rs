//! 质量矩阵乘法包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::MassMultiplyComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::MassMultiplyStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::MassMultiplyValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::MassMultiplyComputeMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::MassMultiplyValidateMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::validate_mass_multiply_metadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::mass_multiply_compute_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::mass_multiply_compute_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::mass_multiply_validate_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::mass_multiply_validate_plan;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn static_dims() -> MassMultiplyStaticDims {
    MassMultiplyStaticDims {
        n: 3,
        mc: 5,
        count: 2,
    }
}

#[cfg(feature = "cuda-probe")]
fn valid_metadata() -> Vec<i32> {
    // 行偏移 [0,1,2,2]、列 [0,2]、地址 [0,1]、两个世界的跳过标志 [0,0]。
    vec![0, 1, 2, 2, 0, 2, 0, 1, 0, 0]
}

#[test]
#[cfg(feature = "cuda-probe")]
fn lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let dims = MassMultiplyValidateDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    assert_eq!(
        mass_multiply_validate_lengths(&dims).unwrap(),
        [10, 18, 14, 10]
    );
    let compute = MassMultiplyComputeDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    assert_eq!(
        mass_multiply_compute_lengths(&compute).unwrap(),
        [10, 18, 14, 14]
    );
    assert_eq!(dims.abi().unwrap(), [3, 5, 8, 2]);
    assert_eq!(compute.abi().unwrap(), [3, 5, 8, 6]);

    for static_dims in [
        MassMultiplyStaticDims {
            n: 0,
            ..static_dims()
        },
        MassMultiplyStaticDims {
            mc: 0,
            ..static_dims()
        },
    ] {
        assert!(
            mass_multiply_validate_lengths(&MassMultiplyValidateDims {
                static_dims,
                worlds: 1,
            })
            .is_err()
        );
    }
    assert!(
        mass_multiply_validate_lengths(&MassMultiplyValidateDims {
            static_dims: static_dims(),
            worlds: 0,
        })
        .is_err()
    );
    assert!(
        mass_multiply_validate_lengths(&MassMultiplyValidateDims {
            static_dims: static_dims(),
            worlds: u32::MAX - 255,
        })
        .is_err()
    );
    // 工作量乘自由度溢出。
    assert!(matches!(
        mass_multiply_compute_lengths(&MassMultiplyComputeDims {
            static_dims: static_dims(),
            worlds: u32::MAX - 255,
        }),
        Err(LauncherError::Overflow("mass_multiply_work"))
    ));
    // 元数据长度超过i32表示上限。
    assert!(matches!(
        mass_multiply_validate_lengths(&MassMultiplyValidateDims {
            static_dims: MassMultiplyStaticDims {
                n: 2,
                count: i32::MAX as u32 / 2,
                ..static_dims()
            },
            worlds: 1,
        }),
        Err(LauncherError::Overflow("mass_multiply_metadata"))
    ));
}

#[test]
#[cfg(feature = "cuda-probe")]
fn metadata_proof_rejects_bad_offsets_columns_addresses_and_skip() {
    let dims = static_dims();
    assert!(validate_mass_multiply_metadata(&valid_metadata(), &dims, 2).is_ok());
    for (index, value, reason) in [
        (0, 1, "首行偏移"),
        (2, 0, "行偏移递减"),
        (3, 3, "行总数不符"),
        (4, 3, "列越界"),
        (4, -1, "列负值"),
        (6, 5, "地址越界"),
        (6, -1, "地址负值"),
        (8, 2, "跳过标志"),
    ] {
        let mut values = valid_metadata();
        values[index] = value;
        assert!(
            validate_mass_multiply_metadata(&values, &dims, 2).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_mass_multiply_metadata(&valid_metadata()[..9], &dims, 2).is_err());
    assert!(validate_mass_multiply_metadata(&valid_metadata(), &dims, 1).is_err());
}

#[test]
#[cfg(feature = "cuda-probe")]
fn plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = MassMultiplyValidateDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    let lengths = mass_multiply_validate_lengths(&dims).unwrap();
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
        Checked::<MassMultiplyValidateMetadata>::assume_uploaded(
            bindings[0].span,
            &values,
            dims,
            revision,
        )
    }
    .unwrap();
    assert!(mass_multiply_validate_plan(&dims, &bindings, &[proof.binding(0)]).is_ok());

    // 容量：第二槽少一个元素。
    let mut short = bindings;
    short[1].span = span(7, 0x2000, (lengths[1] - 1) * 4);
    assert!(matches!(
        mass_multiply_validate_plan(&dims, &short, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 1, .. })
    ));
    // 缺证明：元数据槽声明为地址类。
    assert!(matches!(
        mass_multiply_validate_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    // 别名：状态槽与矩阵槽重叠。
    let mut alias = bindings;
    alias[3].span = bindings[1].span;
    assert!(matches!(
        mass_multiply_validate_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    // 错会话：向量槽属于另一上下文。
    let mut foreign = bindings;
    foreign[2].span = span(8, 0x3000, lengths[2] * 4);
    assert!(matches!(
        mass_multiply_validate_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));

    // 计算计划同样核对证明、跨度、会话与别名。
    let compute = MassMultiplyComputeDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    let compute_lengths = mass_multiply_compute_lengths(&compute).unwrap();
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
        Checked::<MassMultiplyComputeMetadata>::assume_uploaded(
            compute_bindings[0].span,
            &values,
            compute,
            MetadataRevision::new(),
        )
    }
    .unwrap();
    assert!(
        mass_multiply_compute_plan(&compute, &compute_bindings, &[compute_proof.binding(0)])
            .is_ok()
    );
    let mut short_output = compute_bindings;
    short_output[3].span = span(7, 0x4000, (compute_lengths[3] - 1) * 4);
    assert!(matches!(
        mass_multiply_compute_plan(&compute, &short_output, &[compute_proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 3, .. })
    ));
}
