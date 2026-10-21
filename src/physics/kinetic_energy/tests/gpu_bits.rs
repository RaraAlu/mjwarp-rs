//! 动能GPU精确位与配对状态测试。

#[cfg(feature = "cuda-probe")]
use super::tests_support::*;
#[cfg(feature = "cuda-probe")]
use super::*;

#[test]
#[cfg(feature = "cuda-probe")]
#[ignore = "new U047 kinetic consumer, independent diagonal M, warp bits and paired state lifecycle"]
fn gpu_reduces_exact_dyadic_and_fixed_warp_cases_with_paired_world_states() {
    let session = TransferSession::new(0).unwrap();
    let start = std::time::Instant::now();
    let mut batches = 0;
    let mut worlds_checked = 0;
    let mut capacity = 0;
    for n in [1, 7, 31, 32, 33, 65, 129, 513] {
        let plan = KineticEnergyPlan::new(&session, 47, n).unwrap();
        for worlds in [1, 2, 257] {
            let mut d = plan.create_data(worlds).unwrap();
            assert!(plan.update(&mut d).is_err());
            assert!(d.readback().is_err());
            capacity = capacity.max(
                4 * (d.pair.len()
                    + d.current.len()
                    + d.staged.len()
                    + d.status.len()
                    + plan.metadata.len()),
            );
            for state in 0..2 {
                let (q, m, expected, p) = scene(n, worlds, state);
                d.write_pair(tagged(&q, state as u64), tagged(&m, state as u64))
                    .unwrap();
                d.write_potential(&p).unwrap();
                plan.update(&mut d).unwrap();
                let out = d.readback().unwrap();
                assert_eq!(out.model_id(), 47);
                assert_eq!(out.worlds(), worlds);
                assert!(out.world(worlds).is_err());
                assert!(out.state_id(worlds).is_err());
                for w in 0..worlds {
                    assert_eq!(out.world(w).unwrap()[0].to_bits(), p[w].to_bits());
                    assert_eq!(
                        out.world(w).unwrap()[1].to_bits(),
                        (expected[w] as f32).to_bits()
                    );
                    assert_eq!(out.state_id(w).unwrap(), state as u64);
                }
                batches += 1;
                worlds_checked += worlds;
            }
            // Invalid tags/values and foreign plan preserve ready output,
            // both input fields and all state associations bit-for-bit.
            let before = bits(&raw(&d.pair));
            let result = bits(&raw(&d.current));
            let states = d.states.clone();
            let q = vec![1.0; n];
            let m = vec![2.0; n];
            assert!(
                d.write_world_pair(worlds, tagged(&q, 8), tagged(&m, 8))
                    .is_err()
            );
            assert!(d.write_world_pair(0, tagged(&q, 8), tagged(&m, 9)).is_err());
            for bad in [
                KineticEnergyVector {
                    model_id: 48,
                    ..tagged(&m, 8)
                },
                tagged(&[], 8),
            ] {
                assert!(d.write_world_pair(0, tagged(&q, 8), bad).is_err());
                assert!(d.write_world_pair(0, bad, tagged(&m, 8)).is_err());
            }
            let bad = vec![f32::NAN; n];
            assert!(
                d.write_world_pair(0, tagged(&q, 8), tagged(&bad, 8))
                    .is_err()
            );
            assert!(d.write_potential(&vec![f32::INFINITY; worlds]).is_err());
            assert!(d.write_world_potential(0, f32::NAN).is_err());
            assert!(d.ready);
            assert_eq!(before, bits(&raw(&d.pair)));
            assert_eq!(result, bits(&raw(&d.current)));
            assert_eq!(states, d.states);
            if worlds == 257 {
                d.write_world_pair(256, tagged(&q, 17), tagged(&m, 17))
                    .unwrap();
                assert!(d.readback().is_err());
                let after = bits(&raw(&d.pair));
                let a = 4 + 512 * n;
                assert_eq!(&before[..a], &after[..a]);
                assert_eq!(&before[a + 2 * n..], &after[a + 2 * n..]);
                assert_eq!(after[a..a + n], bits(&q));
                assert_eq!(after[a + n..a + 2 * n], bits(&m));
                assert_eq!(&d.states[..256], &states[..256]);
                assert_eq!(d.states[256], Some(17));
                d.write_world_potential(256, -0.0).unwrap();
                let cur = bits(&raw(&d.current));
                assert_eq!(&cur[..516], &result[..516]);
                assert_eq!(&cur[517..], &result[517..]);
                assert_eq!(cur[516], 0x80000000);
                plan.update(&mut d).unwrap();
                let out = d.readback().unwrap();
                assert_eq!(out.state_id(256).unwrap(), 17);
                assert_eq!(out.world(256).unwrap(), &[-0.0, n as f32]);
            }
        }
    }
    for c in fixture()["cases"].as_array().unwrap() {
        let (q, m, expected) = stress(c);
        let plan = KineticEnergyPlan::new(&session, 47, q.len()).unwrap();
        let mut d = plan.create_data(2).unwrap();
        for w in 0..2 {
            d.write_world_pair(w, tagged(&q, w as u64), tagged(&m, w as u64))
                .unwrap();
        }
        d.write_potential(&[-0.0, 3.5]).unwrap();
        plan.update(&mut d).unwrap();
        let out = d.readback().unwrap();
        for w in 0..2 {
            assert_eq!(out.world(w).unwrap()[1].to_bits(), expected);
        }
    }
    let plan = KineticEnergyPlan::new(&session, 47, 7).unwrap();
    let mut d = plan.create_data(2).unwrap();
    let q = vec![1.0; 14];
    let m = vec![2.0; 14];
    d.write_pair(tagged(&q, 55), tagged(&m, 55)).unwrap();
    plan.update(&mut d).unwrap();
    let foreign = KineticEnergyPlan::new(&session, 47, 7).unwrap();
    let before = bits(&raw(&d.current));
    assert!(foreign.update(&mut d).is_err());
    assert!(d.ready);
    assert_eq!(before, bits(&raw(&d.current)));
    let snapshot = d.readback().unwrap();
    drop(d);
    drop(foreign);
    drop(plan);
    drop(session);
    assert_eq!(snapshot.world(1).unwrap(), &[0.0, 7.0]);
    assert_eq!(snapshot.state_id(1).unwrap(), 55);
    assert_eq!((batches, worlds_checked), (48, 4160));
    assert_eq!(capacity, 1060000);
    println!(
        "kinetic_energy32 batches={batches} worlds={worlds_checked} kinetic_slots=4160 potential_slots=4160 fixed_bit_worlds=12 max_bit_difference=0 phase_ms={} owned_buffer_capacity_bytes={capacity}",
        start.elapsed().as_millis()
    );
}
