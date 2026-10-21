//! 偏置力测试共用参考模型。

#[cfg(feature = "cuda-probe")]
use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) fn reference_model(v: &serde_json::Value) -> AttachedModelInput {
    use crate::model::{InertialFields, InertialModelInput, KinematicFields, KinematicModelInput};

    let model = &v["model"];
    let floats = |key: &str| {
        model[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_f64().unwrap() as f32)
            .collect()
    };
    let integers = |key: &str| {
        model[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| i32::try_from(value.as_i64().unwrap()).unwrap())
            .collect()
    };
    let kinematics = KinematicModelInput::new(
        v["nv"].as_u64().unwrap() as usize,
        KinematicFields {
            qpos0: floats("qpos0"),
            body_parentid: integers("body_parentid"),
            body_jntadr: integers("body_jntadr"),
            body_jntnum: integers("body_jntnum"),
            body_pos: floats("body_pos"),
            body_quat: floats("body_quat"),
            jnt_type: integers("jnt_type"),
            jnt_bodyid: integers("jnt_bodyid"),
            jnt_qposadr: integers("jnt_qposadr"),
            jnt_dofadr: integers("jnt_dofadr"),
            jnt_pos: floats("jnt_pos"),
            jnt_axis: floats("jnt_axis"),
        },
    )
    .unwrap();
    let rigid = InertialModelInput::new(
        kinematics,
        InertialFields {
            body_ipos: floats("body_ipos"),
            body_iquat: floats("body_iquat"),
            body_mass: floats("body_mass"),
            body_inertia: floats("body_inertia"),
            dof_bodyid: integers("dof_bodyid"),
            dof_jntid: integers("dof_jntid"),
            dof_parentid: integers("dof_parentid"),
            dof_armature: floats("dof_armature"),
            dof_damping: floats("dof_damping"),
        },
    )
    .unwrap();
    AttachedModelInput::new(rigid, crate::model::AttachedFields::default()).unwrap()
}
