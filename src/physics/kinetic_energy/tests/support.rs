//! 动能测试共用标签与场景构造。

use super::*;

pub(super) fn tagged(values: &[f32], state_id: u64) -> KineticEnergyVector<'_> {
    KineticEnergyVector {
        model_id: 47,
        state_id,
        values,
    }
}
pub(super) fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/kinetic-energy/reference.json"
    )))
    .unwrap()
}
pub(super) fn stress(v: &serde_json::Value) -> (Vec<f32>, Vec<f32>, u32) {
    let n = v["nv"].as_u64().unwrap() as usize;
    let expected = u32::from_str_radix(v["expected_bits"].as_str().unwrap(), 16).unwrap();
    let mut q = vec![1.0; n];
    let mut m = vec![0.0; n];
    match v["kind"].as_str().unwrap() {
        "cancel" | "cancel_tail" => {
            m[0] = 33554432.0;
            m[1] = 1.0;
            m[16] = -33554432.0;
            if n == 33 {
                m[32] = -1.0;
            }
        }
        "negative_zero" => q.fill(-0.0),
        "subnormal" => m.fill(f32::from_bits(1)),
        _ => panic!("unknown certificate"),
    }
    (q, m, expected)
}
// Diagnostic only: fixed Warp lane/register/shuffle dependency graph.
// This never computes a product output or supplies a device result.
pub(super) fn warp_certificate(q: &[f32], m: &[f32]) -> f32 {
    let mut lanes = [0.0f32; 32];
    let active = q.len().min(32);
    for (lane, sum) in lanes.iter_mut().enumerate().take(active) {
        *sum = q[lane] * m[lane];
        for i in (lane + 32..q.len()).step_by(32) {
            *sum += q[i] * m[i];
        }
    }
    let mask = if active == 32 {
        u32::MAX
    } else {
        (1u32 << active) - 1
    };
    for offset in [16, 8, 4, 2, 1] {
        let old = lanes;
        for lane in 0..active {
            if mask == u32::MAX || mask & (1 << ((lane + offset) % 32)) != 0 {
                // Out-of-warp shuffles return the requesting lane's value.
                lanes[lane] += old[if lane + offset < 32 {
                    lane + offset
                } else {
                    lane
                }];
            }
        }
    }
    0.5 * lanes[0]
}

#[cfg(feature = "cuda-probe")]
pub(super) fn raw(b: &TransferBuffer<f32>) -> Vec<f32> {
    let mut v = vec![0.0; b.len()];
    b.read_range_into(0, &mut v).unwrap();
    v
}
#[cfg(feature = "cuda-probe")]
pub(super) fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|v| v.to_bits()).collect()
}
#[cfg(feature = "cuda-probe")]
pub(super) fn scene(
    n: usize,
    worlds: usize,
    state: usize,
) -> (Vec<f32>, Vec<f32>, Vec<f64>, Vec<f32>) {
    let mut q = Vec::new();
    let mut m = Vec::new();
    let mut expected = Vec::new();
    let mut p = Vec::new();
    for w in 0..worlds {
        let mut energy = 0.0;
        for i in 0..n {
            // Independent positive diagonal model, dyadic inputs. Mqvel
            // derives from that known M; no old mass producer executes.
            let mass = ((i + state) % 7 + 1) as f64 / 4.0;
            let velocity = ((i + 3 * w + state) % 9) as f64 / 8.0 - 0.5;
            q.push(velocity as f32);
            m.push((mass * velocity) as f32);
            energy += 0.5 * mass * velocity * velocity;
        }
        assert_eq!(energy as f32 as f64, energy);
        expected.push(energy);
        p.push(if (w + state).is_multiple_of(7) {
            -0.0
        } else {
            ((w + state) % 17) as f32 / 4.0
        });
    }
    (q, m, expected, p)
}
