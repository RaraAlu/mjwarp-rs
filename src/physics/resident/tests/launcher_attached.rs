//! 常驻附着运动学 FIELD_BATCH 包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::attached_launcher_abi::ResidentAttachedDims;
#[cfg(feature = "cuda-probe")]
use super::attached_launcher_abi::ResidentAttachedStaticDims;
#[cfg(feature = "cuda-probe")]
use super::attached_launcher_meta::ResidentAttachedMetadata;
#[cfg(feature = "cuda-probe")]
use super::attached_launcher_meta::validate_resident_attached_metadata;
#[cfg(feature = "cuda-probe")]
use super::attached_launcher_specs::resident_attached_lengths;
#[cfg(feature = "cuda-probe")]
use super::attached_launcher_specs::resident_attached_plan;
#[cfg(feature = "cuda-probe")]
use super::attached_launcher_specs::resident_initialize_attached_plan;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn static_dims() -> ResidentAttachedStaticDims {
    ResidentAttachedStaticDims {
        nb: 3,
        nj: 2,
        ng: 2,
        ns: 1,
        parameters_len: 21,
    }
}

#[cfg(feature = "cuda-probe")]
fn valid_metadata() -> Vec<i32> {
    // 跨度96、几何体 [1,2]、site体 [0]、静态几何 [0,1]、
    // 描述符：geom_pos(0,1)、geom_quat(6,1)、site_pos(14,1)、site_quat(17,1)。
    vec![96, 1, 2, 0, 0, 1, 0, 1, 6, 1, 14, 1, 17, 1]
}

#[test]
#[cfg(feature = "cuda-probe")]
fn lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let dims = ResidentAttachedDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    assert_eq!(resident_attached_lengths(&dims).unwrap(), [14, 21, 200, 80]);
    assert_eq!(dims.abi(), [3, 2, 1, 2]);

    // 空附着集合：元数据只剩跨度与描述符，参数保留一项。
    let empty = ResidentAttachedStaticDims {
        nb: 1,
        nj: 0,
        ng: 0,
        ns: 0,
        parameters_len: 1,
    };
    assert_eq!(
        resident_attached_lengths(&ResidentAttachedDims {
            static_dims: empty,
            worlds: 2,
        })
        .unwrap(),
        [9, 1, 64, 8]
    );
    assert!(
        resident_attached_lengths(&ResidentAttachedDims {
            static_dims: ResidentAttachedStaticDims {
                nb: 0,
                ..static_dims()
            },
            worlds: 1,
        })
        .is_err()
    );
    assert!(
        resident_attached_lengths(&ResidentAttachedDims {
            static_dims: static_dims(),
            worlds: 0,
        })
        .is_err()
    );
    assert!(
        resident_attached_lengths(&ResidentAttachedDims {
            static_dims: static_dims(),
            worlds: u32::MAX - 254,
        })
        .is_err()
    );
}

#[test]
#[cfg(feature = "cuda-probe")]
fn metadata_proof_rejects_bad_stride_bodies_flags_and_descriptors() {
    let dims = static_dims();
    assert!(validate_resident_attached_metadata(&valid_metadata(), &dims).is_ok());
    for (index, value, reason) in [
        (0, 95, "刚体跨度不符"),
        (1, 3, "几何体越界"),
        (3, -1, "site体为负"),
        (4, 2, "静态几何标志非布尔"),
        (9, 0, "geom_quat批次为零"),
        (10, 19, "site_pos区间越界"),
        (12, 18, "site_quat区间越界"),
    ] {
        let mut values = valid_metadata();
        values[index] = value;
        assert!(
            validate_resident_attached_metadata(&values, &dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_resident_attached_metadata(&valid_metadata()[..13], &dims).is_err());
}

#[test]
#[cfg(feature = "cuda-probe")]
fn plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = ResidentAttachedDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    let lengths = resident_attached_lengths(&dims).unwrap();
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
            span: span(7, 0x9000, lengths[3] * 4),
            elem: ElemType::F32,
        },
    ];
    let values = valid_metadata();
    let revision = MetadataRevision::new();
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    let proof = unsafe {
        Checked::<ResidentAttachedMetadata>::assume_uploaded(
            bindings[0].span,
            &values,
            dims,
            revision,
        )
    }
    .unwrap();
    assert!(resident_attached_plan(&dims, &bindings, &[proof.binding(0)]).is_ok());

    // 容量：参数槽少一个元素。
    let mut short = bindings;
    short[1].span = span(7, 0x2000, (lengths[1] - 1) * 4);
    assert!(matches!(
        resident_attached_plan(&dims, &short, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 1, .. })
    ));
    // 缺证明：元数据槽声明为地址类。
    assert!(matches!(
        resident_attached_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    // 别名：输出槽与刚体输入槽重叠。
    let mut alias = bindings;
    alias[3].span = bindings[2].span;
    assert!(matches!(
        resident_attached_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    // 错会话：参数槽属于另一上下文。
    let mut foreign = bindings;
    foreign[1].span = span(8, 0x2000, lengths[1] * 4);
    assert!(matches!(
        resident_attached_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));

    // 初始化入口与附着入口共用 ABI 与证明。
    assert!(resident_initialize_attached_plan(&dims, &bindings, &[proof.binding(0)]).is_ok());
    let mut overlap = bindings;
    overlap[3].span = bindings[2].span;
    assert!(matches!(
        resident_initialize_attached_plan(&dims, &overlap, &[proof.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
}
