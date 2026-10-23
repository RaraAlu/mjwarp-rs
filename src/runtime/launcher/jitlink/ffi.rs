//! nvJitLink 的手写 C ABI 与动态加载。
//! 本文件集中全部 unsafe；上层只见安全接口。
//! 签名逐项对照钉死版本的 `nvJitLink.h`。

use std::ffi::{c_char, c_int, c_uint, c_void};
use std::path::Path;

use super::{LauncherError, NVJITLINK_VERSION};

/// 不透明句柄，对应`nvJitLinkHandle`。
pub(super) type Handle = *mut c_void;
/// 对应`nvJitLinkResult`枚举，按C `int`传递。
pub(super) type LinkResult = c_int;

pub(super) const SUCCESS: LinkResult = 0;

type Create = unsafe extern "C" fn(*mut Handle, u32, *const *const c_char) -> LinkResult;
type Destroy = unsafe extern "C" fn(*mut Handle) -> LinkResult;
type AddData =
    unsafe extern "C" fn(Handle, c_int, *const c_void, usize, *const c_char) -> LinkResult;
type Complete = unsafe extern "C" fn(Handle) -> LinkResult;
type SizeQuery = unsafe extern "C" fn(Handle, *mut usize) -> LinkResult;
type BytesQuery = unsafe extern "C" fn(Handle, *mut c_void) -> LinkResult;
type LogQuery = unsafe extern "C" fn(Handle, *mut c_char) -> LinkResult;
type Version = unsafe extern "C" fn(*mut c_uint, *mut c_uint) -> LinkResult;

/// 导出名带`_<主>_<次>`后缀；头文件的无后缀名只是内联包装。
/// 后缀取钉死版本，签名须与该版头文件一致。
pub(super) const SYMBOLS: [&str; 10] = [
    "__nvJitLinkCreate",
    "__nvJitLinkDestroy",
    "__nvJitLinkAddData",
    "__nvJitLinkComplete",
    "__nvJitLinkGetLinkedCubinSize",
    "__nvJitLinkGetLinkedCubin",
    "__nvJitLinkGetErrorLogSize",
    "__nvJitLinkGetErrorLog",
    "__nvJitLinkGetInfoLogSize",
    "__nvJitLinkGetInfoLog",
];
const VERSION_SYMBOL: &str = "nvJitLinkVersion";

/// 已解析的函数表；`library`保证指针存活。
pub(super) struct Api {
    create: Create,
    destroy: Destroy,
    add_data: AddData,
    complete: Complete,
    cubin_size: SizeQuery,
    cubin: BytesQuery,
    error_log_size: SizeQuery,
    error_log: LogQuery,
    info_log_size: SizeQuery,
    info_log: LogQuery,
    version: (u32, u32),
    // 最后析构：函数指针不得比库活得更久。
    _library: libloading::Library,
}

impl Api {
    /// 只按绝对路径装载，不走系统搜索路径。
    pub(super) fn open(path: &Path) -> Result<Self, LauncherError> {
        if !path.is_absolute() {
            return Err(LauncherError::Linker(format!(
                "库路径须为绝对路径：{}",
                path.display()
            )));
        }
        if !path.is_file() {
            return Err(LauncherError::Linker(format!("库缺失：{}", path.display())));
        }
        let library = open_library(path)?;
        // SAFETY: 每个符号按头文件签名取出，类型与C ABI一一对应。
        // 指针随`_library`一同存放，结构析构前库不会卸载。
        unsafe {
            let version: Version = symbol(&library, VERSION_SYMBOL)?;
            let (mut major, mut minor) = (0, 0);
            let status = version(&mut major, &mut minor);
            if status != SUCCESS {
                return Err(LauncherError::Linker(format!("版本查询失败：{status}")));
            }
            // 先核版本：旧版库缺少新后缀符号，应报版本不符。
            super::check_version((major, minor))?;
            Ok(Self {
                create: symbol(&library, &versioned(SYMBOLS[0]))?,
                destroy: symbol(&library, &versioned(SYMBOLS[1]))?,
                add_data: symbol(&library, &versioned(SYMBOLS[2]))?,
                complete: symbol(&library, &versioned(SYMBOLS[3]))?,
                cubin_size: symbol(&library, &versioned(SYMBOLS[4]))?,
                cubin: symbol(&library, &versioned(SYMBOLS[5]))?,
                error_log_size: symbol(&library, &versioned(SYMBOLS[6]))?,
                error_log: symbol(&library, &versioned(SYMBOLS[7]))?,
                info_log_size: symbol(&library, &versioned(SYMBOLS[8]))?,
                info_log: symbol(&library, &versioned(SYMBOLS[9]))?,
                version: (major, minor),
                _library: library,
            })
        }
    }

    pub(super) fn version(&self) -> (u32, u32) {
        self.version
    }

