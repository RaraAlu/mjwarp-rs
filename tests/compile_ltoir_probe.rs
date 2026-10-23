//! 产品编译链 LTOIR 接口的真实 NVRTC → 链接 → 发射对照。
//! 需要`CUDA_PATH`指向项目工具链（NVRTC 12.8.1 与 nvJitLink 12.9.86）。
//! 对照输入取 2026-10-14 离线证据与 W13 证据的钉死字节。
//! 本测试不调用 warp.dll，也不启动 Python。
#![cfg(feature = "cuda-probe")]

use std::path::PathBuf;

use cudarc::driver::{CudaContext, LaunchConfig, PushKernelArg, sys as driver_sys};
use cudarc::nvrtc::Ptx;
use mjwarp_rs::runtime::launcher::{
    LinkInput, LinkInputKind, LtoirInput, NvJitLink, compile_ltoir, link_ltoir,
};
use sha2::{Digest, Sha256};

const LTO_SHA: &str = "98c01ce64f55d31af5d4eb001c1e9fc4c8456173a9f0bbe97be479c804a981eb";
const FATBIN_SHA: &str = "c00d6412780b18f67e11a1d93fc34d51db9cdd687f26ec7bc926efbff5ddbd1f";
const CALL_SHA: &str = "abfc8c777014708ed9b33c7a80798e8d2160f102f49a21d01fbf203911968853";
/// W13 证据中的调用 LTOIR 与链接产物摘要。
const W13_CALL_LTOIR_BYTES: usize = 3632;
const W13_CALL_LTOIR_SHA: &str = "1415be10987abafe56520b7f4e525023702bbd7c475164641f774271c934c34b";
const W13_CUBIN_SHA: &str = "1086284ba5d1bd525d7320faee63978f6db79f955599d02232128da783433fe1";
/// 离线证据的设备输出摘要。
const EVIDENCE_OUTPUT_SHA: &str =
    "51e1a99b11990d259411de58816a6344cec85f224c6a541fa088030749126d3f";

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

fn compile_call_ltoir(call: &[u8]) -> mjwarp_rs::runtime::launcher::Ltoir {
    let source = std::str::from_utf8(call).unwrap();
    let input = LtoirInput::new("native-mathdx7-call.cu", source, 89).unwrap();
    compile_ltoir(&input).unwrap()
}

#[test]
#[ignore = "需要 CUDA_PATH 下的 NVRTC 12.8.1 与 nvJitLink 12.9.86"]
fn product_ltoir_interface_reproduces_evidence() {
    let lto = pinned("target/native-mathdx7/attempt-1/potrf7.lto", LTO_SHA, false);
    let fatbin = pinned(
        "target/native-mathdx7/attempt-1/potrf7_fatbin.lto",
        FATBIN_SHA,
        false,
    );
    let call = pinned("scripts/native-mathdx7-call.cu", CALL_SHA, true);
    let linker = NvJitLink::load_from_cuda_path().unwrap();

    let first = compile_call_ltoir(&call);
    let second = compile_call_ltoir(&call);
    assert_eq!(first.nvrtc_version().0, 12, "NVRTC 主版本不符");
    assert_eq!(
        first.sha256(),
        W13_CALL_LTOIR_SHA,
        "LTOIR 摘要与 W13 证据不一致"
    );
    assert_eq!(first.bytes().len(), W13_CALL_LTOIR_BYTES);
    assert_eq!(first.sha256(), second.sha256(), "同一源两次编译摘要不一致");

    let extras = [
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
    let first_cubin = link_ltoir(&linker, 89, &first, &extras).unwrap();
    let second_cubin = link_ltoir(&linker, 89, &second, &extras).unwrap();
    assert_eq!(
        first_cubin.sha256(),
        W13_CUBIN_SHA,
        "链接产物摘要与 W13 证据不一致"
    );
    assert_eq!(
        first_cubin.sha256(),
        second_cubin.sha256(),
        "同一输入两次链接摘要不一致"
    );

    let context = CudaContext::new(0).unwrap();
    let major = context
        .attribute(driver_sys::CUdevice_attribute::CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MAJOR)
        .unwrap();
    let minor = context
        .attribute(driver_sys::CUdevice_attribute::CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MINOR)
        .unwrap();
    assert_eq!((major, minor), (8, 9), "冻结 LTO 需要 SM89");
    let module = context
        .load_module(Ptx::from_binary(first_cubin.bytes().to_vec()))
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
            "nvrtc_version": [first.nvrtc_version().0, first.nvrtc_version().1],
            "nvjitlink_version": [linker.version().0, linker.version().1],
            "call_ltoir_bytes": first.bytes().len(),
            "call_ltoir_sha256": first.sha256(),
            "cubin_bytes": first_cubin.bytes().len(),
            "cubin_sha256": first_cubin.sha256(),
            "repeat_compile_identical": first.sha256() == second.sha256(),
            "repeat_link_identical": first_cubin.sha256() == second_cubin.sha256(),
            "device_output_bytes": archive.len(),
            "device_output_sha256": output_sha,
            "launches": 8,
            "device_phase_ms": elapsed_ms,
            "gpu_cc": [major, minor],
        })
    );
    assert_eq!(archive.len(), 104_528, "设备输出归档长度不符");
    assert_eq!(output_sha, EVIDENCE_OUTPUT_SHA, "设备输出与离线证据不一致");
}

#[test]
#[ignore = "需要 CUDA_PATH 下的 NVRTC"]
fn compile_failure_reports_nvrtc_log() {
    let input = LtoirInput::new("broken.cu", "void broken( { }", 89).unwrap();
    let error = compile_ltoir(&input).unwrap_err();
    let text = error.to_string();
    assert!(text.contains("NVRTC编译失败"), "错误面不符：{text}");
    assert!(
        text.len() > "NVRTC编译失败".len(),
        "缺少真实NVRTC日志：{text}"
    );
    println!("compile_error={text}");
}
