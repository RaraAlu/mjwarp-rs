//! MathDx libmathdx 0.3.1 求解器 LTO 生成与冻结产物的字节对照。
//! NVRTC 12.9.86 从显式 pinned 组件解析（`target/toolchains/cuda-12.9.1-nvrtc/bin`，
//! 可用`MATHDX_NVRTC_DIR`覆盖），与`CUDA_PATH`解耦；缺组件时显式失败。
//! 还需要组件 `target/toolchains/libmathdx-0.3.1/`（2026-10-24-mathdx-libmathdx-prep）。
//! libmathdx 会把进程工作目录写进 LTOIR；本探针在冻结证据的工作目录
//! （默认`C:\Rust\mjwarp-rs`，可用`MATHDX_EVIDENCE_CWD`覆盖）下生成，结束后恢复。
//! 本测试不调用 warp.dll，也不启动 Python。
#![cfg(feature = "cuda-probe")]

use std::path::PathBuf;

use cudarc::driver::{CudaContext, LaunchConfig, PushKernelArg, sys as driver_sys};
use cudarc::nvrtc::Ptx;
use mjwarp_rs::runtime::launcher::{
    LinkInput, LinkInputKind, LtoirInput, MathDx, NvJitLink, SolverSpec, compile_ltoir, link_ltoir,
};
use sha2::{Digest, Sha256};

const LTOIR_BYTES: usize = 1_034_872;
const LTOIR_SHA: &str = "98c01ce64f55d31af5d4eb001c1e9fc4c8456173a9f0bbe97be479c804a981eb";
const FATBIN_BYTES: usize = 506_960;
const FATBIN_SHA: &str = "c00d6412780b18f67e11a1d93fc34d51db9cdd687f26ec7bc926efbff5ddbd1f";
const CALL_SHA: &str = "abfc8c777014708ed9b33c7a80798e8d2160f102f49a21d01fbf203911968853";
/// 冻结证据（2026-10-14）的链接产物与设备输出摘要。
const CUBIN_BYTES: usize = 2_086_544;
const CUBIN_EVIDENCE_SHA: &str = "de660b5d5095516549c053bf51e13c05b9bc55a3dfe5cfeb0a0833c068be55ae";
const OUTPUT_BYTES: usize = 104_528;
const OUTPUT_SHA: &str = "51e1a99b11990d259411de58816a6344cec85f224c6a541fa088030749126d3f";

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

fn mathdx_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/toolchains/libmathdx-0.3.1/bin")
}

/// pinned NVRTC 12.9.86 的 bin 目录；显式`MATHDX_NVRTC_DIR`优先。
fn nvrtc_dir() -> PathBuf {
    std::env::var_os("MATHDX_NVRTC_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target/toolchains/cuda-12.9.1-nvrtc/bin")
        })
}

fn load_mathdx() -> MathDx {
    let directory = mathdx_dir();
    assert!(
        directory.is_dir(),
        "缺少 libmathdx 组件：{}（先运行 scripts/prepare-cuda-probe.ps1）",
        directory.display()
    );
    let nvrtc = nvrtc_dir();
    assert!(
        nvrtc.join("nvrtc64_120_0.dll").is_file(),
        "缺少pinned NVRTC 12.9.86组件：{}（先运行 scripts/prepare-cuda-probe.ps1）",
        nvrtc.display()
    );
    MathDx::load_from_dir_with_nvrtc(&directory, &nvrtc).unwrap()
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

/// 冻结证据（2026-10-14）在`C:\Rust\mjwarp-rs`下生成；libmathdx 会把工作目录写进 LTOIR。
/// 本守卫只用于测试：生成期间切到该目录，结束或 panic 时恢复。
struct EvidenceWorkingDirectory(PathBuf);

impl Drop for EvidenceWorkingDirectory {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.0);
    }
}

fn enter_evidence_working_directory() -> EvidenceWorkingDirectory {
    let original = std::env::current_dir().expect("当前工作目录");
    let target = std::env::var_os("MATHDX_EVIDENCE_CWD")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Rust\mjwarp-rs"));
    assert!(
        target.is_dir(),
        "冻结证据工作目录缺失：{}（libmathdx 会把工作目录写进 LTOIR，可用 MATHDX_EVIDENCE_CWD 指定）",
        target.display()
    );
    std::env::set_current_dir(&target).expect("切换到冻结证据工作目录");
    EvidenceWorkingDirectory(original)
}

