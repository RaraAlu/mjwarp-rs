//! 施加外力测试共用参考与工具。

use super::*;

pub(super) fn reference() -> serde_json::Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/applied-force/reference.json"
    )))
    .unwrap()
}
pub(super) fn ints(v: &serde_json::Value, key: &str) -> Vec<i32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap() as i32)
        .collect()
}
pub(super) fn floats(v: &serde_json::Value, key: &str) -> Vec<f32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect()
}
// Independent virtual work: project torque+offset×force, not the GPU jac
// permutation or dot6+dot3 decomposition. Diagnostic f64 never feeds GPU.
pub(super) fn oracle(
    t: &Topology,
    xpos: &[f32],
    com: &[f32],
    cdof: &[f32],
    ft: &[f32],
) -> Vec<f64> {
    let mut out = vec![0.0; t.body.len()];
    for (i, value) in out.iter_mut().enumerate() {
        let d = t.body[i];
        if d == 0 {
            continue;
        }
        for b in 1..t.parent.len() {
            let mut p = b as i32;
            while p != 0 && p != d {
                p = t.parent[p as usize];
            }
            if p != d {
                continue;
            }
            let mut r = [0.0; 3];
            let mut f = [0.0; 3];
            for k in 0..3 {
                r[k] = f64::from(xpos[3 * b + k]) - f64::from(com[3 * t.root[b] as usize + k]);
                f[k] = f64::from(ft[6 * b + k]);
            }
            let moment = [
                r[1] * f[2] - r[2] * f[1],
                r[2] * f[0] - r[0] * f[2],
                r[0] * f[1] - r[1] * f[0],
            ];
            for k in 0..3 {
                *value += f64::from(cdof[6 * i + k]) * (f64::from(ft[6 * b + 3 + k]) + moment[k]);
                *value += f64::from(cdof[6 * i + 3 + k]) * f[k];
            }
        }
    }
    out
}

#[cfg(feature = "cuda-probe")]
pub(super) fn model(nb: usize, n: usize) -> Topology {
    let parent = (0..nb)
        .map(|b| {
            if b == 0 || (b - 1) % 17 == 0 {
                0
            } else {
                (b - 1) as i32
            }
        })
        .collect::<Vec<_>>();
    let mut root = vec![0; nb];
    for b in 1..nb {
        root[b] = if parent[b] == 0 {
            b as i32
        } else {
            root[parent[b] as usize]
        };
    }
    topology(
        parent,
        root,
        (0..n).map(|i| ((i * 17 + 1) % nb) as i32).collect(),
    )
    .unwrap()
}
#[cfg(feature = "cuda-probe")]
type Scene = (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>);

#[cfg(feature = "cuda-probe")]
pub(super) fn scene(t: &Topology, w: usize, sample: usize) -> Scene {
    let nb = t.parent.len();
    let n = t.body.len();
    let x = (0..3 * nb)
        .map(|j| ((j / 3 + j % 3 + sample) % 9) as f32 * 0.25 - 1.0 + (w % 3) as f32 * 0.5)
        .collect();
    let c = (0..3 * nb)
        .map(|j| ((2 * (j / 3) + j % 3 + sample) % 7) as f32 * 0.25 - 0.75 + (w % 3) as f32 * 0.5)
        .collect();
    let d = (0..6 * n)
        .map(|j| ((j / 6 + 2 * (j % 6) + sample) % 7) as f32 * 0.125 - 0.375)
        .collect();
    let ft = (0..6 * nb)
        .map(|j| {
            if j < 6 {
                ((j + sample) % 7) as f32 * 128.0
            } else if (j / 6) % 4 == 0 {
                if j % 2 == 0 { 0.0 } else { -0.0 }
            } else {
                ((j / 6 + 3 * (j % 6) + w + sample) % 9) as f32 * 0.25 - 1.0
            }
        })
        .collect();
    let q = (0..n)
        .map(|i| {
            let v = ((w + i + sample) % 9) as f32 * 0.25 - 1.0;
            if v == 0.0 && i % 2 == 0 { -0.0 } else { v }
        })
        .collect();
    (x, c, d, ft, q)
}
#[cfg(feature = "cuda-probe")]
pub(super) fn fill(
    plan: &AppliedForcePlan,
    data: &mut AppliedForceData,
    sample: usize,
) -> Vec<Vec<f64>> {
    let mut x = Vec::new();
    let mut c = Vec::new();
    let mut d = Vec::new();
    let mut f = Vec::new();
    let mut q = Vec::new();
    let mut expected = Vec::new();
    for w in 0..data.worlds() {
        let (wx, wc, wd, wf, wq) = scene(&plan.input, w, sample);
        expected.push(oracle(&plan.input, &wx, &wc, &wd, &wf));
        x.extend(wx);
        c.extend(wc);
        d.extend(wd);
        f.extend(wf);
        q.extend(wq);
    }
    data.write_geometry(AppliedForceGeometry::Xipos, &x)
        .unwrap();
    data.write_geometry(AppliedForceGeometry::SubtreeCom, &c)
        .unwrap();
    data.write_geometry(AppliedForceGeometry::Cdof, &d).unwrap();
    data.write_ft(&f).unwrap();
    data.write_qfrc(&q).unwrap();
    expected
}
#[cfg(feature = "cuda-probe")]
pub(super) fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|v| v.to_bits()).collect()
}
#[cfg(feature = "cuda-probe")]
pub(super) fn raw(v: &TransferBuffer<f32>) -> Vec<f32> {
    let mut out = vec![0.0; v.len()];
    v.read_range_into(0, &mut out).unwrap();
    out
}
#[cfg(feature = "cuda-probe")]
pub(super) fn raw_meta(v: &TransferBuffer<i32>) -> Vec<i32> {
    let mut out = vec![0; v.len()];
    v.read_range_into(0, &mut out).unwrap();
    out
}
#[cfg(feature = "cuda-probe")]
pub(super) fn tokens(d: &AppliedForceData) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    (
        bits(&raw(&d.geometry)),
        bits(&raw(&d.ft)),
        bits(&raw(&d.current)),
    )
}
