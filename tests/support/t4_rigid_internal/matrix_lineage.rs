//! G03 矩阵修复的源码逆变换（仅证据测试支撑，不进入产品内核）。

// 当前 KINEMATICS 中由 G03 矩阵链对齐引入的片段；按历史捕获重建源码时整体还原。
pub(super) const MATRIX_REPAIRED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/support/t4_rigid_internal/matrix-repaired.txt"
));

// 历史捕获 cd781388… 对应的旧 matrix 写法与调用。
pub(super) const MATRIX_HISTORICAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/support/t4_rigid_internal/matrix-historical.txt"
));

/// 把当前矩阵修复片段还原为历史写法；锚点缺失时拒绝。
pub(super) fn before(source: &str) -> Result<String, &'static str> {
    let source = source.replace("\r\n", "\n");
    let repaired = MATRIX_REPAIRED.replace("\r\n", "\n");
    let historical = MATRIX_HISTORICAL.replace("\r\n", "\n");
    if !source.contains(&repaired) {
        return Err("matrix repair anchor");
    }
    Ok(source.replacen(&repaired, &historical, 1))
}
