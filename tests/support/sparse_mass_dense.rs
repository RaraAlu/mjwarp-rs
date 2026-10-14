use super::engine::model::{SparseMassFields, SparseMassInput};
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
        .map(|v| v.as_i64().unwrap() as i32)
        .collect()
}
pub fn matrix(v: &Value) -> Vec<f32> {
    v["matrix"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect()
}
pub fn native(v: &Value) -> SparseMassInput {
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
pub fn fork(n: usize) -> SparseMassInput {
    let mut f = SparseMassFields {
        dof_parentid: vec![-1],
        m_rownnz: vec![1],
        m_rowadr: vec![0],
        m_colind: vec![0],
        tree_dofadr: vec![0],
        tree_dofnum: vec![n as i32],
    };
    for i in 1..n {
        f.dof_parentid.push(0);
        f.m_rownnz.push(2);
        f.m_rowadr.push((2 * i - 1) as i32);
        f.m_colind.extend([0, i as i32]);
    }
    SparseMassInput::new(f).unwrap()
}
pub fn small() -> SparseMassInput {
    SparseMassInput::new(SparseMassFields {
        dof_parentid: vec![-1, 0, -1, 2, 3],
        m_rownnz: vec![1, 2, 1, 2, 3],
        m_rowadr: vec![0, 1, 3, 4, 6],
        m_colind: vec![0, 0, 1, 2, 2, 3, 2, 3, 4],
        tree_dofadr: vec![0, 2],
        tree_dofnum: vec![2, 3],
    })
    .unwrap()
}
