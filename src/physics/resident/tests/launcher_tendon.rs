//! 常驻肌腱合并、索引与唤醒 FIELD_BATCH 包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::field_batch::field_plan;
#[cfg(feature = "cuda-probe")]
use super::tendon_launcher::TendonDims;
#[cfg(feature = "cuda-probe")]
use super::tendon_launcher::TendonIndicesSpec;
#[cfg(feature = "cuda-probe")]
use super::tendon_launcher::TendonMetadata;
#[cfg(feature = "cuda-probe")]
use super::tendon_launcher::TendonStaticDims;
#[cfg(feature = "cuda-probe")]
use super::tendon_launcher::TendonValuesSpec;
#[cfg(feature = "cuda-probe")]
use super::tendon_launcher::tendon_indices_lengths;
#[cfg(feature = "cuda-probe")]
use super::tendon_launcher::tendon_values_lengths;
#[cfg(feature = "cuda-probe")]
use super::tendon_launcher::validate_tendon_metadata;
#[cfg(feature = "cuda-probe")]
use super::tendon_wake_launcher::TendonWakeDims;
#[cfg(feature = "cuda-probe")]
use super::tendon_wake_launcher::TendonWakeMetadata;
#[cfg(feature = "cuda-probe")]
use super::tendon_wake_launcher::TendonWakeSpec;
#[cfg(feature = "cuda-probe")]
use super::tendon_wake_launcher::TendonWakeStaticDims;
#[cfg(feature = "cuda-probe")]
use super::tendon_wake_launcher::tendon_wake_lengths;
#[cfg(feature = "cuda-probe")]
use super::tendon_wake_launcher::validate_wake_metadata;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn static_dims() -> TendonStaticDims {
    TendonStaticDims::new((2, 3, 2), (1, 2), (1, 1, 2), 1).unwrap()
}

/// 头、每肌腱类别/局部编号/局部偏移/全局行/计数。
#[cfg(feature = "cuda-probe")]
fn tendon_metadata() -> Vec<i32> {
    vec![2, 3, 2, 1, 2, 1, 1, 2, 0, 0, 0, 0, 2, 1, 0, 0, 2, 1]
}

#[cfg(feature = "cuda-probe")]
fn wake_static_dims() -> TendonWakeStaticDims {
    TendonWakeStaticDims {
        ntree: 1,
        metadata_elements: 12,
        parameter_elements: 3,
        lengths_stride: 5,
        scratch_stride: 5,
    }
}

/// 头、肌腱行、限位标志与树编号。
#[cfg(feature = "cuda-probe")]
fn wake_metadata() -> Vec<i32> {
    vec![1, 1, 2, 1, 1, 5, 0, 0, 2, 0, -1, 0]
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
fn tendon_lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let values = TendonDims {
        static_dims: static_dims(),
        worlds: 2,
        kind: 0,
    };
    assert_eq!(tendon_values_lengths(&values).unwrap(), [18, 20, 14, 42]);
    assert_eq!(values.abi(), [0, 0, 0, 2]);
    let spatial = TendonDims { kind: 1, ..values };
    assert_eq!(tendon_values_lengths(&spatial).unwrap(), [18, 20, 88, 42]);
    let indices = TendonDims {
        static_dims: static_dims(),
        worlds: 2,
        kind: 0,
    };
    assert_eq!(tendon_indices_lengths(&indices).unwrap(), [18, 1, 20, 24]);
    assert!(
        tendon_values_lengths(&TendonDims {
            static_dims: static_dims(),
            worlds: 0,
            kind: 0,
        })
        .is_err()
    );
    assert!(
        tendon_values_lengths(&TendonDims {
            static_dims: static_dims(),
            worlds: 2,
            kind: 2,
        })
        .is_err()
    );
    assert!(TendonStaticDims::new((usize::MAX, 1, 1), (0, 0), (0, 0, 0), 1).is_err());
}