#[test]
#[ignore = "需要 libmathdx 组件、CUDA_PATH（NVRTC+nvJitLink）与 SM89 GPU"]
fn solver_lto_matches_frozen_evidence_and_device_output() {
    let _cwd = enter_evidence_working_directory();
    let mathdx = load_mathdx();
    assert_eq!(mathdx.version(), (0, 3, 1));
    let spec = SolverSpec::potrf_7d_sm89();
    let first = mathdx.generate_solver_lto(&spec).unwrap();
    let second = mathdx.generate_solver_lto(&spec).unwrap();
    assert_eq!(first.ltoir().len(), LTOIR_BYTES, "LTOIR 字节数不符");
    assert_eq!(first.ltoir_sha256(), LTOIR_SHA, "LTOIR 摘要不符");
    assert_eq!(first.fatbin().len(), FATBIN_BYTES, "FATBIN 字节数不符");
    assert_eq!(first.fatbin_sha256(), FATBIN_SHA, "FATBIN 摘要不符");
    assert_eq!(
        first.ltoir_sha256(),
        second.ltoir_sha256(),
        "重复生成不一致"
    );
    assert_eq!(
        first.fatbin_sha256(),
        second.fatbin_sha256(),
        "重复生成不一致"
    );

    let call = pinned("scripts/native-mathdx7-call.cu", CALL_SHA, true);
    let input = LtoirInput::new(
        "native-mathdx7-call.cu",
        std::str::from_utf8(&call).unwrap(),
        89,
    )
    .unwrap();
    let call_ltoir = compile_ltoir(&input).unwrap();
    assert!(!call_ltoir.bytes().is_empty(), "调用 LTOIR 为空");
    // 证明与 CUDA_PATH 解耦：NVRTC 必须绑定到 pinned 12.9（G01 环境 CUDA_PATH 为 12.8.1）。
    assert_eq!(
        call_ltoir.nvrtc_version().0,
        12,
        "调用 LTOIR 未使用 NVRTC 12.x：{:?}",
        call_ltoir.nvrtc_version()
    );
    assert_eq!(
        call_ltoir.nvrtc_version().1,
        9,
        "调用 LTOIR 未使用 pinned NVRTC 12.9：{:?}",
        call_ltoir.nvrtc_version()
    );

    let linker = NvJitLink::load_from_cuda_path().unwrap();
    let extras = [
        LinkInput {
            kind: LinkInputKind::Ltoir,
            name: "potrf7.lto",
            bytes: first.ltoir(),
        },
        LinkInput {
            kind: LinkInputKind::Fatbin,
            name: "potrf7_fatbin.lto",
            bytes: first.fatbin(),
        },
    ];
    let cubin = link_ltoir(&linker, 89, &call_ltoir, &extras).unwrap();
    let cubin_again = link_ltoir(&linker, 89, &call_ltoir, &extras).unwrap();
    assert_eq!(cubin.bytes().len(), CUBIN_BYTES, "CUBIN 字节数不符");
    assert_eq!(
        cubin.sha256(),
        CUBIN_EVIDENCE_SHA,
        "CUBIN 摘要与冻结证据不符"
    );
    assert_eq!(cubin.sha256(), cubin_again.sha256(), "重复链接不一致");

    let context = CudaContext::new(0).unwrap();
    let major = context
        .attribute(driver_sys::CUdevice_attribute::CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MAJOR)
        .unwrap();
    let minor = context
        .attribute(driver_sys::CUdevice_attribute::CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MINOR)
        .unwrap();
    assert_eq!((major, minor), (8, 9), "冻结 LTO 需要 SM89");
    let module = context
        .load_module(Ptx::from_binary(cubin.bytes().to_vec()))
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
            "mathdx_version": [first.mathdx_version().0, first.mathdx_version().1, first.mathdx_version().2],
            "ltoir_bytes": first.ltoir().len(),
            "ltoir_sha256": first.ltoir_sha256(),
            "fatbin_bytes": first.fatbin().len(),
            "fatbin_sha256": first.fatbin_sha256(),
            "repeat_generate_identical": first.ltoir_sha256() == second.ltoir_sha256(),
            "call_ltoir_bytes": call_ltoir.bytes().len(),
            "call_ltoir_sha256": call_ltoir.sha256(),
            "call_nvrtc_version": [call_ltoir.nvrtc_version().0, call_ltoir.nvrtc_version().1],
            "evidence_cwd": std::env::current_dir().unwrap().to_string_lossy(),
            "cubin_bytes": cubin.bytes().len(),
            "cubin_sha256": cubin.sha256(),
            "repeat_link_identical": cubin.sha256() == cubin_again.sha256(),
            "device_output_bytes": archive.len(),
            "device_output_sha256": output_sha,
            "launches": 8,
            "device_phase_ms": elapsed_ms,
            "gpu_cc": [major, minor],
        })
    );
    assert_eq!(archive.len(), OUTPUT_BYTES, "设备输出归档长度不符");
    assert_eq!(output_sha, OUTPUT_SHA, "设备输出与离线证据不一致");
}

#[test]
#[ignore = "需要 libmathdx 组件"]
fn descriptor_rejection_reports_error() {
    let mathdx = load_mathdx();
    let mut spec = SolverSpec::potrf_7d_sm89();
    spec.n = 8; // POTRF 需要方阵；非方阵应在描述符阶段被拒绝。
    let error = mathdx.generate_solver_lto(&spec).unwrap_err();
    let text = error.to_string();
    assert!(text.contains("MathDx"), "错误面不符：{text}");
    println!("descriptor_error={text}");
}
