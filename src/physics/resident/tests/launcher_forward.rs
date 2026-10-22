//! 常驻固定肌腱、相机与光源 FIELD_BATCH 包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::camlight_launcher::CamLightComSpec;
#[cfg(feature = "cuda-probe")]
use super::camlight_launcher::CamLightDims;
#[cfg(feature = "cuda-probe")]
use super::camlight_launcher::CamLightMetadata;
#[cfg(feature = "cuda-probe")]
use super::camlight_launcher::CamLightRigidSpec;
#[cfg(feature = "cuda-probe")]
use super::camlight_launcher::CamLightStaticDims;
#[cfg(feature = "cuda-probe")]
use super::camlight_launcher::camlight_com_lengths;
#[cfg(feature = "cuda-probe")]
use super::camlight_launcher::camlight_rigid_lengths;
#[cfg(feature = "cuda-probe")]
use super::camlight_launcher::validate_camlight_metadata;
#[cfg(feature = "cuda-probe")]
use super::field_batch::field_plan;
#[cfg(feature = "cuda-probe")]
use super::fixed_tendon_launcher::FixedTendonDims;
#[cfg(feature = "cuda-probe")]
use super::fixed_tendon_launcher::FixedTendonMetadata;
#[cfg(feature = "cuda-probe")]
use super::fixed_tendon_launcher::FixedTendonSpec;
#[cfg(feature = "cuda-probe")]
use super::fixed_tendon_launcher::FixedTendonStaticDims;
#[cfg(feature = "cuda-probe")]
use super::fixed_tendon_launcher::fixed_tendon_lengths;
#[cfg(feature = "cuda-probe")]
use super::fixed_tendon_launcher::validate_fixed_tendon_metadata;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn fixed_tendon_static_dims() -> FixedTendonStaticDims {
    FixedTendonStaticDims::new(4, 2, 3, 3, 1, 3).unwrap()
}

/// 包裹行 [0,1)+[1,3)，qpos地址[0,1,2]，力臂槽[0,1,2]。
#[cfg(feature = "cuda-probe")]
fn fixed_tendon_metadata() -> Vec<i32> {
    vec![3, 0, 1, 1, 2, 0, 1, 2, 0, 1, 2]
}

#[cfg(feature = "cuda-probe")]
fn camlight_static_dims() -> CamLightStaticDims {
    // 相机位置(0)、四元数(3)、质心位置(7)、参考位置(10)、矩阵(13)，
    // 光源位置(22)、方向(25)、质心位置(28)、参考位置(31)、参考方向(34)。
    CamLightStaticDims::new(2, 16, 36, 1, 1, 37).unwrap()
}

#[cfg(feature = "cuda-probe")]
fn camlight_metadata() -> Vec<i32> {
    vec![
        16, 36, 0, 1, -2, 2, 0, -2, 0, 1, 3, 1, 7, 1, 10, 1, 13, 1, 22, 1, 25, 1, 28, 1, 31, 1, 34,
        1,
    ]
}

#[test]
#[cfg(feature = "cuda-probe")]
fn lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let fixed = FixedTendonDims {
        static_dims: fixed_tendon_static_dims(),
        worlds: 2,
    };
    assert_eq!(fixed_tendon_lengths(&fixed).unwrap(), [11, 3, 22, 18]);
    assert_eq!(fixed.abi(), [4, 2, 3, 2]);
    assert!(
        fixed_tendon_lengths(&FixedTendonDims {
            static_dims: fixed_tendon_static_dims(),
            worlds: 0,
        })
        .is_err()
    );
    assert!(
        fixed_tendon_lengths(&FixedTendonDims {
            static_dims: fixed_tendon_static_dims(),
            worlds: u32::MAX - 254,
        })
        .is_err()
    );
    assert!(FixedTendonStaticDims::new(usize::MAX, 1, 1, 1, 0, 1).is_err());

    let cam = CamLightDims {
        static_dims: camlight_static_dims(),
        worlds: 2,
    };
    assert_eq!(camlight_rigid_lengths(&cam).unwrap(), [28, 37, 40, 44]);
    assert_eq!(camlight_com_lengths(&cam).unwrap(), [28, 37, 80, 44]);
    assert_eq!(cam.abi(), [2, 1, 1, 2]);
    assert!(
        camlight_rigid_lengths(&CamLightDims {
            static_dims: camlight_static_dims(),
            worlds: 0,
        })
        .is_err()
    );
    assert!(CamLightStaticDims::new(0, 16, 36, 1, 1, 37).is_err());
    assert!(CamLightStaticDims::new(2, usize::MAX, 36, 1, 1, 37).is_err());
}

