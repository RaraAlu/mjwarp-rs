//! 常驻质量分解包装的宿主故障测试。

#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentMassFactorDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentMassSolveDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::ResidentMassStaticDims;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::ResidentMassFactorMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::ResidentMassSolveMetadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_meta::validate_resident_mass_metadata;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::resident_mass_factor_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::resident_mass_factor_plan;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::resident_mass_solve_lengths;
#[cfg(feature = "cuda-probe")]
use super::launcher_specs::resident_mass_solve_plan;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
fn static_dims() -> ResidentMassStaticDims {
    ResidentMassStaticDims {
        nbody: 4,
        n: 3,
        count: 2,
    }
}

#[cfg(feature = "cuda-probe")]
fn valid_metadata() -> Vec<i32> {
    // 两块连续分区：对角块0，二自由度Cholesky块1。
    vec![2, 0, 1, 0, 1, 2, 1]
}

#[test]
#[cfg(feature = "cuda-probe")]
fn lengths_follow_frozen_layout_and_reject_invalid_dimensions() {
    let factor = ResidentMassFactorDims {
        static_dims: static_dims(),
        worlds: 2,
        rhs_count: 3,
    };
    assert_eq!(
        resident_mass_factor_lengths(&factor).unwrap(),
        [7, 26, 106, 44]
    );
    assert_eq!(factor.abi(), [4, 3, 1, 2]);
    let solve = ResidentMassSolveDims {
        static_dims: static_dims(),
        worlds: 2,
        rhs_count: 3,
    };
    assert_eq!(
        resident_mass_solve_lengths(&solve).unwrap(),
        [7, 26, 44, 56]
    );
    assert_eq!(solve.abi(), [4, 3, 3, 2]);

    // 零自由度边界：只有块数槽，因子与解区仍保留状态三标量与守卫。
    let zero = ResidentMassStaticDims {
        nbody: 4,
        n: 0,
        count: 0,
    };
    assert_eq!(
        resident_mass_factor_lengths(&ResidentMassFactorDims {
            static_dims: zero,
            worlds: 1,
            rhs_count: 1,
        })
        .unwrap(),
        [1, 8, 48, 11]
    );
    assert_eq!(
        resident_mass_solve_lengths(&ResidentMassSolveDims {
            static_dims: zero,
            worlds: 1,
            rhs_count: 1,
        })
        .unwrap(),
        [1, 8, 11, 11]
    );

    assert!(
        resident_mass_factor_lengths(&ResidentMassFactorDims {
            static_dims: ResidentMassStaticDims {
                nbody: 0,
                ..static_dims()
            },
            worlds: 1,
            rhs_count: 1,
        })
        .is_err()
    );
    assert!(
        resident_mass_factor_lengths(&ResidentMassFactorDims {
            static_dims: static_dims(),
            worlds: 0,
            rhs_count: 1,
        })
        .is_err()
    );
    assert!(
        resident_mass_factor_lengths(&ResidentMassFactorDims {
            static_dims: static_dims(),
            worlds: u32::MAX - 254,
            rhs_count: 1,
        })
        .is_err()
    );
    assert!(
        resident_mass_factor_lengths(&ResidentMassFactorDims {
            static_dims: static_dims(),
            worlds: 1,
            rhs_count: 0,
        })
        .is_err()
    );
    // 块表长度超过i32表示上限。
    assert!(matches!(
        resident_mass_factor_lengths(&ResidentMassFactorDims {
            static_dims: ResidentMassStaticDims {
                count: i32::MAX as u32 / 3 + 1,
                ..static_dims()
            },
            worlds: 1,
            rhs_count: 1,
        }),
        Err(LauncherError::Overflow("resident_mass_metadata"))
    ));
}

#[test]
#[cfg(feature = "cuda-probe")]
fn metadata_proof_rejects_bad_block_count_partition_size_and_kind() {
    let dims = static_dims();
    assert!(validate_resident_mass_metadata(&valid_metadata(), &dims).is_ok());
    for (index, value, reason) in [
        (0, 1, "块数不符"),
        (1, 1, "块起点不连续"),
        (2, 0, "块尺寸为零"),
        (3, 2, "单自由度格式不符"),
        (4, 2, "第二块起点不连续"),
        (5, 3, "第二块越界"),
        (6, 0, "二自由度格式不符"),
    ] {
        let mut values = valid_metadata();
        values[index] = value;
        assert!(
            validate_resident_mass_metadata(&values, &dims).is_err(),
            "{reason} 应被拒绝"
        );
    }
    assert!(validate_resident_mass_metadata(&valid_metadata()[..6], &dims).is_err());
    // 零自由度要求零块，非零维度要求覆盖全部自由度。
    let zero = ResidentMassStaticDims {
        nbody: 4,
        n: 0,
        count: 0,
    };
    assert!(validate_resident_mass_metadata(&[0], &zero).is_ok());
    assert!(
        validate_resident_mass_metadata(&[0], &ResidentMassStaticDims { n: 3, ..zero }).is_err()
    );
    assert!(
        validate_resident_mass_metadata(
            &valid_metadata(),
            &ResidentMassStaticDims { n: 0, ..dims }
        )
        .is_err()
    );
}

