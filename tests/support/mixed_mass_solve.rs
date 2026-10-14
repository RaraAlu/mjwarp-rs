use super::engine::model::{SparseMassFields, SparseMassInput};
use serde_json::Value;
pub fn reference() -> Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/mixed-mass-solve/reference.json"
    )))
    .unwrap()
}
pub fn input(model: &Value) -> SparseMassInput {
    let f = &model["fields"];
    let ints = |key: &str| {
        f[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap() as i32)
            .collect()
    };
    SparseMassInput::new(SparseMassFields {
        dof_parentid: ints("dof_parentid"),
        m_rownnz: ints("m_rownnz"),
        m_rowadr: ints("m_rowadr"),
        m_colind: ints("m_colind"),
        tree_dofadr: ints("tree_dofadr"),
        tree_dofnum: ints("tree_dofnum"),
    })
    .unwrap()
}
pub fn matrix(model: &Value, sample: usize) -> Vec<f32> {
    model["cases"][sample]["matrix"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect()
}
pub fn rhs(model: &Value, sample: usize, nr: usize) -> Vec<f32> {
    let n = model["nv"].as_u64().unwrap() as usize;
    let values = &model["cases"][sample]["rhs"];
    (0..nr * n)
        .map(|i| values[(i / n % 3) * n + i % n].as_f64().unwrap() as f32)
        .collect()
}
pub fn solution(model: &Value, sample: usize, nr: usize) -> Vec<f64> {
    let n = model["nv"].as_u64().unwrap() as usize;
    let values = &model["cases"][sample]["solution"];
    (0..nr * n)
        .map(|i| values[(i / n % 3) * n + i % n].as_f64().unwrap())
        .collect()
}
