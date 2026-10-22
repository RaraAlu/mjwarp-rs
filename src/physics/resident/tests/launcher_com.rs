//! 常驻质心位置 FIELD_BATCH 包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::com_launcher_abi::ResidentComDims;
#[cfg(feature = "cuda-probe")]
use super::com_launcher_abi::ResidentComStaticDims;
#[cfg(feature = "cuda-probe")]
use super::com_launcher_meta::ResidentComMetadata;
#[cfg(feature = "cuda-probe")]
use super::com_launcher_meta::validate_resident_com_metadata;
#[cfg(feature = "cuda-probe")]
use super::com_launcher_specs::resident_com_lengths;
#[cfg(feature = "cuda-probe")]
use super::com_launcher_specs::resident_com_plan;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn static_dims() -> ResidentComStaticDims {
    ResidentComStaticDims {
        nb: 3,
        nj: 2,
        nv: 4,
        parameters_len: 21,
    }
}

#[cfg(feature = "cuda-probe")]
fn valid_metadata() -> Vec<i32> {
    // 体父 [0,0,1]、类型 [球,铰]、关节体 [1,2]、自由度 [0,3]、
    // 描述符：质量 offset=0 batches=1；惯量 offset=3 batches=2。
    vec![0, 0, 1, 1, 3, 1, 2, 0, 3, 0, 1, 3, 2]
}

#[test]
#[cfg(feature = "cuda-probe")]
fn lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let dims = ResidentComDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    assert_eq!(resident_com_lengths(&dims).unwrap(), [13, 21, 200, 140]);
    assert_eq!(dims.abi(), [3, 2, 4, 2]);
    assert!(
        resident_com_lengths(&ResidentComDims {
            static_dims: ResidentComStaticDims {
                nb: 0,
                ..static_dims()
            },
            worlds: 1,
        })
        .is_err()
    );
    assert!(
        resident_com_lengths(&ResidentComDims {
            static_dims: static_dims(),
            worlds: 0,
        })
        .is_err()
    );
    assert!(
        resident_com_lengths(&ResidentComDims {
            static_dims: static_dims(),
            worlds: u32::MAX - 254,
        })
        .is_err()
    );
}

#[test]
#[cfg(feature = "cuda-probe")]
fn metadata_proof_rejects_bad_parents_types_addresses_and_descriptors() {
    let dims = static_dims();
    assert!(validate_resident_com_metadata(&valid_metadata(), &dims).is_ok());
    for (index, value, reason) in [
        (0, 1, "根体父引用"),
        (2, -1, "体父链越界"),
        (3, 4, "关节类型越界"),
        (5, 3, "关节体越界"),
        (7, 2, "球关节自由度越界"),
        (8, 4, "铰关节自由度越界"),
        (9, 20, "质量字段区间越界"),
        (10, 0, "质量字段批次为零"),
        (11, 17, "惯量字段区间越界"),
        (12, 0, "惯量字段批次为零"),
    ] {
        let mut values = valid_metadata();
        values[index] = value;
        assert!(
            validate_resident_com_metadata(&values, &dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_resident_com_metadata(&valid_metadata()[..12], &dims).is_err());
}

#[test]
#[cfg(feature = "cuda-probe")]
fn plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = ResidentComDims {
        static_dims: static_dims(),
        worlds: 2,
    };
    let lengths = resident_com_lengths(&dims).unwrap();
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
        Checked::<ResidentComMetadata>::assume_uploaded(bindings[0].span, &values, dims, revision)
    }
    .unwrap();
    assert!(resident_com_plan(&dims, &bindings, &[proof.binding(0)]).is_ok());

    // 容量：参数槽少一个元素。
    let mut short = bindings;
    short[1].span = span(7, 0x2000, (lengths[1] - 1) * 4);
    assert!(matches!(
        resident_com_plan(&dims, &short, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 1, .. })
    ));
    // 缺证明：元数据槽声明为地址类。
    assert!(matches!(
        resident_com_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    // 别名：输出槽与运动学输入槽重叠。
    let mut alias = bindings;
    alias[3].span = bindings[2].span;
    assert!(matches!(
        resident_com_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    // 错会话：参数槽属于另一上下文。
    let mut foreign = bindings;
    foreign[1].span = span(8, 0x2000, lengths[1] * 4);
    assert!(matches!(
        resident_com_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
}