    /// 选项须为NUL结尾的C串；返回的句柄由`Session`销毁。
    pub(super) fn create(&self, options: &[*const c_char]) -> Result<Session<'_>, LinkResult> {
        let mut handle: Handle = std::ptr::null_mut();
        let count = u32::try_from(options.len()).map_err(|_| -1)?;
        // SAFETY: 出参指向本地变量；选项指针由调用方保证存活至返回。
        let status = unsafe { (self.create)(&mut handle, count, options.as_ptr()) };
        if status != SUCCESS {
            return Err(status);
        }
        Ok(Session { api: self, handle })
    }
}

/// 单次链接的句柄守卫；析构时调用`nvJitLinkDestroy`。
/// 句柄不实现Send，不跨线程共享。
pub(super) struct Session<'a> {
    api: &'a Api,
    handle: Handle,
}

impl Session<'_> {
    /// `name`须为NUL结尾的C串；数据在调用期间只读借用。
    pub(super) fn add(&mut self, kind: c_int, data: &[u8], name: *const c_char) -> LinkResult {
        // SAFETY: 句柄有效；数据指针与长度来自同一切片，调用返回后库不再持有。
        unsafe { (self.api.add_data)(self.handle, kind, data.as_ptr().cast(), data.len(), name) }
    }

    pub(super) fn complete(&mut self) -> LinkResult {
        // SAFETY: 句柄有效且未销毁。
        unsafe { (self.api.complete)(self.handle) }
    }

    pub(super) fn cubin(&self) -> Result<Vec<u8>, LinkResult> {
        let mut size = 0usize;
        // SAFETY: 句柄有效；出参指向本地变量。
        let status = unsafe { (self.api.cubin_size)(self.handle, &mut size) };
        if status != SUCCESS {
            return Err(status);
        }
        let mut bytes = vec![0u8; size];
        if size == 0 {
            return Ok(bytes);
        }
        // SAFETY: 缓冲容量等于库报告的产物字节数。
        let status = unsafe { (self.api.cubin)(self.handle, bytes.as_mut_ptr().cast()) };
        if status != SUCCESS {
            return Err(status);
        }
        Ok(bytes)
    }

    pub(super) fn error_log(&self) -> String {
        self.log(self.api.error_log_size, self.api.error_log)
    }

    pub(super) fn info_log(&self) -> String {
        self.log(self.api.info_log_size, self.api.info_log)
    }

    fn log(&self, size_query: SizeQuery, query: LogQuery) -> String {
        let mut size = 0usize;
        // SAFETY: 句柄有效；出参指向本地变量。
        if unsafe { size_query(self.handle, &mut size) } != SUCCESS || size == 0 {
            return String::new();
        }
        // 库报告的尺寸含结尾NUL。
        let mut bytes = vec![0u8; size];
        // SAFETY: 缓冲容量等于库报告的日志字节数。
        if unsafe { query(self.handle, bytes.as_mut_ptr().cast()) } != SUCCESS {
            return String::new();
        }
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        String::from_utf8_lossy(&bytes[..end]).into_owned()
    }
}

impl Drop for Session<'_> {
    fn drop(&mut self) {
        // SAFETY: 句柄由create取得，此处唯一一次销毁。
        let _ = unsafe { (self.api.destroy)(&mut self.handle) };
    }
}

#[cfg(windows)]
fn open_library(path: &Path) -> Result<libloading::Library, LauncherError> {
    use libloading::os::windows::{
        LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR, LOAD_LIBRARY_SEARCH_SYSTEM32, Library,
    };
    // SAFETY: 依赖只从库所在目录与System32解析；DLL初始化不回调本进程。
    unsafe {
        Library::load_with_flags(
            path,
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32,
        )
    }
    .map(Into::into)
    .map_err(|error| LauncherError::Linker(format!("库加载失败：{}：{error}", path.display())))
}

#[cfg(unix)]
fn open_library(path: &Path) -> Result<libloading::Library, LauncherError> {
    use libloading::os::unix::{Library, RTLD_LOCAL, RTLD_NOW};
    // SAFETY: 按绝对路径立即解析全部符号，不导出到全局命名空间。
    unsafe { Library::open(Some(path), RTLD_NOW | RTLD_LOCAL) }
        .map(Into::into)
        .map_err(|error| LauncherError::Linker(format!("库加载失败：{}：{error}", path.display())))
}

/// 按钉死版本拼出导出名。
pub(super) fn versioned(name: &str) -> String {
    format!("{name}_{}_{}", NVJITLINK_VERSION.0, NVJITLINK_VERSION.1)
}

/// 调用方须保证`T`与该符号的C签名一致。
unsafe fn symbol<T: Copy>(library: &libloading::Library, name: &str) -> Result<T, LauncherError> {
    let mut bytes = name.as_bytes().to_vec();
    bytes.push(0);
    // SAFETY: 由调用方保证签名；取出的指针随库存活。
    unsafe { library.get::<T>(&bytes) }
        .map(|symbol| *symbol)
        .map_err(|_| LauncherError::Linker(format!("符号缺失：{name}")))
}
