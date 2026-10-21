//! NVRTC 版本探测与编译封装。

use std::sync::OnceLock;

use cudarc::nvrtc::{CompileOptions, compile_ptx_with_opts, sys};

use super::*;

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

/// 编译选项只取自配方，与L1现行路径逐项一致。
pub(super) fn nvrtc_compile(
    source: &NvrtcSource,
    recipe: &CompileRecipe,
) -> Result<String, LauncherError> {
    if !source.headers.is_empty() {
        return Err(LauncherError::Unsupported("内存头文件编译待W15"));
    }
    let options = CompileOptions {
        fmad: Some(recipe.fmad()),
        use_fast_math: Some(recipe.fast_math()),
        options: recipe.nvrtc_options(),
        name: Some(format!("{}.cu", source.name)),
        ..Default::default()
    };
    let text = source.source.as_ref();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        compile_ptx_with_opts(text, options)
            .map(|ptx| ptx.to_src())
            .map_err(|e| LauncherError::Compiler(format!("{}: {e:?}", source.name)))
    }))
    .unwrap_or_else(|_| Err(LauncherError::Compiler("NVRTC编译异常".into())))
}
