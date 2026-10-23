//! W13 nvJitLink 绑定的真实库与设备对照。
//! 需要`CUDA_PATH`指向项目工具链目录。
//! 该目录含 NVRTC 12.8.1 与 nvJitLink 12.9.86。
//! 对照输入取 2026-10-14 离线证据的钉死字节。
//! 本测试不调用 warp.dll，也不启动 Python。
#![cfg(feature = "cuda-probe")]

use std::ffi::{CString, c_char};
use std::path::PathBuf;

use cudarc::driver::{CudaContext, LaunchConfig, PushKernelArg, sys as driver_sys};
use cudarc::nvrtc::{Ptx, result as nvrtc, sys as nvrtc_sys};
use mjwarp_rs::runtime::launcher::{LinkInput, LinkInputKind, NVJITLINK_VERSION, NvJitLink};
use sha2::{Digest, Sha256};

const LTO_SHA: &str = "98c01ce64f55d31af5d4eb001c1e9fc4c8456173a9f0bbe97be479c804a981eb";
const FATBIN_SHA: &str = "c00d6412780b18f67e11a1d93fc34d51db9cdd687f26ec7bc926efbff5ddbd1f";
const CALL_SHA: &str = "abfc8c777014708ed9b33c7a80798e8d2160f102f49a21d01fbf203911968853";
/// 离线证据 attempt-2 的产物与设备输出摘要。
const EVIDENCE_CUBIN_SHA: &str = "de660b5d5095516549c053bf51e13c05b9bc55a3dfe5cfeb0a0833c068be55ae";
const EVIDENCE_OUTPUT_SHA: &str =
    "51e1a99b11990d259411de58816a6344cec85f224c6a541fa088030749126d3f";
/// 证据记录的 NVRTC 选项；省略未被使用的 Warp 头路径。
const NVRTC_OPTIONS: [&str; 13] = [
    "--gpu-architecture=sm_89",
    "--std=c++17",
    "--Ofast-compile=0",
    "--define-macro=NDEBUG",
    "--undefine-macro=WP_VERIFY_FP",
    "--define-macro=WP_ENABLE_MATHDX=1",
    "--fmad=true",
    "--device-as-default-execution-space",
    "--extra-device-vectorization",
    "--restrict",
    "--diag-suppress=177,550",
    "-dlto",
    "--relocatable-device-code=true",
];

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn pinned(relative: &str, sha: &str, lf: bool) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    let mut bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("缺少钉死输入{}：{error}", path.display()));
    if lf {
        bytes = String::from_utf8(bytes)
            .unwrap()
            .replace("\r\n", "\n")
            .into_bytes();
    }
    assert_eq!(digest(&bytes), sha, "钉死输入摘要不符：{relative}");
    bytes
}

/// 用工具链 NVRTC 生成调用源的 LTOIR。
fn compile_ltoir(source: &[u8]) -> (Vec<u8>, (i32, i32)) {
    let (mut major, mut minor) = (0, 0);
    // SAFETY: 出参指向本地变量；cudarc 已按需加载 NVRTC。
    unsafe { nvrtc_sys::nvrtcVersion(&mut major, &mut minor) }
        .result()
        .unwrap();
    let source = CString::new(source).unwrap();
    let name = CString::new("native-mathdx7-call.cu").unwrap();
    let program = nvrtc::create_program(&source, Some(&name)).unwrap();
    // SAFETY: program 由 create_program 取得，本函数末尾唯一一次销毁。
    let ltoir = unsafe {
        if let Err(error) = nvrtc::compile_program(program, &NVRTC_OPTIONS) {
            let log = nvrtc::get_program_log(program).unwrap_or_default();
            let log: Vec<u8> = log.iter().map(|&c| c as u8).collect();
            panic!("NVRTC 失败：{error:?}：{}", String::from_utf8_lossy(&log));
        }
        let mut size = 0usize;
        nvrtc_sys::nvrtcGetLTOIRSize(program, &mut size)
            .result()
            .unwrap();
        let mut bytes = vec![0u8; size];
        nvrtc_sys::nvrtcGetLTOIR(program, bytes.as_mut_ptr().cast::<c_char>())
            .result()
            .unwrap();
        nvrtc::destroy_program(program).unwrap();
        bytes
    };
    (ltoir, (major, minor))
}

