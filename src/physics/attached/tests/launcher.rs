//! 附着运动学包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::AttachedDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::AttachedStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::AttachedMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::validate_attached_metadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::attached_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::attached_plan;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn static_dims() -> AttachedStaticDims {
    AttachedStaticDims {
        nbody: 3,
        njnt: 1,
        ngeom: 2,
        nsite: 1,
    }
}

#[cfg(feature = "cuda-probe")]
fn valid_metadata() -> Vec<i32> {
    // 刚体跨度、几何体索引 [1,2]、site体索引 [0]。
    vec![90, 1, 2, 0]
}

#[test]
#[cfg(feature = "cuda-probe")]
fn lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let dims = AttachedDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    assert_eq!(attached_lengths(&dims).unwrap(), [4, 21, 188, 80]);
    assert_eq!(dims.abi(), [3, 2, 1, 2]);

    // 空附着集合：元数据只剩跨度，参数保留一项。
    let empty = AttachedStaticDims {
        nbody: 1,
        njnt: 0,
        ngeom: 0,
        nsite: 0,
    };
    assert_eq!(
        attached_lengths(&AttachedDims {
            static_dims: empty,
            worlds: 2,
        })
        .unwrap(),
        [1, 1, 64, 8]
    );
    assert!(
        attached_lengths(&AttachedDims {
            static_dims: AttachedStaticDims {
                nbody: 0,
                ..static_dims()
            },
            worlds: 1,
        })
        .is_err()
    );
    assert!(
        attached_lengths(&AttachedDims {
            static_dims: static_dims(),
            worlds: 0,
        })
        .is_err()
    );
    assert!(
        attached_lengths(&AttachedDims {
            static_dims: static_dims(),
            worlds: u32::MAX - 254,
        })
        .is_err()
    );
    assert!(matches!(
        attached_lengths(&AttachedDims {
            static_dims: AttachedStaticDims {
                ngeom: i32::MAX as u32,
                nsite: 1,
                ..static_dims()
            },
            worlds: 1,
        }),
        Err(LauncherError::Overflow("attached_metadata"))
    ));
}

#[test]
#[cfg(feature = "cuda-probe")]
fn metadata_proof_rejects_bad_stride_and_body_indices() {
    let dims = static_dims();
    assert!(validate_attached_metadata(&valid_metadata(), &dims).is_ok());
    for (index, value, reason) in [
        (0, 89, "刚体跨度不符"),
        (1, 3, "几何体索引越界"),
        (2, -1, "几何体索引为负"),
        (3, -1, "site体索引为负"),
    ] {
        let mut values = valid_metadata();
        values[index] = value;
        assert!(
            validate_attached_metadata(&values, &dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_attached_metadata(&valid_metadata()[..3], &dims).is_err());
    // 关节数变化会改变冻结的刚体跨度。
    assert!(
        validate_attached_metadata(&valid_metadata(), &AttachedStaticDims { njnt: 0, ..dims })
            .is_err()
    );
}

#[test]
#[cfg(feature = "cuda-probe")]
fn plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = AttachedDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    let lengths = attached_lengths(&dims).unwrap();
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
        Checked::<AttachedMetadata>::assume_uploaded(bindings[0].span, &values, dims, revision)
    }
    .unwrap();
    assert!(attached_plan(&dims, &bindings, &[proof.binding(0)]).is_ok());

    // 容量：参数槽少一个元素。
    let mut short = bindings;
    short[1].span = span(7, 0x2000, (lengths[1] - 1) * 4);
    assert!(matches!(
        attached_plan(&dims, &short, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 1, .. })
    ));
    // 缺证明：元数据槽声明为地址类。
    assert!(matches!(
        attached_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    // 别名：输出槽与刚体输入槽重叠。
    let mut alias = bindings;
    alias[3].span = bindings[2].span;
    assert!(matches!(
        attached_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    // 错会话：刚体输入槽属于另一上下文。
    let mut foreign = bindings;
    foreign[2].span = span(8, 0x3000, lengths[2] * 4);
    assert!(matches!(
        attached_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
}
