//! 同一Rust内核生成两路PTX。
//! 驱动探针不借用CubeCL运行时。

use super::{BLOCK_THREADS, ProbeBackend, ProbeKernel};
use crate::diagnostics::ProbeError;
use cubecl_core as cubecl;
use cubecl_core::Compiler;
use cubecl_core::prelude::*;
use cudarc::nvrtc::Ptx;

pub(super) const REVISION: &str = "1f73b9f63de50a17398c1d5278e2a5f11612c7e1";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum KernelStage {
    Probe(ProbeKernel),
    ScanTotals,
    ScanOffsets,
}

impl KernelStage {
    fn entrypoint(self) -> &'static str {
        match self {
            Self::Probe(kernel) => kernel.entrypoint(),
            Self::ScanTotals => "cubecl_scan_totals_probe",
            Self::ScanOffsets => "cubecl_scan_offsets_probe",
        }
    }
}

#[cube]
fn affine(input: &[f32], output: &mut [f32]) {
    if ABSOLUTE_POS < input.len() {
        output[ABSOLUTE_POS] = input[ABSOLUTE_POS] * 2.0 + 1.0;
    }
}

#[cube]
fn atomic_sum(input: &[u32], output: &mut [Atomic<u32>]) {
    if ABSOLUTE_POS < input.len() {
        output[0].fetch_add(input[ABSOLUTE_POS]);
    }
}

#[cube]
fn float_atomic_sum(input: &[f32], output: &mut [Atomic<f32>]) {
    if ABSOLUTE_POS < input.len() {
        output[0].fetch_add(input[ABSOLUTE_POS]);
    }
}

#[cube]
fn block_reduce(input: &[u32], output: &mut [u32]) {
    let mut shared = Shared::<[u32]>::new_slice(128usize);
    let unit = UNIT_POS as usize;
    let mut value = 0u32;
    if ABSOLUTE_POS < input.len() {
        value = input[ABSOLUTE_POS];
    }
    shared[unit] = value;
    // 尾块线程同样参加屏障。
    sync_cube();
    let mut stride = 64usize;
    while stride > 0 {
        if unit < stride {
            shared[unit] += shared[unit + stride];
        }
        sync_cube();
        stride /= 2;
    }
    if UNIT_POS == 0 {
        output[CUBE_POS] = shared[0];
    }
}

#[cube]
fn block_scan(input: &[u32], output: &mut [u32]) {
    let mut shared = Shared::<[u32]>::new_slice(128usize);
    let unit = UNIT_POS as usize;
    let mut value = 0u32;
    if ABSOLUTE_POS < input.len() {
        value = input[ABSOLUTE_POS];
    }
    shared[unit] = value;
    sync_cube();
    let mut offset = 1usize;
    while offset < 128 {
        let mut preceding = 0u32;
        if unit >= offset {
            preceding = shared[unit - offset];
        }
        // 所有线程先读，再统一写入。
        sync_cube();
        shared[unit] += preceding;
        sync_cube();
        offset *= 2;
    }
    if ABSOLUTE_POS < input.len() {
        output[ABSOLUTE_POS] = shared[unit];
    }
}

#[cube]
fn scan_totals(input: &[u32], output: &mut [u32]) {
    if ABSOLUTE_POS < output.len() {
        let mut last = (ABSOLUTE_POS + 1) * 128 - 1;
        if last >= input.len() {
            last = input.len() - 1;
        }
        output[ABSOLUTE_POS] = input[last];
    }
}

#[cube]
fn scan_offsets(input: &[u32], output: &mut [u32]) {
    if ABSOLUTE_POS < output.len() && CUBE_POS > 0 {
        output[ABSOLUTE_POS] += input[CUBE_POS - 1];
    }
}

#[cube]
fn control_flow(input: &[u32], output: &mut [u32]) {
    if ABSOLUTE_POS < input.len() {
        let value = input[ABSOLUTE_POS];
        let limit = value % 7 + 1;
        let mut iteration = 0u32;
        let mut total = 0u32;
        while iteration < limit {
            if value & 1 == 0 {
                total += value + iteration;
            } else {
                total += 2 * value - iteration;
            }
            iteration += 1;
        }
        output[ABSOLUTE_POS] = total;
    }
}

#[cube]
fn small_solve(input: &[f32], output: &mut [f32]) {
    if ABSOLUTE_POS < input.len() / 2 {
        let base = ABSOLUTE_POS * 2;
        // 固定2x2正定矩阵消元。
        let rhs_x = input[base];
        let rhs_y = input[base + 1];
        let y = (rhs_y - 0.25 * rhs_x) / 1.75;
        let x = (rhs_x - y) / 4.0;
        output[base] = x;
        output[base + 1] = y;
    }
}

