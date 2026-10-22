//! 常驻柔体失效与位置 FIELD_BATCH 包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::field_batch::field_plan;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher::FlexDims;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher::FlexStaticDims;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher::flex_invalidate_lengths;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher::flex_positions_lengths;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher::validate_flex_metadata;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher_specs::FlexInvalidateSpec;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher_specs::FlexMetadata;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher_specs::FlexPositionsSpec;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn static_dims() -> FlexStaticDims {
    FlexStaticDims::new(1, 8, 2, 2, 62, 36).unwrap()
}

/// 头、单柔体插值行、节点体与顶点体。
#[cfg(feature = "cuda-probe")]
fn valid_metadata() -> Vec<i32> {
    vec![
        1, 8, 2, 2, 62, // 五项头
        1, 1, 1, 1, 0, 8, 0, 2, 0, // 插值类型、cell维度、行区间、居中标志
        0, 1, 0, 1, 0, 1, 0, 1, // 节点体
        1, 0, // 顶点体
    ]
}

#[cfg(feature = "cuda-probe")]
fn dims() -> FlexDims {
    FlexDims {
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
    assert_eq!(flex_invalidate_lengths(&dims).unwrap(), [24, 36, 132, 10]);
    assert_eq!(flex_positions_lengths(&dims).unwrap(), [24, 36, 132, 68]);
    assert_eq!(dims.abi(), [0, 0, 0, 2]);
    assert!(
        flex_positions_lengths(&FlexDims {
            static_dims: static_dims(),
            worlds: 0,
        })
        .is_err()
    );
    // 参数跨度必须等于6*nv+3*nn。
    assert!(FlexStaticDims::new(1, 8, 2, 2, 62, 35).is_err());
    // 刚体跨度必须至少覆盖28*nb与16*nb下界。
    assert!(FlexStaticDims::new(1, 8, 2, 2, 31, 36).is_err());
    assert!(FlexStaticDims::new(0, 8, 2, 2, 62, 36).is_err());
    assert!(FlexStaticDims::new(1, 8, 2, usize::MAX, 62, 36).is_err());
}

#[test]
#[cfg(feature = "cuda-probe")]
fn metadata_proof_rejects_rows_grids_and_bodies() {
    let dims = static_dims();
    assert!(validate_flex_metadata(&valid_metadata(), dims).is_ok());
    for (index, value, reason) in [
        (0, 2, "柔体数不符"),
        (4, 61, "刚体跨度不符"),
        (5, 4, "插值类型越界"),
        (6, 0, "cell维度无效"),
        (6, 2, "插值网格越界"),
        (9, -1, "节点地址为负"),
        (10, 9, "节点区间越界"),
        (12, 3, "顶点区间越界"),
        (13, 2, "居中标志无效"),
        (14, 2, "节点体越界"),
    ] {
        let mut values = valid_metadata();
        values[index] = value;
        assert!(
            validate_flex_metadata(&values, dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_flex_metadata(&valid_metadata()[..23], dims).is_err());
}

#[cfg(feature = "cuda-probe")]
fn proof(bindings: &[SlotBinding; 4]) -> Checked<FlexMetadata> {
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    unsafe {
        Checked::<FlexMetadata>::assume_uploaded(
            bindings[0].span,
            &valid_metadata(),
            dims(),
            MetadataRevision::new(),
        )
    }
    .unwrap()
}

#[test]
#[cfg(feature = "cuda-probe")]
fn invalidate_plan_rejects_capacity_context_and_missing_proof() {
    let dims = dims();
    let bindings = [
        binding(0, 7, 0x1000, 24, ElemType::I32),
        binding(1, 7, 0x2000, 36, ElemType::F32),
        binding(2, 7, 0x3000, 132, ElemType::F32),
        binding(3, 7, 0x9000, 10, ElemType::I32),
    ];
    let checked = proof(&bindings);
    assert!(field_plan::<FlexInvalidateSpec>(&dims, &bindings, &[checked.binding(0)]).is_ok());
    let mut short = bindings;
    short[3].span = span(7, 0x9000, 9 * 4);
    assert!(matches!(
        field_plan::<FlexInvalidateSpec>(&dims, &short, &[checked.binding(0)]),
        Err(LauncherError::Capacity { slot: 3, .. })
    ));
    assert!(matches!(
        field_plan::<FlexInvalidateSpec>(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    let mut foreign = bindings;
    foreign[2].span = span(8, 0x3000, 132 * 4);
    assert!(matches!(
        field_plan::<FlexInvalidateSpec>(&dims, &foreign, &[checked.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
}

#[test]
#[cfg(feature = "cuda-probe")]
fn positions_plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = dims();
    let bindings = [
        binding(0, 7, 0x1000, 24, ElemType::I32),
        binding(1, 7, 0x2000, 36, ElemType::F32),
        binding(2, 7, 0x3000, 132, ElemType::F32),
        binding(3, 7, 0x9000, 68, ElemType::F32),
    ];
    let checked = proof(&bindings);
    assert!(field_plan::<FlexPositionsSpec>(&dims, &bindings, &[checked.binding(0)]).is_ok());
    let mut short = bindings;
    short[2].span = span(7, 0x3000, 131 * 4);
    assert!(matches!(
        field_plan::<FlexPositionsSpec>(&dims, &short, &[checked.binding(0)]),
        Err(LauncherError::Capacity { slot: 2, .. })
    ));
    assert!(matches!(
        field_plan::<FlexPositionsSpec>(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    // 参数槽与可写输出槽重叠：输出跨度大于参数，容量先通过再报别名。
    let mut alias = bindings;
    alias[1].span = bindings[3].span;
    assert!(matches!(
        field_plan::<FlexPositionsSpec>(&dims, &alias, &[checked.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    let mut foreign = bindings;
    foreign[1].span = span(8, 0x2000, 36 * 4);
    assert!(matches!(
        field_plan::<FlexPositionsSpec>(&dims, &foreign, &[checked.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
}
#[cfg(feature = "cuda-probe")]
fn vertex_static_dims() -> FlexStaticDims {
    FlexStaticDims::new(1, 0, 2, 2, 62, 12).unwrap()
}

#[test]
#[cfg(feature = "cuda-probe")]
fn vertex_rows_require_valid_vertex_bodies() {
    let dims = vertex_static_dims();
    // 顶点类型：无节点、两个顶点、顶点体[0,1]。
    let mut values = vec![1, 0, 2, 2, 62, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 1];
    assert!(validate_flex_metadata(&values, dims).is_ok());
    // 顶点类型缺省节点体检查不读取节点体表。
    values[14] = -1;
    assert!(validate_flex_metadata(&values, dims).is_err());
    values[14] = 0;
    values[15] = 2;
    assert!(validate_flex_metadata(&values, dims).is_err());
}
