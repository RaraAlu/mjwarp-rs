//! 动能类型化计划与校验测试。

#[cfg(feature = "cuda-probe")]
use super::metadata::KineticEnergy32Spec;
#[cfg(feature = "cuda-probe")]
use super::metadata::KineticLaunchMetadata;
#[cfg(feature = "cuda-probe")]
use super::metadata::KineticValidationSpec;
#[cfg(feature = "cuda-probe")]
use super::metadata::kinetic_lengths;
#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(feature = "cuda-probe")]
#[test]
fn typed_kinetic_plan_rejects_capacity_alias_session_and_stale_proof() {
    use crate::runtime::launcher::DeviceSpan;

    let dims = [65, 2, 0, 2];
    let (pair_len, energy_len) = kinetic_lengths(&dims).unwrap();
    let metadata_span = DeviceSpan::new(7, 0x1000, 4).unwrap();
    let revision = MetadataRevision::new();
    // SAFETY: 本测试把宿主值绑定到等价合成跨度。
    let metadata = unsafe {
        Checked::<KineticLaunchMetadata>::assume_uploaded(
            metadata_span,
            &[65],
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
            span: DeviceSpan::new(7, 0x2000, pair_len * 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: DeviceSpan::new(7, 0x4000, energy_len * 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: DeviceSpan::new(7, 0x5000, energy_len * 4).unwrap(),
            elem: ElemType::F32,
        },
    ];
    let kernel = Kernel::<KineticEnergy32Spec>::new().unwrap();
    kernel
        .plan(&dims, &bindings, &[metadata.binding(0)])
        .unwrap();

    let mut capacity = bindings;
    capacity[1].span = DeviceSpan::new(7, 0x2000, pair_len * 4 - 4).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &capacity, &[metadata.binding(0)]),
        Err(LauncherError::Capacity { slot: 1, .. })
    ));

    let mut alias = bindings;
    alias[3].span = alias[2].span;
    assert!(matches!(
        kernel.plan(&dims, &alias, &[metadata.binding(0)]),
        Err(LauncherError::Alias {
            first: 2,
            second: 3,
        })
    ));

    let mut foreign = bindings;
    foreign[1].span = DeviceSpan::new(8, 0x2000, pair_len * 4).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &foreign, &[metadata.binding(0)]),
        Err(LauncherError::ContextMismatch { slot: 1 })
    ));

    revision.invalidate();
    assert!(matches!(
        kernel.plan(&dims, &bindings, &[metadata.binding(0)]),
        Err(LauncherError::StaleMetadata)
    ));
}

