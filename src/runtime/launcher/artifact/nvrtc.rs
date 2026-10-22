//! NVRTC 版本探测与编译封装。

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use cudarc::nvrtc::{CompileOptions, compile_ptx_with_opts, sys};

use super::*;

static HEADER_DIR_ID: AtomicU64 = AtomicU64::new(0);

/// 进程只加载一份NVRTC；成功结果缓存，失败每次重查。
pub(super) fn nvrtc_version() -> Result<(i32, i32), LauncherError> {
    static VERSION: OnceLock<(i32, i32)> = OnceLock::new();
    if let Some(version) = VERSION.get() {
        return Ok(*version);
    }
    let version = query_nvrtc_version()?;
    Ok(*VERSION.get_or_init(|| version))
}

fn query_nvrtc_version() -> Result<(i32, i32), LauncherError> {
    std::panic::catch_unwind(|| {
        // SAFETY: Only probes whether the compiler library can be loaded.
        if !unsafe { sys::is_culib_present() } {
            return Err(LauncherError::Compiler("NVRTC不可用".into()));
        }
        let (mut major, mut minor) = (0, 0);
        // SAFETY: The library is loaded; both out-pointers are valid locals.
        unsafe { sys::nvrtcVersion(&mut major, &mut minor) }
            .result()
            .map_err(|e| LauncherError::Compiler(format!("NVRTC版本：{e:?}")))?;
        Ok((major, minor))
    })
    .unwrap_or_else(|_| Err(LauncherError::Compiler("NVRTC加载异常".into())))
}

/// 把内存头文件按相对路径落到唯一临时目录；调用方负责清理。
pub(super) fn materialize_headers(headers: &[HeaderFile]) -> Result<PathBuf, LauncherError> {
    let base = std::env::temp_dir().join("mjwarp-launcher-headers");
    std::fs::create_dir_all(&base)
        .map_err(|error| LauncherError::Compiler(format!("头文件临时目录：{error}")))?;
    let directory = base.join(format!(
        "{}-{}",
        std::process::id(),
        HEADER_DIR_ID.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&directory)
        .map_err(|error| LauncherError::Compiler(format!("头文件临时目录：{error}")))?;
    let write = || -> Result<(), LauncherError> {
        for header in headers {
            let path = directory.join(header.name());
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| LauncherError::Compiler(format!("头文件目录：{error}")))?;
            }
            std::fs::write(&path, header.content().as_bytes())
                .map_err(|error| LauncherError::Compiler(format!("头文件落盘：{error}")))?;
        }
        Ok(())
    };
    if let Err(error) = write() {
        remove_headers_dir(&directory);
        return Err(error);
    }
    Ok(directory)
}

pub(super) fn remove_headers_dir(directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
}

/// 编译选项只取自配方，与L1现行路径逐项一致。
pub(super) fn nvrtc_compile(
    source: &NvrtcSource,
    recipe: &CompileRecipe,
) -> Result<String, LauncherError> {
    let header_dir = if source.headers.is_empty() {
        None
    } else {
        Some(materialize_headers(&source.headers)?)
    };
    let mut include_paths = Vec::with_capacity(source.include_paths.len() + 1);
    if let Some(directory) = &header_dir {
        include_paths.push(directory.to_string_lossy().into_owned());
    }
    include_paths.extend(source.include_paths.iter().cloned());
    let mut nvrtc_options = recipe.nvrtc_options();
    if !source.headers.is_empty() {
        // 上游头树含未标注执行空间的函数，NVRTC 需按设备端默认处理。
        nvrtc_options.push("--device-as-default-execution-space".to_owned());
    }
    let options = CompileOptions {
        fmad: Some(recipe.fmad()),
        use_fast_math: Some(recipe.fast_math()),
        options: nvrtc_options,
        include_paths,
        name: Some(format!("{}.cu", source.name)),
        ..Default::default()
    };
    let text = source.source.as_ref();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        compile_ptx_with_opts(text, options)
            .map(|ptx| ptx.to_src())
            .map_err(|e| LauncherError::Compiler(format!("{}: {e:?}", source.name)))
    }))
    .unwrap_or_else(|_| Err(LauncherError::Compiler("NVRTC编译异常".into())));
    if let Some(directory) = header_dir {
        remove_headers_dir(&directory);
    }
    result
}
