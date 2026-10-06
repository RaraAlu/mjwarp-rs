//! 同一Rust内核生成两路PTX。
//! 驱动探针不借用CubeCL运行时。

use super::ProbeBackend;
use crate::diagnostics::ProbeError;
use cubecl_core as cubecl;
use cubecl_core::Compiler;
use cubecl_core::prelude::*;
use cudarc::nvrtc::Ptx;

pub(super) const REVISION: &str = "1f73b9f63de50a17398c1d5278e2a5f11612c7e1";
const ENTRYPOINT: &str = "cubecl_affine_probe";

#[cube]
fn affine(input: &[f32], output: &mut [f32]) {
    if ABSOLUTE_POS < input.len() {
        output[ABSOLUTE_POS] = input[ABSOLUTE_POS] * 2.0 + 1.0;
    }
}

fn definition() -> KernelDefinition {
    // 固定128线程与u32索引。
    // 两个缓冲仅携带静态长度。
    let settings = KernelSettings::new(
        *CubeDim::new_1d(128),
        ExecutionMode::Checked,
        AddressType::U32,
    )
    .kernel_name(ENTRYPOINT);
    let mut builder = KernelBuilder::new(settings);
    let arg = BufferCompilationArg { inplace: None };
    let input = <[f32] as LaunchArg>::expand(&arg, &mut builder);
    let mut output = <[f32] as LaunchArg>::expand(&arg, &mut builder);
    affine::expand(&builder.scope, &input, &mut output);
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
    sm: u32,
    driver: i32,
) -> Result<CompiledProbe, ProbeError> {
    match backend {
        #[cfg(feature = "cubecl-cpp-probe")]
        ProbeBackend::CubeClCpp => compile_cpp(sm),
        #[cfg(feature = "cubecl-llvm-probe")]
        ProbeBackend::CubeClLlvm => compile_llvm(sm, driver),
        _ => {
            let _ = (sm, driver);
            Err(ProbeError::FeatureDisabled(backend.required_feature()))
        }
    }
}

#[cfg(feature = "cubecl-cpp-probe")]
fn cpp_source() -> Result<cubecl_cpp::ComputeKernel, ProbeError> {
    use cubecl_cpp::{
        shared::{CompilationOptions, CppCompiler},
        target::Cuda,
    };
    // 不用默认CudaBackend，避免特性合并改路。
    CppCompiler::<Cuda>::default()
        .compile(definition(), &CompilationOptions::default())
        .map_err(|error| compilation_error("cubecl-cpp", error))
}

#[cfg(feature = "cubecl-cpp-probe")]
fn compile_cpp(sm: u32) -> Result<CompiledProbe, ProbeError> {
    use cudarc::nvrtc::{CompileOptions, compile_ptx_with_opts, sys};
    // SAFETY: 仅探测编译器库，不传用户指针。
    if !unsafe { sys::is_culib_present() } {
        return Err(ProbeError::NvrtcUnavailable);
    }
    let kernel = cpp_source()?;
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
            name: Some("cubecl_affine_probe.cu".into()),
            ..Default::default()
        },
    )
    .map_err(|error| compilation_error("nvrtc", format!("{error:?}")))?;
    Ok(CompiledProbe {
        ptx,
        entrypoint: ENTRYPOINT.into(),
        shared_memory: kernel.shared_memory_size as u32,
    })
}

#[cfg(feature = "cubecl-llvm-probe")]
fn compile_llvm(sm: u32, driver: i32) -> Result<CompiledProbe, ProbeError> {
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
        .compile(definition(), &options)
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
        let kernel = definition();
        assert!(kernel.info.scalars.is_empty());
        assert!(!kernel.info.has_dynamic_meta);
        assert_eq!(kernel.info.metadata.num_meta(), 2);
        assert_eq!(kernel.info.metadata.buffer_len_index(0), 0);
        assert_eq!(kernel.info.metadata.buffer_len_index(1), 1);
        assert_eq!(kernel.info.dynamic_meta_offset, 8);
        assert_eq!(kernel.settings.cube_dim, *CubeDim::new_1d(128));
        assert_eq!(kernel.settings.address_type, AddressType::U32);
    }

    #[cfg(feature = "cubecl-cpp-probe")]
    #[test]
    fn lowers_rust_kernel_to_cpp_without_gpu() {
        let kernel = cpp_source().unwrap();
        assert!(kernel.source.contains(ENTRYPOINT));
        assert!(kernel.source.contains("info_st"));
        assert_eq!(kernel.buffers.len(), 2);
        assert_eq!(kernel.shared_memory_size, 0);
    }
}
