//! 探针配置与编译路线选择。

use super::{BLOCK_THREADS, MAX_ELEMENTS, MAX_REPLAYS, buffer_bytes};
use crate::diagnostics::ProbeError;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProbeKernel {
    #[default]
    Affine,
    AtomicSum,
    FloatAtomicSum,
    BlockReduce,
    BlockScan,
    GlobalScan,
    ControlFlow,
    SmallSolve,
}

impl ProbeKernel {
    pub const ALL: [Self; 8] = [
        Self::Affine,
        Self::AtomicSum,
        Self::FloatAtomicSum,
        Self::BlockReduce,
        Self::BlockScan,
        Self::GlobalScan,
        Self::ControlFlow,
        Self::SmallSolve,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Affine => "affine",
            Self::AtomicSum => "atomic-sum",
            Self::FloatAtomicSum => "float-atomic-sum",
            Self::BlockReduce => "block-reduce",
            Self::BlockScan => "block-scan",
            Self::GlobalScan => "global-scan",
            Self::ControlFlow => "control-flow",
            Self::SmallSolve => "small-solve",
        }
    }

    pub fn parse(value: &str) -> Result<Self, ProbeError> {
        Self::ALL
            .into_iter()
            .find(|kernel| kernel.name() == value)
            .ok_or(ProbeError::InvalidArgument("未知探针内核"))
    }

    pub(super) fn output_elements(self, elements: usize) -> Result<usize, ProbeError> {
        match self {
            Self::AtomicSum | Self::FloatAtomicSum => Ok(1),
            Self::BlockReduce => Ok(elements.div_ceil(BLOCK_THREADS)),
            Self::SmallSolve => elements
                .checked_mul(2)
                .ok_or(ProbeError::InvalidArgument("矩阵缓冲长度溢出")),
            _ => Ok(elements),
        }
    }

    #[cfg(feature = "cuda-probe")]
    pub(super) fn entrypoint(self) -> &'static str {
        match self {
            Self::Affine => "cubecl_affine_probe",
            Self::AtomicSum => "cubecl_atomic_sum_probe",
            Self::FloatAtomicSum => "cubecl_float_atomic_sum_probe",
            Self::BlockReduce => "cubecl_block_reduce_probe",
            Self::BlockScan => "cubecl_block_scan_probe",
            Self::GlobalScan => "cubecl_global_scan_probe",
            Self::ControlFlow => "cubecl_control_flow_probe",
            Self::SmallSolve => "cubecl_small_solve_probe",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProbeBackend {
    #[default]
    NativePtx,
    CubeClCpp,
    CubeClLlvm,
}

impl ProbeBackend {
    pub fn name(self) -> &'static str {
        match self {
            Self::NativePtx => "native-ptx",
            Self::CubeClCpp => "cubecl-cpp",
            Self::CubeClLlvm => "cubecl-llvm",
        }
    }

    pub fn parse(value: &str) -> Result<Self, ProbeError> {
        match value {
            "native-ptx" => Ok(Self::NativePtx),
            "cubecl-cpp" => Ok(Self::CubeClCpp),
            "cubecl-llvm" => Ok(Self::CubeClLlvm),
            _ => Err(ProbeError::InvalidArgument("未知编译路线")),
        }
    }

    pub(super) fn required_feature(self) -> &'static str {
        match self {
            Self::NativePtx => "cuda-probe",
            Self::CubeClCpp => "cubecl-cpp-probe",
            Self::CubeClLlvm => "cubecl-llvm-probe",
        }
    }

    pub(super) fn enabled(self) -> bool {
        match self {
            Self::NativePtx => cfg!(feature = "cuda-probe"),
            Self::CubeClCpp => cfg!(feature = "cubecl-cpp-probe"),
            Self::CubeClLlvm => cfg!(feature = "cubecl-llvm-probe"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ProbeConfig {
    pub backend: ProbeBackend,
    pub kernel: ProbeKernel,
    pub device: usize,
    pub elements: usize,
    pub replays: usize,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            backend: ProbeBackend::NativePtx,
            kernel: ProbeKernel::Affine,
            device: 0,
            elements: 257,
            replays: 3,
        }
    }
}

impl ProbeConfig {
    pub fn validate(&self) -> Result<(), ProbeError> {
        if !(1..=MAX_ELEMENTS).contains(&self.elements) {
            return Err(ProbeError::InvalidArgument("元素数须在1至1048576"));
        }
        if !(1..=MAX_REPLAYS).contains(&self.replays) {
            return Err(ProbeError::InvalidArgument("重放数须在1至1000"));
        }
        if self.backend == ProbeBackend::NativePtx && self.kernel != ProbeKernel::Affine {
            return Err(ProbeError::InvalidArgument("原生PTX只支持affine探针"));
        }
        buffer_bytes(self.kernel.output_elements(self.elements)?)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ResourceProbeConfig {
    pub backend: ProbeBackend,
    pub device: usize,
    pub elements: usize,
}

impl Default for ResourceProbeConfig {
    fn default() -> Self {
        let config = ProbeConfig::default();
        Self {
            backend: config.backend,
            device: config.device,
            elements: config.elements,
        }
    }
}

impl ResourceProbeConfig {
    pub(super) fn kernel_config(self) -> ProbeConfig {
        ProbeConfig {
            backend: self.backend,
            device: self.device,
            elements: self.elements,
            kernel: ProbeKernel::Affine,
            replays: 1,
        }
    }
}
