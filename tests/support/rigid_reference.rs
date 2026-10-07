//! 两项GPU探针共用静态DTO。
use mjwarp_rs::model::{InertialFields, InertialModelInput, KinematicFields, KinematicModelInput};
use serde_json::Value;

pub fn floats(v: &Value, key: &str) -> Vec<f32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap() as f32)
        .collect()
}
fn integers(v: &Value, key: &str) -> Vec<i32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| i32::try_from(x.as_i64().unwrap()).unwrap())
        .collect()
}
pub fn model(v: &Value) -> InertialModelInput {
    let m = &v["model"];
    let k = KinematicModelInput::new(
        v["nv"].as_u64().unwrap() as usize,
        KinematicFields {
            qpos0: floats(m, "qpos0"),
            body_parentid: integers(m, "body_parentid"),
            body_jntadr: integers(m, "body_jntadr"),
            body_jntnum: integers(m, "body_jntnum"),
            body_pos: floats(m, "body_pos"),
            body_quat: floats(m, "body_quat"),
            jnt_type: integers(m, "jnt_type"),
            jnt_bodyid: integers(m, "jnt_bodyid"),
            jnt_qposadr: integers(m, "jnt_qposadr"),
            jnt_dofadr: integers(m, "jnt_dofadr"),
            jnt_pos: floats(m, "jnt_pos"),
            jnt_axis: floats(m, "jnt_axis"),
        },
    )
    .unwrap();
    assert_eq!(k.nq(), v["nq"].as_u64().unwrap() as usize);
    assert_eq!(k.nbody(), v["nbody"].as_u64().unwrap() as usize);
    assert_eq!(k.njnt(), v["njnt"].as_u64().unwrap() as usize);
    InertialModelInput::new(
        k,
        InertialFields {
            body_ipos: floats(m, "body_ipos"),
            body_iquat: floats(m, "body_iquat"),
            body_mass: floats(m, "body_mass"),
            body_inertia: floats(m, "body_inertia"),
            dof_bodyid: integers(m, "dof_bodyid"),
            dof_jntid: integers(m, "dof_jntid"),
            dof_parentid: integers(m, "dof_parentid"),
            dof_armature: floats(m, "dof_armature"),
            dof_damping: floats(m, "dof_damping"),
        },
    )
    .unwrap()
}
