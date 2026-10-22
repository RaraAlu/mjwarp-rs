//! 质量求解GPU包装路径与恢复测试。

#[cfg(feature = "cuda-probe")]
use super::buffer::RigidMassSolveMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::kernel::RigidMassSolveKernel;
#[cfg(feature = "cuda-probe")]
use super::probe::solve_device;
#[cfg(feature = "cuda-probe")]
use super::validate::pack_rigid_mass_solve_blocks;
#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn runs_rigid_mass_solve_wrapper_paths_and_recovers() {
    fn captured<T>(label: &str, run: impl FnOnce() -> T) -> T {
        let capture = std::env::var_os("MJWARP_PHYSICS_PTX_DIR");
        if let Some(child) = crate::runtime::capture_child(module_path!(), label) {
            assert!(!child.exists(), "捕获目录必须为空：{}", child.display());
            // SAFETY: 本测试单线程运行，并在本次编译后恢复环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", &child) };
        }
        let result = run();
        if let Some(root) = capture.as_ref() {
            // SAFETY: 本测试单线程运行，并恢复调用方环境。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root) };
        } else {
            // SAFETY: 本测试单线程运行，调用前也没有该变量。
            unsafe { std::env::remove_var("MJWARP_PHYSICS_PTX_DIR") };
        }
        result
    }

    let session = TransferSession::new(0).unwrap();
    let worlds = 257usize;
    let nv = 79usize;
    let rhs_count = 2usize;
    let nbody = 1usize;
    let blocks = vec![
        MassFactorBlock {
            start: 0,
            size: 1,
            kind: MassFactorKind::Diagonal,
        },
        MassFactorBlock {
            start: 1,
            size: 6,
            kind: MassFactorKind::Cholesky,
        },
        MassFactorBlock {
            start: 7,
            size: 7,
            kind: MassFactorKind::Cholesky,
        },
        MassFactorBlock {
            start: 14,
            size: 65,
            kind: MassFactorKind::ReverseLdl,
        },
    ];
    let layout = MassSolveLayout::new(worlds, nv, rhs_count).unwrap();
    let abi = [nbody as u32, nv as u32, rhs_count as u32, worlds as u32];
    let packed = pack_rigid_mass_solve_blocks(&blocks, nv).unwrap();
    let mut metadata = RigidMassSolveMetadataBuffer::upload(&session, packed, abi).unwrap();
    let rhs = vec![1.0; worlds * rhs_count * nv];
    let rhs_buffer = session.upload(&rhs).unwrap();
    let state_width = 10 * nbody + nv * nv;
    let mut state = vec![-131072.0; 8 + worlds * state_width];
    for world in 0..worlds {
        let start = 4 + world * state_width;
        state[start..start + 10 * nbody].fill(0.0);
        let matrix = start + 10 * nbody;
        state[matrix..matrix + nv * nv].fill(0.0);
        for dof in 0..nv {
            state[matrix + dof * nv + dof] = 4.0;
        }
    }
    let state_buffer = session.upload(&state).unwrap();
    let mut initial = vec![f32::NAN; layout.guarded];
    initial[..4].fill(-131072.0);
    let guard = initial.len() - 4;
    initial[guard..].fill(-131072.0);
    let mut output = session.upload(&initial).unwrap();
    let kernel = captured("typed-main", || RigidMassSolveKernel::compile(&session)).unwrap();

    // SAFETY: 复用产品缓冲和冻结私有ABI，建立迁移前位基线。
    unsafe {
        kernel
            .kernel
            .launch(
                metadata.as_buffer(),
                &rhs_buffer,
                &state_buffer,
                &mut output,
                abi,
            )
            .unwrap();
    }
    let mut raw = vec![0.0; output.len()];
    output.read_range_into(0, &mut raw).unwrap();
    output.write_range(0, &initial).unwrap();
    let launch = kernel
        .preflight(&session, &metadata, &rhs_buffer, &state_buffer, &output)
        .unwrap();
    kernel
        .launch(&launch, &metadata, &rhs_buffer, &state_buffer, &mut output)
        .unwrap();
    let mut typed = vec![0.0; output.len()];
    output.read_range_into(0, &mut typed).unwrap();
    assert_eq!(
        raw.iter().map(|value| value.to_bits()).collect::<Vec<_>>(),
        typed
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    );
    check_result(layout, &typed).unwrap();
    let typed_output = MassSolveOutput {
        layout,
        values: typed,
        blocks: blocks.clone(),
    };
    for world in 0..worlds {
        assert!(
            typed_output
                .world(world)
                .unwrap()
                .solution
                .iter()
                .all(|&value| (value - 0.25).abs() <= 2.0e-5)
        );
    }

    let before = {
        let mut values = vec![0.0; output.len()];
        output.read_range_into(0, &mut values).unwrap();
        values
    };
    let short_rhs = session.upload(&rhs[..rhs.len() - 1]).unwrap();
    assert!(
        kernel
            .preflight(&session, &metadata, &short_rhs, &state_buffer, &output,)
            .is_err()
    );
    let mut rejected = vec![0.0; output.len()];
    output.read_range_into(0, &mut rejected).unwrap();
    assert_eq!(
        before
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        rejected
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    );

    let stale = kernel
        .preflight(&session, &metadata, &rhs_buffer, &state_buffer, &output)
        .unwrap();
    metadata.overwrite_for_test(0, 1).unwrap();
    assert!(
        kernel
            .launch(&stale, &metadata, &rhs_buffer, &state_buffer, &mut output,)
            .is_err()
    );
    assert!(
        kernel
            .preflight(&session, &metadata, &rhs_buffer, &state_buffer, &output)
            .is_err()
    );
    let mut rejected_stale = vec![0.0; output.len()];
    output.read_range_into(0, &mut rejected_stale).unwrap();
    assert_eq!(
        before
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        rejected_stale
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    );
    metadata.restore_for_test().unwrap();
    let recovered = kernel
        .preflight(&session, &metadata, &rhs_buffer, &state_buffer, &output)
        .unwrap();
    kernel
        .launch(
            &recovered,
            &metadata,
            &rhs_buffer,
            &state_buffer,
            &mut output,
        )
        .unwrap();

    let foreign_session = TransferSession::new(0).unwrap();
    let foreign = captured("foreign-session", || {
        RigidMassSolveKernel::compile(&foreign_session)
    })
    .unwrap();
    assert!(matches!(
        foreign.preflight(&session, &metadata, &rhs_buffer, &state_buffer, &output,),
        Err(TransferError::SessionMismatch)
    ));

    let scalar_layout = MassSolveLayout::new(2, 1, 1).unwrap();
    let scalar_blocks = factor_blocks(&[-1]);
    let mut scalar_state = vec![-131072.0; 30];
    scalar_state[4..14].fill(0.0);
    scalar_state[15..25].fill(0.0);
    scalar_state[14] = 4.0;
    scalar_state[25] = -1.0;
    let scalar_buffer = session.upload(&scalar_state).unwrap();
    assert!(matches!(
        captured("product-invalid", || solve_device(
            &session,
            1,
            scalar_layout,
            &scalar_buffer,
            &[1.0; 2],
            scalar_blocks.clone(),
        )),
        Err(TransferError::InvalidPivot {
            world: 1,
            dof: 0,
            ..
        })
    ));

    scalar_state[14] = 1.0e-39;
    scalar_state[25] = 4.0;
    let scalar_buffer = session.upload(&scalar_state).unwrap();
    assert!(matches!(
        captured("product-nonfinite", || solve_device(
            &session,
            1,
            scalar_layout,
            &scalar_buffer,
            &[1.0; 2],
            scalar_blocks.clone(),
        )),
        Err(TransferError::Input(InputError::NonFinite {
            field: "mass_solve_diagonal_inverse",
            index: 0
        }))
    ));

    scalar_state[14] = 4.0;
    let scalar_buffer = session.upload(&scalar_state).unwrap();
    let recovered = captured("product-recovery", || {
        solve_device(
            &session,
            1,
            scalar_layout,
            &scalar_buffer,
            &[1.0; 2],
            scalar_blocks,
        )
    })
    .unwrap();
    assert_eq!(recovered.world(0).unwrap().solution, &[0.25]);
    assert_eq!(recovered.world(1).unwrap().solution, &[0.25]);

    let zero_layout = MassSolveLayout::new(2, 0, 3).unwrap();
    let zero_state = session.upload(&[-131072.0; 28]).unwrap();
    let zero = captured("zero-dof", || {
        solve_device(&session, 1, zero_layout, &zero_state, &[], vec![])
    })
    .unwrap();
    assert!(zero.world(0).unwrap().solution.is_empty());
    assert!(zero.world(1).unwrap().solution.is_empty());
}
