//! MathDx（libmathdx 0.3.1）动态绑定：cuSOLVERDx 描述符 API（W13 后续）。
//! 只按绝对路径加载 `mathdx64_0.dll`，版本固定 0.3.1。
//! 生成 7 维 POTRF 的 LTOIR 与 universal FATBIN。
//! 描述符调用顺序逐项对照 Warp 1.15.0 `warp.cu` 的 `wp_cuda_compile_solver`。
//! 注意：libmathdx 会把进程工作目录写进 LTOIR；同参数换工作目录摘要即不同。
//! 冻结证据在 `C:\Rust\mjwarp-rs` 下生成，逐字节对照需在该工作目录运行。
//! 不调用 warp.dll，不启动 Python；不新增 C/C++ 组件。

use std::ffi::{CString, c_int, c_longlong, c_void};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::LauncherError;

/// 工具链固定的 libmathdx 主次补丁版本。
pub const MATHDX_VERSION: (i32, i32, i32) = (0, 3, 1);
/// 描述符句柄上限内的尺寸/线程范围。
pub const MAX_DIM: i32 = 4096;
/// 线程数上限。
pub const MAX_THREADS: i32 = 1024;
/// 合法SM下限。
pub const MIN_ARCH: u32 = 50;
/// 合法SM上限。
pub const MAX_ARCH: u32 = 129;

#[cfg(windows)]
const LIBRARY_FILE: &str = "mathdx64_0.dll";
#[cfg(unix)]
const LIBRARY_FILE: &str = "libmathdx.so.0";

const SUCCESS: c_int = 0;

/// 对应`cusolverdxOperatorType`。
const OP_SIZE: c_int = 0;
const OP_TYPE: c_int = 1;
const OP_PRECISION: c_int = 2;
const OP_SM: c_int = 3;
const OP_EXECUTION: c_int = 4;
const OP_BLOCK_DIM: c_int = 5;
const OP_API: c_int = 6;
const OP_FUNCTION: c_int = 7;
const OP_ARRANGEMENT: c_int = 8;
const OP_FILL_MODE: c_int = 9;
const OP_SIDE: c_int = 10;
const OP_DIAG: c_int = 11;

/// 对应`commondxOption::COMMONDX_OPTION_SYMBOL_NAME`。
const OPTION_SYMBOL_NAME: c_int = 0;
/// 对应`cusolverdxType::CUSOLVERDX_TYPE_REAL`与`cusolverdxApi::CUSOLVERDX_API_SMEM`。
const TYPE_REAL: c_longlong = 0;
const API_SMEM: c_longlong = 0;
/// 对应`commondxExecution::COMMONDX_EXECUTION_BLOCK`。
const EXECUTION_BLOCK: c_longlong = 1;

mod deps;
mod ffi;

/// 求解器规格；数值直接对应冻结证据的 contract。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SolverSpec<'a> {
    pub m: i32,
    pub n: i32,
    pub k: i32,
    pub function: i32,
    /// 负值表示不设置该运算符。
    pub side: i32,
    /// 负值表示不设置该运算符。
    pub diag: i32,
    pub precision: i32,
    pub arrangement_a: i32,
    pub arrangement_b: i32,
    pub fill_mode: i32,
    pub num_threads: i32,
    pub arch: u32,
    pub symbol: &'a str,
}

impl<'a> SolverSpec<'a> {
    /// 冻结证据的 7 维 POTRF（M=N=7、K=1、function=POTRF、F32、行主序、UPPER、32线程、sm_89）。
    pub const fn potrf_7d_sm89() -> Self {
        Self {
            m: 7,
            n: 7,
            k: 1,
            function: 2,
            side: -1,
            diag: -1,
            precision: 5,
            arrangement_a: 1,
            arrangement_b: 1,
            fill_mode: 0,
            num_threads: 32,
            arch: 89,
            symbol: "potrf_7_7_1_89_32_1_1_5_x_x_0",
        }
    }