#[test]
#[cfg(feature = "cuda-probe")]
fn tendon_metadata_proof_rejects_rows_and_length() {
    let dims = static_dims();
    assert!(validate_tendon_metadata(&tendon_metadata(), dims).is_ok());
    for (index, value, reason) in [
        (0, 3, "肌腱数不符"),
        (3, 2, "固定肌腱数不符"),
        (8, 2, "合并类别越界"),
        (9, 1, "固定局部编号越界"),
        (10, -1, "局部偏移为负"),
        (11, 3, "全局行越界"),
        (12, -1, "全局计数为负"),
        (14, 1, "空间局部编号越界"),
        (16, 3, "全局行越界"),
    ] {
        let mut values = tendon_metadata();
        values[index] = value;
        assert!(
            validate_tendon_metadata(&values, dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_tendon_metadata(&tendon_metadata()[..17], dims).is_err());
}

#[test]
#[cfg(feature = "cuda-probe")]
fn wake_lengths_and_metadata_proof_reject_invalid_inputs() {
    let dims = TendonWakeDims {
        abi: [0, 0, 0, 2],
        static_dims: wake_static_dims(),
    };
    assert_eq!(tendon_wake_lengths(&dims).unwrap(), [12, 3, 18, 18]);
    let static_dims = wake_static_dims();
    assert!(validate_wake_metadata(&wake_metadata(), static_dims).is_ok());
    for (index, value, reason) in [
        (0, 2, "树数量不符"),
        (1, -1, "肌腱数为负"),
        (2, -1, "包裹数为负"),
        (5, 4, "长度跨度不符"),
        (6, 2, "使能标志无效"),
        (8, 3, "行未连续覆盖"),
        (9, 2, "限位标志无效"),
        (11, 1, "树索引越界"),
    ] {
        let mut values = wake_metadata();
        values[index] = value;
        assert!(
            validate_wake_metadata(&values, static_dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    // 冻结内核在禁用或零肌腱时提前返回：空批次合法；
    // 启用且存在肌腱时必须正批次，否则周期取参数会除零。
    let mut disabled = wake_metadata();
    disabled[3] = 0;
    disabled[4] = 0;
    assert!(validate_wake_metadata(&disabled, static_dims).is_ok());
    let mut enabled = wake_metadata();
    enabled[6] = 1;
    assert!(validate_wake_metadata(&enabled, static_dims).is_ok());
    enabled[3] = 0;
    assert!(validate_wake_metadata(&enabled, static_dims).is_err());
    enabled[3] = 1;
    enabled[4] = 0;
    assert!(validate_wake_metadata(&enabled, static_dims).is_err());
    assert!(validate_wake_metadata(&wake_metadata()[..11], static_dims).is_err());
    // 状态跨度必须为2*ntree+3。
    let bad = TendonWakeStaticDims {
        scratch_stride: 6,
        ..static_dims
    };
    assert!(validate_wake_metadata(&wake_metadata(), bad).is_err());
}

#[cfg(feature = "cuda-probe")]
fn tendon_proof(values: &[SlotBinding; 4], dims: TendonDims) -> Checked<TendonMetadata> {
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    unsafe {
        Checked::<TendonMetadata>::assume_uploaded(
            values[0].span,
            &tendon_metadata(),
            dims,
            MetadataRevision::new(),
        )
    }
    .unwrap()
}

#[test]
#[cfg(feature = "cuda-probe")]
fn tendon_values_and_indices_plans_reject_capacity_alias_context_and_missing_proof() {
    let dims = TendonDims {
        static_dims: static_dims(),
        worlds: 2,
        kind: 0,
    };
    let values = [
        binding(0, 7, 0x1000, 18, ElemType::I32),
        binding(1, 7, 0x2000, 20, ElemType::I32),
        binding(2, 7, 0x3000, 14, ElemType::F32),
        binding(3, 7, 0x9000, 42, ElemType::F32),
    ];
    let checked = tendon_proof(&values, dims);
    assert!(field_plan::<TendonValuesSpec>(&dims, &values, &[checked.binding(0)]).is_ok());
    let mut short = values;
    short[2].span = span(7, 0x3000, 13 * 4);
    assert!(matches!(
        field_plan::<TendonValuesSpec>(&dims, &short, &[checked.binding(0)]),
        Err(LauncherError::Capacity { slot: 2, .. })
    ));
    assert!(matches!(
        field_plan::<TendonValuesSpec>(&dims, &values, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    let mut alias = values;
    alias[1].span = values[3].span;
    assert!(matches!(
        field_plan::<TendonValuesSpec>(&dims, &alias, &[checked.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    let mut foreign = values;
    foreign[1].span = span(8, 0x2000, 20 * 4);
    assert!(matches!(
        field_plan::<TendonValuesSpec>(&dims, &foreign, &[checked.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));

    let indices = [
        binding(0, 7, 0x1000, 18, ElemType::I32),
        binding(1, 7, 0x2000, 1, ElemType::F32),
        binding(2, 7, 0x3000, 20, ElemType::I32),
        binding(3, 7, 0x9000, 24, ElemType::I32),
    ];
    assert!(field_plan::<TendonIndicesSpec>(&dims, &indices, &[checked.binding(0)]).is_ok());
    let mut wrong = indices;
    wrong[1].span = span(7, 0x2000, 2 * 4);
    assert!(matches!(
        field_plan::<TendonIndicesSpec>(&dims, &wrong, &[checked.binding(0)]),
        Err(LauncherError::Capacity { slot: 1, .. })
    ));
}

#[test]
#[cfg(feature = "cuda-probe")]
fn wake_plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = TendonWakeDims {
        abi: [0, 0, 0, 2],
        static_dims: wake_static_dims(),
    };
    let bindings = [
        binding(0, 7, 0x1000, 12, ElemType::I32),
        binding(1, 7, 0x2000, 3, ElemType::F32),
        binding(2, 7, 0x3000, 18, ElemType::F32),
        binding(3, 7, 0x9000, 18, ElemType::I32),
    ];
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    let checked = unsafe {
        Checked::<TendonWakeMetadata>::assume_uploaded(
            bindings[0].span,
            &wake_metadata(),
            dims,
            MetadataRevision::new(),
        )
    }
    .unwrap();
    assert!(field_plan::<TendonWakeSpec>(&dims, &bindings, &[checked.binding(0)]).is_ok());
    let mut short = bindings;
    short[3].span = span(7, 0x9000, 17 * 4);
    assert!(matches!(
        field_plan::<TendonWakeSpec>(&dims, &short, &[checked.binding(0)]),
        Err(LauncherError::Capacity { slot: 3, .. })
    ));
    assert!(matches!(
        field_plan::<TendonWakeSpec>(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    let mut alias = bindings;
    alias[2].span = bindings[3].span;
    assert!(matches!(
        field_plan::<TendonWakeSpec>(&dims, &alias, &[checked.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    let mut foreign = bindings;
    foreign[1].span = span(8, 0x2000, 3 * 4);
    assert!(matches!(
        field_plan::<TendonWakeSpec>(&dims, &foreign, &[checked.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));
}
