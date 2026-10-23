//! libmathdx 的 NVRTC 依赖预加载。
//! Windows：`mathdx64_0.dll`静态依赖`nvrtc64_120_0.dll`；
//! 按绝对路径先加载 pinned NVRTC 与 builtins，使按名解析命中已加载模块，
//! 不修改进程 DLL 搜索目录。Linux 部署未验收，依赖交由系统动态链接器。

use std::path::{Path, PathBuf};

use super::LauncherError;

/// 解析顺序：显式`MATHDX_NVRTC_DIR` → 库同目录 → `CUDA_PATH/bin`。
/// 找到后连同 builtins 一起按绝对路径预加载。
#[cfg(windows)]
pub(super) fn preload_dependencies(path: &Path) -> Result<Vec<libloading::Library>, LauncherError> {
    if let Some(directory) = std::env::var_os("MATHDX_NVRTC_DIR") {
        return preload_nvrtc(&PathBuf::from(directory));
    }
    let mut directories = Vec::new();
    if let Some(parent) = path.parent() {
        directories.push(parent.to_path_buf());
    }
    if let Some(root) = std::env::var_os("CUDA_PATH") {
        directories.push(PathBuf::from(root).join("bin"));
    }
    for directory in directories {
        if directory.join("nvrtc64_120_0.dll").is_file() {
            return preload_nvrtc(&directory);
        }
    }
    Ok(Vec::new())
}

/// 从显式目录严格预加载 pinned NVRTC 与 builtins。
/// 缺任一组件都显式报错，不回退到其他版本。
#[cfg(windows)]
pub(super) fn preload_nvrtc(directory: &Path) -> Result<Vec<libloading::Library>, LauncherError> {
    let nvrtc = directory.join("nvrtc64_120_0.dll");
    if !nvrtc.is_file() {
        return Err(LauncherError::Linker(format!(
            "缺少pinned NVRTC 12.9.86组件：{}（先运行 scripts/prepare-cuda-probe.ps1）",
            nvrtc.display()
        )));
    }
    let mut libraries = vec![super::ffi::preload(&nvrtc)?];
    let entries = std::fs::read_dir(directory).map_err(|error| {
        LauncherError::Linker(format!(
            "读取NVRTC目录失败：{}：{error}",
            directory.display()
        ))
    })?;
    let mut builtins: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("nvrtc-builtins64_") && name.ends_with(".dll"))
        })
        .collect();
    builtins.sort();
    if builtins.is_empty() {
        return Err(LauncherError::Linker(format!(
            "缺少NVRTC builtins组件：{}",
            directory.display()
        )));
    }
    for path in builtins {
        libraries.push(super::ffi::preload(&path)?);
    }
    Ok(libraries)
}

#[cfg(unix)]
pub(super) fn preload_dependencies(
    _path: &Path,
) -> Result<Vec<libloading::Library>, LauncherError> {
    Ok(Vec::new())
}

#[cfg(unix)]
pub(super) fn preload_nvrtc(_directory: &Path) -> Result<Vec<libloading::Library>, LauncherError> {
    Ok(Vec::new())
}
