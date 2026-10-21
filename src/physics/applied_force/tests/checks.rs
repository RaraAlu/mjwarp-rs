//! 施加外力边界与故障测试。

use super::data::check;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::AppliedForceDeltaMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::AppliedForceValidateMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::AppliedForceDeltaSpec;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::AppliedForcePublishSpec;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::AppliedForceValidateSpec;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::applied_force_delta_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::applied_force_publish_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::applied_force_publish_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::applied_force_validate_lengths;
use super::tests_support::*;
use super::*;

#[test]
fn rejects_cycles_wrong_roots_and_bad_counts_before_device_allocation() {
    for p in [vec![], vec![1], vec![0, -1], vec![0, 1], vec![0, 2, 1]] {
        assert!(topology(p.clone(), vec![0; p.len()], vec![0]).is_err());
    }
    for r in [vec![0], vec![0, -1], vec![0, 2], vec![0, 0]] {
        assert!(topology(vec![0, 0], r, vec![1]).is_err());
    }
    assert!(topology(vec![0], vec![0], vec![]).is_err());
    for d in [-1, 1, i32::MAX] {
        assert!(topology(vec![0], vec![0], vec![d]).is_err());
    }
    for args in [
        (0, 4, 4),
        (1, 0, 1),
        (1, 1, 0),
        (usize::MAX, 1, 1),
        (1, usize::MAX, 1),
        (1, 1, usize::MAX),
    ] {
        assert!(layout(args.0, args.1, args.2).is_err());
    }
    let l = layout(257, 273, 513).unwrap();
    assert_eq!(l.geometry_len, 257 * 6 * (273 + 513));
    assert_eq!(l.ft.total_elements(), 257 * 6 * 273);
    assert!(check(&[f32::NAN], 1, "ft").is_err());
    assert!(check(&[], 1, "ft").is_err());
}
#[test]
fn independently_certifies_frame_component_order_and_subtree_exclusion() {
    let v = reference();
    let t = topology(
        ints(&v, "parents"),
        ints(&v, "roots"),
        ints(&v, "dof_bodyid"),
    )
    .unwrap();
    let q = oracle(
        &t,
        &floats(&v, "xipos"),
        &floats(&v, "subtree_com"),
        &floats(&v, "cdof"),
        &floats(&v, "ft_force_torque"),
    );
    assert_eq!(q, vec![4.0, -12.0, -40.0, 0.0]);
    for (i, &x) in q.iter().enumerate() {
        assert_eq!(
            (x as f32).to_bits(),
            u32::from_str_radix(v["delta_bits"][i].as_str().unwrap(), 16).unwrap()
        );
    }
}
#[test]
#[cfg(feature = "cuda-probe")]
fn delta_wrapper_rejects_capacity_context_alias_metadata_and_work_faults() {
    use crate::runtime::launcher::DeviceSpan;

    let dims = [2, 1, 2, 2];
    let values = [
        -131072, -131072, -131072, -131072, 0, 0, 0, 1, 1, -131072, -131072, -131072, -131072,
    ];
    let revision = MetadataRevision::new();
    let meta = DeviceSpan::new(7, 0x1000, 52).unwrap();
    // SAFETY: 测试跨度代表与宿主值一致的只读元数据。
    let proof = unsafe {
        Checked::<AppliedForceDeltaMetadata>::assume_uploaded(meta, &values, dims, revision.clone())
    }
    .unwrap();
    let bindings = [
        SlotBinding {
            slot: 0,
            span: meta,
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: DeviceSpan::new(7, 0x2000, 176).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: DeviceSpan::new(7, 0x3000, 128).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: DeviceSpan::new(7, 0x4000, 40).unwrap(),
            elem: ElemType::F32,
        },
    ];
    let kernel = Kernel::<AppliedForceDeltaSpec>::new().unwrap();
    assert!(kernel.plan(&dims, &bindings, &[proof.binding(0)]).is_ok());

    let mut short = bindings;
    short[3].span = DeviceSpan::new(7, 0x4000, 36).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &short, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 3, .. })
    ));
    let mut foreign = bindings;
    foreign[2].span = DeviceSpan::new(8, 0x3000, 128).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { slot: 2 })
    ));
    let mut alias = bindings;
    alias[3].span = DeviceSpan::new(7, 0x2000, 40).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias {
            first: 1,
            second: 3
        })
    ));
    revision.invalidate();
    assert!(matches!(
        kernel.plan(&dims, &bindings, &[proof.binding(0)]),
        Err(LauncherError::StaleMetadata)
    ));

    let mut bad = values;
    bad[8] = 2;
    assert!(AppliedForceDeltaMetadata::validate(&bad, &dims).is_err());
    assert!(applied_force_delta_lengths(&[2, 1, 2, 1]).is_err());
    assert!(applied_force_delta_lengths(&[1, 1, u32::MAX, u32::MAX]).is_err());
}
#[test]
#[cfg(feature = "cuda-probe")]
fn publish_wrapper_rejects_capacity_context_alias_and_dimension_faults() {
    use crate::runtime::launcher::DeviceSpan;

    let dims = [2, 3, 0, 6];
    let bindings = [
        SlotBinding {
            slot: 0,
            span: DeviceSpan::new(7, 0x1000, 4).unwrap(),
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: DeviceSpan::new(7, 0x2000, 56).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: DeviceSpan::new(7, 0x3000, 56).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: DeviceSpan::new(7, 0x4000, 56).unwrap(),
            elem: ElemType::F32,
        },
    ];
    assert_eq!(AppliedForcePublishSpec::BLOCK, BlockRule::Fixed(256));
    assert_eq!(
        applied_force_publish_lengths(&dims).unwrap(),
        [1, 14, 14, 14]
    );
    assert!(applied_force_publish_plan(&dims, &bindings).is_ok());
    assert!(applied_force_publish_plan(&[2, 3, 1, 6], &bindings).is_ok());

    for invalid in [[0, 3, 0, 6], [2, 0, 0, 6], [2, 3, 0, 0]] {
        assert!(matches!(
            applied_force_publish_lengths(&invalid),
            Err(LauncherError::InvalidArray("外力发布维度无效"))
        ));
    }
    assert!(matches!(
        applied_force_publish_lengths(&[2, 3, 2, 6]),
        Err(LauncherError::InvalidArray("外力发布累加开关无效"))
    ));
    assert!(matches!(
        applied_force_publish_lengths(&[2, 3, 0, 5]),
        Err(LauncherError::InvalidArray("外力发布工作量不匹配"))
    ));
    assert!(matches!(
        applied_force_publish_lengths(&[65_536, 65_536, 0, 1]),
        Err(LauncherError::InvalidArray("外力发布工作量不匹配"))
    ));

    const MAX_WORK: u32 = u32::MAX - 255;
    let boundary = [1, MAX_WORK, 0, MAX_WORK];
    let vector = u64::from(MAX_WORK) + 8;
    assert_eq!(
        applied_force_publish_lengths(&boundary).unwrap(),
        [1, vector, vector, vector]
    );
    assert_eq!(
        AppliedForcePublishSpec::launch_dims(&boundary)
            .unwrap()
            .grid(),
        (16_777_215, 1, 1)
    );
    assert!(matches!(
        applied_force_publish_lengths(&[1, MAX_WORK + 1, 0, MAX_WORK + 1]),
        Err(LauncherError::InvalidArray("外力发布维度无效"))
    ));
    // work受u32上界约束；转为u64后加八不会溢出。

    for (slot, bytes) in [(0, 3), (1, 52), (2, 52), (3, 52)] {
        let mut short = bindings;
        short[slot].span = DeviceSpan::new(7, 0x5000 + 0x1000 * slot as u64, bytes).unwrap();
        assert!(matches!(
            applied_force_publish_plan(&dims, &short),
            Err(LauncherError::Capacity { slot: actual, .. }) if actual == slot
        ));
    }
    for slot in 0..4 {
        let mut foreign = bindings;
        foreign[slot].span =
            DeviceSpan::new(8, foreign[slot].span.addr(), foreign[slot].span.bytes()).unwrap();
        let expected = if slot == 0 { 1 } else { slot };
        assert!(matches!(
            applied_force_publish_plan(&dims, &foreign),
            Err(LauncherError::ContextMismatch { slot: actual }) if actual == expected
        ));
    }

    let mut current_delta_alias = bindings;
    current_delta_alias[2].span = current_delta_alias[1].span;
    assert!(matches!(
        applied_force_publish_plan(&dims, &current_delta_alias),
        Err(LauncherError::Alias {
            first: 1,
            second: 2
        })
    ));
    for input in 0..3 {
        let mut staged_alias = bindings;
        staged_alias[3].span = DeviceSpan::new(
            7,
            staged_alias[input].span.addr(),
            staged_alias[3].span.bytes(),
        )
        .unwrap();
        assert!(matches!(
            applied_force_publish_plan(&dims, &staged_alias),
            Err(LauncherError::Alias { first, second })
                if first == input && second == 3
        ));
    }
}
#[test]
#[cfg(feature = "cuda-probe")]
fn validate_wrapper_rejects_capacity_context_alias_metadata_and_work_faults() {
    use crate::runtime::launcher::DeviceSpan;

    let dims = [2, 2, 2, 2];
    let values = [
        -131072, -131072, -131072, -131072, 0, 0, 0, 1, 1, 1, -131072, -131072, -131072, -131072,
    ];
    let revision = MetadataRevision::new();
    let meta = DeviceSpan::new(7, 0x1000, 56).unwrap();
    // SAFETY: 测试跨度代表与宿主值一致的只读元数据。
    let proof = unsafe {
        Checked::<AppliedForceValidateMetadata>::assume_uploaded(
            meta,
            &values,
            dims,
            revision.clone(),
        )
    }
    .unwrap();
    let bindings = [
        SlotBinding {
            slot: 0,
            span: meta,
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: DeviceSpan::new(7, 0x2000, 224).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: DeviceSpan::new(7, 0x3000, 128).unwrap(),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: DeviceSpan::new(7, 0x4000, 40).unwrap(),
            elem: ElemType::F32,
        },
    ];
    let kernel = Kernel::<AppliedForceValidateSpec>::new().unwrap();
    assert_eq!(AppliedForceValidateSpec::BLOCK, BlockRule::Fixed(256));
    assert!(kernel.plan(&dims, &bindings, &[proof.binding(0)]).is_ok());
    assert!(applied_force_delta_lengths(&dims).is_err());

    let mut short = bindings;
    short[3].span = DeviceSpan::new(7, 0x4000, 36).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &short, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 3, .. })
    ));
    let mut foreign = bindings;
    foreign[2].span = DeviceSpan::new(8, 0x3000, 128).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { slot: 2 })
    ));
    let mut alias = bindings;
    alias[3].span = DeviceSpan::new(7, 0x2000, 40).unwrap();
    assert!(matches!(
        kernel.plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias {
            first: 1,
            second: 3
        })
    ));
    revision.invalidate();
    assert!(matches!(
        kernel.plan(&dims, &bindings, &[proof.binding(0)]),
        Err(LauncherError::StaleMetadata)
    ));

    let mut bad = values;
    bad[9] = 2;
    assert!(AppliedForceValidateMetadata::validate(&bad, &dims).is_err());
    assert!(matches!(
        applied_force_validate_lengths(&[2, 2, 2, 1]),
        Err(LauncherError::InvalidArray("外力验证工作量不匹配"))
    ));
    for invalid in [
        [0, 2, 2, 2],
        [2, 0, 2, 2],
        [2, 2, 0, 2],
        [2, 2, 2, 0],
        [2, 2, u32::MAX - 254, u32::MAX - 254],
    ] {
        assert!(matches!(
            applied_force_validate_lengths(&invalid),
            Err(LauncherError::InvalidArray("外力验证维度无效"))
        ));
    }

    const MAX_NB: u32 = (i32::MAX as u32 - 9) / 2;
    let accepted = applied_force_validate_lengths(&[MAX_NB, 1, 1, 1]).unwrap();
    assert_eq!(accepted[0], i32::MAX as u64);
    assert!(matches!(
        applied_force_validate_lengths(&[MAX_NB, 2, 1, 1]),
        Err(LauncherError::Overflow("applied_force_validate_metadata"))
    ));
    assert!(matches!(
        applied_force_validate_lengths(&[MAX_NB + 1, 1, 1, 1]),
        Err(LauncherError::Overflow("applied_force_validate_metadata"))
    ));
    assert!(matches!(
        applied_force_validate_lengths(&[MAX_NB, 1, u32::MAX - 255, u32::MAX - 255]),
        Err(LauncherError::Overflow("applied_force_validate_geometry"))
    ));
    // FT小于已验证的几何跨度；状态仅为受限work加八。
}
#[test]
#[cfg(feature = "cuda-probe")]
fn delta_metadata_span_accepts_i32_max_and_rejects_next_nb() {
    const MAX_NB: u32 = (i32::MAX as u32 - 9) / 2;

    let accepted = applied_force_delta_lengths(&[MAX_NB, 1, 1, 1]).unwrap();
    assert_eq!(accepted[0], i32::MAX as u64);

    assert!(matches!(
        applied_force_delta_lengths(&[MAX_NB, 2, 1, 2]),
        Err(LauncherError::Overflow("applied_force_delta_metadata"))
    ));
    assert!(matches!(
        applied_force_delta_lengths(&[MAX_NB + 1, 1, 1, 1]),
        Err(LauncherError::Overflow("applied_force_delta_metadata"))
    ));
}
