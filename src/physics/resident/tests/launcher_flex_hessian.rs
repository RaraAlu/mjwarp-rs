//! 常驻柔体 Hessian 缓存与发布 FIELD_BATCH 包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::field_batch::field_plan;
#[cfg(feature = "cuda-probe")]
use super::flex_hessian_launcher::FlexHessianDims;
#[cfg(feature = "cuda-probe")]
use super::flex_hessian_launcher::FlexHessianDimsInput;
#[cfg(feature = "cuda-probe")]
use super::flex_hessian_launcher::FlexHessianStaticDims;
#[cfg(feature = "cuda-probe")]
use super::flex_hessian_launcher::flex_hessian_lengths;
#[cfg(feature = "cuda-probe")]
use super::flex_hessian_launcher::flex_hessian_validate_lengths;
#[cfg(feature = "cuda-probe")]
use super::flex_hessian_launcher::validate_flex_hessian_metadata;
#[cfg(feature = "cuda-probe")]
use super::flex_hessian_launcher_specs::FlexHessianMetadata;
#[cfg(feature = "cuda-probe")]
use super::flex_hessian_launcher_specs::FlexHessianSpec;
#[cfg(feature = "cuda-probe")]
use super::flex_hessian_launcher_specs::FlexHessianValidateSpec;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn input() -> FlexHessianDimsInput {
    FlexHessianDimsInput {
        nf: 1,
        nv: 3,
        nn: 0,
        ne: 1,
        pos_stride: 9,
        edge_stride: 2,
        parameters_len: 22,
        metadata_elements: 28,
    }
}

#[cfg(feature = "cuda-probe")]
fn static_dims() -> FlexHessianStaticDims {
    FlexHessianStaticDims::new(input()).unwrap()
}
/// 头、单柔体行、单元顶点、单元边索引与边端点。
#[cfg(feature = "cuda-probe")]
fn valid_metadata() -> Vec<i32> {
    vec![
        1, 3, 1, 9, 2, 20, 23, 26, 0, // 九项头
        2, 1, 0, 3, 0, 1, 0, 1, 0, 0,
        0, // 单元维度/激活、顶点/边/元素区间与地址
        0, 1, 2, // 单元顶点
        0, 0, 0, // 单元边索引
        0, 1, // 边端点
    ]
}

#[cfg(feature = "cuda-probe")]
fn dims() -> FlexHessianDims {
    FlexHessianDims {
        static_dims: static_dims(),
        worlds: 2,
    }
}

#[cfg(feature = "cuda-probe")]
fn span(context: u64, addr: u64, bytes: u64) -> DeviceSpan {
    DeviceSpan::new(context, addr, bytes).unwrap()
}

#[cfg(feature = "cuda-probe")]
fn binding(slot: usize, context: u64, addr: u64, length: u64, elem: ElemType) -> SlotBinding {
    SlotBinding {
        slot,
        span: span(context, addr, length * 4),
        elem,
    }
}

#[test]
#[cfg(feature = "cuda-probe")]
fn lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let dims = dims();
    assert_eq!(
        flex_hessian_lengths(&dims).unwrap(),
        [28, 22, 26, 12, 10, 62]
    );
    assert_eq!(
        flex_hessian_validate_lengths(&dims).unwrap(),
        [28, 22, 62, 10]
    );
    assert_eq!(dims.abi(), [0, 0, 0, 2]);
    assert!(
        flex_hessian_lengths(&FlexHessianDims {
            static_dims: static_dims(),
            worlds: 0,
        })
        .is_err()
    );
    // 位置跨度必须为3*nn+3*nv；边长跨度至少2*ne。
    assert!(
        FlexHessianStaticDims::new(FlexHessianDimsInput {
            pos_stride: 8,
            ..input()
        })
        .is_err()
    );
    assert!(
        FlexHessianStaticDims::new(FlexHessianDimsInput {
            edge_stride: 1,
            ..input()
        })
        .is_err()
    );
    assert!(FlexHessianStaticDims::new(FlexHessianDimsInput { nf: 0, ..input() }).is_err());
    assert!(FlexHessianStaticDims::new(FlexHessianDimsInput { ne: 0, ..input() }).is_err());
}

