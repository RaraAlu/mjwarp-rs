//! 历史刚体模板回溯。

use super::super::require;

/// 旧捕获仅证明历史源码，不充当修复后的完整捕获。
/// 只回溯已实测的一个分支；其他漂移仍触发原哈希检查。
pub(crate) fn historical_rigid_kernel(text: &str) -> Result<String, String> {
    let current = "if(n==0.0f) return {0,0,0,1};";
    require(text.matches(current).count() == 1, "zero fallback drift")?;
    Ok(text.replace(current, "if(n==0.0f) return {0,0,0,0};"))
}
