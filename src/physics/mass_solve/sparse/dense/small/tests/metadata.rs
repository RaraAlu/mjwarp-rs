//! Small主机元数据与类型化计划测试。

#[cfg(feature = "cuda-probe")]
use super::factor_meta::SmallFactorLaunchMetadata;
#[cfg(feature = "cuda-probe")]
use super::factor_meta::SmallFactorSpec;
#[cfg(feature = "cuda-probe")]
use super::factor_meta::StaticSmallFactorMetadata;
#[cfg(feature = "cuda-probe")]
use super::factor_meta::small_factor_lengths;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::SmallValidationSpec;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::small_validation_lengths;
use super::*;

#[test]
fn host_metadata_allows_empty_small_set_and_rejects_bad_ranges() {
    assert!(StaticSmallFactorMetadata::validate(&[0], &0).is_ok());
    assert!(StaticSmallFactorMetadata::validate(&[0, 0, 2], &0).is_err());
    assert!(StaticSmallFactorMetadata::validate(&[0, 2, 2], &4).is_err());
    assert!(StaticSmallFactorMetadata::validate(&[0, 0, 2, 2, 2, 2], &8).is_err());
}

#[test]
fn typed_small_factor_plan_rejects_capacity_alias_session_and_stale_proof() {
    use crate::runtime::launcher::DeviceSpan;

    let dims = [4, 0, 1, 257];
    let (metadata_len, buffer_len) = small_factor_lengths(&dims).unwrap();
    let metadata_span = DeviceSpan::new(7, 0x1000, metadata_len * 4).unwrap();
    let revision = MetadataRevision::new();
    // SAFETY: 本测试绑定等价合成跨度。
    let metadata = unsafe {
        Checked::<SmallFactorLaunchMetadata>::assume_uploaded(
            metadata_span,
            &[0, 0, 2],
            dims,
            revision.clone(),
        )
    }
    .unwrap();
    let bindings = [
        SlotBinding {
            slot: 0,
            span: metadata_span,
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: DeviceSpan::new(7, 0x10000, buffer_len * 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: DeviceSpan::new(7, 0x30000, buffer_len * 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: DeviceSpan::new(7, 0x50000, buffer_len * 4).unwrap(),
            elem: ElemType::F32,
        },
    ];
    let kernel = Kernel::<SmallFactorSpec>::new().unwrap();
    let plan = kernel
        .plan(&dims, &bindings, &[metadata.binding(0)])
        .unwrap();
    assert_eq!(plan.dims().grid(), (5, 1, 1));
    assert_eq!(plan.dims().block(), (64, 1, 1));

    let mut capacity = bindings;
    capacity[3].span = DeviceSpan::new(7, 0x50000, buffer_len * 4 - 4).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &capacity, &[metadata.binding(0)]),
        Err(LauncherError::Capacity { slot: 3, .. })
    ));

    let mut alias = bindings;
    alias[3].span = alias[1].span;
    assert!(matches!(
        kernel.plan(&dims, &alias, &[metadata.binding(0)]),
        Err(LauncherError::Alias {
            first: 1,
            second: 3,
        })
    ));

    let mut foreign = bindings;
    foreign[2].span = DeviceSpan::new(8, 0x30000, buffer_len * 4).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &foreign, &[metadata.binding(0)]),
        Err(LauncherError::ContextMismatch { slot: 2 })
    ));

    revision.invalidate();
    assert!(matches!(
        kernel.plan(&dims, &bindings, &[metadata.binding(0)]),
        Err(LauncherError::StaleMetadata)
    ));
}

#[test]
fn typed_small_validation_plan_tracks_only_dereferenced_spans() {
    use crate::runtime::launcher::DeviceSpan;

    let dims = [5, 0, 0, 257];
    let (matrix_len, status_len) = small_validation_lengths(&dims).unwrap();
    let bindings = [
        SlotBinding {
            slot: 0,
            span: DeviceSpan::new(7, 0x1000, 4).unwrap(),
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: DeviceSpan::new(7, 0x2000, matrix_len * 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: DeviceSpan::new(7, 0x4000, 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: DeviceSpan::new(7, 0x6000, status_len * 4).unwrap(),
            elem: ElemType::F32,
        },
    ];
    let kernel = Kernel::<SmallValidationSpec>::new().unwrap();
    let plan = kernel.plan(&dims, &bindings, &[]).unwrap();
    assert_eq!(plan.dims().grid(), (2, 1, 1));
    assert_eq!(plan.dims().block(), (256, 1, 1));

    let mut capacity = bindings;
    capacity[1].span = DeviceSpan::new(7, 0x2000, matrix_len * 4 - 4).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &capacity, &[]),
        Err(LauncherError::Capacity { slot: 1, .. })
    ));

    let mut alias = bindings;
    alias[3].span = alias[1].span;
    assert!(matches!(
        kernel.plan(&dims, &alias, &[]),
        Err(LauncherError::Alias {
            first: 1,
            second: 3,
        })
    ));

    let mut unused_alias = bindings;
    unused_alias[2].span = unused_alias[3].span;
    assert!(matches!(
        kernel.plan(&dims, &unused_alias, &[]),
        Err(LauncherError::Alias {
            first: 2,
            second: 3,
        })
    ));

    let mut foreign = bindings;
    foreign[2].span = DeviceSpan::new(8, 0x4000, 4).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &foreign, &[]),
        Err(LauncherError::ContextMismatch { slot: 2 })
    ));

    assert!(kernel.plan(&[5, 1, 0, 257], &bindings, &[]).is_err());
    let zero_dims = [0, 0, 0, 257];
    let mut zero = bindings;
    zero[1].span = DeviceSpan::new(7, 0x2000, 8 * 4).unwrap();
    assert!(kernel.plan(&zero_dims, &zero, &[]).is_ok());
}