    /// 拒绝非法形状、线程、枚举、SM 与符号。
    pub fn validate(&self) -> Result<(), LauncherError> {
        for (name, value) in [("M", self.m), ("N", self.n), ("K", self.k)] {
            if !(1..=MAX_DIM).contains(&value) {
                return Err(invalid(&format!("{name}超出范围：{value}")));
            }
        }
        if self.num_threads < 1 || self.num_threads > MAX_THREADS || self.num_threads % 32 != 0 {
            return Err(invalid(&format!("线程数非法：{}", self.num_threads)));
        }
        if !(MIN_ARCH..=MAX_ARCH).contains(&self.arch) {
            return Err(invalid(&format!("架构超出范围：sm_{}", self.arch)));
        }
        if !(0..=14).contains(&self.function) {
            return Err(invalid(&format!("function非法：{}", self.function)));
        }
        if !(0..=14).contains(&self.precision) {
            return Err(invalid(&format!("precision非法：{}", self.precision)));
        }
        for (name, value) in [
            ("arrangement_a", self.arrangement_a),
            ("arrangement_b", self.arrangement_b),
        ] {
            if !(0..=1).contains(&value) {
                return Err(invalid(&format!("{name}非法：{value}")));
            }
        }
        if !(0..=1).contains(&self.fill_mode) {
            return Err(invalid(&format!("fill_mode非法：{}", self.fill_mode)));
        }
        for (name, value) in [("side", self.side), ("diag", self.diag)] {
            if !(-1..=1).contains(&value) {
                return Err(invalid(&format!("{name}非法：{value}")));
            }
        }
        if !valid_symbol(self.symbol) {
            return Err(invalid("符号名为空、过长或含非法字符"));
        }
        Ok(())
    }
}

fn invalid(reason: &str) -> LauncherError {
    LauncherError::Compiler(format!("MathDx求解器规格无效：{reason}"))
}

/// 符号名限定ASCII字母数字与`_-.`，长度1–128。
fn valid_symbol(symbol: &str) -> bool {
    !symbol.is_empty()
        && symbol.len() <= 128
        && symbol
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}

/// 求解器 LTOIR/FATBIN 产物与摘要。
pub struct SolverLto {
    ltoir: Vec<u8>,
    fatbin: Vec<u8>,
    ltoir_sha256: String,
    fatbin_sha256: String,
    mathdx_version: (i32, i32, i32),
}

impl SolverLto {
    pub fn ltoir(&self) -> &[u8] {
        &self.ltoir
    }

    pub fn fatbin(&self) -> &[u8] {
        &self.fatbin
    }

    pub fn ltoir_sha256(&self) -> &str {
        &self.ltoir_sha256
    }

    pub fn fatbin_sha256(&self) -> &str {
        &self.fatbin_sha256
    }

    pub fn mathdx_version(&self) -> (i32, i32, i32) {
        self.mathdx_version
    }
}

impl std::fmt::Debug for SolverLto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SolverLto")
            .field("ltoir_bytes", &self.ltoir.len())
            .field("fatbin_bytes", &self.fatbin.len())
            .field("ltoir_sha256", &self.ltoir_sha256)
            .field("fatbin_sha256", &self.fatbin_sha256)
            .field("mathdx_version", &self.mathdx_version)
            .finish()
    }
}

/// 已加载并核对版本的 libmathdx。
/// 可跨线程共享；生成调用在实例内串行。
pub struct MathDx {
    api: ffi::Api,
    path: PathBuf,
    serial: Mutex<()>,
    // 静态依赖（Windows 上为 nvrtc64_120_0.dll）须比函数指针活得更久。
    _dependencies: Vec<libloading::Library>,
}

impl std::fmt::Debug for MathDx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MathDx")
            .field("path", &self.path)
            .field("version", &self.api.version())
            .finish()
    }
}

impl MathDx {
    /// 从显式目录加载平台库文件（依赖解析见`load_file`）。
    pub fn load_from_dir(directory: &Path) -> Result<Self, LauncherError> {
        Self::load_file(&directory.join(LIBRARY_FILE))
    }

    /// 从显式目录加载库，并从显式 NVRTC bin 目录预加载 pinned 12.9.86 依赖。
    /// 缺 pinned 组件时显式报错，不回退到其他 NVRTC。
    pub fn load_from_dir_with_nvrtc(
        directory: &Path,
        nvrtc_bin_directory: &Path,
    ) -> Result<Self, LauncherError> {
        Self::load_file_with_nvrtc(&directory.join(LIBRARY_FILE), nvrtc_bin_directory)
    }

