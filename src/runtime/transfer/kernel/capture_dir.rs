//! 测试侧PTX捕获目录分配（仅测试构建）。
//!
//! `MJWARP_PHYSICS_PTX_DIR` 统一按“捕获根目录”解释。
//! 根目录存在时按调用作用域与标签分配唯一子目录。
//! 根目录不存在时由调用方决定沿用旧口径。
//! 同一进程内多次捕获因此不会共享目录。

use std::path::PathBuf;

fn root() -> Option<PathBuf> {
    std::env::var_os("MJWARP_PHYSICS_PTX_DIR").map(PathBuf::from)
}

fn unique(root: &PathBuf, scope: &str, label: &str) -> PathBuf {
    let scope = scope.replace("::", "-");
    root.join(format!("{scope}-{label}"))
}

/// 捕获根目录下的唯一子目录。
///
/// 无论根目录是否存在都返回`根/作用域-标签`。
/// 产品捕获代码会按需创建父目录。
/// 未设置环境变量时返回`None`。
pub(crate) fn capture_child(scope: &str, label: &str) -> Option<PathBuf> {
    root().map(|root| unique(&root, scope, label))
}

/// 重采集与归档使用的输出目录。
///
/// 根目录存在时返回唯一子目录，避免与同进程其他捕获冲突。
/// 根目录不存在时沿用旧口径：变量本身即本次唯一目录。
/// 未设置环境变量时返回`None`。
pub(crate) fn capture_output(scope: &str, label: &str) -> Option<PathBuf> {
    let root = root()?;
    Some(if root.exists() {
        unique(&root, scope, label)
    } else {
        root
    })
}