fn definition(kernel: KernelStage) -> KernelDefinition {
    // 固定128线程与u32索引。
    // 两个缓冲仅携带静态长度。
    let settings = KernelSettings::new(
        *CubeDim::new_1d(BLOCK_THREADS as u32),
        ExecutionMode::Checked,
        AddressType::U32,
    )
    .kernel_name(kernel.entrypoint());
    let mut builder = KernelBuilder::new(settings);
    let arg = BufferCompilationArg { inplace: None };
    match kernel {
        KernelStage::Probe(ProbeKernel::Affine | ProbeKernel::SmallSolve) => {
            let input = <[f32] as LaunchArg>::expand(&arg, &mut builder);
            let mut output = <[f32] as LaunchArg>::expand(&arg, &mut builder);
            if kernel == KernelStage::Probe(ProbeKernel::Affine) {
                affine::expand(&builder.scope, &input, &mut output);
            } else {
                small_solve::expand(&builder.scope, &input, &mut output);
            }
        }
        KernelStage::Probe(ProbeKernel::AtomicSum) => {
            let input = <[u32] as LaunchArg>::expand(&arg, &mut builder);
            let mut output = <[Atomic<u32>] as LaunchArg>::expand(&arg, &mut builder);
            atomic_sum::expand(&builder.scope, &input, &mut output);
        }
        KernelStage::Probe(ProbeKernel::FloatAtomicSum) => {
            let input = <[f32] as LaunchArg>::expand(&arg, &mut builder);
            let mut output = <[Atomic<f32>] as LaunchArg>::expand(&arg, &mut builder);
            float_atomic_sum::expand(&builder.scope, &input, &mut output);
        }
        _ => {
            let input = <[u32] as LaunchArg>::expand(&arg, &mut builder);
            let mut output = <[u32] as LaunchArg>::expand(&arg, &mut builder);
            match kernel {
                KernelStage::Probe(ProbeKernel::BlockReduce) => {
                    block_reduce::expand(&builder.scope, &input, &mut output)
                }
                KernelStage::Probe(ProbeKernel::BlockScan | ProbeKernel::GlobalScan) => {
                    block_scan::expand(&builder.scope, &input, &mut output)
                }
                KernelStage::ScanTotals => scan_totals::expand(&builder.scope, &input, &mut output),
                KernelStage::ScanOffsets => {
                    scan_offsets::expand(&builder.scope, &input, &mut output)
                }
                KernelStage::Probe(ProbeKernel::ControlFlow) => {
                    control_flow::expand(&builder.scope, &input, &mut output)
                }
                _ => unreachable!(),
            }
        }
    }
    builder.build()
}

pub(super) struct CompiledProbe {
    pub ptx: Ptx,
    pub entrypoint: String,
    pub shared_memory: u32,
}

fn compilation_error(stage: &'static str, detail: impl ToString) -> ProbeError {
    ProbeError::Compilation {
        stage,
        detail: detail.to_string(),
    }
}

pub(super) fn compile(
    backend: ProbeBackend,
    kernel: KernelStage,
    sm: u32,
    driver: i32,
) -> Result<CompiledProbe, ProbeError> {
    match backend {
        #[cfg(feature = "cubecl-cpp-probe")]
        ProbeBackend::CubeClCpp => compile_cpp(kernel, sm),
        #[cfg(feature = "cubecl-llvm-probe")]
        ProbeBackend::CubeClLlvm => compile_llvm(kernel, sm, driver),
        _ => {
            let _ = (kernel, sm, driver);
            Err(ProbeError::FeatureDisabled(backend.required_feature()))
        }
    }
}

#[cfg(feature = "cubecl-cpp-probe")]
fn cpp_source(kernel: KernelStage) -> Result<cubecl_cpp::ComputeKernel, ProbeError> {
    use cubecl_cpp::{
        shared::{CompilationOptions, CppCompiler},
        target::Cuda,
    };
    // 不用默认CudaBackend，避免特性合并改路。
    CppCompiler::<Cuda>::default()
        .compile(definition(kernel), &CompilationOptions::default())
        .map_err(|error| compilation_error("cubecl-cpp", error))
}

#[cfg(feature = "cubecl-cpp-probe")]
fn compile_cpp(kernel: KernelStage, sm: u32) -> Result<CompiledProbe, ProbeError> {
    use cudarc::nvrtc::{CompileOptions, compile_ptx_with_opts, sys};
    // SAFETY: 仅探测编译器库，不传用户指针。
    if !unsafe { sys::is_culib_present() } {
        return Err(ProbeError::NvrtcUnavailable);
    }
    let entrypoint = kernel.entrypoint();
    let kernel = cpp_source(kernel)?;
    let mut include_paths = Vec::new();
    if let Some(root) = std::env::var_os("CUDA_PATH") {
        include_paths.push(
            std::path::Path::new(&root)
                .join("include")
                .to_string_lossy()
                .into_owned(),
        );
    }
    let ptx = compile_ptx_with_opts(
        kernel.source,
        CompileOptions {
            fmad: Some(false),
            use_fast_math: Some(false),
            include_paths,
            options: vec![
                "--std=c++17".into(),
                format!("--gpu-architecture=compute_{sm}"),
            ],
            name: Some(format!("{entrypoint}.cu")),
            ..Default::default()
        },
    )
    .map_err(|error| compilation_error("nvrtc", format!("{error:?}")))?;
    Ok(CompiledProbe {
        ptx,
        entrypoint: entrypoint.into(),
        shared_memory: kernel.shared_memory_size as u32,
    })
}

