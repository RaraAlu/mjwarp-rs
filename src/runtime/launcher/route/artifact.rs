//! 路线产物与 MathDx LTO 管线。
//! 产物元数据记录路线、版本组合与各级摘要；`verify`复核键与内容一致。
//! 只经显式路线启用；默认路径不启用，不切换任何物理配方。

use serde::Serialize;

use super::{
    KernelSourceRoute, RouteKey, RouteOptIn, require_mathdx_nvrtc, require_route_enabled,
    route_error, sha256_hex,
};
use crate::runtime::launcher::{
    LauncherError, LinkInput, LinkInputKind, LtoirInput, MATHDX_VERSION, MathDx, NVJITLINK_VERSION,
    NvJitLink, SolverSpec, compile_ltoir, link_ltoir,
};

/// 产物元数据：路线、版本组合与各级摘要。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct KernelArtifactMetadata {
    pub route: KernelSourceRoute,
    pub key_sha256: String,
    pub source_sha256: String,
    pub header_sha256: String,
    pub arch: u32,
    pub nvrtc_version: (i32, i32),
    pub nvjitlink_version: (u32, u32),
    pub libmathdx_version: (i32, i32, i32),
    pub call_ltoir_bytes: usize,
    pub call_ltoir_sha256: String,
    pub solver_ltoir_bytes: usize,
    pub solver_ltoir_sha256: String,
    pub solver_fatbin_bytes: usize,
    pub solver_fatbin_sha256: String,
    pub cubin_bytes: usize,
    pub cubin_sha256: String,
}

/// 经显式路线构建的内核产物；`verify`可复核键与内容一致。
/// 字段仅 crate 内可见，外部只经 getter 与`verify`访问。
pub struct KernelArtifact {
    pub(crate) key: RouteKey,
    pub(crate) metadata: KernelArtifactMetadata,
    pub(crate) call_ltoir: Vec<u8>,
    pub(crate) solver_ltoir: Vec<u8>,
    pub(crate) solver_fatbin: Vec<u8>,
    pub(crate) cubin: Vec<u8>,
}

impl std::fmt::Debug for KernelArtifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KernelArtifact")
            .field("route", &self.key.route())
            .field("key_sha256", &self.metadata.key_sha256)
            .field("cubin_bytes", &self.cubin.len())
            .field("cubin_sha256", &self.metadata.cubin_sha256)
            .finish()
    }
}

impl KernelArtifact {
    pub fn key(&self) -> &RouteKey {
        &self.key
    }

    pub fn metadata(&self) -> &KernelArtifactMetadata {
        &self.metadata
    }

    pub fn call_ltoir(&self) -> &[u8] {
        &self.call_ltoir
    }

    pub fn solver_ltoir(&self) -> &[u8] {
        &self.solver_ltoir
    }

    pub fn solver_fatbin(&self) -> &[u8] {
        &self.solver_fatbin
    }

    pub fn cubin(&self) -> &[u8] {
        &self.cubin
    }

    /// 复核键摘要、各级产物摘要与 CUBIN ELF；任何不一致都显式报错。
    pub fn verify(&self) -> Result<(), LauncherError> {
        if self.metadata.route != self.key.route() {
            return Err(route_error("产物键与内容不一致：路线"));
        }
        if self.metadata.key_sha256 != self.key.sha256() {
            return Err(route_error("产物键与内容不一致：键摘要"));
        }
        if self.metadata.source_sha256 != self.key.source_sha256()
            || self.metadata.header_sha256 != self.key.header_sha256()
            || self.metadata.arch != self.key.arch()
        {
            return Err(route_error("产物键与内容不一致：键字段"));
        }
        let parts = [
            (
                "调用LTOIR",
                self.call_ltoir.as_slice(),
                &self.metadata.call_ltoir_sha256,
            ),
            (
                "MathDx LTOIR",
                self.solver_ltoir.as_slice(),
                &self.metadata.solver_ltoir_sha256,
            ),
            (
                "MathDx FATBIN",
                self.solver_fatbin.as_slice(),
                &self.metadata.solver_fatbin_sha256,
            ),
        ];
        for (name, bytes, recorded) in parts {
            if bytes.is_empty() {
                return Err(route_error(format!("空产物：{name}")));
            }
            if sha256_hex(bytes) != *recorded {
                return Err(route_error(format!("产物键与内容不一致：{name}摘要")));
            }
        }
        if self.cubin.is_empty() {
            return Err(route_error("空产物：CUBIN"));
        }
        if sha256_hex(&self.cubin) != self.metadata.cubin_sha256 {
            return Err(route_error("产物键与内容不一致：CUBIN摘要"));
        }
        check_cubin(&self.cubin)
    }
}

