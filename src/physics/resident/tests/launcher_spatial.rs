//! 常驻空间肌腱三项 FIELD_BATCH 包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::field_batch::field_plan;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher::SpatialTendonDims;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher::SpatialTendonDimsInput;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher::SpatialTendonStaticDims;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher::spatial_moment_lengths;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher::spatial_site_lengths;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher::spatial_wrap_lengths;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher::validate_spatial_tendon_metadata;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher_specs::SpatialMomentSpec;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher_specs::SpatialSiteSpec;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher_specs::SpatialTendonMetadata;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher_specs::SpatialWrapSpec;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher_specs::spatial_tendon_launch_error;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn static_dims_input() -> SpatialTendonDimsInput {
    SpatialTendonDimsInput {
        nb: 2,
        nv: 2,
        nt: 1,
        nnz: 1,
        nw: 2,
        ngeom: 1,
        nsite: 1,
        attached_stride: 24,
        parameters_len: 5,
    }
}

#[cfg(feature = "cuda-probe")]
fn static_dims() -> SpatialTendonStaticDims {
    SpatialTendonStaticDims::new(static_dims_input()).unwrap()
}

/// 头、体表、自由度表、肌腱行、包裹表与列索引。
#[cfg(feature = "cuda-probe")]
fn valid_metadata() -> Vec<i32> {
    vec![
        1, 2, 1, 2, 24, 12, 1, 1, 2, // 九项头
        -1, 0, // 体父链
        -1, 1, // 根体表
        0, 0, // 自由度地址
        0, 2, // 自由度计数
        0, // 肌腱行起始
        2, // 肌腱行包裹数
        0, // CSR行起始
        1, // CSR行非零数
        3, 3, // 包裹类型
        0, 0, // site索引
        1, 1, // 体索引
        -1, -1, // 侧点索引
        0,  // 列索引
    ]
}

#[cfg(feature = "cuda-probe")]
fn span(context: u64, addr: u64, bytes: u64) -> DeviceSpan {
    DeviceSpan::new(context, addr, bytes).unwrap()
}

#[cfg(feature = "cuda-probe")]
fn bindings(lengths: [u64; 4], out_elem: ElemType) -> [SlotBinding; 4] {
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
            elem: out_elem,
        },
    ]
}

#[cfg(feature = "cuda-probe")]
fn dims() -> SpatialTendonDims {
    SpatialTendonDims {
        static_dims: static_dims(),
        worlds: 2,
    }
}

#[test]
#[cfg(feature = "cuda-probe")]
fn lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let dims = dims();
    assert_eq!(spatial_site_lengths(&dims).unwrap(), [30, 5, 56, 88]);
    assert_eq!(spatial_moment_lengths(&dims).unwrap(), [30, 5, 88, 88]);
    assert_eq!(spatial_wrap_lengths(&dims).unwrap(), [30, 5, 88, 20]);
    assert_eq!(dims.abi(), [2, 0, 0, 2]);
    assert!(
        spatial_site_lengths(&SpatialTendonDims {
            static_dims: static_dims(),
            worlds: 0,
        })
        .is_err()
    );
    // 附着跨度必须等于12*(几何+site)对象。
    let mut input = static_dims_input();
    input.attached_stride = 19;
    assert!(SpatialTendonStaticDims::new(input).is_err());
    let mut input = static_dims_input();
    input.nb = 0;
    assert!(SpatialTendonStaticDims::new(input).is_err());
    let mut input = static_dims_input();
    input.nw = usize::MAX;
    assert!(SpatialTendonStaticDims::new(input).is_err());
}

