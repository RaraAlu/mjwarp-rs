//! 内核来源路线标准化与 MathDx LTO 编译链接口。
//! 四类路线：手写 CUDA、冻结 PTX、W15 上游源码、MathDx LTO。
//! 产物键纳入源/头文件摘要、选项与顺序、arch 与三级工具版本；同输入稳定。
//! MathDx LTO 只经显式路线启用，默认路径不启用，不切换任何物理配方。
//! 既有 PTX/cubin 编译路径、内核字符串、容差与 f32 均不改动。

use serde::Serialize;
use sha2::{Digest, Sha256};

use super::{
    ArchTarget, CompileRecipe, FrozenPtx, HeaderFile, LauncherError, LtoirInput, NvrtcSource,
    SolverSpec, ltoir_options,
};

/// MathDx LTO 路线要求的 NVRTC 主次版本。
pub const MATHDX_NVRTC_VERSION: (i32, i32) = (12, 9);

/// 内核来源路线。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum KernelSourceRoute {
    /// 手写 CUDA 源码经 NVRTC 编译。
    HandWrittenCuda,
    /// 已冻结的 PTX 文本。
    FrozenPtx,
    /// W15 原样复制的上游源码树经 NVRTC 编译。
    UpstreamSource,
    /// libmathdx 描述符生成的 LTOIR/FATBIN，经 nvJitLink 链接。
    MathDxLto,
}

impl KernelSourceRoute {
    pub const ALL: [Self; 4] = [
        Self::HandWrittenCuda,
        Self::FrozenPtx,
        Self::UpstreamSource,
        Self::MathDxLto,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HandWrittenCuda => "HandWrittenCuda",
            Self::FrozenPtx => "FrozenPtx",
            Self::UpstreamSource => "UpstreamSource",
            Self::MathDxLto => "MathDxLto",
        }
    }

    /// 新路线默认不启用；须显式`RouteOptIn`。
    pub const fn enabled_by_default(self) -> bool {
        false
    }
}

/// 路线显式启用凭证；默认为未启用。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RouteOptIn {
    route: Option<KernelSourceRoute>,
}

impl RouteOptIn {
    pub const fn none() -> Self {
        Self { route: None }
    }

    pub const fn enable(route: KernelSourceRoute) -> Self {
        Self { route: Some(route) }
    }

    pub const fn route(self) -> Option<KernelSourceRoute> {
        self.route
    }
}

/// 路线未启用或启用了其他路线时显式报错。
pub fn require_route_enabled(
    opt_in: RouteOptIn,
    expected: KernelSourceRoute,
) -> Result<(), LauncherError> {
    match opt_in.route() {
        Some(route) if route == expected => Ok(()),
        Some(route) => Err(route_error(format!(
            "路线不符：已启用{}，需要{}",
            route.as_str(),
            expected.as_str()
        ))),
        None => Err(route_error(format!(
            "路线未启用：{}（新路线默认不启用，须显式 RouteOptIn::enable）",
            expected.as_str()
        ))),
    }
}

/// MathDx LTO 路线要求的 NVRTC 版本；不符即拒绝，不回退。
pub fn require_mathdx_nvrtc(actual: (i32, i32)) -> Result<(), LauncherError> {
    if actual != MATHDX_NVRTC_VERSION {
        return Err(route_error(format!(
            "NVRTC版本不符：MathDx路线需要{}.{}，实际{}.{}；不回退",
            MATHDX_NVRTC_VERSION.0, MATHDX_NVRTC_VERSION.1, actual.0, actual.1
        )));
    }
    Ok(())
}

/// 确定性产物键；任一因子变化键即变化。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RouteKey {
    route: KernelSourceRoute,
    source_sha256: String,
    header_sha256: String,
    include_paths: Vec<String>,
    options: Vec<String>,
    arch: u32,
    recipe_fingerprint: Option<String>,
    solver_fingerprint: Option<String>,
    nvrtc: Option<(i32, i32)>,
    nvjitlink: Option<(u32, u32)>,
    libmathdx: Option<(i32, i32, i32)>,
}

