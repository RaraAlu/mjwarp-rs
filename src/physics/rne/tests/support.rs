//! RNE测试共用参考模型。

#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) fn reference_model(v: &serde_json::Value) -> AttachedModelInput {
    use crate::model::{InertialFields, InertialModelInput, KinematicFields, KinematicModelInput};
    let m = &v["model"];
    let f = |key: &str| {
        m[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect()
    };
    let i = |key: &str| {
        m[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| i32::try_from(v.as_i64().unwrap()).unwrap())
            .collect()
    };
    let k = KinematicModelInput::new(
        v["nv"].as_u64().unwrap() as usize,
        KinematicFields {
            qpos0: f("qpos0"),
            body_parentid: i("body_parentid"),
            body_jntadr: i("body_jntadr"),
            body_jntnum: i("body_jntnum"),
            body_pos: f("body_pos"),
            body_quat: f("body_quat"),
            jnt_type: i("jnt_type"),
            jnt_bodyid: i("jnt_bodyid"),
            jnt_qposadr: i("jnt_qposadr"),
            jnt_dofadr: i("jnt_dofadr"),
            jnt_pos: f("jnt_pos"),
            jnt_axis: f("jnt_axis"),
        },
    )
    .unwrap();
    let model = InertialModelInput::new(
        k,
        InertialFields {
            body_ipos: f("body_ipos"),
            body_iquat: f("body_iquat"),
            body_mass: f("body_mass"),
            body_inertia: f("body_inertia"),
            dof_bodyid: i("dof_bodyid"),
            dof_jntid: i("dof_jntid"),
            dof_parentid: i("dof_parentid"),
            dof_armature: f("dof_armature"),
            dof_damping: f("dof_damping"),
        },
    )
    .unwrap();
    AttachedModelInput::new(model, crate::model::AttachedFields::default()).unwrap()
}