#[test]
#[cfg(feature = "cuda-probe")]
fn metadata_proof_rejects_offsets_rows_elements_and_coefficients() {
    let dims = static_dims();
    assert!(validate_flex_hessian_metadata(&valid_metadata(), dims).is_ok());
    for (index, value, reason) in [
        (0, 2, "柔体数不符"),
        (3, 8, "位置跨度不符"),
        (5, 21, "元素偏移不符"),
        (8, 3, "节点偏移不符"),
        (9, 1, "单元维度无效"),
        (10, 2, "激活标志无效"),
        (12, 4, "顶点区间越界"),
        (14, 2, "边区间越界"),
        (17, 2, "元素数据区间越界"),
        (18, 1, "元素边区间越界"),
        (19, 2, "系数区间越界"),
        (20, 3, "单元顶点越界"),
        (23, 1, "单元边索引越界"),
        (26, 3, "边端点越界"),
    ] {
        let mut values = valid_metadata();
        values[index] = value;
        assert!(
            validate_flex_hessian_metadata(&values, dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_flex_hessian_metadata(&valid_metadata()[..27], dims).is_err());
    // 元数据边数据长度必须等于2*ne。
    let bad = FlexHessianStaticDims::new(FlexHessianDimsInput {
        metadata_elements: 29,
        ..input()
    })
    .unwrap();
    assert!(validate_flex_hessian_metadata(&valid_metadata(), bad).is_err());
}

#[cfg(feature = "cuda-probe")]
fn proof(bindings: &[SlotBinding; 6]) -> Checked<FlexHessianMetadata> {
    let dims = dims();
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    unsafe {
        Checked::<FlexHessianMetadata>::assume_uploaded(
            bindings[0].span,
            &valid_metadata(),
            dims,
            MetadataRevision::new(),
        )
    }
    .unwrap()
}

#[test]
#[cfg(feature = "cuda-probe")]
fn hessian_plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = dims();
    let bindings = [
        binding(0, 7, 0x1000, 28, ElemType::I32),
        binding(1, 7, 0x2000, 22, ElemType::F32),
        binding(2, 7, 0x3000, 26, ElemType::F32),
        binding(3, 7, 0x4000, 12, ElemType::F32),
        binding(4, 7, 0x5000, 10, ElemType::I32),
        binding(5, 7, 0x9000, 62, ElemType::F32),
    ];
    let checked = proof(&bindings);
    assert!(field_plan::<FlexHessianSpec>(&dims, &bindings, &[checked.binding(0)]).is_ok());
    let mut short = bindings;
    short[5].span = span(7, 0x9000, 61 * 4);
    assert!(matches!(
        field_plan::<FlexHessianSpec>(&dims, &short, &[checked.binding(0)]),
        Err(LauncherError::Capacity { slot: 5, .. })
    ));
    assert!(matches!(
        field_plan::<FlexHessianSpec>(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    let mut alias = bindings;
    alias[3].span = bindings[5].span;
    assert!(matches!(
        field_plan::<FlexHessianSpec>(&dims, &alias, &[checked.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    let mut foreign = bindings;
    foreign[4].span = span(8, 0x5000, 10 * 4);
    assert!(matches!(
        field_plan::<FlexHessianSpec>(&dims, &foreign, &[checked.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
}

#[test]
#[cfg(feature = "cuda-probe")]
fn validate_plan_rejects_capacity_context_and_missing_proof() {
    let dims = dims();
    let bindings = [
        binding(0, 7, 0x1000, 28, ElemType::I32),
        binding(1, 7, 0x2000, 22, ElemType::F32),
        binding(2, 7, 0x3000, 62, ElemType::F32),
        binding(3, 7, 0x9000, 10, ElemType::I32),
    ];
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    let checked = unsafe {
        Checked::<FlexHessianMetadata>::assume_uploaded(
            bindings[0].span,
            &valid_metadata(),
            dims,
            MetadataRevision::new(),
        )
    }
    .unwrap();
    assert!(field_plan::<FlexHessianValidateSpec>(&dims, &bindings, &[checked.binding(0)]).is_ok());
    let mut short = bindings;
    short[3].span = span(7, 0x9000, 9 * 4);
    assert!(matches!(
        field_plan::<FlexHessianValidateSpec>(&dims, &short, &[checked.binding(0)]),
        Err(LauncherError::Capacity { slot: 3, .. })
    ));
    assert!(matches!(
        field_plan::<FlexHessianValidateSpec>(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    let mut foreign = bindings;
    foreign[2].span = span(8, 0x3000, 62 * 4);
    assert!(matches!(
        field_plan::<FlexHessianValidateSpec>(&dims, &foreign, &[checked.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
}
