use super::super::spec::*;
#[cfg(feature = "cuda-probe")]
use super::super::validation::*;
use super::*;

#[cfg(feature = "cuda-probe")]
#[test]
fn typed_source_validation_rejects_capacity_alias_session_and_stale_proof() {
    use crate::physics::kinetic_energy::KineticLaunchMetadata;
    use crate::runtime::launcher::{Checked, DeviceSpan, MetadataRevision};

    let dims = [65, 257, 0, 257];
    let (source_len, status_len) = source_validation_lengths(&dims).unwrap();
    let metadata_span = DeviceSpan::new(7, 0x1000, 4).unwrap();
    let revision = MetadataRevision::new();
    // SAFETY: 本测试绑定等价合成跨度。
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
            span: DeviceSpan::new(7, 0x10000, source_len * 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: DeviceSpan::new(7, 0x30000, source_len * 4).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: DeviceSpan::new(7, 0x50000, status_len * 4).unwrap(),
            elem: ElemType::F32,
        },
    ];
    let kernel = Kernel::<DeviceEnergySourceValidationSpec>::new().unwrap();
    let plan = kernel
        .plan(&dims, &bindings, &[metadata.binding(0)])
        .unwrap();
    assert_eq!(plan.dims().grid(), (2, 1, 1));
    assert_eq!(plan.dims().block(), (256, 1, 1));

    let mut capacity = bindings;
    capacity[1].span = DeviceSpan::new(7, 0x10000, source_len * 4 - 4).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &capacity, &[metadata.binding(0)]),
        Err(LauncherError::Capacity { slot: 1, .. })
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
    foreign[2].span = DeviceSpan::new(8, 0x30000, source_len * 4).unwrap();
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

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn gpu_source_validation_wrapper_binds_real_spans_and_metadata() {
    let session = TransferSession::new(0).unwrap();
    let foreign = TransferSession::new(0).unwrap();
    let validator = DeviceEnergySourceValidationKernel::compile(&session, 65).unwrap();
    let mut metadata = KineticEnergyMetadataBuffer::upload(&session, 65).unwrap();
    let foreign_metadata = KineticEnergyMetadataBuffer::upload(&foreign, 65).unwrap();
    let wrong_metadata = KineticEnergyMetadataBuffer::upload(&session, 64).unwrap();
    let dims = [65, 257, 0, 257];
    let mut source_values = vec![-131072.0; 65 * 257 + 8];
    let source_end = source_values.len() - 4;
    source_values[4..source_end].fill(0.25);
    let status_values = vec![-131072.0; 257 + 8];
    let q = session.upload(&source_values).unwrap();
    let mq = session.upload(&source_values).unwrap();
    let mut status = session.upload(&status_values).unwrap();

    validator
        .launch(&metadata, &q, &mq, &mut status, dims)
        .unwrap();
    let mut result = vec![0.0; status.len()];
    status.read_range_into(0, &mut result).unwrap();
    assert_eq!(&result[..4], &status_values[..4]);
    assert_eq!(
        &result[result.len() - 4..],
        &status_values[status_values.len() - 4..]
    );
    assert!(result[4..4 + 257].iter().all(|&value| value == 1.0));

    let short_q = session
        .upload(&source_values[..source_values.len() - 1])
        .unwrap();
    assert!(
        validator
            .launch(&metadata, &short_q, &mq, &mut status, dims)
            .is_err()
    );
    let foreign_q = foreign.upload(&source_values).unwrap();
    assert!(matches!(
        validator.launch(&metadata, &foreign_q, &mq, &mut status, dims),
        Err(TransferError::SessionMismatch)
    ));
    assert!(matches!(
        validator.launch(&foreign_metadata, &q, &mq, &mut status, dims),
        Err(TransferError::SessionMismatch)
    ));
    assert!(matches!(
        validator.launch(&wrong_metadata, &q, &mq, &mut status, dims),
        Err(TransferError::Input(InputError::InvalidDimension {
            field: "device_energy_source_validate_static_nv"
        }))
    ));
    assert!(matches!(
        validator.launch(&metadata, &q, &mq, &mut status, [64, 257, 0, 257]),
        Err(TransferError::Input(InputError::InvalidDimension {
            field: "device_energy_source_validate_static_nv"
        }))
    ));

    metadata.write_for_test(65).unwrap();
    assert!(matches!(
        validator.launch(&metadata, &q, &mq, &mut status, dims),
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
        .launch(&metadata, &q, &mq, &mut status, dims)
        .unwrap();
}

#[test]
fn host_checks_pair_world_strides_and_original_scalar32_limits() {
    for n in [1, 3, 33, 65, 1024] {
        for w in [1, 2, 257] {
            let l = layout(n, w).unwrap();
            assert_eq!(l.pair.total_elements(), 2 * n * w);
            assert_eq!(l.energy.total_elements(), 2 * w);
            assert!(l.energy.world_elements(w).is_err());
        }
    }
    for (n, w) in [
        (0, 1),
        (1025, 1),
        (3, 0),
        (3, usize::MAX),
        (3, i32::MAX as usize + 1),
    ] {
        assert!(layout(n, w).is_err());
    }
}
#[test]
fn host_requires_actual_owner_arcs_and_every_current_world() {
    let mass = Arc::new(Spec {
        nbody: 4,
        nv: 3,
        meta: vec![],
    });
    let plan = Arc::new(EnergySpec { mass: mass.clone() });
    let other = Arc::new(EnergySpec { mass: mass.clone() });
    let twin = Arc::new(Spec {
        nbody: 4,
        nv: 3,
        meta: vec![],
    });
    assert!(binding(&plan, &plan, &mass));
    assert!(!binding(&plan, &other, &mass));
    assert!(!binding(&plan, &plan, &twin));
    for flags in [
        &[][..],
        &[false][..],
        &[true, false][..],
        &[false, true][..],
    ] {
        assert!(all_current(flags).is_err());
    }
    all_current(&[true]).unwrap();
    all_current(&[true; 257]).unwrap();
}
