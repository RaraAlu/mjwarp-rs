//! 肌腱速度包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonVelocityComputeDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonVelocityStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::TendonVelocityValidateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::TendonVelocityValidateMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::validate_tendon_velocity_metadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::tendon_velocity_compute_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::tendon_velocity_validate_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::tendon_velocity_validate_plan;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn static_dims() -> TendonVelocityStaticDims {
    TendonVelocityStaticDims {
        nv: 3,
        nt: 2,
        nnz: 2,
        stride: 4,
    }
}

#[cfg(feature = "cuda-probe")]
fn valid_metadata() -> Vec<i32> {
    vec![
        -131072, -131072, -131072, -131072, 0, 1, 1, 1, 0, 2, -131072, -131072, -131072, -131072,
    ]
}

#[test]
#[cfg(feature = "cuda-probe")]
fn lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let dims = TendonVelocityValidateDims {
        static_dims: static_dims(),
        worlds: 3,
    };
    assert_eq!(
        tendon_velocity_validate_lengths(&dims).unwrap(),
        [14, 20, 17, 11]
    );
    let compute = TendonVelocityComputeDims {
        static_dims: static_dims(),
        worlds: 3,
    };
    assert_eq!(
        tendon_velocity_compute_lengths(&compute).unwrap(),
        [14, 20, 17, 14]
    );
    assert_eq!(dims.abi(), [0, 0, 3, 3]);
    assert_eq!(compute.abi().unwrap(), [0, 0, 3, 6]);

    for static_dims in [
        TendonVelocityStaticDims {
            nv: 0,
            ..static_dims()
        },
        TendonVelocityStaticDims {
            nt: 0,
            ..static_dims()
        },
        // 跨度必须至少容纳肌腱数与列数。
        TendonVelocityStaticDims {
            stride: 3,
            ..static_dims()
        },
    ] {
        assert!(
            tendon_velocity_validate_lengths(&TendonVelocityValidateDims {
                static_dims,
                worlds: 1,
            })
            .is_err()
        );
    }
    assert!(
        tendon_velocity_validate_lengths(&TendonVelocityValidateDims {
            static_dims: static_dims(),
            worlds: 0,
        })
        .is_err()
    );
    // 工作量乘肌腱数溢出。
    assert!(
        tendon_velocity_compute_lengths(&TendonVelocityComputeDims {
            static_dims: static_dims(),
            worlds: u32::MAX - 255,
        })
        .is_err()
    );
    // 元数据长度超过i32表示上限。
    let wide = TendonVelocityStaticDims {
        nt: (i32::MAX as u32 - 7) / 2,
        stride: (i32::MAX as u32 - 7) / 2 + 2,
        ..static_dims()
    };
    assert!(matches!(
        tendon_velocity_validate_lengths(&TendonVelocityValidateDims {
            static_dims: wide,
            worlds: 1,
        }),
        Err(LauncherError::Overflow("tendon_velocity_metadata"))
    ));
}

#[test]
#[cfg(feature = "cuda-probe")]
fn metadata_proof_rejects_bad_guards_rows_and_columns() {
    let dims = static_dims();
    assert!(validate_tendon_velocity_metadata(&valid_metadata(), &dims).is_ok());
    for (index, value, reason) in [
        (0, 0, "前守卫"),
        (13, 0, "后守卫"),
        (7, 2, "行数越界"),
        (4, 3, "行偏移越界"),
        (8, 3, "列越界"),
        (8, -1, "列负值"),
    ] {
        let mut values = valid_metadata();
        values[index] = value;
        assert!(
            validate_tendon_velocity_metadata(&values, &dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_tendon_velocity_metadata(&valid_metadata()[..13], &dims).is_err());
}

#[test]
#[cfg(feature = "cuda-probe")]
fn plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = TendonVelocityValidateDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    let lengths = tendon_velocity_validate_lengths(&dims).unwrap();
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
        Checked::<TendonVelocityValidateMetadata>::assume_uploaded(
            bindings[0].span,
            &values,
            dims,
            revision,
        )
    }
    .unwrap();
    assert!(tendon_velocity_validate_plan(&dims, &bindings, &[proof.binding(0)]).is_ok());

    // 容量：第二槽少一个元素。
    let mut short = bindings;
    short[1].span = span(7, 0x2000, (lengths[1] - 1) * 4);
    assert!(matches!(
        tendon_velocity_validate_plan(&dims, &short, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 1, .. })
    ));
    // 缺证明：元数据槽声明为地址类。
    assert!(matches!(
        tendon_velocity_validate_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    // 别名：暂存槽与雅可比槽重叠。
    let mut alias = bindings;
    alias[3].span = bindings[1].span;
    assert!(matches!(
        tendon_velocity_validate_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    // 错会话：第三槽属于另一上下文。
    let mut foreign = bindings;
    foreign[2].span = span(8, 0x3000, lengths[2] * 4);
    assert!(matches!(
        tendon_velocity_validate_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
}
