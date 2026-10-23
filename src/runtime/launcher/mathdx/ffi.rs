//! libmathdx 的手写 C ABI 与动态加载。
//! 本文件集中全部 unsafe；上层只见安全接口。
//! 签名逐项对照钉死的 `libcusolverdx.h` 与 `libcommondx.h`。
//! Windows 上`LIBMATHDX_CALL`是`__stdcall`，x64 与 C 一致，按`extern "system"`声明。

use std::ffi::{c_char, c_int, c_longlong, c_void};
use std::path::Path;

use super::{LauncherError, check_version};

/// `cusolverdxDescriptor`是`long long int`不透明值，不是指针。
pub(super) type DescriptorHandle = c_longlong;
/// 状态码按C `int`传递。
pub(super) type Status = c_int;

type CreateDescriptor = unsafe extern "system" fn(*mut DescriptorHandle) -> Status;
type DestroyDescriptor = unsafe extern "system" fn(DescriptorHandle) -> Status;
type SetOperatorInt64 = unsafe extern "system" fn(DescriptorHandle, c_int, c_longlong) -> Status;
type SetOperatorInt64s =
    unsafe extern "system" fn(DescriptorHandle, c_int, usize, *mut c_longlong) -> Status;
type SetOptionStr = unsafe extern "system" fn(DescriptorHandle, c_int, *const c_char) -> Status;
type GetSize = unsafe extern "system" fn(DescriptorHandle, *mut usize) -> Status;
type GetBytes = unsafe extern "system" fn(DescriptorHandle, usize, *mut c_void) -> Status;
type GetVersion = unsafe extern "system" fn(*mut c_int, *mut c_int, *mut c_int) -> Status;
type GetLastErrorSize = unsafe extern "system" fn(*mut usize) -> Status;
type GetLastError = unsafe extern "system" fn(*mut c_int, usize, *mut c_char) -> Status;

