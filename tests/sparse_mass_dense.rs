//! 新块稠密化用途；不执行旧factor/solve或参考作者。
use mjwarp_rs as engine;
#[path = "support/sparse_mass_dense.rs"]
mod input;
use engine::model::{SparseMassFields, SparseMassInput};

fn fork_sources(n: usize) -> Vec<i32> {
    (0..n)
        .flat_map(|i| {
            (0..n).map(move |j| {
                if i == j {
                    (2 * i) as i32
                } else if i == 0 || j == 0 {
                    (2 * i.max(j) - 1) as i32
                } else {
                    (2 * n - 1) as i32
                }
            })
        })
        .collect()
}
#[test]
fn new_dense_sources_match_native_prefix_and_independent_closed_forms() {
    let refs = input::reference();
    for m in refs["models"].as_array().unwrap().iter().take(3) {
        assert!(input::native(m).dense_block_sources().unwrap().is_empty());
    }
    let m = &refs["models"][3];
    let native = input::native(m);
    assert_eq!(native.nv(), 72);
    assert_eq!(native.nc(), 2173);
    assert_eq!(input::matrix(&m["cases"][0]).len(), 2173);
    let expected: Vec<i32> = (0..7)
        .flat_map(|i| (0..7).map(move |j| 2145 + i.max(j) * (i.max(j) + 1) / 2 + i.min(j)))
        .collect();
    assert_eq!(native.dense_block_sources().unwrap(), expected);
    for n in [7, 64] {
        assert_eq!(
            input::fork(n).dense_block_sources().unwrap(),
            fork_sources(n)
        );
    }
    let small = input::small();
    assert_eq!(small.qld_block_adr(), [0, 0, 4, 4, 4]);
    assert_eq!(
        small.dense_block_sources().unwrap(),
        [0, 1, 1, 2, 3, 4, 6, 4, 5, 7, 6, 7, 8]
    );
    assert!(
        SparseMassInput::new(SparseMassFields::default())
            .unwrap()
            .dense_block_sources()
            .unwrap()
            .is_empty()
    );
}

