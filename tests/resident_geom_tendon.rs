#![cfg(feature = "cuda-probe")]
//! G01球柱绕行辅助验收。
//! 产品测试只读取静态参考。
use mjwarp_rs::{
    diagnostics::{InputError, TransferError},
    model::{
        AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, CamLightParameters,
        FixedTendonFields, FixedTendonModelInput, InertialFields, InertialModelInput,
        KinematicFields, KinematicModelInput, KinematicsParameter as F, KinematicsParameters,
        MocapModelInput, ParameterBatch, SpatialTendonFields, SpatialTendonGeometry,
        SpatialTendonModelInput,
    },
    physics::{KinematicsPlan, KinematicsSnapshot},
    runtime::TransferSession,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
#[path = "support/geom_tendon_input.rs"]
mod input;
use input::{fixture, floats, ids, model, motion_fixture, parts, plan, write};
#[path = "support/two_reference.rs"]
mod two_reference;
#[path = "support/mjwarp_geom_reference.rs"]
mod upstream_reference;
use two_reference::{Audit, MJWARP_RELATIVE_SLACK, SecondaryReference, evaluate, native_reference};
fn compare(a: &[f32], e: &[f64]) -> f64 {
    // 解析不变量不进入参考回退。
    assert_eq!(a.len(), e.len());
    let mut max: f64 = 0.0;
    for (i, (&a, &e)) in a.iter().zip(e).enumerate() {
        let error = (f64::from(a) - e).abs();
        max = max.max(error);
        assert!(
            a.is_finite() && error <= 2e-5 + 2e-5 * e.abs(),
            "G01 geom index={i} actual={a} expected={e} error={error}"
        );
    }
    max
}
fn native(out: &KinematicsSnapshot, w: usize, c: &Value) -> f64 {
    let a = out.spatial_tendon().world(w).unwrap();
    assert_eq!(a.ten_wrapadr, ids(c, "ten_wrapadr"));
    assert_eq!(a.ten_wrapnum, ids(c, "ten_wrapnum"));
    assert_eq!(a.wrap_obj, ids(c, "wrap_obj"));
    let e: Vec<_> = ["ten_length", "ten_J", "wrap_xpos"]
        .into_iter()
        .flat_map(|k| c[k].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()))
        .collect();
    two_reference::assert_native(
        "MuJoCo3.12.0:geom-tendon:combined",
        &[a.ten_length, a.ten_jacobian, a.wrap_xpos].concat(),
        e,
        2e-5,
        2e-5,
    )
}

#[path = "resident_geom_tendon/cylinder.rs"]
mod cylinder;
#[path = "resident_geom_tendon/paths.rs"]
mod paths;
#[path = "resident_geom_tendon/readiness.rs"]
mod readiness;
#[path = "resident_geom_tendon/reference.rs"]
mod reference;