/// 已解析的函数表；`_library`保证指针存活。
pub(super) struct Api {
    create_descriptor: CreateDescriptor,
    destroy_descriptor: DestroyDescriptor,
    set_operator_int64: SetOperatorInt64,
    set_operator_int64s: SetOperatorInt64s,
    set_option_str: SetOptionStr,
    get_ltoir_size: GetSize,
    get_ltoir: GetBytes,
    get_fatbin_size: GetSize,
    get_fatbin: GetBytes,
    get_last_error_size: GetLastErrorSize,
    get_last_error: GetLastError,
    version: (i32, i32, i32),
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
            // `mathdxGetVersionEx` 报告 libmathdx 包版本（0.3.1）；
            // `cusolverdxGetVersion` 是组件自身版本（0.3.0），不用于钉版。
            let get_version: GetVersion = symbol(&library, "mathdxGetVersionEx")?;
            let (mut major, mut minor, mut patch) = (0, 0, 0);
            let status = get_version(&mut major, &mut minor, &mut patch);
            if status != super::SUCCESS {
                return Err(LauncherError::Linker(format!(
                    "MathDx版本查询失败：{status}"
                )));
            }
            // 先核版本，再取其余符号。
            check_version((major, minor, patch))?;
            Ok(Self {
                create_descriptor: symbol(&library, "cusolverdxCreateDescriptor")?,
                destroy_descriptor: symbol(&library, "cusolverdxDestroyDescriptor")?,
                set_operator_int64: symbol(&library, "cusolverdxSetOperatorInt64")?,
                set_operator_int64s: symbol(&library, "cusolverdxSetOperatorInt64s")?,
                set_option_str: symbol(&library, "cusolverdxSetOptionStr")?,
                get_ltoir_size: symbol(&library, "cusolverdxGetLTOIRSize")?,
                get_ltoir: symbol(&library, "cusolverdxGetLTOIR")?,
                get_fatbin_size: symbol(&library, "cusolverdxGetUniversalFATBINSize")?,
                get_fatbin: symbol(&library, "cusolverdxGetUniversalFATBIN")?,
                get_last_error_size: symbol(&library, "commondxGetLastErrorStrSize")?,
                get_last_error: symbol(&library, "commondxGetLastErrorStr")?,
                version: (major, minor, patch),
                _library: library,
            })
        }
    }

    pub(super) fn version(&self) -> (i32, i32, i32) {
        self.version
    }

    /// 创建描述符；守卫析构时调用`cusolverdxDestroyDescriptor`。
    pub(super) fn create_descriptor(&self) -> Result<Descriptor<'_>, Status> {
        let mut handle: DescriptorHandle = 0;
        // SAFETY: 出参指向本地变量。
        let status = unsafe { (self.create_descriptor)(&mut handle) };
        if status != super::SUCCESS {
            return Err(status);
        }
        Ok(Descriptor { api: self, handle })
    }

    pub(super) fn set_operator_int64(
        &self,
        handle: DescriptorHandle,
        operator: c_int,
        value: c_longlong,
    ) -> Status {
        // SAFETY: 句柄由`create_descriptor`取得且未销毁；值类型按头文件。
        unsafe { (self.set_operator_int64)(handle, operator, value) }
    }

    pub(super) fn set_operator_int64s(
        &self,
        handle: DescriptorHandle,
        operator: c_int,
        len: usize,
        values: *mut c_longlong,
    ) -> Status {
        // SAFETY: 句柄有效；数组指针与长度来自同一切片，调用返回后库不再持有。
        unsafe { (self.set_operator_int64s)(handle, operator, len, values) }
    }

    pub(super) fn set_option_str(
        &self,
        handle: DescriptorHandle,
        option: c_int,
        value: *const c_char,
    ) -> Status {
        // SAFETY: 句柄有效；`value`是由调用方持有的NUL结尾C串。
        unsafe { (self.set_option_str)(handle, option, value) }
    }

    pub(super) fn get_ltoir_size(&self, handle: DescriptorHandle, size: &mut usize) -> Status {
        // SAFETY: 句柄有效；出参指向本地变量。
        unsafe { (self.get_ltoir_size)(handle, size) }
    }

    pub(super) fn get_ltoir(
        &self,
        handle: DescriptorHandle,
        size: usize,
        out: *mut c_void,
    ) -> Status {
        // SAFETY: 缓冲容量等于头文件要求的大小。
        unsafe { (self.get_ltoir)(handle, size, out) }
    }

    pub(super) fn get_fatbin_size(&self, handle: DescriptorHandle, size: &mut usize) -> Status {
        // SAFETY: 句柄有效；出参指向本地变量。
        unsafe { (self.get_fatbin_size)(handle, size) }
    }

    pub(super) fn get_fatbin(
        &self,
        handle: DescriptorHandle,
        size: usize,
        out: *mut c_void,
    ) -> Status {
        // SAFETY: 缓冲容量等于头文件要求的大小。
        unsafe { (self.get_fatbin)(handle, size, out) }
    }

    pub(super) fn get_last_error_size(&self, size: &mut usize) -> Status {
        // SAFETY: 出参指向本地变量；取的是当前线程的最后错误。
        unsafe { (self.get_last_error_size)(size) }
    }

    pub(super) fn get_last_error(
        &self,
        code: &mut c_int,
        size: usize,
        value: *mut c_char,
    ) -> Status {
        // SAFETY: 缓冲容量等于`get_last_error_size`返回的字节数。
        unsafe { (self.get_last_error)(code, size, value) }
    }
}

/// 单次生成的描述符守卫；析构时销毁。
/// 句柄不实现Send，不跨线程共享。
pub(super) struct Descriptor<'a> {
    api: &'a Api,
    handle: DescriptorHandle,
}

impl Descriptor<'_> {
    pub(super) fn handle(&self) -> DescriptorHandle {
        self.handle
    }
}

impl Drop for Descriptor<'_> {
    fn drop(&mut self) {
        // SAFETY: 句柄由`create_descriptor`取得，此处唯一一次销毁。
        unsafe {
            let _ = (self.api.destroy_descriptor)(self.handle);
        }
    }
}

/// 按绝对路径预加载依赖库；句柄须由调用方保存到进程结束。
pub(super) fn preload(path: &Path) -> Result<libloading::Library, LauncherError> {
    if !path.is_absolute() {
        return Err(LauncherError::Linker(format!(
            "依赖库路径须为绝对路径：{}",
            path.display()
        )));
    }
    if !path.is_file() {
        return Err(LauncherError::Linker(format!(
            "依赖库缺失：{}",
            path.display()
        )));
    }
    open_library(path)
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

/// 调用方须保证`T`与该符号的C签名一致。
unsafe fn symbol<T: Copy>(library: &libloading::Library, name: &str) -> Result<T, LauncherError> {
    let mut bytes = name.as_bytes().to_vec();
    bytes.push(0);
    // SAFETY: 由调用方保证签名；取出的指针随库存活。
    unsafe { library.get::<T>(&bytes) }
        .map(|symbol| *symbol)
        .map_err(|_| LauncherError::Linker(format!("MathDx符号缺失：{name}")))
}
