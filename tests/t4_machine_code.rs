//! 捕获本轮真实加载产物。
//! 不运行物理样本或内核。
#![cfg(all(windows, feature = "cuda-probe"))]

use cudarc::{driver::CudaContext, driver::sys, nvrtc::Ptx};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    ffi::{CStr, c_char, c_void},
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
    time::Instant,
};

const TOOL: &str = "target/toolchains/cuda-12.8.1";
const CUPTI: &str = "cupti/cuda_cupti-windows-x86_64-12.8.90-archive/lib/cupti64_2025.1.1.dll";
const DISASM: &str = "nvdisasm/cuda_nvdisasm-windows-x86_64-12.8.90-archive/bin/nvdisasm.exe";

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryW(path: *const u16) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const c_char) -> *mut c_void;
    fn FreeLibrary(module: *mut c_void) -> i32;
}

struct Library(*mut c_void);
impl Drop for Library {
    fn drop(&mut self) {
        // SAFETY: 此对象独占一次成功加载。
        unsafe { FreeLibrary(self.0) };
    }
}
impl Library {
    fn load(path: &Path) -> Self {
        let wide: Vec<u16> = path
            .as_os_str()
            .to_string_lossy()
            .encode_utf16()
            .chain([0])
            .collect();
        // SAFETY: 路径终止符有效；仅加载已校验的官方库。
        let handle = unsafe { LoadLibraryW(wide.as_ptr()) };
        assert!(!handle.is_null());
        Self(handle)
    }
    fn symbol(&self, name: &CStr) -> *mut c_void {
        // SAFETY: 库仍存活；名称具有终止符。
        let address = unsafe { GetProcAddress(self.0, name.as_ptr()) };
        assert!(!address.is_null());
        address
    }
}

// cupti_callbacks.h固定以下64位C布局。
#[repr(C)]
struct ResourceData {
    context: *mut c_void,
    stream: *mut c_void,
    descriptor: *mut c_void,
}
#[repr(C)]
struct ModuleData {
    id: u32,
    size: usize,
    cubin: *const u8,
}
struct Snapshot {
    id: u32,
    context: usize,
    bytes: Vec<u8>,
}
type Callback = unsafe extern "system" fn(*mut c_void, u32, u32, *const c_void);
type Subscribe = unsafe extern "system" fn(*mut *mut c_void, Callback, *mut c_void) -> u32;
type Enable = unsafe extern "system" fn(u32, *mut c_void, u32, u32) -> u32;
type Unsubscribe = unsafe extern "system" fn(*mut c_void) -> u32;

unsafe extern "system" fn callback(user: *mut c_void, domain: u32, id: u32, data: *const c_void) {
    if domain != 3 || id != 6 || user.is_null() || data.is_null() {
        return;
    }
    // SAFETY: 仅订阅RESOURCE/MODULE_LOADED。官方回调保证布局与寿命。
    // 在回调返回前复制CUBIN；不保留库提供的指针。
    // 订阅器持有user的Box；注销先于释放。互斥锁隔离回调线程。
    unsafe {
        let resource = &*data.cast::<ResourceData>();
        if resource.descriptor.is_null() {
            return;
        }
        let module = &*resource.descriptor.cast::<ModuleData>();
        if module.cubin.is_null() || module.size == 0 || module.size > 128 * 1024 * 1024 {
            return;
        }
        let list = &*user.cast::<Mutex<Vec<Snapshot>>>();
        if let Ok(mut list) = list.lock() {
            list.push(Snapshot {
                id: module.id,
                context: resource.context as usize,
                bytes: std::slice::from_raw_parts(module.cubin, module.size).to_vec(),
            });
        }
    }
}

