// Test-only transcription of the pinned XML declarations, not a physics reference.
use super::engine::model::{
    AttachedFields, AttachedModelInput, InertialFields, InertialModelInput, KinematicFields,
    KinematicModelInput, SparseMassFields, SparseMassInput,
};
use serde_json::Value;
pub fn reference() -> Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/sparse-mass-factor/native.json"
    )))
    .unwrap()
}
pub fn ints(v: &Value, key: &str) -> Vec<i32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_i64().unwrap() as i32)
        .collect()
}
pub fn floats(v: &Value, key: &str) -> Vec<f32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap() as f32)
        .collect()
}
pub fn sparse(v: &Value) -> SparseMassInput {
    SparseMassInput::new(SparseMassFields {
        dof_parentid: ints(v, "dof_parentid"),
        m_rownnz: ints(v, "m_rownnz"),
        m_rowadr: ints(v, "m_rowadr"),
        m_colind: ints(v, "m_colind"),
        tree_dofadr: ints(v, "tree_dofadr"),
        tree_dofnum: ints(v, "tree_dofnum"),
    })
    .unwrap()
}
pub fn model(v: &Value) -> AttachedModelInput {
    let parents = ints(v, "dof_parentid");
    let n = parents.len();
    let nb = n + 1;
    let mut body_parentid = vec![0];
    body_parentid.extend(parents.iter().map(|&p| p + 1));
    let mut body_jntadr = vec![-1];
    body_jntadr.extend(0..n as i32);
    let mut body_jntnum = vec![0];
    body_jntnum.extend(vec![1; n]);
    let mut body_pos = vec![0.0; 3];
    body_pos.extend([0.08, 0.01, 0.02].repeat(n));
    let k = KinematicModelInput::new(
        n,
        KinematicFields {
            qpos0: vec![0.0; n],
            body_parentid,
            body_jntadr,
            body_jntnum,
            body_pos,
            body_quat: [1.0, 0.0, 0.0, 0.0].repeat(nb),
            jnt_type: vec![3; n],
            jnt_bodyid: (1..nb as i32).collect(),
            jnt_qposadr: (0..n as i32).collect(),
            jnt_dofadr: (0..n as i32).collect(),
            jnt_pos: vec![0.0; 3 * n],
            jnt_axis: [0.0, 0.0, 1.0].repeat(n),
        },
    )
    .unwrap();
    let mut body_ipos = vec![0.0; 3];
    body_ipos.extend([0.03, 0.01, 0.0].repeat(n));
    let mut body_mass = vec![0.0];
    body_mass.extend(vec![0.01; n]);
    let mut body_inertia = vec![0.0; 3];
    body_inertia.extend(vec![0.001; 3 * n]);
    let m = InertialModelInput::new(
        k,
        InertialFields {
            body_ipos,
            body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(nb),
            body_mass,
            body_inertia,
            dof_bodyid: (1..nb as i32).collect(),
            dof_jntid: (0..n as i32).collect(),
            dof_parentid: parents,
            dof_armature: vec![2.0; n],
            dof_damping: vec![0.0; n],
        },
    )
    .unwrap();
    AttachedModelInput::new(m, AttachedFields::default()).unwrap()
}
