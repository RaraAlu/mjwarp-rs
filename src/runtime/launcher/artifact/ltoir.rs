//! 产品编译链 LTOIR 输出接口（W13 后续）。
//! 冻结 13 项 NVRTC 选项与顺序；本接口只输出 LTOIR。
//! 端到端复用 nvJitLink 绑定：程序 LTOIR + 可选 LTOIR/FATBIN → CUBIN。
//! 不接 MathDx SDK，不调用 warp.dll，不启动 Python。
//! 既有 PTX/CUBIN 编译路径保持不变。
//! 评审与绑定见`docs/development-evidence/layers-w13.md`与`jitlink.rs`。

use std::ffi::{CString, c_char};

use cudarc::nvrtc::{result as nvrtc, sys as nvrtc_sys};
use sha2::{Digest, Sha256};

use super::nvrtc::{materialize_headers, nvrtc_version, remove_headers_dir};
use super::{HeaderFile, LauncherError};
use crate::runtime::launcher::jitlink::{LinkInput, LinkInputKind, LinkedCubin, NvJitLink};

/// 源码字节上限；与既有产物键的文本上限一致。
pub const MAX_LTOIR_SOURCE_BYTES: usize = super::MAX_TEXT_BYTES;
/// 目标架构下限（sm_50）。
pub const MIN_ARCH: u32 = 50;
/// 目标架构上限（sm_129）。
pub const MAX_ARCH: u32 = 129;
/// 冻结选项数；不含随头文件追加的`--include-path`。
pub const LTOIR_OPTION_COUNT: usize = 13;

/// LTOIR 编译输入。头文件与 include 目录只影响本次编译。
pub struct LtoirInput<'a> {
    name: &'a str,
    source: &'a str,
    headers: &'a [HeaderFile],
    include_paths: &'a [String],
    arch: u32,
}

impl<'a> LtoirInput<'a> {
    /// 无头文件、无外部 include 目录的输入。
    pub fn new(name: &'a str, source: &'a str, arch: u32) -> Result<Self, LauncherError> {
        Self::with_headers(name, source, &[], &[], arch)
    }

    /// 带头文件集合与 include 目录的输入；两类路径都进入本次编译选项。
    pub fn with_headers(
        name: &'a str,
        source: &'a str,
        headers: &'a [HeaderFile],
        include_paths: &'a [String],
        arch: u32,
    ) -> Result<Self, LauncherError> {
        if !super::valid_name(name) {
            return Err(LauncherError::InvalidSource("源名为空、过长或含非法字符"));
        }
        if !super::valid_text(source) {
            return Err(LauncherError::InvalidSource("源码为空、过大或含NUL"));
        }
        if let Some(path) = include_paths
            .iter()
            .find(|path| path.is_empty() || path.len() > 1024 || path.contains('\0'))
        {
            return Err(LauncherError::Compiler(format!("include路径非法：{path}")));
        }
        if !(MIN_ARCH..=MAX_ARCH).contains(&arch) {
            return Err(LauncherError::Compiler(format!(
                "架构超出范围：{arch}，要求{MIN_ARCH}–{MAX_ARCH}"
            )));
        }
        Ok(Self {
            name,
            source,
            headers,
            include_paths,
            arch,
        })
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn source(&self) -> &'a str {
        self.source
    }

    pub fn headers(&self) -> &'a [HeaderFile] {
        self.headers
    }

    pub fn include_paths(&self) -> &'a [String] {
        self.include_paths
    }

    pub fn arch(&self) -> u32 {
        self.arch
    }
}

/// LTOIR 产物：字节、摘要与实际 NVRTC 版本。
pub struct Ltoir {
    name: String,
    bytes: Vec<u8>,
    sha256: String,
    nvrtc_version: (i32, i32),
}

impl Ltoir {
    /// 程序名；作为链接输入名使用。
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    /// 实际加载的 NVRTC 主次版本，不按工具链写死。
    pub fn nvrtc_version(&self) -> (i32, i32) {
        self.nvrtc_version
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

impl std::fmt::Debug for Ltoir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ltoir")
            .field("name", &self.name)
            .field("bytes", &self.bytes.len())
            .field("sha256", &self.sha256)
            .field("nvrtc_version", &self.nvrtc_version)
            .finish()
    }
}

/// 端到端链接产物：CUBIN 与摘要。
pub struct LtoirCubin {
    cubin: LinkedCubin,
    sha256: String,
}

impl LtoirCubin {
    pub fn bytes(&self) -> &[u8] {
        self.cubin.bytes()
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    pub fn info_log(&self) -> &str {
        self.cubin.info_log()
    }

    pub fn into_cubin(self) -> LinkedCubin {
        self.cubin
    }
}

impl std::fmt::Debug for LtoirCubin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LtoirCubin")
            .field("bytes", &self.cubin.bytes().len())
            .field("sha256", &self.sha256)
            .finish()
    }
}

/// 编译 CUDA 源为 LTOIR。
/// 每次调用创建独立 NVRTC 程序并在返回前销毁；可跨线程并发调用。
/// NVRTC 不可用或编译失败都显式返回错误，不静默回退。
pub fn compile_ltoir(input: &LtoirInput<'_>) -> Result<Ltoir, LauncherError> {
    // SAFETY: 只探测动态库是否可加载，不创建或编译程序。
    ensure_nvrtc(unsafe { nvrtc_sys::is_culib_present() })?;
    let version = nvrtc_version()?;
    let header_dir = if input.headers.is_empty() {
        None
    } else {
        Some(materialize_headers(input.headers)?)
    };
    let mut options = ltoir_options(input.arch)?;
    if let Some(directory) = &header_dir {
        options.push(format!("--include-path={}", directory.display()));
    }
    options.extend(
        input
            .include_paths
            .iter()
            .map(|path| format!("--include-path={path}")),
    );
    let result = compile_to_ltoir(input.name, input.source, &options);
    if let Some(directory) = header_dir {
        remove_headers_dir(&directory);
    }
    let bytes = result?;
    Ok(Ltoir {
        name: input.name.to_owned(),
        sha256: digest(&bytes),
        bytes,
        nvrtc_version: version,
    })
}

