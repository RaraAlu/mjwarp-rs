use super::*;
use crate as engine;
use crate::runtime::TransferSession;
mod input {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/support/sparse_mass_model.rs"
    ));
}
#[test]
fn rejects_zero_or_overflowing_rhs_layouts_without_allocation() {
    for (w, n, r) in [
        (0, 65, 1),
        (1, 65, 0),
        (usize::MAX, 65, 1),
        (1, usize::MAX, 2),
        (2, 0, i32::MAX as usize),
    ] {
        assert!(layout(w, n, r).is_err());
    }
    assert_eq!(layout(2, 0, 3).unwrap().total_elements(), 0);
    assert_eq!(layout(257, 65, 5).unwrap().total_elements(), 83525);
}
fn factor_values(data: &SparseMassSolveData) -> Vec<f32> {
    let mut values = vec![0.0; data.mass.factor.output_layout.total_elements() + 8];
    data.mass
        .factor
        .output
        .read_range_into(0, &mut values)
        .unwrap();
    values
}
#[test]
#[ignore = "requires NVIDIA driver/NVRTC; new fused solve source/cache fault purpose"]
fn cached_solve_ignores_dense_poison_and_rejects_all_guarded_source_faults() {
    let refs = input::reference();
    let m = &refs["models"][0];
    let session = TransferSession::new(0).unwrap();
    let plan = SparseMassSolvePlan::new(&session, input::model(m), input::sparse(m)).unwrap();
    let mut data = plan.create_data(2, 3).unwrap();
    data.write_qpos(&input::floats(&m["cases"][0], "qpos").repeat(2))
        .unwrap();
    data.write_rhs(&vec![0.5; 390]).unwrap();
    plan.update(&mut data).unwrap();
    let mut rhs_before = vec![0.0; 398];
    data.rhs.read_range_into(0, &mut rhs_before).unwrap();
    data.write_world_rhs(1, &vec![0.5; 195]).unwrap();
    let mut rhs_after = vec![0.0; 398];
    data.rhs.read_range_into(0, &mut rhs_after).unwrap();
    for i in (0..199).chain(394..398) {
        assert_eq!(rhs_before[i].to_bits(), rhs_after[i].to_bits());
    }
    assert!(
        rhs_after[199..394]
            .iter()
            .all(|v| v.to_bits() == 0.5f32.to_bits())
    );
    plan.update_solve(&mut data).unwrap();
    let before = data.readback().unwrap();
    let saved = factor_values(&data);
    let dense_slot = 4 + 10 * 66 + 1;
    data.mass
        .matrix
        .corrupt_output_for_test(dense_slot, f32::NAN)
        .unwrap();
    data.write_rhs(&vec![0.5; 390]).unwrap();
    plan.update_solve(&mut data).unwrap();
    assert_eq!(
        factor_values(&data)
            .iter()
            .map(|v| v.to_bits())
            .collect::<Vec<_>>(),
        saved.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
    );
    let after = data.readback().unwrap();
    for w in 0..2 {
        for (&a, &b) in after
            .world(w)
            .unwrap()
            .solution
            .iter()
            .zip(before.world(w).unwrap().solution)
        {
            assert!((a - b).abs() <= 2e-4 + 2e-4 * b.abs());
        }
    }
    assert!(plan.update_gather(&mut data).is_err());
    assert!(plan.update_factor(&mut data).is_err());
    assert!(plan.update_solve(&mut data).is_err());
    assert!(data.readback().is_err());
    plan.update(&mut data).unwrap();
    let mut csr = [0.0];
    data.mass
        .factor
        .matrix
        .read_range_into(4, &mut csr)
        .unwrap();
    data.mass.factor.matrix.write_range(4, &[f32::NAN]).unwrap();
    data.write_rhs(&vec![0.5; 390]).unwrap();
    plan.update_solve(&mut data).unwrap();
    assert!(data.readback().is_ok());
    assert!(plan.update_factor(&mut data).is_err());
    assert!(data.readback().is_err());
    data.mass.factor.matrix.write_range(4, &csr).unwrap();
    plan.update_factor(&mut data).unwrap();
    plan.update_solve(&mut data).unwrap();
    let saved = factor_values(&data);
    let inverse = 4 + plan.input().nc();
    for (offset, bad) in [
        (0, 0.0),
        (saved.len() - 4, 0.0),
        (5, f32::NAN),
        (inverse, -1.0),
        (inverse, f32::INFINITY),
    ] {
        data.mass.factor.output.write_range(offset, &[bad]).unwrap();
        assert!(data.readback().is_err());
        assert!(plan.update_solve(&mut data).is_err());
        assert!(!data.mass.factor.ready);
        assert!(data.readback().is_err());
        data.mass
            .factor
            .output
            .write_range(offset, &[saved[offset]])
            .unwrap();
        plan.update_factor(&mut data).unwrap();
        plan.update_solve(&mut data).unwrap();
    }
    let len = data.layout.total_elements() + 8;
    for (offset, bad, restore) in [
        (0, 0.0, -131072.0),
        (len - 4, 0.0, -131072.0),
        (4, f32::NAN, 0.5),
    ] {
        data.rhs.write_range(offset, &[bad]).unwrap();
        assert!(data.readback().is_err());
        assert!(plan.update_solve(&mut data).is_err());
        assert!(data.mass.factor.ready);
        assert!(data.readback().is_err());
        data.rhs.write_range(offset, &[restore]).unwrap();
        plan.update_solve(&mut data).unwrap();
    }
    for offset in [0, len - 4] {
        data.output.write_range(offset, &[0.0]).unwrap();
        assert!(data.readback().is_err());
        assert!(plan.update_solve(&mut data).is_err());
        assert!(data.mass.factor.ready);
        assert!(data.readback().is_err());
        data.output.write_range(offset, &[-131072.0]).unwrap();
        plan.update_solve(&mut data).unwrap();
    }
    data.output.write_range(4, &[f32::NAN]).unwrap();
    assert!(data.readback().is_err());
    plan.update_solve(&mut data).unwrap();
    for offset in [0, data.worlds() + 4] {
        data.mass.status.write_range(offset, &[0.0]).unwrap();
        assert!(plan.update_solve(&mut data).is_err());
        assert!(data.readback().is_err());
        assert!(data.mass.factor.ready);
        data.mass.status.write_range(offset, &[-131072.0]).unwrap();
        plan.update_solve(&mut data).unwrap();
    }
    assert!(data.readback().is_ok());
}