#[test]
#[ignore = "需要 CUDA_PATH 下的 nvJitLink 12.9"]
fn pinned_library_reports_version_and_rejects_bad_input() {
    let linker = NvJitLink::load_from_cuda_path().unwrap();
    assert_eq!(linker.version(), NVJITLINK_VERSION);
    let garbage = [0x5au8; 64];
    let error = linker
        .link_cubin(
            89,
            &[LinkInput {
                kind: LinkInputKind::Ltoir,
                name: "garbage",
                bytes: &garbage,
            }],
        )
        .unwrap_err();
    println!("bad_input_error={error}");
    assert!(matches!(
        error,
        mjwarp_rs::runtime::launcher::LauncherError::Linker(_)
    ));
}

type Matrix = [f32; 49];

fn analytic(sample: usize) -> (Matrix, Matrix) {
    let (mut upper, mut matrix) = ([0f32; 49], [0f32; 49]);
    for i in 0..7 {
        upper[i * 7 + i] = if sample == 0 {
            (1 << i) as f32
        } else {
            (2 * i + 3) as f32
        };
        for j in i + 1..7 {
            let sign = if (i + j) % 2 == 1 { -1 } else { 1 };
            let scale = if sample == 0 { 4.0 } else { 8.0 };
            upper[i * 7 + j] = (sign * (i + j + 1) as i32) as f32 / scale;
        }
    }
    for i in 0..7 {
        for j in 0..7 {
            let gram: f64 = (0..7)
                .map(|k| f64::from(upper[k * 7 + i]) * f64::from(upper[k * 7 + j]))
                .sum();
            matrix[i * 7 + j] = gram as f32;
            assert_eq!(f64::from(matrix[i * 7 + j]), gram, "M 不能精确表示为 f32");
        }
    }
    (matrix, upper)
}

/// 与离线工具相同的守卫布局、发射配置与输出归档顺序。
fn batch(
    stream: &std::sync::Arc<cudarc::driver::CudaStream>,
    kernel: &cudarc::driver::CudaFunction,
    worlds: usize,
    matrix: &Matrix,
    expected_info: i32,
    archive: &mut Vec<u8>,
) {
    const GUARD: f32 = -12345.5;
    const STATUS_GUARD: i32 = 0x1234_5678;
    let count = worlds * 49;
    let mut input = vec![GUARD; count + 2];
    for world in 0..worlds {
        input[1 + world * 49..1 + (world + 1) * 49].copy_from_slice(matrix);
    }
    let source = stream.clone_htod(&input).unwrap();
    let mut output = stream.clone_htod(&vec![GUARD; count + 2]).unwrap();
    let mut status = stream.clone_htod(&vec![STATUS_GUARD; worlds + 2]).unwrap();
    {
        let source_view = source.slice(1..);
        let mut output_view = output.slice_mut(1..);
        let mut status_view = status.slice_mut(1..);
        let config = LaunchConfig {
            grid_dim: (worlds as u32, 1, 1),
            block_dim: (32, 1, 1),
            shared_mem_bytes: 0,
        };
        let mut builder = stream.launch_builder(kernel);
        builder
            .arg(&source_view)
            .arg(&mut output_view)
            .arg(&mut status_view);
        // SAFETY: 签名为(const float*, float*, int*)；每世界读写49项与1项，
        // 守卫偏移后容量充足，三缓冲互不重叠。
        unsafe { builder.launch(config) }.unwrap();
    }
    stream.synchronize().unwrap();
    let after = stream.clone_dtoh(&source).unwrap();
    let output = stream.clone_dtoh(&output).unwrap();
    let status = stream.clone_dtoh(&status).unwrap();
    assert!(
        after
            .iter()
            .zip(&input)
            .all(|(a, b)| a.to_bits() == b.to_bits())
    );
    assert_eq!(output[0].to_bits(), GUARD.to_bits());
    assert_eq!(output[count + 1].to_bits(), GUARD.to_bits());
    assert_eq!(
        (status[0], status[worlds + 1]),
        (STATUS_GUARD, STATUS_GUARD)
    );
    assert!(status[1..=worlds].iter().all(|&info| info == expected_info));
    archive.extend(output.iter().flat_map(|value| value.to_le_bytes()));
    archive.extend(status.iter().flat_map(|value| value.to_le_bytes()));
}

