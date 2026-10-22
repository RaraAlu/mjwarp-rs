//! 常驻柔体边与面 FIELD_BATCH 包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::field_batch::field_plan;
#[cfg(feature = "cuda-probe")]
use super::flex_edges_launcher::FlexEdgesDims;
#[cfg(feature = "cuda-probe")]
use super::flex_edges_launcher::FlexEdgesMetadata;
#[cfg(feature = "cuda-probe")]
use super::flex_edges_launcher::FlexEdgesSpec;
#[cfg(feature = "cuda-probe")]
use super::flex_edges_launcher::FlexEdgesStaticDims;
#[cfg(feature = "cuda-probe")]
use super::flex_edges_launcher::flex_edges_launch_error;
#[cfg(feature = "cuda-probe")]
use super::flex_edges_launcher::flex_edges_lengths;
#[cfg(feature = "cuda-probe")]
use super::flex_edges_launcher::validate_flex_edges_metadata;
#[cfg(feature = "cuda-probe")]
use super::flex_faces_launcher::FlexFacesDims;
#[cfg(feature = "cuda-probe")]
use super::flex_faces_launcher::FlexFacesMetadata;
#[cfg(feature = "cuda-probe")]
use super::flex_faces_launcher::FlexFacesSpec;
#[cfg(feature = "cuda-probe")]
use super::flex_faces_launcher::FlexFacesStaticDims;
#[cfg(feature = "cuda-probe")]
use super::flex_faces_launcher::flex_faces_lengths;
#[cfg(feature = "cuda-probe")]
use super::flex_faces_launcher::validate_flex_faces_metadata;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn edges_static_dims() -> FlexEdgesStaticDims {
    FlexEdgesStaticDims::new(2, 2, 1, 1, 2, 40, 6).unwrap()
}

/// 头、单边行、列索引、顶点体、体父链、根体与自由度体。
#[cfg(feature = "cuda-probe")]
fn edges_metadata() -> Vec<i32> {
    vec![
        2, 2, 1, 1, 40, 6, 2, // 七项头
        0, 1, 0, 1, // 边端点与稀疏行
        0, // 列索引
        1, 1, // 顶点体
        -1, 0, // 体父链
        -1, 1, // 根体表
        0, 0, // 自由度体
    ]
}

#[cfg(feature = "cuda-probe")]
fn faces_static_dims() -> FlexFacesStaticDims {
    FlexFacesStaticDims::new(1, 4, 18, 1).unwrap()
}

/// 头、面轴向与九个节点索引。
#[cfg(feature = "cuda-probe")]
fn faces_metadata() -> Vec<i32> {
    vec![1, 18, 0, -1, 0, 1, 2, 3, -1, 0, 1, 2]
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
fn edge_lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let dims = FlexEdgesDims {
        static_dims: edges_static_dims(),
        worlds: 2,
    };
    assert_eq!(flex_edges_lengths(&dims).unwrap(), [20, 12, 88, 20, 14]);
    assert_eq!(dims.abi(), [0, 0, 0, 2]);
    assert!(
        flex_edges_lengths(&FlexEdgesDims {
            static_dims: edges_static_dims(),
            worlds: 0,
        })
        .is_err()
    );
    // 质心跨度必须为14*nb+6*nv。
    assert!(FlexEdgesStaticDims::new(2, 2, 1, 1, 2, 39, 6).is_err());
    // 柔体位置跨度至少覆盖3*nvert。
    assert!(FlexEdgesStaticDims::new(2, 2, 1, 1, 2, 40, 5).is_err());
    assert!(FlexEdgesStaticDims::new(1, 2, 1, 1, 2, 19, 6).is_err());
}

