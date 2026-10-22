//! 常驻CRB包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbAccumulateDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbInitDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbMatrixDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentCrbStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::ResidentCrbAccumulateMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::ResidentCrbInitMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::validate_resident_crb_metadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::resident_crb_accumulate_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::resident_crb_accumulate_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::resident_crb_init_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::resident_crb_init_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::resident_crb_matrix_lengths;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn static_dims() -> ResidentCrbStaticDims {
    ResidentCrbStaticDims {
        nbody: 4,
        nv: 2,
        depth: 2,
    }
}

#[cfg(feature = "cuda-probe")]
fn valid_metadata() -> Vec<i32> {
    // 刚体数、自由度、体父链、关节体、关节父链、深度表。
    vec![4, 2, 0, 0, 0, 1, 1, 2, -1, 0, 0, 1, 1, 2]
}

#[test]
#[cfg(feature = "cuda-probe")]
fn lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let init = ResidentCrbInitDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    assert_eq!(resident_crb_init_lengths(&init).unwrap(), [14, 2, 144, 96]);
    assert_eq!(init.abi(), [4, 2, 0, 2]);
    let accumulate = ResidentCrbAccumulateDims {
        static_dims: static_dims(),
        worlds: 2,
        level: 2,
    };
    assert_eq!(
        resident_crb_accumulate_lengths(&accumulate).unwrap(),
        [14, 2, 144, 96]
    );
    assert_eq!(accumulate.abi().unwrap(), [4, 2, 2, 8]);
    let matrix = ResidentCrbMatrixDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    assert_eq!(
        resident_crb_matrix_lengths(&matrix).unwrap(),
        [14, 2, 144, 96]
    );
    assert_eq!(matrix.abi().unwrap(), [4, 2, 0, 4]);

    // 层级必须落在实际深度内。
    for level in [0, 3] {
        assert!(
            resident_crb_accumulate_lengths(&ResidentCrbAccumulateDims {
                static_dims: static_dims(),
                worlds: 1,
                level,
            })
            .is_err()
        );
    }
    assert!(
        resident_crb_init_lengths(&ResidentCrbInitDims {
            static_dims: ResidentCrbStaticDims {
                nbody: 0,
                ..static_dims()
            },
            worlds: 1,
        })
        .is_err()
    );
    assert!(
        resident_crb_init_lengths(&ResidentCrbInitDims {
            static_dims: static_dims(),
            worlds: 0,
        })
        .is_err()
    );
    assert!(
        resident_crb_init_lengths(&ResidentCrbInitDims {
            static_dims: static_dims(),
            worlds: u32::MAX - 254,
        })
        .is_err()
    );
    // 元数据超过i32表示上限。
    assert!(matches!(
        resident_crb_init_lengths(&ResidentCrbInitDims {
            static_dims: ResidentCrbStaticDims {
                nbody: i32::MAX as u32 / 2,
                nv: 0,
                depth: 0,
            },
            worlds: 1,
        }),
        Err(LauncherError::Overflow("resident_crb_metadata"))
    ));
}

#[test]
#[cfg(feature = "cuda-probe")]
fn metadata_proof_rejects_bad_parents_joints_and_depth() {
    let dims = static_dims();
    assert!(validate_resident_crb_metadata(&valid_metadata(), &dims).is_ok());
    for (index, value, reason) in [
        (0, 3, "刚体数不符"),
        (1, 1, "自由度不符"),
        (2, 1, "根体父引用"),
        (5, 3, "体父链越界"),
        (6, -1, "关节体为负"),
        (8, 0, "根关节父非负一"),
        (9, 1, "关节父链越界"),
        (10, 1, "根深度非零"),
        (11, 0, "深度表不符"),
        (13, 1, "深度表末项不符"),
    ] {
        let mut values = valid_metadata();
        values[index] = value;
        assert!(
            validate_resident_crb_metadata(&values, &dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_resident_crb_metadata(&valid_metadata()[..13], &dims).is_err());
    // 多棵树允许非零自由度作为根，父引用为负一。
    let mut multi_root = valid_metadata();
    multi_root[9] = -1;
    assert!(validate_resident_crb_metadata(&multi_root, &dims).is_ok());
    assert!(
        validate_resident_crb_metadata(
            &valid_metadata(),
            &ResidentCrbStaticDims { depth: 1, ..dims }
        )
        .is_err()
    );
}

#[test]
#[cfg(feature = "cuda-probe")]
fn plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = ResidentCrbInitDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    let lengths = resident_crb_init_lengths(&dims).unwrap();
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
        Checked::<ResidentCrbInitMetadata>::assume_uploaded(
            bindings[0].span,
            &values,
            dims,
            revision,
        )
    }
    .unwrap();
    assert!(resident_crb_init_plan(&dims, &bindings, &[proof.binding(0)]).is_ok());

    // 容量：拱臂槽少一个元素。
    let mut short = bindings;
    short[1].span = span(7, 0x2000, (lengths[1] - 1) * 4);
    assert!(matches!(
        resident_crb_init_plan(&dims, &short, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 1, .. })
    ));
    // 缺证明：拓扑槽声明为地址类。
    assert!(matches!(
        resident_crb_init_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    // 别名：输出槽与质心输入槽重叠。
    let mut alias = bindings;
    alias[3].span = bindings[2].span;
    assert!(matches!(
        resident_crb_init_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    // 错会话：拱臂槽属于另一上下文。
    let mut foreign = bindings;
    foreign[1].span = span(8, 0x2000, lengths[1] * 4);
    assert!(matches!(
        resident_crb_init_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));

    // 累加计划：层级合法时通过，越界层级在长度阶段被拒绝。
    let accumulate = ResidentCrbAccumulateDims {
        static_dims: static_dims(),
        worlds: 2,
        level: 2,
    };
    let accumulate_lengths = resident_crb_accumulate_lengths(&accumulate).unwrap();
    let accumulate_bindings = [
        SlotBinding {
            slot: 0,
            span: bindings[0].span,
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: span(7, 0x2000, accumulate_lengths[1] * 4),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: span(7, 0x3000, accumulate_lengths[2] * 4),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: span(7, 0x4000, accumulate_lengths[3] * 4),
            elem: ElemType::F32,
        },
    ];
    let accumulate_proof = unsafe {
        Checked::<ResidentCrbAccumulateMetadata>::assume_uploaded(
            accumulate_bindings[0].span,
            &values,
            accumulate,
            MetadataRevision::new(),
        )
    }
    .unwrap();
    assert!(
        resident_crb_accumulate_plan(
            &accumulate,
            &accumulate_bindings,
            &[accumulate_proof.binding(0)]
        )
        .is_ok()
    );
    let mut foreign = accumulate_bindings;
    foreign[2].span = span(8, 0x3000, accumulate_lengths[2] * 4);
    assert!(matches!(
        resident_crb_accumulate_plan(&accumulate, &foreign, &[accumulate_proof.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
}