#[test]
#[cfg(feature = "cuda-probe")]
fn fixed_tendon_metadata_proof_rejects_rows_addresses_and_length() {
    let dims = fixed_tendon_static_dims();
    assert!(validate_fixed_tendon_metadata(&fixed_tendon_metadata(), dims).is_ok());
    for (index, value, reason) in [
        (0, 2, "包裹数不符"),
        (1, 1, "行起始未从零开始"),
        (3, 3, "行覆盖超出包裹数"),
        (5, 4, "qpos地址越界"),
        (8, 3, "力臂槽越界"),
    ] {
        let mut values = fixed_tendon_metadata();
        values[index] = value;
        assert!(
            validate_fixed_tendon_metadata(&values, dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_fixed_tendon_metadata(&fixed_tendon_metadata()[..10], dims).is_err());
}

#[test]
#[cfg(feature = "cuda-probe")]
fn camlight_metadata_proof_rejects_modes_bodies_and_descriptors() {
    let dims = camlight_static_dims();
    assert!(validate_camlight_metadata(&camlight_metadata(), dims).is_ok());
    for (index, value, reason) in [
        (0, 17, "刚体跨度头不符"),
        (1, 35, "质心跨度头不符"),
        (2, 5, "相机模式越界"),
        (3, 2, "相机体越界"),
        (4, 2, "相机目标体越界"),
        (5, 5, "光源模式越界"),
        (6, -1, "光源体为负"),
        (7, 2, "光源目标体越界"),
        (8, -1, "相机位置偏移为负"),
        (9, 0, "相机位置批次为零"),
        (27, 2, "参考方向批次越界"),
    ] {
        let mut values = camlight_metadata();
        values[index] = value;
        assert!(
            validate_camlight_metadata(&values, dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_camlight_metadata(&camlight_metadata()[..27], dims).is_err());
}

#[cfg(feature = "cuda-probe")]
fn span(context: u64, addr: u64, bytes: u64) -> DeviceSpan {
    DeviceSpan::new(context, addr, bytes).unwrap()
}

#[cfg(feature = "cuda-probe")]
fn camlight_bindings(lengths: [u64; 4]) -> [SlotBinding; 4] {
    [
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
    ]
}

#[test]
#[cfg(feature = "cuda-probe")]
fn fixed_tendon_plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = FixedTendonDims {
        static_dims: fixed_tendon_static_dims(),
        worlds: 2,
    };
    let lengths = fixed_tendon_lengths(&dims).unwrap();
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
    let values = fixed_tendon_metadata();
    let revision = MetadataRevision::new();
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    let proof = unsafe {
        Checked::<FixedTendonMetadata>::assume_uploaded(bindings[0].span, &values, dims, revision)
    }
    .unwrap();
    assert!(field_plan::<FixedTendonSpec>(&dims, &bindings, &[proof.binding(0)]).is_ok());
    let mut short = bindings;
    short[2].span = span(7, 0x3000, (lengths[2] - 1) * 4);
    assert!(matches!(
        field_plan::<FixedTendonSpec>(&dims, &short, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 2, .. })
    ));
    assert!(matches!(
        field_plan::<FixedTendonSpec>(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    let mut alias = bindings;
    alias[3].span = bindings[2].span;
    assert!(matches!(
        field_plan::<FixedTendonSpec>(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    let mut foreign = bindings;
    foreign[1].span = span(8, 0x2000, lengths[1] * 4);
    assert!(matches!(
        field_plan::<FixedTendonSpec>(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
}

#[test]
#[cfg(feature = "cuda-probe")]
fn camlight_rigid_plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = CamLightDims {
        static_dims: camlight_static_dims(),
        worlds: 2,
    };
    let lengths = camlight_rigid_lengths(&dims).unwrap();
    let bindings = camlight_bindings(lengths);
    let values = camlight_metadata();
    let revision = MetadataRevision::new();
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    let proof = unsafe {
        Checked::<CamLightMetadata>::assume_uploaded(
            bindings[0].span,
            &values,
            dims,
            revision.clone(),
        )
    }
    .unwrap();
    assert!(field_plan::<CamLightRigidSpec>(&dims, &bindings, &[proof.binding(0)]).is_ok());
    let mut short = bindings;
    short[2].span = span(7, 0x3000, (lengths[2] - 1) * 4);
    assert!(matches!(
        field_plan::<CamLightRigidSpec>(&dims, &short, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 2, .. })
    ));
    assert!(matches!(
        field_plan::<CamLightRigidSpec>(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    let mut alias = bindings;
    alias[1].span = bindings[3].span;
    assert!(matches!(
        field_plan::<CamLightRigidSpec>(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    let mut foreign = bindings;
    foreign[1].span = span(8, 0x2000, lengths[1] * 4);
    assert!(matches!(
        field_plan::<CamLightRigidSpec>(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
}

#[test]
#[cfg(feature = "cuda-probe")]
fn camlight_com_plan_accepts_wide_com_input_and_rejects_alias() {
    let dims = CamLightDims {
        static_dims: camlight_static_dims(),
        worlds: 2,
    };
    let lengths = camlight_com_lengths(&dims).unwrap();
    let bindings = camlight_bindings(lengths);
    let values = camlight_metadata();
    let revision = MetadataRevision::new();
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    let proof = unsafe {
        Checked::<CamLightMetadata>::assume_uploaded(bindings[0].span, &values, dims, revision)
    }
    .unwrap();
    assert!(field_plan::<CamLightComSpec>(&dims, &bindings, &[proof.binding(0)]).is_ok());
    // 质心输入比刚体输入宽：同字节数会因精确跨度不符被拒绝。
    let narrow = camlight_bindings(camlight_rigid_lengths(&dims).unwrap());
    assert!(matches!(
        field_plan::<CamLightComSpec>(&dims, &narrow, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 2, .. })
    ));
    let mut alias = bindings;
    alias[3].span = bindings[2].span;
    assert!(matches!(
        field_plan::<CamLightComSpec>(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
}
