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
                run_external_resource_probe(ResourceProbeConfig {
                    backend,
                    ..Default::default()
                })
                .unwrap_err(),
                ProbeError::FeatureDisabled(backend.required_feature())
            );
            assert_eq!(
                run_resource_probe(ResourceProbeConfig {
                    backend,
                    ..Default::default()
                })
                .unwrap_err(),
                ProbeError::FeatureDisabled(backend.required_feature())
            );
            for kernel in ProbeKernel::ALL {
                if backend == ProbeBackend::NativePtx && kernel != ProbeKernel::Affine {
                    continue;
                }
                assert_eq!(
                    run_probe(ProbeConfig {
                        backend,
                        kernel,
                        ..Default::default()
                    })
                    .unwrap_err(),
                    ProbeError::FeatureDisabled(backend.required_feature())
                );
            }
        }
    }
}

#[test]
fn rejects_invalid_resource_dimensions_before_loading_driver() {
    for elements in [0, MAX_ELEMENTS + 1, usize::MAX] {
        assert!(matches!(
            run_external_resource_probe(ResourceProbeConfig {
                elements,
                ..Default::default()
            }),
            Err(ProbeError::InvalidArgument(_))
        ));
        assert!(matches!(
            run_resource_probe(ResourceProbeConfig {
                elements,
                ..Default::default()
            }),
            Err(ProbeError::InvalidArgument(_))
        ));
    }
}

#[test]
fn parses_kernels_and_rejects_native_route_substitution() {
    for kernel in ProbeKernel::ALL {
        assert_eq!(ProbeKernel::parse(kernel.name()).unwrap(), kernel);
        let config = ProbeConfig {
            kernel,
            ..Default::default()
        };
        assert_eq!(config.validate().is_ok(), kernel == ProbeKernel::Affine);
        ProbeConfig {
            backend: ProbeBackend::CubeClCpp,
            ..config
        }
        .validate()
        .unwrap();
    }
    assert!(ProbeKernel::parse("auto").is_err());
    assert!(ProbeKernel::parse("unknown").is_err());
}

#[test]
fn calculates_kernel_output_capacity() {
    assert_eq!(ProbeKernel::AtomicSum.output_elements(257).unwrap(), 1);
    assert_eq!(ProbeKernel::FloatAtomicSum.output_elements(257).unwrap(), 1);
    assert_eq!(ProbeKernel::BlockReduce.output_elements(257).unwrap(), 3);
    assert_eq!(ProbeKernel::BlockScan.output_elements(257).unwrap(), 257);
    assert_eq!(ProbeKernel::GlobalScan.output_elements(257).unwrap(), 257);
    assert_eq!(ProbeKernel::SmallSolve.output_elements(257).unwrap(), 514);
    assert!(ProbeKernel::SmallSolve.output_elements(usize::MAX).is_err());
}
