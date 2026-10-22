//! 质心位置包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ComPositionDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ComPositionStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::ComPositionMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::validate_com_position_metadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::com_position_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::com_position_plan;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn static_dims() -> ComPositionStaticDims {
    ComPositionStaticDims {
        nbody: 3,
        njnt: 2,
        nv: 4,
    }
}

#[cfg(feature = "cuda-probe")]
fn valid_metadata() -> Vec<i32> {
    // 体父 [0,0,1]、关节类型 [球,铰]、关节体 [1,2]、自由度地址 [0,3]。
    vec![0, 0, 1, 1, 3, 1, 2, 0, 3]
}

#[test]
#[cfg(feature = "cuda-probe")]
fn lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let dims = ComPositionDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    assert_eq!(com_position_lengths(&dims).unwrap(), [9, 12, 200, 140]);
    assert_eq!(dims.abi(), [3, 2, 4, 2]);

    // 无关节、无自由度边界：元数据只含体父。
    let zero = ComPositionStaticDims {
        nbody: 3,
        njnt: 0,
        nv: 0,
    };
    assert_eq!(
        com_position_lengths(&ComPositionDims {
            static_dims: zero,
            worlds: 2,
        })
        .unwrap(),
        [3, 12, 176, 92]
    );
    assert!(
        com_position_lengths(&ComPositionDims {
            static_dims: ComPositionStaticDims {
                nbody: 0,
                ..static_dims()
            },
            worlds: 1,
        })
        .is_err()
    );
    assert!(
        com_position_lengths(&ComPositionDims {
            static_dims: static_dims(),
            worlds: 0,
        })
        .is_err()
    );
    assert!(
        com_position_lengths(&ComPositionDims {
            static_dims: static_dims(),
            worlds: u32::MAX - 254,
        })
        .is_err()
    );
    assert!(matches!(
        com_position_lengths(&ComPositionDims {
            static_dims: ComPositionStaticDims {
                nbody: i32::MAX as u32 / 3,
                njnt: i32::MAX as u32 / 3,
                nv: 0,
            },
            worlds: 1,
        }),
        Err(LauncherError::Overflow("com_position_metadata"))
    ));
}

#[test]
#[cfg(feature = "cuda-probe")]
fn metadata_proof_rejects_bad_parents_types_bodies_and_addresses() {
    let dims = static_dims();
    assert!(validate_com_position_metadata(&valid_metadata(), &dims).is_ok());
    for (index, value, reason) in [
        (0, 1, "根体父引用"),
        (2, -1, "体父链越界"),
        (3, 4, "关节类型越界"),
        (4, -1, "关节类型为负"),
        (5, 3, "关节体越界"),
        (7, 2, "球关节自由度越界"),
        (8, 4, "铰关节自由度越界"),
    ] {
        let mut values = valid_metadata();
        values[index] = value;
        assert!(
            validate_com_position_metadata(&values, &dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_com_position_metadata(&valid_metadata()[..8], &dims).is_err());
}

#[test]
#[cfg(feature = "cuda-probe")]
fn plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = ComPositionDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    let lengths = com_position_lengths(&dims).unwrap();
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
        Checked::<ComPositionMetadata>::assume_uploaded(bindings[0].span, &values, dims, revision)
    }
    .unwrap();
    assert!(com_position_plan(&dims, &bindings, &[proof.binding(0)]).is_ok());

    // 容量：运动学输入槽少一个元素。
    let mut short = bindings;
    short[2].span = span(7, 0x3000, (lengths[2] - 1) * 4);
    assert!(matches!(
        com_position_plan(&dims, &short, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 2, .. })
    ));
    // 缺证明：元数据槽声明为地址类。
    assert!(matches!(
        com_position_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    // 别名：输出槽与运动学输入槽重叠。
    let mut alias = bindings;
    alias[3].span = bindings[2].span;
    assert!(matches!(
        com_position_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    // 错会话：参数槽属于另一上下文。
    let mut foreign = bindings;
    foreign[1].span = span(8, 0x2000, lengths[1] * 4);
    assert!(matches!(
        com_position_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
}