#[cfg(feature = "cubecl-llvm-probe")]
fn compile_llvm(kernel: KernelStage, sm: u32, driver: i32) -> Result<CompiledProbe, ProbeError> {
    use cubecl_core::ir::nvidia::SmArch;
    use cubecl_llvm::{
        LlvmTarget, PlironArtifact, PlironCompiler, PlironOptions, nvptx::ptx_version::PtxVersion,
    };
    let mut compiler = PlironCompiler {
        target: LlvmTarget::Nvptx,
    };
    let options = PlironOptions {
        sm_arch: Some(SmArch::new(sm, false)),
        ptx_version: PtxVersion::for_driver(driver),
        grid_constants: false,
        ..Default::default()
    };
    let artifact = compiler
        .compile(definition(kernel), &options)
        .map_err(|error| compilation_error("cubecl-llvm", error))?;
    let PlironArtifact::NvptxCode(module) = artifact else {
        return Err(compilation_error("cubecl-llvm", "编译器未返回PTX"));
    };
    // 先检查NUL，避免越界读取编译产物。
    let bytes: Vec<u8> = module.ptx.into_iter().map(|byte| byte as u8).collect();
    let ptx = std::ffi::CStr::from_bytes_with_nul(&bytes)
        .map_err(|error| compilation_error("llvm-ptx", error))?
        .to_str()
        .map_err(|error| compilation_error("llvm-ptx", error))?
        .to_owned();
    Ok(CompiledProbe {
        ptx: Ptx::from_src(ptx),
        entrypoint: module.entrypoint,
        shared_memory: module.shared_memory_size as u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stages() -> impl Iterator<Item = KernelStage> {
        ProbeKernel::ALL
            .into_iter()
            .map(KernelStage::Probe)
            .chain([KernelStage::ScanTotals, KernelStage::ScanOffsets])
    }

    #[test]
    fn linker_retains_slice_interface_registration() {
        use cubecl_core::ir::{
            interfaces::memory_slot::DestructurableTypeInterface, types::aggregate::SliceType,
        };
        assert!(
            cubecl_core::ir::pliron::utils::trait_cast::impls_trait_static::<
                SliceType,
                dyn DestructurableTypeInterface,
            >()
        );
    }

    #[test]
    fn freezes_metadata_layout_and_launch_dimensions() {
        for kind in stages() {
            let kernel = definition(kind);
            assert!(kernel.info.scalars.is_empty());
            assert!(!kernel.info.has_dynamic_meta);
            assert_eq!(kernel.info.metadata.num_meta(), 2);
            assert_eq!(kernel.info.metadata.buffer_len_index(0), 0);
            assert_eq!(kernel.info.metadata.buffer_len_index(1), 1);
            assert_eq!(kernel.info.dynamic_meta_offset, 8);
            assert_eq!(kernel.settings.cube_dim, *CubeDim::new_1d(128));
            assert_eq!(kernel.settings.address_type, AddressType::U32);
        }
    }

    #[cfg(feature = "cubecl-cpp-probe")]
    #[test]
    fn lowers_rust_kernel_to_cpp_without_gpu() {
        for kind in stages() {
            let kernel = cpp_source(kind).unwrap();
            assert!(kernel.source.contains(kind.entrypoint()));
            assert!(kernel.source.contains("info_st"));
            assert_eq!(kernel.buffers.len(), 2);
            if matches!(
                kind,
                KernelStage::Probe(
                    ProbeKernel::BlockReduce | ProbeKernel::BlockScan | ProbeKernel::GlobalScan
                )
            ) {
                assert!(kernel.source.contains("__syncthreads"));
                assert!(kernel.source.contains("__shared__"));
            }
            if kind == KernelStage::Probe(ProbeKernel::AtomicSum) {
                assert!(kernel.source.contains("atom.relaxed.add.u32"));
            }
            if kind == KernelStage::Probe(ProbeKernel::FloatAtomicSum) {
                assert!(kernel.source.contains("atom.relaxed.add.f32"));
            }
        }
    }

    #[cfg(feature = "cubecl-llvm-probe")]
    #[test]
    fn lowers_all_rust_kernels_to_ptx_without_gpu() {
        for kind in stages() {
            // 固定编译目标，不访问驱动。
            let kernel = compile_llvm(kind, 89, 13020).unwrap();
            let ptx = kernel.ptx.to_src();
            assert_eq!(kernel.entrypoint, kind.entrypoint());
            assert!(ptx.contains(kind.entrypoint()));
            if matches!(
                kind,
                KernelStage::Probe(
                    ProbeKernel::BlockReduce | ProbeKernel::BlockScan | ProbeKernel::GlobalScan
                )
            ) {
                assert!(ptx.contains("bar.sync"));
                assert!(ptx.contains(".shared"));
            }
            if matches!(
                kind,
                KernelStage::Probe(ProbeKernel::AtomicSum | ProbeKernel::FloatAtomicSum)
            ) {
                assert!(ptx.contains("atom.") || ptx.contains("red."));
            }
        }
    }
}
