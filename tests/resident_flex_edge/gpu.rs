use super::*;
use mjwarp_rs::{
    diagnostics::TransferError,
    model::{
        CamLightParameters, KinematicsParameters, ParameterBatch, TendonWakeFields,
        TendonWakeModelInput,
    },
    physics::{KinematicsData, KinematicsPlan, KinematicsSnapshot},
    runtime::TransferSession,
};
fn plan(s: &TransferSession, b: &Value, e: &Value) -> KinematicsPlan {
    KinematicsPlan::with_flex_positions(
        s,
        input(b, e),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap()
}
fn write(d: &mut KinematicsData, w: usize, c: &Value) {
    d.write_world_qpos(w, &floats(c, "qpos")).unwrap();
    d.write_world_qvel(w, &floats(c, "qvel")).unwrap();
    d.write_world_mocap(w, &floats(c, "mocap_pos"), &floats(c, "mocap_quat"))
        .unwrap();
}
fn compare(a: &[f32], e: &Value) -> f64 {
    let e = e.as_array().unwrap();
    assert_eq!(a.len(), e.len());
    let mut max: f64 = 0.0;
    for (i, (&a, e)) in a.iter().zip(e).enumerate() {
        let e = e.as_f64().unwrap();
        let err = (f64::from(a) - e).abs();
        max = max.max(err);
        assert!(
            a.is_finite() && err <= 2e-5 + 2e-5 * e.abs(),
            "G01 edge index={i} actual={a} expected={e} error={err}"
        );
    }
    max
}
fn native(o: &KinematicsSnapshot, w: usize, c: &Value) -> f64 {
    // Frozen clear semantics, not a native Hessian field comparison.
    assert_eq!(
        o.flex_positions()
            .unwrap()
            .world(w)
            .unwrap()
            .flex_hessian_valid,
        &[false; 4]
    );
    let f = o.flex_edges().unwrap().world(w).unwrap();
    compare(f.flexedge_length, &c["flexedge_length"])
        .max(compare(f.flexedge_velocity, &c["flexedge_velocity"]))
        .max(compare(f.flexedge_jacobian, &c["flexedge_J"]))
}
fn bits(o: &KinematicsSnapshot, w: usize) -> Vec<u32> {
    let f = o.flex_edges().unwrap().world(w).unwrap();
    f.flexedge_length
        .iter()
        .chain(f.flexedge_velocity)
        .chain(f.flexedge_jacobian)
        .map(|v| v.to_bits())
        .collect()
}

#[path = "gpu/edges.rs"]
mod edges;
#[path = "gpu/native.rs"]
mod native;
#[path = "gpu/ownership.rs"]
mod ownership;
#[path = "gpu/wake.rs"]
mod wake;
