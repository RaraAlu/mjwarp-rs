//! nvJitLink 动态绑定（W13）。
//! 只链接 LTOIR 与 FATBIN，产出 CUBIN ELF。
//! 库版本必须等于工具链固定的 12.9，不回退。
//! 12.9.86 与冻结 Warp 1.15.0 内置链接器同版。
//! 每次链接独占句柄，同一实例内串行。
//! 评审见`docs/development-evidence/layers-w13.md`。

mod ffi;

use std::ffi::{CString, c_int};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::LauncherError;

/// 工具链固定的 nvJitLink 主次版本（组件 12.9.86）。
pub const NVJITLINK_VERSION: (u32, u32) = (12, 9);
/// 单个输入的字节上限。
pub const MAX_LINK_INPUT_BYTES: usize = 1 << 30;
/// 单次链接的输入数上限。
pub const MAX_LINK_INPUTS: usize = 64;

#[cfg(windows)]
const LIBRARY_FILE: &str = "nvJitLink_120_0.dll";
#[cfg(unix)]
const LIBRARY_FILE: &str = "libnvJitLink.so.12";

/// ELF机器号`EM_CUDA`。
const EM_CUDA: u16 = 190;

/// 链接输入类型；取值对照`nvJitLinkInputType`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkInputKind {
    /// `NVJITLINK_INPUT_LTOIR`。
    Ltoir,
    /// `NVJITLINK_INPUT_FATBIN`。
    Fatbin,
}

impl LinkInputKind {
    fn raw(self) -> c_int {
        match self {
            Self::Ltoir => 3,
            Self::Fatbin => 4,
        }
    }
}

/// 单项链接输入；字节只在调用期间借用。
#[derive(Clone, Copy, Debug)]
pub struct LinkInput<'a> {
    pub kind: LinkInputKind,
    pub name: &'a str,
    pub bytes: &'a [u8],
}

/// 链接产物与信息日志。
#[derive(Clone, Debug)]
pub struct LinkedCubin {
    bytes: Vec<u8>,
    info_log: String,
}

impl LinkedCubin {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub fn info_log(&self) -> &str {
        &self.info_log
    }
}

/// 已加载并核对版本的 nvJitLink。
/// 可跨线程共享；链接调用在实例内串行。
pub struct NvJitLink {
    api: ffi::Api,
    path: PathBuf,
    serial: Mutex<()>,
}

impl std::fmt::Debug for NvJitLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NvJitLink")
            .field("path", &self.path)
            .field("version", &self.api.version())
            .finish()
    }
}

impl NvJitLink {
    /// 从显式目录加载平台库文件。
    pub fn load_from_dir(directory: &Path) -> Result<Self, LauncherError> {
        Self::load_file(&directory.join(LIBRARY_FILE))
    }

    /// 从`CUDA_PATH/bin`加载；变量缺失时报错，不搜索其他位置。
    pub fn load_from_cuda_path() -> Result<Self, LauncherError> {
        let root = std::env::var_os("CUDA_PATH")
            .ok_or_else(|| LauncherError::Linker("未设置CUDA_PATH".into()))?;
        Self::load_from_dir(&PathBuf::from(root).join("bin"))
    }

    /// 按绝对路径加载指定库文件。
    pub fn load_file(path: &Path) -> Result<Self, LauncherError> {
        let api = ffi::Api::open(path)?;
        Ok(Self {
            api,
            path: path.to_path_buf(),
            serial: Mutex::new(()),
        })
    }