    /// 从`MATHDX_PATH`安装根加载`bin`目录下的库文件；变量缺失时报错。
    /// 设置`MATHDX_NVRTC_DIR`时按其显式预加载 pinned NVRTC。
    pub fn load_from_env() -> Result<Self, LauncherError> {
        let root = std::env::var_os("MATHDX_PATH")
            .ok_or_else(|| LauncherError::Linker("未设置MATHDX_PATH".into()))?;
        Self::load_from_dir(&PathBuf::from(root).join("bin"))
    }

    /// 按绝对路径加载指定库文件。
    /// 依赖预加载顺序：`MATHDX_NVRTC_DIR`（显式）→ 库同目录 → `CUDA_PATH/bin`。
    /// 显式 pinned 路径请用`load_file_with_nvrtc`。
    pub fn load_file(path: &Path) -> Result<Self, LauncherError> {
        let dependencies = deps::preload_dependencies(path)?;
        Self::finish(path, dependencies)
    }

    /// 按绝对路径加载，强制从显式 NVRTC bin 目录预加载 pinned 12.9.86 依赖。
    /// 缺组件时显式报错，不静默回退。
    pub fn load_file_with_nvrtc(
        path: &Path,
        nvrtc_bin_directory: &Path,
    ) -> Result<Self, LauncherError> {
        let dependencies = deps::preload_nvrtc(nvrtc_bin_directory)?;
        Self::finish(path, dependencies)
    }

    fn finish(path: &Path, dependencies: Vec<libloading::Library>) -> Result<Self, LauncherError> {
        let api = ffi::Api::open(path)?;
        Ok(Self {
            api,
            path: path.to_path_buf(),
            serial: Mutex::new(()),
            _dependencies: dependencies,
        })
    }