#[test]
#[cfg(feature = "cuda-probe")]
fn plan_rejects_capacity_alias_context_and_missing_proof() {
    let dims = ResidentMassFactorDims {
        static_dims: static_dims(),
        worlds: 2,
        rhs_count: 3,
    };
    let lengths = resident_mass_factor_lengths(&dims).unwrap();
    let span = |context: u64, addr: u64, bytes: u64| DeviceSpan::new(context, addr, bytes).unwrap();
    let bindings = [
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
            span: span(7, 0x4000, lengths[3] * 4),
            elem: ElemType::F32,
        },
    ];
    let values = valid_metadata();
    let revision = MetadataRevision::new();
    // SAFETY: 测试用宿主镜像与实际跨度一致；不发射。
    let proof = unsafe {
        Checked::<ResidentMassFactorMetadata>::assume_uploaded(
            bindings[0].span,
            &values,
            dims,
            revision,
        )
    }
    .unwrap();
    assert!(resident_mass_factor_plan(&dims, &bindings, &[proof.binding(0)]).is_ok());

    // 容量：因子槽少一个元素。
    let mut short = bindings;
    short[3].span = span(7, 0x4000, (lengths[3] - 1) * 4);
    assert!(matches!(
        resident_mass_factor_plan(&dims, &short, &[proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 3, .. })
    ));
    // 缺证明：块表槽声明为地址类。
    assert!(matches!(
        resident_mass_factor_plan(&dims, &bindings, &[]),
        Err(LauncherError::MissingMetadataProof { slot: 0 })
    ));
    // 别名：因子结果槽与矩阵源槽重叠。
    let mut alias = bindings;
    alias[3].span = bindings[2].span;
    assert!(matches!(
        resident_mass_factor_plan(&dims, &alias, &[proof.binding(0)]),
        Err(LauncherError::Alias { .. })
    ));
    // 错会话：右端项槽属于另一上下文。
    let mut foreign = bindings;
    foreign[1].span = span(8, 0x2000, lengths[1] * 4);
    assert!(matches!(
        resident_mass_factor_plan(&dims, &foreign, &[proof.binding(0)]),
        Err(LauncherError::ContextMismatch { .. })
    ));

    // 回代计划同样核对证明、跨度、会话与别名。
    let solve = ResidentMassSolveDims {
        static_dims: static_dims(),
        worlds: 2,
        rhs_count: 3,
    };
    let solve_lengths = resident_mass_solve_lengths(&solve).unwrap();
    let solve_bindings = [
        SlotBinding {
            slot: 0,
            span: span(7, 0x1000, solve_lengths[0] * 4),
            elem: ElemType::I32,
        },
        SlotBinding {
            slot: 1,
            span: span(7, 0x2000, solve_lengths[1] * 4),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 2,
            span: span(7, 0x3000, solve_lengths[2] * 4),
            elem: ElemType::F32,
        },
        SlotBinding {
            slot: 3,
            span: span(7, 0x4000, solve_lengths[3] * 4),
            elem: ElemType::F32,
        },
    ];
    let solve_proof = unsafe {
        Checked::<ResidentMassSolveMetadata>::assume_uploaded(
            solve_bindings[0].span,
            &values,
            solve,
            MetadataRevision::new(),
        )
    }
    .unwrap();
    assert!(resident_mass_solve_plan(&solve, &solve_bindings, &[solve_proof.binding(0)]).is_ok());
    let mut short_output = solve_bindings;
    short_output[3].span = span(7, 0x4000, (solve_lengths[3] - 1) * 4);
    assert!(matches!(
        resident_mass_solve_plan(&solve, &short_output, &[solve_proof.binding(0)]),
        Err(LauncherError::Capacity { slot: 3, .. })
    ));
}
