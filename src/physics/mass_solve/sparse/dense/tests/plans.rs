//! 稠密类型化计划测试。

#[cfg(feature = "cuda-probe")]
use super::gather_meta::DenseGatherLaunchMetadata;
#[cfg(feature = "cuda-probe")]
use super::gather_meta::DenseGatherSpec;
#[cfg(feature = "cuda-probe")]
use super::gather_meta::StaticDenseGatherMetadata;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::DenseValidationSpec;
#[cfg(feature = "cuda-probe")]
use super::validation_meta::dense_validation_lengths;
use super::*;

#[test]
fn typed_dense_validation_plan_tracks_actual_access_spans() {
    use crate::runtime::launcher::DeviceSpan;

    let dims = [3, 0, 0, 257];
    let (matrix_len, status_len) = dense_validation_lengths(&dims).unwrap();
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
    let kernel = Kernel::<DenseValidationSpec>::new().unwrap();
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

    assert!(kernel.plan(&[3, 0, 1, 257], &bindings, &[]).is_err());
    assert!(kernel.plan(&[3, 0, 0, 0], &bindings, &[]).is_err());
}

#[test]
fn typed_dense_gather_plan_binds_sources_and_exact_spans() {
    use crate::runtime::launcher::{DeviceSpan, MetadataRevision};

    assert!(StaticDenseGatherMetadata::validate(&[0], &[3, 0]).is_ok());
    assert!(StaticDenseGatherMetadata::validate(&[], &[3, 0]).is_err());
    assert!(StaticDenseGatherMetadata::validate(&[-1], &[3, 1]).is_err());
    assert!(StaticDenseGatherMetadata::validate(&[4], &[3, 1]).is_err());
    assert!(StaticDenseGatherMetadata::validate(&[3], &[3, 1]).is_ok());
    assert!(StaticDenseGatherMetadata::validate(&[0, 1], &[3, 1]).is_err());
    assert!(StaticDenseGatherMetadata::validate(&[0], &[i32::MAX as u32 + 1, 1]).is_err());

    let dims = [3, 257, 0, 514];
    let values = vec![3; 257];
    let metadata_span = DeviceSpan::new(7, 0x1000, 257 * 4).unwrap();
    let revision = MetadataRevision::new();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let proof = unsafe {
        Checked::<DenseGatherLaunchMetadata>::assume_uploaded(
            metadata_span,
            &values,
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
            span: DeviceSpan::new(7, 0x2000, 14 * 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: DeviceSpan::new(7, 0x3000, 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: DeviceSpan::new(7, 0x4000, 522 * 4).unwrap(),
            elem: ElemType::F32,
        },
    ];
    let kernel = Kernel::<DenseGatherSpec>::new().unwrap();
    let plan = kernel.plan(&dims, &bindings, &[proof.binding(0)]).unwrap();
    assert_eq!(plan.dims().grid(), (3, 1, 1));
    assert_eq!(plan.dims().block(), (256, 1, 1));
    assert!(matches!(
        kernel.plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));

    let mut replaced_metadata = bindings;
    replaced_metadata[0].span = DeviceSpan::new(7, 0x6000, 257 * 4).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &replaced_metadata, &[proof.binding(0)]),
        Err(LauncherError::MetadataMismatch(_))
    ));

    let wrong_dims = [3, 257, 0, 771];
    let wrong_revision = MetadataRevision::new();
    // SAFETY: 测试跨度代表同一份宿主快照。
    let wrong_dims_proof = unsafe {
        Checked::<DenseGatherLaunchMetadata>::assume_uploaded(
            metadata_span,
            &values,
            wrong_dims,
            wrong_revision,
        )
    }
    .unwrap();
    assert!(matches!(
        kernel.plan(&dims, &bindings, &[wrong_dims_proof.binding(0)]),
        Err(LauncherError::MetadataMismatch(_))
    ));

    let mut short_matrix = bindings;
    short_matrix[1].span = DeviceSpan::new(7, 0x2000, 13 * 4).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &short_matrix, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 1, .. })
    ));

    let mut short_output = bindings;
    short_output[3].span = DeviceSpan::new(7, 0x4000, 521 * 4).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &short_output, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 3, .. })
    ));

    let mut alias = bindings;
    alias[1].span = DeviceSpan::new(7, 0x2000, 4096).unwrap();
    alias[3].span = alias[1].span;
    assert!(matches!(
        kernel.plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias {
            first: 1,
            second: 3,
        })
    ));

    let mut foreign = bindings;
    foreign[2].span = DeviceSpan::new(8, 0x3000, 4).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { slot: 2 })
    ));

    revision.invalidate();
    assert!(matches!(
        kernel.plan(&dims, &bindings, &[proof.binding(0)]),
        Err(LauncherError::StaleMetadata)
    ));
    assert!(kernel.plan(&[3, 0, 0, 0], &bindings, &[]).is_err());
    assert!(kernel.plan(&[3, 257, 1, 514], &bindings, &[]).is_err());
    assert!(kernel.plan(&[3, 257, 0, 513], &bindings, &[]).is_err());
    assert!(
        kernel
            .plan(&[i32::MAX as u32 + 1, 257, 0, 514], &bindings, &[])
            .is_err()
    );
}