    pub fn version(&self) -> (i32, i32, i32) {
        self.api.version()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 按冻结参数生成 LTOIR 与 universal FATBIN。
    pub fn generate_solver_lto(&self, spec: &SolverSpec<'_>) -> Result<SolverLto, LauncherError> {
        spec.validate()?;
        let symbol = CString::new(spec.symbol).map_err(|_| invalid("符号名含NUL"))?;
        let _serial = self
            .serial
            .lock()
            .map_err(|_| LauncherError::Compiler("MathDx锁已中毒".into()))?;
        let descriptor = self
            .api
            .create_descriptor()
            .map_err(|status| self.status_error("创建描述符", status))?;
        let mut size = [spec.m, spec.n, spec.k].map(c_longlong::from);
        self.set_operator_int64s(&descriptor, OP_SIZE, &mut size, "设置尺寸")?;
        let mut block = [spec.num_threads, 1, 1].map(c_longlong::from);
        self.set_operator_int64s(&descriptor, OP_BLOCK_DIM, &mut block, "设置块大小")?;
        self.set_operator_int64(&descriptor, OP_TYPE, TYPE_REAL, "设置类型")?;
        self.set_operator_int64(&descriptor, OP_API, API_SMEM, "设置API")?;
        self.set_operator_int64(
            &descriptor,
            OP_FUNCTION,
            c_longlong::from(spec.function),
            "设置功能",
        )?;
        if spec.side >= 0 {
            self.set_operator_int64(
                &descriptor,
                OP_SIDE,
                c_longlong::from(spec.side),
                "设置side",
            )?;
        }
        if spec.diag >= 0 {
            self.set_operator_int64(
                &descriptor,
                OP_DIAG,
                c_longlong::from(spec.diag),
                "设置diag",
            )?;
        }
        self.set_operator_int64(&descriptor, OP_EXECUTION, EXECUTION_BLOCK, "设置执行模式")?;
        self.set_operator_int64(
            &descriptor,
            OP_PRECISION,
            c_longlong::from(spec.precision),
            "设置精度",
        )?;
        let mut arrangement = [spec.arrangement_a, spec.arrangement_b].map(c_longlong::from);
        self.set_operator_int64s(&descriptor, OP_ARRANGEMENT, &mut arrangement, "设置排列")?;
        self.set_operator_int64(
            &descriptor,
            OP_FILL_MODE,
            c_longlong::from(spec.fill_mode),
            "设置填充模式",
        )?;
        self.set_operator_int64(
            &descriptor,
            OP_SM,
            c_longlong::from(spec.arch) * 10,
            "设置SM",
        )?;
        let status =
            self.api
                .set_option_str(descriptor.handle(), OPTION_SYMBOL_NAME, symbol.as_ptr());
        if status != SUCCESS {
            return Err(self.status_error("设置符号名", status));
        }
        let ltoir = self.read_output(&descriptor, OutputKind::Ltoir)?;
        let fatbin = self.read_output(&descriptor, OutputKind::Fatbin)?;
        Ok(SolverLto {
            ltoir_sha256: sha256_hex(&ltoir),
            fatbin_sha256: sha256_hex(&fatbin),
            ltoir,
            fatbin,
            mathdx_version: self.version(),
        })
    }

    fn set_operator_int64(
        &self,
        descriptor: &ffi::Descriptor<'_>,
        operator: c_int,
        value: c_longlong,
        stage: &str,
    ) -> Result<(), LauncherError> {
        let status = self
            .api
            .set_operator_int64(descriptor.handle(), operator, value);
        if status != SUCCESS {
            return Err(self.status_error(stage, status));
        }
        Ok(())
    }

    fn set_operator_int64s(
        &self,
        descriptor: &ffi::Descriptor<'_>,
        operator: c_int,
        values: &mut [c_longlong],
        stage: &str,
    ) -> Result<(), LauncherError> {
        let status = self.api.set_operator_int64s(
            descriptor.handle(),
            operator,
            values.len(),
            values.as_mut_ptr(),
        );
        if status != SUCCESS {
            return Err(self.status_error(stage, status));
        }
        Ok(())
    }

    fn read_output(
        &self,
        descriptor: &ffi::Descriptor<'_>,
        kind: OutputKind,
    ) -> Result<Vec<u8>, LauncherError> {
        let mut size = 0usize;
        let status = match kind {
            OutputKind::Ltoir => self.api.get_ltoir_size(descriptor.handle(), &mut size),
            OutputKind::Fatbin => self.api.get_fatbin_size(descriptor.handle(), &mut size),
        };
        if status != SUCCESS {
            return Err(self.status_error("读取产物大小", status));
        }
        let name = match kind {
            OutputKind::Ltoir => "LTOIR",
            OutputKind::Fatbin => "FATBIN",
        };
        if size == 0 {
            return Err(LauncherError::Compiler(format!("MathDx {name}为空")));
        }
        let mut bytes = vec![0u8; size];
        let status = match kind {
            OutputKind::Ltoir => self.api.get_ltoir(
                descriptor.handle(),
                bytes.len(),
                bytes.as_mut_ptr().cast::<c_void>(),
            ),
            OutputKind::Fatbin => self.api.get_fatbin(
                descriptor.handle(),
                bytes.len(),
                bytes.as_mut_ptr().cast::<c_void>(),
            ),
        };
        if status != SUCCESS {
            return Err(self.status_error("读取产物", status));
        }
        Ok(bytes)
    }

    fn status_error(&self, stage: &str, status: c_int) -> LauncherError {
        let log = self.last_error();
        let text = status_text(status);
        if log.is_empty() {
            LauncherError::Compiler(format!("MathDx{stage}失败：{text}（{status}）"))
        } else {
            LauncherError::Compiler(format!("MathDx{stage}失败：{text}（{status}）：{log}"))
        }
    }

    fn last_error(&self) -> String {
        let mut size = 0usize;
        if self.api.get_last_error_size(&mut size) != SUCCESS || size == 0 {
            return String::new();
        }
        let mut bytes = vec![0u8; size];
        let mut code: c_int = 0;
        if self
            .api
            .get_last_error(&mut code, bytes.len(), bytes.as_mut_ptr().cast())
            != SUCCESS
        {
            return String::new();
        }
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        String::from_utf8_lossy(&bytes[..end]).into_owned()
    }
}

#[derive(Clone, Copy)]
enum OutputKind {
    Ltoir,
    Fatbin,
}

/// 对应`commondxStatusType`的短文本。
fn status_text(status: c_int) -> &'static str {
    match status {
        0 => "成功",
        1 => "取值非法",
        2 => "库内部错误",
        3 => "编译失败",
        4 => "库依赖错误",
        _ => "未知状态",
    }
}

/// 版本须与工具链固定组件完全一致。
pub(crate) fn check_version(version: (i32, i32, i32)) -> Result<(), LauncherError> {
    if version != MATHDX_VERSION {
        return Err(LauncherError::Linker(format!(
            "MathDx版本不符：需要{}.{}.{}，实际{}.{}.{}",
            MATHDX_VERSION.0, MATHDX_VERSION.1, MATHDX_VERSION.2, version.0, version.1, version.2
        )));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests;
