//! P1驱动级探针。
//! 探针不冻结运行时接口。

use crate::diagnostics::ProbeError;

#[cfg(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe"))]
mod cubecl;
#[cfg(feature = "cuda-probe")]
mod cuda;

const GUARD_ELEMENTS: usize = 16;
#[cfg(any(feature = "cuda-probe", test))]
const GUARD_VALUE: f32 = -12345.0;
const MAX_ELEMENTS: usize = 1 << 20;
const MAX_REPLAYS: usize = 1000;

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

    fn required_feature(self) -> &'static str {
        match self {
            Self::NativePtx => "cuda-probe",
            Self::CubeClCpp => "cubecl-cpp-probe",
            Self::CubeClLlvm => "cubecl-llvm-probe",
        }
    }

    fn enabled(self) -> bool {
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
    pub device: usize,
    pub elements: usize,
    pub replays: usize,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            backend: ProbeBackend::NativePtx,
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
        buffer_bytes(self.elements)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ProbeReport {
    pub backend: ProbeBackend,
    /// 仅CubeCL路线记录冻结提交。
    pub compiler_revision: Option<&'static str>,
    pub os: &'static str,
    pub arch: &'static str,
    pub device: usize,
    pub gpu_name: String,
    pub compute_capability: (i32, i32),
    /// CUDA驱动API版本，不是驱动发行号。
    pub driver_api_version: i32,
    pub elements: usize,
    pub buffer_bytes: usize,
    pub graph_replays: usize,
}

/// 同步返回探针结果。
/// 函数不导出指针或在途资源。
pub fn run_probe(config: ProbeConfig) -> Result<ProbeReport, ProbeError> {
    config.validate()?;
    if !config.backend.enabled() {
        return Err(ProbeError::FeatureDisabled(
            config.backend.required_feature(),
        ));
    }
    #[cfg(feature = "cuda-probe")]
    {
        // 驱动加载器会对缺少符号执行panic。
        // 适配层返回错误，且不选择CPU。
        std::panic::catch_unwind(|| cuda::run(config)).map_err(|_| ProbeError::BackendPanic)?
    }
    #[cfg(not(feature = "cuda-probe"))]
    Err(ProbeError::FeatureDisabled("cuda-probe"))
}

fn buffer_bytes(elements: usize) -> Result<usize, ProbeError> {
    elements
        .checked_add(GUARD_ELEMENTS)
        .and_then(|length| length.checked_mul(size_of::<f32>()))
        .ok_or(ProbeError::InvalidArgument("缓冲字节数溢出"))
}

#[cfg(any(feature = "cuda-probe", test))]
fn input_values(elements: usize, epoch: usize) -> Vec<f32> {
    // 二进制分数确保精确比较。
    (0..elements)
        .map(|index| ((index % 31) as f32 - 15.0) * 0.25 + epoch as f32)
        .collect()
}

#[cfg(any(feature = "cuda-probe", test))]
fn validate_output(input: &[f32], output: &[f32]) -> Result<(), ProbeError> {
    let expected_length = input.len() + GUARD_ELEMENTS;
    if output.len() != expected_length {
        return Err(ProbeError::InvalidOutputLength {
            expected: expected_length,
            actual: output.len(),
        });
    }
    for (index, &actual) in output.iter().enumerate() {
        let expected = if index < input.len() {
            input[index] * 2.0 + 1.0
        } else {
            GUARD_VALUE
        };
        if actual != expected {
            return Err(ProbeError::Mismatch {
                index,
                expected,
                actual,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_default_and_capacity_boundaries() {
        ProbeConfig::default().validate().unwrap();
        ProbeConfig {
            elements: 1,
            replays: 1,
            ..Default::default()
        }
        .validate()
        .unwrap();
        ProbeConfig {
            elements: MAX_ELEMENTS,
            replays: MAX_REPLAYS,
            ..Default::default()
        }
        .validate()
        .unwrap();
    }

    #[test]
    fn rejects_invalid_elements_and_replays() {
        for elements in [0, MAX_ELEMENTS + 1, usize::MAX] {
            assert!(
                ProbeConfig {
                    elements,
                    ..Default::default()
                }
                .validate()
                .is_err()
            );
        }
        for replays in [0, MAX_REPLAYS + 1, usize::MAX] {
            assert!(
                ProbeConfig {
                    replays,
                    ..Default::default()
                }
                .validate()
                .is_err()
            );
        }
    }

    #[test]
    fn rejects_byte_count_overflow() {
        assert!(buffer_bytes(usize::MAX).is_err());
        assert!(buffer_bytes(usize::MAX / size_of::<f32>()).is_err());
        assert_eq!(buffer_bytes(257).unwrap(), 1092);
    }

    #[test]
    fn validates_exact_results_and_guard() {
        let input = input_values(257, 3);
        let output: Vec<_> = input
            .iter()
            .map(|value| value * 2.0 + 1.0)
            .chain([GUARD_VALUE; GUARD_ELEMENTS])
            .collect();
        validate_output(&input, &output).unwrap();
    }

    #[test]
    fn rejects_short_and_long_output() {
        for output in [vec![], vec![3.0; GUARD_ELEMENTS + 2]] {
            assert!(matches!(
                validate_output(&[1.0], &output),
                Err(ProbeError::InvalidOutputLength { .. })
            ));
        }
    }

    #[test]
    fn detects_bad_values_nonfinite_values_and_tail_writes() {
        let mut output = vec![GUARD_VALUE; 1 + GUARD_ELEMENTS];
        for value in [0.0, f32::NAN, f32::INFINITY] {
            output[0] = value;
            assert!(matches!(
                validate_output(&[1.0], &output),
                Err(ProbeError::Mismatch { index: 0, .. })
            ));
        }
        output[0] = 3.0;
        output[1] = 0.0;
        assert!(matches!(
            validate_output(&[1.0], &output),
            Err(ProbeError::Mismatch { index: 1, .. })
        ));
    }

    #[test]
    fn changes_inputs_between_graph_replays() {
        let input = input_values(1, 0);
        let old_output: Vec<_> = [input[0] * 2.0 + 1.0]
            .into_iter()
            .chain([GUARD_VALUE; GUARD_ELEMENTS])
            .collect();
        assert!(validate_output(&input_values(1, 1), &old_output).is_err());
    }

    #[cfg(not(feature = "cuda-probe"))]
    #[test]
    fn rejects_missing_feature_without_cpu_fallback() {
        assert_eq!(
            run_probe(ProbeConfig::default()).unwrap_err(),
            ProbeError::FeatureDisabled("cuda-probe")
        );
    }

    #[test]
    fn parses_only_explicit_backend_names() {
        for backend in [
            ProbeBackend::NativePtx,
            ProbeBackend::CubeClCpp,
            ProbeBackend::CubeClLlvm,
        ] {
            assert_eq!(ProbeBackend::parse(backend.name()).unwrap(), backend);
        }
        assert!(ProbeBackend::parse("auto").is_err());
        assert!(ProbeBackend::parse("cpu").is_err());
    }

    #[test]
    fn rejects_disabled_routes_without_substitution() {
        for backend in [
            ProbeBackend::NativePtx,
            ProbeBackend::CubeClCpp,
            ProbeBackend::CubeClLlvm,
        ] {
            if !backend.enabled() {
                assert_eq!(
                    run_probe(ProbeConfig {
                        backend,
                        ..Default::default()
                    })
                    .unwrap_err(),
                    ProbeError::FeatureDisabled(backend.required_feature())
                );
            }
        }
    }
}