impl RouteKey {
    /// 通用构造；各路线专用构造函数见下。
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        route: KernelSourceRoute,
        source_sha256: String,
        header_sha256: String,
        include_paths: Vec<String>,
        options: Vec<String>,
        arch: u32,
        recipe_fingerprint: Option<String>,
        solver_fingerprint: Option<String>,
        nvrtc: Option<(i32, i32)>,
        nvjitlink: Option<(u32, u32)>,
        libmathdx: Option<(i32, i32, i32)>,
    ) -> Result<Self, LauncherError> {
        if source_sha256.len() != 64 || !source_sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(route_error("源摘要非法"));
        }
        if header_sha256.len() != 64 || !header_sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(route_error("头文件摘要非法"));
        }
        Ok(Self {
            route,
            source_sha256,
            header_sha256,
            include_paths,
            options,
            arch,
            recipe_fingerprint,
            solver_fingerprint,
            nvrtc,
            nvjitlink,
            libmathdx,
        })
    }

    /// 手写 CUDA 与 W15 上游源码路线：键取自`NvrtcSource`与`CompileRecipe`。
    pub fn for_cuda_route(
        route: KernelSourceRoute,
        source: &NvrtcSource,
        recipe: &CompileRecipe,
        nvrtc: (i32, i32),
    ) -> Result<Self, LauncherError> {
        if !matches!(
            route,
            KernelSourceRoute::HandWrittenCuda | KernelSourceRoute::UpstreamSource
        ) {
            return Err(route_error(format!(
                "路线{}不接受 CUDA 源构造",
                route.as_str()
            )));
        }
        recipe.validate()?;
        let ArchTarget::Compute(arch) = recipe.arch();
        Self::new(
            route,
            sha256_hex(source.source().as_bytes()),
            headers_fingerprint(source.headers()),
            source.include_paths().to_vec(),
            recipe.nvrtc_options(),
            arch,
            Some(recipe.fingerprint()),
            None,
            Some(nvrtc),
            None,
            None,
        )
    }

    /// 冻结 PTX 路线：键取冻结文本摘要；无 NVRTC/nvJitLink/libmathdx 版本。
    pub fn for_frozen_ptx(frozen: &FrozenPtx) -> Result<Self, LauncherError> {
        Self::new(
            KernelSourceRoute::FrozenPtx,
            frozen.sha256().to_owned(),
            sha256_hex(&[]),
            Vec::new(),
            Vec::new(),
            0,
            None,
            None,
            None,
            None,
            None,
        )
    }

    /// MathDx LTO 路线：源=调用源，头文件=调用源头文件，选项=冻结 LTOIR 选项+include 路径。
    pub fn for_mathdx_lto(
        call: &LtoirInput<'_>,
        solver: &SolverSpec<'_>,
        nvrtc: (i32, i32),
        nvjitlink: (u32, u32),
        libmathdx: (i32, i32, i32),
    ) -> Result<Self, LauncherError> {
        solver.validate()?;
        if call.arch() != solver.arch {
            return Err(route_error(format!(
                "架构不一致：调用源sm_{}，求解器sm_{}",
                call.arch(),
                solver.arch
            )));
        }
        let mut options = ltoir_options(call.arch())?;
        options.extend(
            call.include_paths()
                .iter()
                .map(|path| format!("--include-path={path}")),
        );
        Self::new(
            KernelSourceRoute::MathDxLto,
            sha256_hex(call.source().as_bytes()),
            headers_fingerprint(call.headers()),
            call.include_paths().to_vec(),
            options,
            call.arch(),
            None,
            Some(solver_fingerprint(solver)),
            Some(nvrtc),
            Some(nvjitlink),
            Some(libmathdx),
        )
    }

    pub fn sha256(&self) -> String {
        sha256_hex(&serde_json::to_vec(self).expect("route key serializes"))
    }

    pub fn route(&self) -> KernelSourceRoute {
        self.route
    }

    pub fn source_sha256(&self) -> &str {
        &self.source_sha256
    }

    pub fn header_sha256(&self) -> &str {
        &self.header_sha256
    }

    pub fn options(&self) -> &[String] {
        &self.options
    }

    pub fn arch(&self) -> u32 {
        self.arch
    }

    pub fn nvrtc(&self) -> Option<(i32, i32)> {
        self.nvrtc
    }

    pub fn nvjitlink(&self) -> Option<(u32, u32)> {
        self.nvjitlink
    }

    pub fn libmathdx(&self) -> Option<(i32, i32, i32)> {
        self.libmathdx
    }
}

mod artifact;
pub use artifact::{KernelArtifact, KernelArtifactMetadata, build_mathdx_lto_kernel};

fn route_error(reason: impl Into<String>) -> LauncherError {
    LauncherError::Compiler(format!("内核来源路线：{}", reason.into()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn headers_fingerprint(headers: &[HeaderFile]) -> String {
    let mut hasher = Sha256::new();
    for header in headers {
        hasher.update(header.name().as_bytes());
        hasher.update([0u8]);
        hasher.update(header.content().as_bytes());
        hasher.update([0u8]);
    }
    format!("{:x}", hasher.finalize())
}

fn solver_fingerprint(spec: &SolverSpec<'_>) -> String {
    format!(
        "mathdx-solver-v1;m={};n={};k={};function={};side={};diag={};precision={};arrangement={};{};fill={};threads={};arch={};symbol={}",
        spec.m,
        spec.n,
        spec.k,
        spec.function,
        spec.side,
        spec.diag,
        spec.precision,
        spec.arrangement_a,
        spec.arrangement_b,
        spec.fill_mode,
        spec.num_threads,
        spec.arch,
        spec.symbol
    )
}

#[cfg(test)]
mod tests;