struct Capture {
    _library: Library,
    subscriber: *mut c_void,
    unsubscribe: Unsubscribe,
    snapshots: Box<Mutex<Vec<Snapshot>>>,
}
impl Capture {
    fn start(path: &Path) -> Self {
        let library = Library::load(path);
        let snapshots = Box::new(Mutex::new(Vec::new()));
        let mut subscriber = std::ptr::null_mut();
        // SAFETY: 签名与冻结官方头文件一致。库与Box覆盖整个订阅期。
        let unsubscribe = unsafe {
            let subscribe: Subscribe = std::mem::transmute(library.symbol(c"cuptiSubscribe"));
            let enable: Enable = std::mem::transmute(library.symbol(c"cuptiEnableCallback"));
            let unsubscribe: Unsubscribe = std::mem::transmute(library.symbol(c"cuptiUnsubscribe"));
            let code = subscribe(
                &mut subscriber,
                callback,
                (&*snapshots as *const Mutex<Vec<Snapshot>>)
                    .cast_mut()
                    .cast(),
            );
            assert_eq!(code, 0, "CUPTI subscribe");
            // 启用失败时也先注销，防止回调保留已释放的Box。
            let code = enable(1, subscriber, 3, 6);
            if code != 0 {
                unsubscribe(subscriber);
            }
            assert_eq!(code, 0, "CUPTI resource callback");
            unsubscribe
        };
        Self {
            _library: library,
            subscriber,
            unsubscribe,
            snapshots,
        }
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        // SAFETY: 句柄属于此对象。先停止回调，再释放Box与动态库。
        unsafe { (self.unsubscribe)(self.subscriber) };
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
#[ignore = "CUPTI module-load capture and disassembly only; no kernel launch"]
fn captures_loaded_face_machine_code_without_launching_physics() {
    let started = Instant::now();
    assert_eq!(size_of::<ResourceData>(), 24);
    assert_eq!(size_of::<ModuleData>(), 24);
    assert_eq!(std::mem::offset_of!(ResourceData, descriptor), 16);
    assert_eq!(std::mem::offset_of!(ModuleData, size), 8);
    assert_eq!(std::mem::offset_of!(ModuleData, cubin), 16);
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = PathBuf::from(std::env::var_os("MJWARP_MACHINE_DIR").unwrap());
    assert!(out.is_absolute() && !out.exists());
    assert!(
        out.parent()
            .unwrap()
            .canonicalize()
            .unwrap()
            .starts_with(root.join("target").canonicalize().unwrap())
    );
    let cupti = root.join(TOOL).join(CUPTI);
    let disasm = root.join(TOOL).join(DISASM);
    assert_eq!(
        digest(&fs::read(&cupti).unwrap()),
        "d026870a2c503d1b6e38a793170823ceca83d776f9b887f63fb0bb5c93d5c525"
    );
    assert_eq!(
        digest(&fs::read(&disasm).unwrap()),
        "f0ab98df225b9f2568e8f56c24a919a4e1362425fb7c02101afbf85022324add"
    );
    let header = root
        .join(TOOL)
        .join("cupti/cuda_cupti-windows-x86_64-12.8.90-archive/include/cupti_callbacks.h");
    assert_eq!(
        digest(&fs::read(header).unwrap()),
        "da47efc991e0d46c0ae1576eaa6f4c06935cf735d26c3d3566ddebfd162d5d51"
    );
    let context = CudaContext::new(0).unwrap();
    context.bind_to_thread().unwrap();
    let major = context
        .attribute(sys::CUdevice_attribute::CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MAJOR)
        .unwrap();
    let minor = context
        .attribute(sys::CUdevice_attribute::CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MINOR)
        .unwrap();
    assert_eq!((major, minor), (8, 9));
    let mut driver = 0;
    // SAFETY: 驱动只写一个有效i32，不提交设备任务。
    unsafe { sys::cuDriverGetVersion(&mut driver) }
        .result()
        .unwrap();
    let capture = Capture::start(&cupti);
    fs::create_dir(&out).unwrap();
    let mut artifacts = Vec::new();
    for (id, path, expected, entry) in [
        (
            "product",
            "target/t4-face-operators/product.ptx",
            "6832e3250af25b8108dd65eed170194e3e961385adb24e699f01bcd528376827",
            "flex_faces",
        ),
        (
            "upstream",
            "target/mjwarp-parallel/warp-cache/1.15.0/wp_mujoco_warp._src.smooth_ca013d6/wp_mujoco_warp._src.smooth_ca013d6.sm89.ptx",
            "1d9e4e3a983ca35090ade3a48fd83aa0dc86a23228bb0865010f681698ab75dd",
            "_flex_face_kinematics_9a1620ae_cuda_kernel_forward",
        ),
    ] {
        let raw = fs::read(root.join(path)).unwrap();
        assert_eq!(digest(&raw), expected);
        let text = String::from_utf8(raw).unwrap();
        let source = text.trim_end_matches('\0');
        capture.snapshots.lock().unwrap().clear();
        // 与原适配器相同：Ptx::from_src→load_module→cuModuleLoadData。
        // 只加载模块和查询函数；不创建流、物理缓冲或启动内核。
        let module = context.load_module(Ptx::from_src(source)).unwrap();
        let _function = module.load_function(entry).unwrap();
        let snapshots = std::mem::take(&mut *capture.snapshots.lock().unwrap());
        assert_eq!(snapshots.len(), 1, "one target load callback");
        let snapshot = &snapshots[0];
        assert_eq!(snapshot.context, context.cu_ctx() as usize);
        assert_eq!(&snapshot.bytes[..4], b"\x7fELF");
        let binary = out.join(format!("{id}.cubin"));
        fs::write(&binary, &snapshot.bytes).unwrap();
        let result = Command::new(&disasm)
            .arg("-hex")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let sass = String::from_utf8(result.stdout).unwrap();
        assert!(sass.contains(entry));
        assert!(sass.contains(".target\tsm_89"));
        let warning = String::from_utf8(result.stderr).unwrap();
        fs::write(out.join(format!("{id}.disasm-stderr")), &warning).unwrap();
        fs::write(out.join(format!("{id}.sass")), &sass).unwrap();
        artifacts.push(json!({"id":id,"ptx_path":path,"ptx_raw_sha256":expected,"loaded_ptx_sha256":digest(source.as_bytes()),"cubin_sha256":digest(&snapshot.bytes),"cubin_bytes":snapshot.bytes.len(),"sass_sha256":digest(sass.as_bytes()),"module_callback_id":snapshot.id,"callbacks":snapshots.len(),"entry":entry,"disassembly_warning":warning}));
    }
    let report = json!({"schema":1,"scope":"current-cuModuleLoadData-CUPTI-capture-no-kernel-launch","base_revision":"38803deb7f6264f745896a59e785616d92518692","gpu":context.name().unwrap(),"sm":[major,minor],"driver_api_version":driver,"module_loader":"cuModuleLoadData via cudarc 0.18.2; no JIT option overrides","cuLink_used":false,"historical_session_binary":false,"current_original_load_path_binary":true,"nvdisasm_version":"12.8.90","cupti_version":"12.8.90","artifacts":artifacts,"kernel_launches":0,"physical_inputs":0,"physical_buffer_allocations":0,"python_executions":0,"full_acceptance":false,"elapsed_ms":started.elapsed().as_millis()});
    fs::write(
        out.join("capture.json"),
        serde_json::to_string_pretty(&report).unwrap() + "\n",
    )
    .unwrap();
    println!("{report}");
}