/// MathDx LTO 路线端到端入口：
/// 生成调用 LTOIR 与求解器 LTOIR/FATBIN，经 nvJitLink 链接为 CUBIN。
/// 路线默认不启用；未显式`RouteOptIn`即拒绝。
pub fn build_mathdx_lto_kernel(
    opt_in: RouteOptIn,
    mathdx: &MathDx,
    linker: &NvJitLink,
    call: &LtoirInput<'_>,
    solver: &SolverSpec<'_>,
) -> Result<KernelArtifact, LauncherError> {
    require_route_enabled(opt_in, KernelSourceRoute::MathDxLto)?;
    solver.validate()?;
    if mathdx.version() != MATHDX_VERSION {
        return Err(route_error(format!(
            "libmathdx版本不符：需要{}.{}.{}，实际{}.{}.{}",
            MATHDX_VERSION.0,
            MATHDX_VERSION.1,
            MATHDX_VERSION.2,
            mathdx.version().0,
            mathdx.version().1,
            mathdx.version().2
        )));
    }
    if linker.version() != NVJITLINK_VERSION {
        return Err(route_error(format!(
            "nvJitLink版本不符：需要{}.{}，实际{}.{}",
            NVJITLINK_VERSION.0,
            NVJITLINK_VERSION.1,
            linker.version().0,
            linker.version().1
        )));
    }
    let call_ltoir = compile_ltoir(call)?;
    require_mathdx_nvrtc(call_ltoir.nvrtc_version())?;
    let solver_lto = mathdx.generate_solver_lto(solver)?;
    let solver_ltoir = solver_lto.ltoir().to_vec();
    let solver_fatbin = solver_lto.fatbin().to_vec();
    let extras = [
        LinkInput {
            kind: LinkInputKind::Ltoir,
            name: "mathdx-solver.ltoir",
            bytes: &solver_ltoir,
        },
        LinkInput {
            kind: LinkInputKind::Fatbin,
            name: "mathdx-solver.fatbin",
            bytes: &solver_fatbin,
        },
    ];
    let linked = link_ltoir(linker, call.arch(), &call_ltoir, &extras)?;
    let nvrtc = call_ltoir.nvrtc_version();
    let call_sha = call_ltoir.sha256().to_owned();
    let call_bytes = call_ltoir.into_bytes();
    let key = RouteKey::for_mathdx_lto(call, solver, nvrtc, linker.version(), mathdx.version())?;
    let cubin_sha = linked.sha256().to_owned();
    let cubin = linked.into_cubin().into_bytes();
    let metadata = KernelArtifactMetadata {
        route: KernelSourceRoute::MathDxLto,
        key_sha256: key.sha256(),
        source_sha256: key.source_sha256().to_owned(),
        header_sha256: key.header_sha256().to_owned(),
        arch: key.arch(),
        nvrtc_version: nvrtc,
        nvjitlink_version: linker.version(),
        libmathdx_version: mathdx.version(),
        call_ltoir_bytes: call_bytes.len(),
        call_ltoir_sha256: call_sha,
        solver_ltoir_bytes: solver_ltoir.len(),
        solver_ltoir_sha256: solver_lto.ltoir_sha256().to_owned(),
        solver_fatbin_bytes: solver_fatbin.len(),
        solver_fatbin_sha256: solver_lto.fatbin_sha256().to_owned(),
        cubin_bytes: cubin.len(),
        cubin_sha256: cubin_sha,
    };
    let artifact = KernelArtifact {
        key,
        metadata,
        call_ltoir: call_bytes,
        solver_ltoir,
        solver_fatbin,
        cubin,
    };
    artifact.verify()?;
    Ok(artifact)
}

/// 产物须为非空64位小端 CUDA ELF。
fn check_cubin(bytes: &[u8]) -> Result<(), LauncherError> {
    if bytes.len() < 64 || bytes[..4] != *b"\x7fELF" {
        return Err(route_error("产物不是CUBIN ELF"));
    }
    if bytes[4] != 2 || bytes[5] != 1 {
        return Err(route_error("产物不是64位小端ELF"));
    }
    let machine = u16::from_le_bytes([bytes[18], bytes[19]]);
    if machine != 190 {
        return Err(route_error(format!("产物机器号不是CUDA：{machine}")));
    }
    Ok(())
}