#[cfg(feature = "cuda-probe")]
mod gpu {
    use super::*;
    use engine::{physics::SparseMassDensePlan, runtime::TransferSession};
    use serde_json::json;
    use std::time::Instant;
    fn bits(actual: &[f32], expected: &[f32]) {
        assert_eq!(
            actual.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            expected.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
    }
    fn expected(values: &[f32], sources: &[i32]) -> Vec<f32> {
        sources
            .iter()
            .map(|&a| {
                if a as usize == values.len() {
                    0.0
                } else {
                    values[a as usize]
                }
            })
            .collect()
    }
    #[test]
    #[ignore = "requires NVIDIA/NVRTC; new fourth-model block densify purpose"]
    fn gpu_fourth_native_model_copies_two_states_and_world256_without_factor_claim() {
        let refs = input::reference();
        let m = &refs["models"][3];
        let session = TransferSession::new(0).unwrap();
        let plan = SparseMassDensePlan::new(&session, input::native(m)).unwrap();
        assert!(plan.create_data(0).is_err());
        assert!(plan.create_data(u32::MAX as usize).is_err());
        let sources: Vec<i32> = (0..7)
            .flat_map(|i| (0..7).map(move |j| 2145 + i.max(j) * (i.max(j) + 1) / 2 + i.min(j)))
            .collect();
        let mut batches = Vec::new();
        let mut count = 0;
        for worlds in [1, 2, 257] {
            let start = Instant::now();
            let mut data = plan.create_data(worlds).unwrap();
            assert!(data.readback().is_err());
            for sample in 0..2 {
                let matrix = input::matrix(&m["cases"][sample]);
                let wanted = expected(&matrix, &sources);
                data.write_matrix(&matrix.repeat(worlds)).unwrap();
                assert!(data.readback().is_err());
                plan.update(&mut data).unwrap();
                let output = data.readback().unwrap();
                for w in 0..worlds {
                    bits(output.world(w).unwrap(), &wanted);
                    count += wanted.len();
                }
            }
            batches.push(json!({"worlds":worlds,"states":2,"slots":worlds*98,"phase_elapsed_ms":start.elapsed().as_millis(),"product_device_bytes":4*((2173+49+1)*worlds+24+49)}));
        }
        let mut data = plan.create_data(257).unwrap();
        let m0 = input::matrix(&m["cases"][0]);
        let m1 = input::matrix(&m["cases"][1]);
        data.write_matrix(&m0.repeat(257)).unwrap();
        plan.update(&mut data).unwrap();
        let before = data.readback().unwrap();
        data.write_world_matrix(256, &m1).unwrap();
        plan.update(&mut data).unwrap();
        let after = data.readback().unwrap();
        for w in [0, 255] {
            bits(after.world(w).unwrap(), before.world(w).unwrap());
        }
        bits(after.world(256).unwrap(), &expected(&m1, &sources));
        drop(data);
        drop(plan);
        drop(session);
        assert_eq!(after.worlds(), 257);
        assert!(after.world(257).is_err());
        let result = json!({"schema":1,"purpose":"new fourth-model dense prefix input; not factors","copied_slots":count,"world_edit_slots":147,"batches":batches});
        if let Some(dir) = std::env::var_os("MJWARP_SPARSE_DENSE_RESULTS") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                std::path::Path::new(&dir).join("native.json"),
                serde_json::to_vec_pretty(&result).unwrap(),
            )
            .unwrap();
        }
        println!("{result}");
    }
    #[test]
    #[ignore = "requires NVIDIA/NVRTC; new structural-zero/small-prefix block densify purpose"]
    fn gpu_structural_holes_negative_zero_and_multiple_prefix_blocks_copy_bits() {
        let session = TransferSession::new(0).unwrap();
        let mut batches = Vec::new();
        let mut total = 0;
        let mut zeros = 0;
        for kind in 0..3 {
            let (input, sources, base) = if kind == 2 {
                (
                    input::small(),
                    vec![0, 1, 1, 2, 3, 4, 6, 4, 5, 7, 6, 7, 8],
                    vec![4.0, 1.0, 5.0, 6.0, 2.0, 7.0, 3.0, 4.0, 8.0],
                )
            } else {
                let n = if kind == 0 { 7 } else { 64 };
                let mut v = vec![11.0];
                for i in 1..n {
                    v.extend([if i == 1 { -0.0 } else { i as f32 + 0.25 }, 20.0 + i as f32]);
                }
                (input::fork(n), fork_sources(n), v)
            };
            let nc = input.nc();
            let prefix = input.qld_block_total();
            let plan = SparseMassDensePlan::new(&session, input).unwrap();
            for worlds in [1, 2, 257] {
                let start = Instant::now();
                let mut data = plan.create_data(worlds).unwrap();
                for phase in 0..2 {
                    let values: Vec<f32> = (0..worlds)
                        .flat_map(|w| {
                            base.iter().map(move |&v| {
                                if v == 0.0 {
                                    v
                                } else {
                                    v + (4096 * w + phase) as f32
                                }
                            })
                        })
                        .collect();
                    data.write_matrix(&values).unwrap();
                    plan.update(&mut data).unwrap();
                    let out = data.readback().unwrap();
                    for w in 0..worlds {
                        bits(
                            out.world(w).unwrap(),
                            &expected(&values[w * nc..(w + 1) * nc], &sources),
                        );
                        total += prefix;
                        zeros += sources.iter().filter(|&&s| s == nc as i32).count();
                    }
                }
                batches.push(json!({"kind":kind,"worlds":worlds,"states":2,"prefix":prefix,"slots":2*worlds*prefix,"phase_elapsed_ms":start.elapsed().as_millis(),"product_device_bytes":4*((nc+prefix+1)*worlds+24+prefix.max(1))}));
            }
        }
        for source in [
            SparseMassInput::new(SparseMassFields::default()).unwrap(),
            input::native(&input::reference()["models"][0]),
        ] {
            let plan = SparseMassDensePlan::new(&session, source).unwrap();
            let mut data = plan.create_data(2).unwrap();
            data.write_matrix(&vec![0.0; 2 * plan.input().nc()])
                .unwrap();
            plan.update(&mut data).unwrap();
            assert!(data.readback().unwrap().world(1).unwrap().is_empty());
        }
        let result = json!({"schema":1,"purpose":"new structural block densify; no Cholesky/solve","copied_slots":total,"structural_positive_zero_slots":zeros,"negative_zero_copy_asserted":true,"batches":batches});
        if let Some(dir) = std::env::var_os("MJWARP_SPARSE_DENSE_RESULTS") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                std::path::Path::new(&dir).join("synthetic.json"),
                serde_json::to_vec_pretty(&result).unwrap(),
            )
            .unwrap();
        }
        println!("{result}");
    }
}
