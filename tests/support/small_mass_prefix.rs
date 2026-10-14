use super::engine::model::{SparseMassFields, SparseMassInput};
use serde_json::Value;
pub fn analytic() -> Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/small-mass-prefix/reference.json"
    )))
    .unwrap()
}
pub fn base() -> Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/sparse-mass-factor/native.json"
    )))
    .unwrap()
}
pub fn sparse(v: &Value) -> SparseMassInput {
    let ints = |key: &str| {
        v[key]
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
pub fn mixed_input() -> SparseMassInput {
    let mut f = sparse(&base()["models"][0]).fields().clone();
    for n in 2..=6 {
        let start = f.dof_parentid.len();
        f.tree_dofadr.push(start as i32);
        f.tree_dofnum.push(n as i32);
        for i in 0..n {
            f.dof_parentid
                .push(if i == 0 { -1 } else { (start + i - 1) as i32 });
            f.m_rowadr.push(f.m_colind.len() as i32);
            f.m_rownnz.push((i + 1) as i32);
            f.m_colind.extend((start..=start + i).map(|v| v as i32));
        }
    }
    let start = f.dof_parentid.len();
    f.tree_dofadr.push(start as i32);
    f.tree_dofnum.push(1);
    f.dof_parentid.push(-1);
    f.m_rowadr.push(f.m_colind.len() as i32);
    f.m_rownnz.push(1);
    f.m_colind.push(start as i32);
    SparseMassInput::new(f).unwrap()
}
pub fn mixed_matrix(sample: usize) -> Vec<f32> {
    let refs = base();
    let mut v: Vec<f32> = refs["models"][0]["cases"][sample]["matrix"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect();
    v.extend(
        analytic()["cases"][sample]["csr"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32),
    );
    v.push(4.0);
    v
}
