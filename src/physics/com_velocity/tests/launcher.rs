//! 空间速度包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ComVelocityDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ComVelocityStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::ComVelocityMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::validate_com_velocity_metadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::com_velocity_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::com_velocity_plan;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn static_dims() -> ComVelocityStaticDims {
    ComVelocityStaticDims {
        nbody: 3,
        njnt: 2,
        nv: 4,
    }
}

#[cfg(feature = "cuda-probe")]
fn valid_metadata() -> Vec<i32> {
    // 体父 [0,0,1]、体关节数 [0,1,1]、体关节地址 [-1,0,1]、
    // 关节类型 [球,铰]、自由度地址 [0,3]。
    vec![0, 0, 1, 0, 1, 1, -1, 0, 1, 1, 3, 0, 3]
}

#[test]
#[cfg(feature = "cuda-probe")]
fn lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let dims = ComVelocityDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    assert_eq!(com_velocity_lengths(&dims).unwrap(), [13, 16, 140, 92]);
    assert_eq!(dims.abi(), [3, 2, 4, 2]);
    assert!(
        com_velocity_lengths(&ComVelocityDims {
            static_dims: ComVelocityStaticDims {
                nbody: 0,
                ..static_dims()
            },
            worlds: 1,
        })
        .is_err()
    );
    assert!(
        com_velocity_lengths(&ComVelocityDims {
            static_dims: static_dims(),
            worlds: 0,
        })
        .is_err()
    );
    assert!(
        com_velocity_lengths(&ComVelocityDims {
            static_dims: static_dims(),
            worlds: u32::MAX - 254,
        })
        .is_err()
    );
    assert!(matches!(
        com_velocity_lengths(&ComVelocityDims {
            static_dims: ComVelocityStaticDims {
                nbody: i32::MAX as u32 / 3 + 1,
                njnt: 0,
                nv: 0,
            },
            worlds: 1,
        }),
        Err(LauncherError::Overflow("com_velocity_metadata"))
    ));
}

#[test]
#[cfg(feature = "cuda-probe")]
fn metadata_proof_rejects_bad_parents_counts_addresses_and_dofs() {
    let dims = static_dims();
    assert!(validate_com_velocity_metadata(&valid_metadata(), &dims).is_ok());
    for (index, value, reason) in [
        (0, 1, "根体父引用"),
        (2, -1, "体父链越界"),
        (3, -1, "体关节数为负"),
        (8, 2, "体关节地址越界"),
        (9, 4, "关节类型越界"),
        (11, 2, "球关节自由度越界"),
        (12, 4, "铰关节自由度越界"),
    ] {
        let mut values = valid_metadata();
        values[index] = value;
        assert!(
            validate_com_velocity_metadata(&values, &dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_com_velocity_metadata(&valid_metadata()[..12], &dims).is_err());
    // 无关节边界：体关节数全零，关节表为空。
    let bare = ComVelocityStaticDims {
        nbody: 3,
        njnt: 0,
        nv: 0,
    };
    assert!(validate_com_velocity_metadata(&[0, 0, 1, 0, 0, 0, -1, -1, -1], &bare).is_ok());
}

#[test]
#[cfg(feature = "cuda-probe")]
fn plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = ComVelocityDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    let lengths = com_velocity_lengths(&dims).unwrap();
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
        Checked::<ComVelocityMetadata>::assume_uploaded(bindings[0].span, &values, dims, revision)
    }
    .unwrap();
    assert!(com_velocity_plan(&dims, &bindings, &[proof.binding(0)]).is_ok());

    // 容量：qvel槽少一个元素。
    let mut short = bindings;
    short[1].span = span(7, 0x2000, (lengths[1] - 1) * 4);
    assert!(matches!(
        com_velocity_plan(&dims, &short, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 1, .. })
    ));
    // 缺证明：元数据槽声明为地址类。
    assert!(matches!(
        com_velocity_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    // 别名：输出槽与质心输入槽重叠。
    let mut alias = bindings;
    alias[3].span = bindings[2].span;
    assert!(matches!(
        com_velocity_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    // 错会话：qvel槽属于另一上下文。
    let mut foreign = bindings;
    foreign[1].span = span(8, 0x2000, lengths[1] * 4);
    assert!(matches!(
        com_velocity_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
}
