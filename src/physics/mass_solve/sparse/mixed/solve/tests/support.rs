//! 混合求解测试共用样例与参考数据。

use super::*;
#[cfg(feature = "cuda-probe")]
use crate as engine;
#[cfg(feature = "cuda-probe")]
pub(super) mod samples {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/support/mixed_mass_solve.rs"
    ));
}
pub(super) fn input() -> SparseMassInput {
    let refs: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fixtures/mixed-mass-solve/reference.json"
    )))
    .unwrap();
    let f = &refs["models"][0]["fields"];
    let ints = |key: &str| {
        f[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap() as i32)
            .collect()
    };
    SparseMassInput::new(crate::model::SparseMassFields {
        dof_parentid: ints("dof_parentid"),
        m_rownnz: ints("m_rownnz"),
        m_rowadr: ints("m_rowadr"),
        m_colind: ints("m_colind"),
        tree_dofadr: ints("tree_dofadr"),
        tree_dofnum: ints("tree_dofnum"),
    })
    .unwrap()
}
pub(super) fn validation_metadata() -> Vec<i32> {
    vec![0, 4, 3, 0, -1, -1, -1, 0, 3, 9]
}
pub(super) fn fused_metadata() -> Vec<i32> {
    vec![
        1, 4, 2, 1, // 层数、Small前缀、CSR跨度、Small数量。
        -1, -1, 0, 0, // Sparse与Small路由。
        6, 7, 2, 3, // 校验入口使用的对角地址。
        0, 1, // 单层更新区间。
        0, 1, 0, // 同层写0、读1、L地址0。
        2, 2, 0, // Small从2开始，大小2，前缀地址0。
    ]
}