#[cfg(feature = "cuda-probe")]
#[test]
fn typed_kinetic_validation_rejects_capacity_alias_session_and_stale_proof() {
    use crate::runtime::launcher::DeviceSpan;

    let dims = [65, 257, 0, 257];
    let (pair_len, energy_len) = kinetic_lengths(&dims).unwrap();
    let status_len = u64::from(dims[1]) + 8;
    let metadata_span = DeviceSpan::new(7, 0x1000, 4).unwrap();
    let revision = MetadataRevision::new();
    // SAFETY: 本测试把宿主值绑定到等价合成跨度。
    let metadata = unsafe {
        Checked::<KineticLaunchMetadata>::assume_uploaded(
            metadata_span,
            &[65],
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
            span: DeviceSpan::new(7, 0x10000, pair_len * 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: DeviceSpan::new(7, 0x40000, energy_len * 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: DeviceSpan::new(7, 0x50000, status_len * 4).unwrap(),
            elem: ElemType::F32,
        },
    ];
    let kernel = Kernel::<KineticValidationSpec>::new().unwrap();
    let plan = kernel
        .plan(&dims, &bindings, &[metadata.binding(0)])
        .unwrap();
    assert_eq!(plan.dims().grid(), (2, 1, 1));
    assert_eq!(plan.dims().block(), (256, 1, 1));

    let mut capacity = bindings;
    capacity[3].span = DeviceSpan::new(7, 0x50000, status_len * 4 - 4).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &capacity, &[metadata.binding(0)]),
        Err(LauncherError::Capacity { slot: 3, .. })
    ));

    let mut alias = bindings;
    alias[3].span = alias[2].span;
    assert!(matches!(
        kernel.plan(&dims, &alias, &[metadata.binding(0)]),
        Err(LauncherError::Alias {
            first: 2,
            second: 3,
        })
    ));

    let mut foreign = bindings;
    foreign[1].span = DeviceSpan::new(8, 0x10000, pair_len * 4).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &foreign, &[metadata.binding(0)]),
        Err(LauncherError::ContextMismatch { slot: 1 })
    ));

    revision.invalidate();
    assert!(matches!(
        kernel.plan(&dims, &bindings, &[metadata.binding(0)]),
        Err(LauncherError::StaleMetadata)
    ));
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn gpu_safe_wrapper_binds_session_static_nv_revision_and_exact_bits() {
    let session = TransferSession::new(0).unwrap();
    let foreign = TransferSession::new(0).unwrap();
    let kernel = KineticEnergy32Kernel::compile(&session, 65).unwrap();
    let mut metadata = KineticEnergyMetadataBuffer::upload(&session, 65).unwrap();
    let foreign_metadata = KineticEnergyMetadataBuffer::upload(&foreign, 65).unwrap();
    let wrong_nv = KineticEnergyMetadataBuffer::upload(&session, 64).unwrap();
    let dims = [65, 1, 0, 1];
    let mut pair_values = vec![-131072.0; 138];
    pair_values[4..69].fill(1.0);
    pair_values[69..134].fill(2.0);
    let pair = session.upload(&pair_values).unwrap();
    let foreign_pair = foreign.upload(&pair_values).unwrap();
    let mut energy_values = vec![-131072.0; 10];
    energy_values[4] = 3.5;
    let current = session.upload(&energy_values).unwrap();
    let mut staged = session.upload(&energy_values).unwrap();

    assert!(matches!(
        kernel.launch(&foreign_metadata, &pair, &current, &mut staged, dims),
        Err(TransferError::SessionMismatch)
    ));
    assert!(matches!(
        kernel.launch(&metadata, &foreign_pair, &current, &mut staged, dims),
        Err(TransferError::SessionMismatch)
    ));
    assert!(matches!(
        kernel.launch(&wrong_nv, &pair, &current, &mut staged, dims),
        Err(TransferError::Input(InputError::InvalidDimension {
            field: "kinetic_energy32_abi"
        }))
    ));

    metadata.write_for_test(65).unwrap();
    assert!(matches!(
        kernel.launch(&metadata, &pair, &current, &mut staged, dims),
        Err(TransferError::Input(InputError::InvalidTopology {
            field: "kinetic_energy_metadata",
            reason: "stale_proof",
            ..
        }))
    ));
    metadata.restore_for_test().unwrap();
    kernel
        .launch(&metadata, &pair, &current, &mut staged, dims)
        .unwrap();
    let mut result = vec![0.0; staged.len()];
    staged.read_range_into(0, &mut result).unwrap();
    assert_eq!(result[4].to_bits(), 3.5f32.to_bits());
    assert_eq!(result[5].to_bits(), 65.0f32.to_bits());
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn gpu_validation_wrapper_checks_real_spans_and_preserves_status_guards() {
    let session = TransferSession::new(0).unwrap();
    let foreign = TransferSession::new(0).unwrap();
    let validator = KineticValidationKernel::compile(&session, 65).unwrap();
    let mut metadata = KineticEnergyMetadataBuffer::upload(&session, 65).unwrap();
    let foreign_metadata = KineticEnergyMetadataBuffer::upload(&foreign, 65).unwrap();
    let wrong_metadata = KineticEnergyMetadataBuffer::upload(&session, 64).unwrap();
    let dims = [65, 257, 0, 257];
    let mut pair_values = vec![-131072.0; 2 * 65 * 257 + 8];
    let pair_end = pair_values.len() - 4;
    pair_values[4..pair_end].fill(0.25);
    let mut current_values = vec![-131072.0; 2 * 257 + 8];
    for world in 0..257 {
        current_values[4 + 2 * world] = world as f32;
    }
    let status_values = vec![-131072.0; 257 + 8];
    let pair = session.upload(&pair_values).unwrap();
    let current = session.upload(&current_values).unwrap();
    let mut status = session.upload(&status_values).unwrap();

    validator
        .launch(&metadata, &pair, &current, &mut status, dims)
        .unwrap();
    let mut result = vec![0.0; status.len()];
    status.read_range_into(0, &mut result).unwrap();
    assert_eq!(&result[..4], &status_values[..4]);
    assert_eq!(
        &result[result.len() - 4..],
        &status_values[status_values.len() - 4..]
    );
    assert!(result[4..4 + 257].iter().all(|&value| value == 1.0));

    let short_pair = session
        .upload(&pair_values[..pair_values.len() - 1])
        .unwrap();
    assert!(
        validator
            .launch(&metadata, &short_pair, &current, &mut status, dims)
            .is_err()
    );
    let foreign_pair = foreign.upload(&pair_values).unwrap();
    assert!(matches!(
        validator.launch(&metadata, &foreign_pair, &current, &mut status, dims),
        Err(TransferError::SessionMismatch)
    ));
    assert!(matches!(
        validator.launch(&foreign_metadata, &pair, &current, &mut status, dims),
        Err(TransferError::SessionMismatch)
    ));
    assert!(matches!(
        validator.launch(&wrong_metadata, &pair, &current, &mut status, dims),
        Err(TransferError::Input(InputError::InvalidDimension {
            field: "kinetic_energy_validate_static_nv"
        }))
    ));
    assert!(matches!(
        validator.launch(&metadata, &pair, &current, &mut status, [64, 257, 0, 257],),
        Err(TransferError::Input(InputError::InvalidDimension {
            field: "kinetic_energy_validate_static_nv"
        }))
    ));

    metadata.write_for_test(65).unwrap();
    assert!(matches!(
        validator.launch(&metadata, &pair, &current, &mut status, dims),
        Err(TransferError::Input(InputError::InvalidTopology {
            field: "kinetic_energy_metadata",
            reason: "stale_proof",
            ..
        }))
    ));
    let mut after = vec![0.0; status.len()];
    status.read_range_into(0, &mut after).unwrap();
    assert_eq!(result, after);
    metadata.restore_for_test().unwrap();
    validator
        .launch(&metadata, &pair, &current, &mut status, dims)
        .unwrap();
}
