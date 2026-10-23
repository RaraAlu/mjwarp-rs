//! 内核来源路线 MathDx LTO 入口的真实对照。
//! 使用新路线 API 生成 7 维 POTRF 的 LTOIR/FATBIN 并链接，对照冻结证据。
//! NVRTC 12.9.86 从显式 pinned 组件解析；libmathdx 组件来自 prepare 脚本。
//! libmathdx 会把进程工作目录写进 LTOIR；本探针在冻结证据工作目录
//! （默认`C:\Rust\mjwarp-rs`，可用`MATHDX_EVIDENCE_CWD`覆盖）下生成，结束后恢复。
#![cfg(feature = "cuda-probe")]

use std::path::PathBuf;

use cudarc::driver::{CudaContext, LaunchConfig, PushKernelArg, sys as driver_sys};
use cudarc::nvrtc::Ptx;
use mjwarp_rs::runtime::launcher::{
    KernelSourceRoute, LtoirInput, MathDx, NvJitLink, RouteOptIn, SolverSpec,
    build_mathdx_lto_kernel,
};
use sha2::{Digest, Sha256};

const CALL_SHA: &str = "abfc8c777014708ed9b33c7a80798e8d2160f102f49a21d01fbf203911968853";
const CALL_LTOIR_BYTES: usize = 3_632;
const CALL_LTOIR_SHA: &str = "a323db381e0793b03837c95179965eb90aeb44c110eae042f16ede1d587da953";
const SOLVER_LTOIR_BYTES: usize = 1_034_872;
const SOLVER_LTOIR_SHA: &str = "98c01ce64f55d31af5d4eb001c1e9fc4c8456173a9f0bbe97be479c804a981eb";
const SOLVER_FATBIN_BYTES: usize = 506_960;
const SOLVER_FATBIN_SHA: &str = "c00d6412780b18f67e11a1d93fc34d51db9cdd687f26ec7bc926efbff5ddbd1f";
const CUBIN_BYTES: usize = 2_086_544;
const CUBIN_SHA: &str = "de660b5d5095516549c053bf51e13c05b9bc55a3dfe5cfeb0a0833c068be55ae";
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

fn nvrtc_dir() -> PathBuf {
    std::env::var_os("MATHDX_NVRTC_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target/toolchains/cuda-12.9.1-nvrtc/bin")
        })
}

fn load_mathdx() -> MathDx {
    let directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/toolchains/libmathdx-0.3.1/bin");
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
#[ignore = "需要 libmathdx 组件、pinned NVRTC 12.9.86、CUDA_PATH 与 SM89 GPU"]
fn mathdx_route_reproduces_frozen_evidence() {
    let _cwd = enter_evidence_working_directory();
    let mathdx = load_mathdx();
    let linker = NvJitLink::load_from_cuda_path().unwrap();
    let call_source = pinned("scripts/native-mathdx7-call.cu", CALL_SHA, true);
    let call = LtoirInput::new(
        "native-mathdx7-call.cu",
        std::str::from_utf8(&call_source).unwrap(),
        89,
    )
    .unwrap();
    let solver = SolverSpec::potrf_7d_sm89();
    let opt_in = RouteOptIn::enable(KernelSourceRoute::MathDxLto);

    // 默认不启用：未显式 opt-in 必须拒绝。
    let error = build_mathdx_lto_kernel(RouteOptIn::none(), &mathdx, &linker, &call, &solver)
        .unwrap_err()
        .to_string();
    assert!(error.contains("路线未启用"), "错误面不符：{error}");

    let first = build_mathdx_lto_kernel(opt_in, &mathdx, &linker, &call, &solver).unwrap();
    let second = build_mathdx_lto_kernel(opt_in, &mathdx, &linker, &call, &solver).unwrap();
    assert_eq!(first.key().route(), KernelSourceRoute::MathDxLto);
    assert_eq!(
        first.key().sha256(),
        second.key().sha256(),
        "同输入键不稳定"
    );
    assert_eq!(first.key().nvrtc(), Some((12, 9)));
    assert_eq!(first.key().nvjitlink(), Some((12, 9)));
    assert_eq!(first.key().libmathdx(), Some((0, 3, 1)));
    assert_eq!(first.metadata(), second.metadata(), "产物元数据不稳定");
    first.verify().unwrap();

    let metadata = first.metadata();
    assert_eq!(metadata.call_ltoir_bytes, CALL_LTOIR_BYTES);
    assert_eq!(metadata.call_ltoir_sha256, CALL_LTOIR_SHA);
    assert_eq!(metadata.solver_ltoir_bytes, SOLVER_LTOIR_BYTES);
    assert_eq!(metadata.solver_ltoir_sha256, SOLVER_LTOIR_SHA);
    assert_eq!(metadata.solver_fatbin_bytes, SOLVER_FATBIN_BYTES);
    assert_eq!(metadata.solver_fatbin_sha256, SOLVER_FATBIN_SHA);
    assert_eq!(metadata.cubin_bytes, CUBIN_BYTES);
    assert_eq!(metadata.cubin_sha256, CUBIN_SHA);

    let context = CudaContext::new(0).unwrap();
    let major = context
        .attribute(driver_sys::CUdevice_attribute::CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MAJOR)
        .unwrap();
    let minor = context
        .attribute(driver_sys::CUdevice_attribute::CU_DEVICE_ATTRIBUTE_COMPUTE_CAPABILITY_MINOR)
        .unwrap();
    assert_eq!((major, minor), (8, 9), "冻结 LTO 需要 SM89");
    let module = context
        .load_module(Ptx::from_binary(first.cubin().to_vec()))
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
            "route": first.key().route().as_str(),
            "route_key_sha256": first.key().sha256(),
            "metadata": first.metadata(),
            "repeat_build_identical": first.metadata() == second.metadata(),
            "device_output_bytes": archive.len(),
            "device_output_sha256": output_sha,
            "launches": 8,
            "device_phase_ms": elapsed_ms,
            "gpu_cc": [major, minor],
            "evidence_cwd": std::env::current_dir().unwrap().to_string_lossy(),
        })
    );
    assert_eq!(archive.len(), OUTPUT_BYTES, "设备输出归档长度不符");
    assert_eq!(output_sha, OUTPUT_SHA, "设备输出与离线证据不一致");
}
