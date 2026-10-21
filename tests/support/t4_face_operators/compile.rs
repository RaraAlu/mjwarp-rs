use super::*;

#[cfg(windows)]
pub(super) fn warp_compile(source: &str, arch: i32, native: &Path, path: &Path) -> String {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LoadLibraryW(path: *const u16) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const c_char) -> *mut c_void;
        fn FreeLibrary(module: *mut c_void) -> i32;
    }
    // warp.h与context.py共同定义冻结的22参数C ABI。
    type Compile = unsafe extern "C" fn(
        *const c_char,
        *const c_char,
        i32,
        *const c_char,
        *const c_char,
        i32,
        *const *const c_char,
        bool,
        i32,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        *const c_char,
        *const c_char,
        usize,
        *mut *mut c_char,
        *mut usize,
        *mut i32,
    ) -> usize;
    let dll = native.parent().unwrap().join("bin/warp.dll");
    assert_eq!(
        hash(&fs::read(&dll).unwrap()),
        "7c006eb348a7359810f4d001cbd4141cc5d6d1641fc2989e4c3711e99e6671d6"
    );
    let wide: Vec<u16> = dll.to_str().unwrap().encode_utf16().chain([0]).collect();
    let src = CString::new(source).unwrap();
    let name = CString::new("face_operators.cu").unwrap();
    let include = CString::new(native.to_str().unwrap()).unwrap();
    let output = CString::new(path.to_str().unwrap()).unwrap();
    // SAFETY: 仅加载哈希冻结的本地库。字符串存活至同步编译返回。
    // 空链接数组与零长度配对。关闭PCH，不启动Python或Warp运行时。
    unsafe {
        let module = LoadLibraryW(wide.as_ptr());
        assert!(!module.is_null());
        let pointer = GetProcAddress(module, c"wp_cuda_compile_program".as_ptr());
        assert!(!pointer.is_null());
        let compile: Compile = std::mem::transmute(pointer);
        let code = compile(
            src.as_ptr(),
            name.as_ptr(),
            arch,
            c"".as_ptr(),
            include.as_ptr(),
            0,
            std::ptr::null(),
            false,
            3,
            false,
            false,
            false,
            true,
            false,
            false,
            false,
            output.as_ptr(),
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        assert_ne!(FreeLibrary(module), 0);
        assert_eq!(code, 0);
    }
    let text = fs::read_to_string(path).unwrap();
    assert!(text.contains("V12.9.86"));
    text.trim_end_matches('\0').to_owned()
}