/// 把程序 LTOIR 与附加输入链接为 CUBIN。
/// 顺序固定为：程序 LTOIR 在前，附加输入按调用方给定顺序；
/// 附加输入可放 LTOIR 或 FATBIN。同一输入两次产物摘要一致。
pub fn link_ltoir(
    linker: &NvJitLink,
    arch: u32,
    program: &Ltoir,
    extra: &[LinkInput<'_>],
) -> Result<LtoirCubin, LauncherError> {
    let mut inputs: Vec<LinkInput<'_>> = Vec::with_capacity(extra.len() + 1);
    inputs.push(LinkInput {
        kind: LinkInputKind::Ltoir,
        name: program.name(),
        bytes: program.bytes(),
    });
    inputs.extend_from_slice(extra);
    let cubin = linker.link_cubin(arch, &inputs)?;
    let sha256 = digest(cubin.bytes());
    Ok(LtoirCubin { cubin, sha256 })
}

/// 冻结的 13 项 NVRTC 选项，顺序固定；目标架构只替换首项。
pub fn ltoir_options(arch: u32) -> Result<Vec<String>, LauncherError> {
    if !(MIN_ARCH..=MAX_ARCH).contains(&arch) {
        return Err(LauncherError::Compiler(format!(
            "架构超出范围：{arch}，要求{MIN_ARCH}–{MAX_ARCH}"
        )));
    }
    let options = [
        format!("--gpu-architecture=sm_{arch}"),
        "--std=c++17".to_owned(),
        "--Ofast-compile=0".to_owned(),
        "--define-macro=NDEBUG".to_owned(),
        "--undefine-macro=WP_VERIFY_FP".to_owned(),
        "--define-macro=WP_ENABLE_MATHDX=1".to_owned(),
        "--fmad=true".to_owned(),
        "--device-as-default-execution-space".to_owned(),
        "--extra-device-vectorization".to_owned(),
        "--restrict".to_owned(),
        "--diag-suppress=177,550".to_owned(),
        "-dlto".to_owned(),
        "--relocatable-device-code=true".to_owned(),
    ];
    debug_assert_eq!(options.len(), LTOIR_OPTION_COUNT);
    Ok(options.into())
}

/// NVRTC 不可用时显式报错；单独成函数便于覆盖两条分支。
pub(crate) fn ensure_nvrtc(available: bool) -> Result<(), LauncherError> {
    if available {
        Ok(())
    } else {
        Err(LauncherError::Compiler("NVRTC不可用".into()))
    }
}

/// 创建、编译、取字节；程序句柄由守卫在返回前销毁。
fn compile_to_ltoir(
    name: &str,
    source: &str,
    options: &[String],
) -> Result<Vec<u8>, LauncherError> {
    let source_c = CString::new(source).map_err(|_| LauncherError::Compiler("源码含NUL".into()))?;
    let name_c = CString::new(name).map_err(|_| LauncherError::Compiler("源名含NUL".into()))?;
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let program = nvrtc::create_program(&source_c, Some(&name_c))
            .map_err(|error| LauncherError::Compiler(format!("NVRTC创建程序：{error}")))?;
        let guard = ProgramGuard(program);
        // SAFETY: `program` 来自`create_program`，守卫保证返回前唯一销毁。
        unsafe { nvrtc::compile_program(guard.0, options) }.map_err(|error| {
            // SAFETY: 程序尚未销毁，取日志是只读查询。
            let log = unsafe { nvrtc::get_program_log(guard.0) }.unwrap_or_default();
            let log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
            LauncherError::Compiler(format!(
                "NVRTC编译失败：{error}：{}",
                String::from_utf8_lossy(&log).trim()
            ))
        })?;
        let mut size = 0usize;
        // SAFETY: 程序已编译成功，出参指向本地变量。
        unsafe { nvrtc_sys::nvrtcGetLTOIRSize(guard.0, &mut size) }
            .result()
            .map_err(|error| LauncherError::Compiler(format!("读取LTOIR大小：{error}")))?;
        if size == 0 {
            return Err(LauncherError::Compiler("LTOIR产物为空".into()));
        }
        let mut bytes = vec![0u8; size];
        // SAFETY: 缓冲区按上一步返回的精确大小分配。
        unsafe { nvrtc_sys::nvrtcGetLTOIR(guard.0, bytes.as_mut_ptr().cast::<c_char>()) }
            .result()
            .map_err(|error| LauncherError::Compiler(format!("读取LTOIR：{error}")))?;
        Ok(bytes)
    }))
    .map_err(|_| LauncherError::Compiler("NVRTC调用异常".into()))?
}

/// NVRTC 程序句柄守卫；所有返回路径都销毁。
struct ProgramGuard(nvrtc_sys::nvrtcProgram);

impl Drop for ProgramGuard {
    fn drop(&mut self) {
        // SAFETY: 句柄只由本守卫持有，且在`create_program`之后调用一次。
        unsafe {
            let _ = nvrtc::destroy_program(self.0);
        }
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests;
