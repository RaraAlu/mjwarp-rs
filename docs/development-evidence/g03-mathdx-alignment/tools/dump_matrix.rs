//! Offline dump of the product mass matrix for precheck analysis (not product code).
use mjwarp_rs::model::{InertialFields, InertialModelInput, KinematicFields, KinematicModelInput};
use mjwarp_rs::physics::probe_mass_matrix;
use mjwarp_rs::runtime::TransferSession;
use serde_json::Value;
use std::fs;

fn floats(v: &Value, key: &str) -> Vec<f32> {
    v[key].as_array().unwrap().iter().map(|x| x.as_f64().unwrap() as f32).collect()
}
fn integers(v: &Value, key: &str) -> Vec<i32> {
    v[key].as_array().unwrap().iter().map(|x| i32::try_from(x.as_i64().unwrap()).unwrap()).collect()
}
fn model(v: &Value) -> InertialModelInput {
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
fn main() {
    let root = std::path::PathBuf::from(std::env::args().nth(1).expect("repo root"));
    let out = std::path::PathBuf::from(std::env::args().nth(2).expect("output f32 path"));
    let matrix_text = fs::read_to_string(root.join("fixtures/mass-matrix/mixed-joints.json")).unwrap();
    let matrix_fixture: Value = serde_json::from_str(&matrix_text).unwrap();
    let solve_fixture: Value = serde_json::from_str(
        &fs::read_to_string(root.join("fixtures/mass-solve/mixed-joints.json")).unwrap(),
    )
    .unwrap();
    let m = model(&matrix_fixture);
    let cases = solve_fixture["cases"].as_array().unwrap();
    let q: Vec<f32> = (0..8).flat_map(|w| floats(&cases[w % 8], "qpos")).collect();
    let session = TransferSession::new(0).unwrap();
    let out_mm = probe_mass_matrix(&session, &m, 8, &q).unwrap();
    let world = out_mm.world(4).unwrap();
    let mut bytes = Vec::with_capacity(world.matrix.len() * 4);
    for v in world.matrix {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    fs::write(out, bytes).unwrap();
    println!("MATRIX_DUMP_OK nv={} values={}", out_mm.nv(), world.matrix.len());
}
