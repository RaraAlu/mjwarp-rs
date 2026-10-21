//! 动能GPU守卫与事务发布测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new kinetic consumer private guards/source/overflow transactional publication"]
fn gpu_rejects_source_guards_and_overflow_without_partial_publication() {
    let session = TransferSession::new(0).unwrap();
    let mut plan = KineticEnergyPlan::new(&session, 47, 65).unwrap();
    let mut d = plan.create_data(257).unwrap();
    let (q, m, _, p) = scene(65, 257, 0);
    d.write_pair(tagged(&q, 1), tagged(&m, 1)).unwrap();
    d.write_potential(&p).unwrap();
    plan.update(&mut d).unwrap();
    for field in 0..4 {
        for edge in 0..8 {
            let b = match field {
                0 => &mut d.pair,
                1 => &mut d.current,
                2 => &mut d.staged,
                _ => &mut d.status,
            };
            let index = if edge < 4 { edge } else { b.len() - 8 + edge };
            b.write_range(index, &[99.0]).unwrap();
            let before = bits(&raw(&d.current));
            assert!(plan.update(&mut d).is_err());
            assert!(!d.ready);
            assert!(d.readback().is_err());
            assert_eq!(before, bits(&raw(&d.current)));
            let b = match field {
                0 => &mut d.pair,
                1 => &mut d.current,
                2 => &mut d.staged,
                _ => &mut d.status,
            };
            b.write_range(index, &[-131072.0]).unwrap();
            plan.update(&mut d).unwrap();
        }
    }
    plan.metadata.write_for_test(64).unwrap();
    let before = bits(&raw(&d.current));
    assert!(plan.update(&mut d).is_err());
    assert_eq!(before, bits(&raw(&d.current)));
    plan.metadata.restore_for_test().unwrap();
    plan.update(&mut d).unwrap();
    for index in [4, 68, 69, 133, 4 + 512 * 65, 4 + 514 * 65 - 1] {
        for bad in [f32::NAN, f32::INFINITY] {
            let old = raw(&d.pair)[index];
            d.pair.write_range(index, &[bad]).unwrap();
            let before = bits(&raw(&d.current));
            assert!(plan.update(&mut d).is_err());
            assert_eq!(before, bits(&raw(&d.current)));
            d.pair.write_range(index, &[old]).unwrap();
            plan.update(&mut d).unwrap();
        }
    }
    let index = 516;
    let old = raw(&d.current)[index];
    d.current.write_range(index, &[f32::NAN]).unwrap();
    let before = bits(&raw(&d.current));
    assert!(plan.update(&mut d).is_err());
    assert_eq!(before, bits(&raw(&d.current)));
    d.current.write_range(index, &[old]).unwrap();
    // Old kinetic contents do not enter the frozen function at all.
    d.current.write_range(517, &[f32::NAN]).unwrap();
    plan.update(&mut d).unwrap();
    for (q, m) in [
        (vec![f32::MAX; 65], vec![2.0; 65]),
        (vec![1.0; 65], vec![f32::MAX; 65]),
    ] {
        d.write_world_pair(256, tagged(&q, 9), tagged(&m, 9))
            .unwrap();
        let before = bits(&raw(&d.current));
        assert!(plan.update(&mut d).is_err());
        assert_eq!(before, bits(&raw(&d.current)));
        assert!(d.readback().is_err());
        let (q, m, _, _) = scene(65, 1, 0);
        d.write_world_pair(256, tagged(&q, 10), tagged(&m, 10))
            .unwrap();
        plan.update(&mut d).unwrap();
    }
    let (q, m, _, _) = scene(65, 1, 0);
    let mut missing = plan.create_data(2).unwrap();
    missing
        .write_world_pair(1, tagged(&q, 0), tagged(&m, 0))
        .unwrap();
    assert!(plan.update(&mut missing).is_err());
    assert_eq!(missing.states, vec![None, Some(0)]);
    println!(
        "kinetic_energy_private guard_faults=32 metadata_faults=1 source_nonfinite_faults=12 potential_faults=1 overflow_faults=2 recovered=true"
    );
}