    pub fn version(&self) -> (u32, u32) {
        self.api.version()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 以`-dlto -arch=sm_<arch>`链接输入，返回 CUBIN ELF。
    pub fn link_cubin(
        &self,
        arch: u32,
        inputs: &[LinkInput<'_>],
    ) -> Result<LinkedCubin, LauncherError> {
        let options = link_options(arch)?;
        let names = validate_inputs(inputs)?;
        let option_ptrs: Vec<_> = options.iter().map(|option| option.as_ptr()).collect();
        let _serial = self
            .serial
            .lock()
            .map_err(|_| LauncherError::Linker("链接锁已中毒".into()))?;
        let mut session = self
            .api
            .create(&option_ptrs)
            .map_err(|status| LauncherError::Linker(format!("创建句柄失败：{status}")))?;
        for (input, name) in inputs.iter().zip(&names) {
            let status = session.add(input.kind.raw(), input.bytes, name.as_ptr());
            if status != ffi::SUCCESS {
                return Err(link_error("添加输入", input.name, status, &session));
            }
        }
        let status = session.complete();
        if status != ffi::SUCCESS {
            return Err(link_error("完成链接", "", status, &session));
        }
        let bytes = session
            .cubin()
            .map_err(|status| link_error("读取产物", "", status, &session))?;
        check_cubin(&bytes)?;
        Ok(LinkedCubin {
            bytes,
            info_log: session.info_log(),
        })
    }
}

fn link_error(
    stage: &str,
    input: &str,
    status: c_int,
    session: &ffi::Session<'_>,
) -> LauncherError {
    let log = session.error_log();
    LauncherError::Linker(format!("{stage}{input}失败：{status}：{log}"))
}

/// 版本须与工具链固定组件完全一致。
pub(crate) fn check_version(version: (u32, u32)) -> Result<(), LauncherError> {
    if version != NVJITLINK_VERSION {
        return Err(LauncherError::Linker(format!(
            "版本不符：需要{}.{}，实际{}.{}",
            NVJITLINK_VERSION.0, NVJITLINK_VERSION.1, version.0, version.1
        )));
    }
    Ok(())
}

/// 冻结选项：只开放目标架构。
pub(crate) fn link_options(arch: u32) -> Result<[CString; 2], LauncherError> {
    if !(50..=129).contains(&arch) {
        return Err(LauncherError::Linker(format!("架构超出范围：sm_{arch}")));
    }
    let arch = CString::new(format!("-arch=sm_{arch}"))
        .map_err(|_| LauncherError::Linker("架构选项含NUL".into()))?;
    let lto = CString::new("-dlto").map_err(|_| LauncherError::Linker("选项含NUL".into()))?;
    Ok([lto, arch])
}

/// 拒绝空列表、空输入、超限输入与非法名称。
pub(crate) fn validate_inputs(inputs: &[LinkInput<'_>]) -> Result<Vec<CString>, LauncherError> {
    if inputs.is_empty() {
        return Err(LauncherError::Linker("没有链接输入".into()));
    }
    if inputs.len() > MAX_LINK_INPUTS {
        return Err(LauncherError::Linker(format!(
            "链接输入过多：{}",
            inputs.len()
        )));
    }
    inputs
        .iter()
        .map(|input| {
            if input.bytes.is_empty() {
                return Err(LauncherError::Linker(format!("输入为空：{}", input.name)));
            }
            if input.bytes.len() > MAX_LINK_INPUT_BYTES {
                return Err(LauncherError::Linker(format!(
                    "输入超限：{}：{}字节",
                    input.name,
                    input.bytes.len()
                )));
            }
            if input.name.is_empty() {
                return Err(LauncherError::Linker("输入名称为空".into()));
            }
            CString::new(input.name)
                .map_err(|_| LauncherError::Linker(format!("输入名称含NUL：{}", input.name)))
        })
        .collect()
}

/// 产物须为非空的64位小端 CUDA ELF。
pub(crate) fn check_cubin(bytes: &[u8]) -> Result<(), LauncherError> {
    if bytes.is_empty() {
        return Err(LauncherError::Linker("链接产物为空".into()));
    }
    if bytes.len() < 64 || bytes[..4] != *b"\x7fELF" {
        return Err(LauncherError::Linker("产物不是ELF".into()));
    }
    if bytes[4] != 2 || bytes[5] != 1 {
        return Err(LauncherError::Linker("产物不是64位小端ELF".into()));
    }
    let machine = u16::from_le_bytes([bytes[18], bytes[19]]);
    if machine != EM_CUDA {
        return Err(LauncherError::Linker(format!(
            "产物机器号不是CUDA：{machine}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