#[test]
#[ignore = "需要 SM89 GPU、CUDA_PATH 与离线证据输入"]
fn mathdx7_link_reproduces_offline_device_output() {
    let lto = pinned("target/native-mathdx7/attempt-1/potrf7.lto", LTO_SHA, false);
    let fatbin = pinned(
        "target/native-mathdx7/attempt-1/potrf7_fatbin.lto",
        FATBIN_SHA,
        false,
    );
    let call = pinned("scripts/native-mathdx7-call.cu", CALL_SHA, true);
    let (call_ltoir, nvrtc_version) = compile_ltoir(&call);
    let linker = NvJitLink::load_from_cuda_path().unwrap();
    // 与 Warp 相同：程序 LTOIR 在前，外部输入按原顺序在后。
    let inputs = [
        LinkInput {
            kind: LinkInputKind::Ltoir,
            name: "native-mathdx7-call",
            bytes: &call_ltoir,
        },
        LinkInput {
            kind: LinkInputKind::Ltoir,
            name: "potrf7.lto",
            bytes: &lto,
        },
        LinkInput {
            kind: LinkInputKind::Fatbin,
            name: "potrf7_fatbin.lto",
            bytes: &fatbin,
        },
    ];
    let first = linker.link_cubin(89, &inputs).unwrap();
    let second = linker.link_cubin(89, &inputs).unwrap();
    let cubin_sha = digest(first.bytes());
    assert_eq!(
        cubin_sha,
        digest(second.bytes()),
        "同输入两次链接产物不一致"
    );
    // 留存产物供离线比对；目录由 Cargo 为集成测试提供。
    let dump = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("nvjitlink-mathdx7.cubin");
    std::fs::write(&dump, first.bytes()).unwrap();

    let context = CudaContext::new(0).unwrap();
    let major = context
        .attribute(driver_sys::CUdevice_attribute::CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MAJOR)
        .unwrap();
    let minor = context
        .attribute(driver_sys::CUdevice_attribute::CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MINOR)
        .unwrap();
    assert_eq!((major, minor), (8, 9), "冻结 LTO 需要 SM89");
    let module = context
        .load_module(Ptx::from_binary(first.bytes().to_vec()))
        .unwrap();
    let kernel = module.load_function("mathdx7_factor").unwrap();
    let stream = context.default_stream();
    let mut archive = Vec::new();
    let started = std::time::Instant::now();
    for sample in 0..2 {
        let (matrix, _) = analytic(sample);
        for worlds in [1, 2, 257] {
            batch(&stream, &kernel, worlds, &matrix, 0, &mut archive);
        }
    }
    for pivot in [0usize, 6] {
        let mut matrix = [0f32; 49];
        for i in 0..7 {
            matrix[i * 7 + i] = if i == pivot { -1.0 } else { 1.0 };
        }
        batch(&stream, &kernel, 1, &matrix, pivot as i32 + 1, &mut archive);
    }
    let elapsed_ms = started.elapsed().as_secs_f64() * 1e3;
    let output_sha = digest(&archive);
    println!(
        "{}",
        serde_json::json!({
            "nvjitlink_version": [linker.version().0, linker.version().1],
            "nvjitlink_path": linker.path().display().to_string(),
            "nvrtc_version": [nvrtc_version.0, nvrtc_version.1],
            "call_ltoir_bytes": call_ltoir.len(),
            "call_ltoir_sha256": digest(&call_ltoir),
            "cubin_bytes": first.bytes().len(),
            "cubin_sha256": cubin_sha,
            "cubin_matches_evidence": cubin_sha == EVIDENCE_CUBIN_SHA,
            "repeat_link_identical": true,
            "device_output_bytes": archive.len(),
            "device_output_sha256": output_sha,
            "device_output_matches_evidence": output_sha == EVIDENCE_OUTPUT_SHA,
            "launches": 8,
            "device_phase_ms": elapsed_ms,
            "gpu_cc": [major, minor],
        })
    );
    assert_eq!(archive.len(), 104_528, "设备输出归档长度不符");
    assert_eq!(output_sha, EVIDENCE_OUTPUT_SHA, "设备输出与离线证据不一致");
}
