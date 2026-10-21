//! 稠密源守卫与恢复测试。

use super::tests_support::*;
use super::*;

#[test]
#[ignore = "requires NVIDIA/NVRTC; new dense prefix source/target fault purpose"]
fn dense_source_guards_unselected_nan_world_writes_and_recovery() {
    let refs = input::reference();
    let m = &refs["models"][3];
    let session = TransferSession::new(0).unwrap();
    let plan = SparseMassDensePlan::new(&session, input::native(m)).unwrap();
    let mut data = plan.create_data(2).unwrap();
    let base = input::matrix(&m["cases"][0]);
    let next = input::matrix(&m["cases"][1]);
    data.write_matrix(&base.repeat(2)).unwrap();
    plan.update(&mut data).unwrap();
    let before = data.readback().unwrap();
    for other in [input::fork(7), input::small()] {
        let foreign = SparseMassDensePlan::new(&session, other).unwrap();
        assert!(foreign.update(&mut data).is_err());
        assert_eq!(
            data.readback().unwrap().world(0).unwrap(),
            before.world(0).unwrap()
        );
    }
    assert!(data.write_matrix(&[]).is_err());
    assert!(data.write_world_matrix(2, &base).is_err());
    assert!(data.write_world_matrix(0, &vec![f32::NAN; 2173]).is_err());
    assert!(
        data.write_world_matrix(0, &vec![f32::INFINITY; 2173])
            .is_err()
    );
    assert!(data.readback().is_ok());
    let nc = data.matrix_layout.elements_per_world();
    let tail = 4 + data.matrix_layout.total_elements();
    let mut original = vec![0.0; tail + 4];
    data.matrix.read_range_into(0, &mut original).unwrap();
    data.write_world_matrix(1, &next).unwrap();
    let mut changed = vec![0.0; original.len()];
    data.matrix.read_range_into(0, &mut changed).unwrap();
    for i in (0..4 + nc).chain(tail..tail + 4) {
        assert_eq!(original[i].to_bits(), changed[i].to_bits());
    }
    for (a, b) in changed[4 + nc..tail].iter().zip(&next) {
        assert_eq!(a.to_bits(), b.to_bits());
    }
    plan.update(&mut data).unwrap();
    for (offset, bad) in [(0, 0.0), (tail, 0.0), (4, f32::NAN), (4 + 2145, f32::NAN)] {
        let mut saved = [0.0];
        data.matrix.read_range_into(offset, &mut saved).unwrap();
        data.matrix.write_range(offset, &[bad]).unwrap();
        assert!(plan.update(&mut data).is_err());
        assert!(data.readback().is_err());
        data.matrix.write_range(offset, &saved).unwrap();
        plan.update(&mut data).unwrap();
    }
    for offset in [0, 4 + data.dense_layout.total_elements()] {
        data.dense.write_range(offset, &[0.0]).unwrap();
        assert!(data.readback().is_err());
        assert!(plan.update(&mut data).is_err());
        assert!(data.readback().is_err());
        data.dense.write_range(offset, &[-131072.0]).unwrap();
        plan.update(&mut data).unwrap();
    }
    data.dense.write_range(4, &[f32::NAN]).unwrap();
    assert!(data.readback().is_err());
    plan.update(&mut data).unwrap();
    for offset in [0, 4 + data.worlds()] {
        data.status.write_range(offset, &[0.0]).unwrap();
        assert!(plan.update(&mut data).is_err());
        assert!(data.readback().is_err());
        data.status.write_range(offset, &[-131072.0]).unwrap();
        plan.update(&mut data).unwrap();
    }
    assert!(data.readback().is_ok());
}