#[test]
#[cfg(feature = "cuda-probe")]
fn metadata_proof_rejects_headers_tables_wraps_and_columns() {
    let dims = static_dims();
    assert!(validate_spatial_tendon_metadata(&valid_metadata(), dims).is_ok());
    for (index, value, reason) in [
        (0, 2, "肌腱数不符"),
        (4, 25, "附着跨度不符"),
        (5, 11, "几何块跨度不符"),
        (7, 0, "几何批次为零"),
        (8, 1, "参数偏移不符"),
        (10, 2, "体父链越界"),
        (12, -2, "根体表越界"),
        (15, 3, "自由度表越界"),
        (17, 1, "肌腱行未从零开始"),
        (21, 2, "稀疏行越界"),
        (22, 6, "包裹类型越界"),
        (24, 1, "site索引越界"),
        (26, 2, "体索引越界"),
        (28, 1, "侧点索引越界"),
        (29, 2, "列索引越界"),
    ] {
        let mut values = valid_metadata();
        values[index] = value;
        assert!(
            validate_spatial_tendon_metadata(&values, dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_spatial_tendon_metadata(&valid_metadata()[..29], dims).is_err());
}

#[cfg(feature = "cuda-probe")]
fn proof(bindings: &[SlotBinding; 4]) -> Result<Checked<SpatialTendonMetadata>, TransferError> {
    let dims = dims();
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    unsafe {
        Checked::<SpatialTendonMetadata>::assume_uploaded(
            bindings[0].span,
            &valid_metadata(),
            dims,
            MetadataRevision::new(),
        )
    }
    .map_err(spatial_tendon_launch_error)
}

#[test]
#[cfg(feature = "cuda-probe")]
fn site_and_moment_plans_reject_capacity_alias_context_and_missing_proof() {
    let dims = dims();
    for lengths in [
        spatial_site_lengths(&dims).unwrap(),
        spatial_moment_lengths(&dims).unwrap(),
    ] {
        let bindings = bindings(lengths, ElemType::F32);
        let checked = proof(&bindings).unwrap();
        let is_site = lengths == spatial_site_lengths(&dims).unwrap();
        if is_site {
            assert!(field_plan::<SpatialSiteSpec>(&dims, &bindings, &[checked.binding(0)]).is_ok());
        } else {
            assert!(
                field_plan::<SpatialMomentSpec>(&dims, &bindings, &[checked.binding(0)]).is_ok()
            );
        }
        let mut short = bindings;
        short[2].span = span(7, 0x3000, (lengths[2] - 1) * 4);
        assert!(matches!(
            if is_site {
                field_plan::<SpatialSiteSpec>(&dims, &short, &[checked.binding(0)])
            } else {
                field_plan::<SpatialMomentSpec>(&dims, &short, &[checked.binding(0)])
            },
            Err(LauncherError::Capacity { slot: 2, .. })
        ));
        assert!(matches!(
            if is_site {
                field_plan::<SpatialSiteSpec>(&dims, &bindings, &[])
            } else {
                field_plan::<SpatialMomentSpec>(&dims, &bindings, &[])
            },
            Err(LauncherError::MissingMetadataProof { slot: 0 })
        ));
        let mut alias = bindings;
        alias[1].span = bindings[3].span;
        assert!(matches!(
            if is_site {
                field_plan::<SpatialSiteSpec>(&dims, &alias, &[checked.binding(0)])
            } else {
                field_plan::<SpatialMomentSpec>(&dims, &alias, &[checked.binding(0)])
            },
            Err(LauncherError::Alias { .. })
        ));
        let mut foreign = bindings;
        foreign[1].span = span(8, 0x2000, lengths[1] * 4);
        assert!(matches!(
            if is_site {
                field_plan::<SpatialSiteSpec>(&dims, &foreign, &[checked.binding(0)])
            } else {
                field_plan::<SpatialMomentSpec>(&dims, &foreign, &[checked.binding(0)])
            },
            Err(LauncherError::ContextMismatch { .. })
        ));
    }
}

#[test]
#[cfg(feature = "cuda-probe")]
fn wrap_plan_accepts_i32_output_and_rejects_alias_and_capacity() {
    let dims = dims();
    let lengths = spatial_wrap_lengths(&dims).unwrap();
    let bindings = bindings(lengths, ElemType::I32);
    assert_eq!(bindings[3].elem, ElemType::I32);
    let checked = proof(&bindings).unwrap();
    assert!(field_plan::<SpatialWrapSpec>(&dims, &bindings, &[checked.binding(0)]).is_ok());
    let mut short = bindings;
    short[3].span = span(7, 0x9000, (lengths[3] - 1) * 4);
    assert!(matches!(
        field_plan::<SpatialWrapSpec>(&dims, &short, &[checked.binding(0)]),
        Err(LauncherError::Capacity { slot: 3, .. })
    ));
    let mut alias = bindings;
    alias[1].span = bindings[3].span;
    assert!(matches!(
        field_plan::<SpatialWrapSpec>(&dims, &alias, &[checked.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
}