#[test]
#[cfg(feature = "cuda-probe")]
fn edge_metadata_proof_rejects_rows_bodies_and_parents() {
    let dims = edges_static_dims();
    assert!(validate_flex_edges_metadata(&edges_metadata(), dims).is_ok());
    for (index, value, reason) in [
        (0, 3, "刚体数不符"),
        (4, 39, "质心跨度不符"),
        (8, 2, "边端点越界"),
        (10, 2, "稀疏行越界"),
        (12, 2, "列索引越界"),
        (13, 0, "顶点体为世界体"),
        (15, 1, "体父链非递增"),
        (16, 2, "根体越界"),
        (18, 2, "自由度体越界"),
        (19, -1, "自由度体为负"),
    ] {
        let mut values = edges_metadata();
        values[index] = value;
        assert!(
            validate_flex_edges_metadata(&values, dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_flex_edges_metadata(&edges_metadata()[..19], dims).is_err());
}

#[test]
#[cfg(feature = "cuda-probe")]
fn face_lengths_and_metadata_proof_reject_invalid_inputs() {
    let dims = FlexFacesDims {
        static_dims: faces_static_dims(),
        worlds: 2,
    };
    assert_eq!(flex_faces_lengths(&dims).unwrap(), [12, 1, 44, 70]);
    let static_dims = faces_static_dims();
    assert!(validate_flex_faces_metadata(&faces_metadata(), static_dims).is_ok());
    for (index, value, reason) in [
        (0, 2, "面数不符"),
        (1, 17, "节点跨度不符"),
        (2, 3, "轴向越界"),
        (3, 4, "节点索引越界"),
        (11, -2, "节点索引为负哨兵之外"),
    ] {
        let mut values = faces_metadata();
        values[index] = value;
        assert!(
            validate_flex_faces_metadata(&values, static_dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_flex_faces_metadata(&faces_metadata()[..11], static_dims).is_err());
    assert!(FlexFacesStaticDims::new(1, 7, 18, 1).is_err());
}

#[test]
#[cfg(feature = "cuda-probe")]
fn edge_plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = FlexEdgesDims {
        static_dims: edges_static_dims(),
        worlds: 2,
    };
    let bindings = [
        binding(0, 7, 0x1000, 20, ElemType::I32),
        binding(1, 7, 0x2000, 12, ElemType::F32),
        binding(2, 7, 0x3000, 88, ElemType::F32),
        binding(3, 7, 0x4000, 20, ElemType::F32),
        binding(4, 7, 0x9000, 14, ElemType::F32),
    ];
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    let checked = unsafe {
        Checked::<FlexEdgesMetadata>::assume_uploaded(
            bindings[0].span,
            &edges_metadata(),
            dims,
            MetadataRevision::new(),
        )
    }
    .map_err(flex_edges_launch_error)
    .unwrap();
    assert!(field_plan::<FlexEdgesSpec>(&dims, &bindings, &[checked.binding(0)]).is_ok());
    let mut short = bindings;
    short[4].span = span(7, 0x9000, 13 * 4);
    assert!(matches!(
        field_plan::<FlexEdgesSpec>(&dims, &short, &[checked.binding(0)]),
        Err(LauncherError::Capacity { slot: 4, .. })
    ));
    assert!(matches!(
        field_plan::<FlexEdgesSpec>(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    let mut alias = bindings;
    alias[1].span = bindings[4].span;
    assert!(matches!(
        field_plan::<FlexEdgesSpec>(&dims, &alias, &[checked.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    let mut foreign = bindings;
    foreign[3].span = span(8, 0x4000, 20 * 4);
    assert!(matches!(
        field_plan::<FlexEdgesSpec>(&dims, &foreign, &[checked.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
}

#[test]
#[cfg(feature = "cuda-probe")]
fn face_plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = FlexFacesDims {
        static_dims: faces_static_dims(),
        worlds: 2,
    };
    let bindings = [
        binding(0, 7, 0x1000, 12, ElemType::I32),
        binding(1, 7, 0x2000, 1, ElemType::F32),
        binding(2, 7, 0x3000, 44, ElemType::F32),
        binding(3, 7, 0x9000, 70, ElemType::F32),
    ];
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    let checked = unsafe {
        Checked::<FlexFacesMetadata>::assume_uploaded(
            bindings[0].span,
            &faces_metadata(),
            dims,
            MetadataRevision::new(),
        )
    }
    .unwrap();
    assert!(field_plan::<FlexFacesSpec>(&dims, &bindings, &[checked.binding(0)]).is_ok());
    let mut short = bindings;
    short[2].span = span(7, 0x3000, 43 * 4);
    assert!(matches!(
        field_plan::<FlexFacesSpec>(&dims, &short, &[checked.binding(0)]),
        Err(LauncherError::Capacity { slot: 2, .. })
    ));
    assert!(matches!(
        field_plan::<FlexFacesSpec>(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    let mut alias = bindings;
    alias[1].span = bindings[3].span;
    assert!(matches!(
        field_plan::<FlexFacesSpec>(&dims, &alias, &[checked.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    let mut foreign = bindings;
    foreign[1].span = span(8, 0x2000, 4);
    assert!(matches!(
        field_plan::<FlexFacesSpec>(&dims, &foreign, &[checked.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
}
