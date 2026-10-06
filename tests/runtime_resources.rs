#![cfg(feature = "cuda-probe")]

use mjwarp_rs::{
    diagnostics::ProbeError,
    runtime::{ProbeBackend, ResourceProbeConfig, run_resource_probe},
};

fn check(backend: ProbeBackend) {
    for elements in [1, 129, 257, 4097, 1048576] {
        let report = run_resource_probe(ResourceProbeConfig {
            backend,
            elements,
            ..Default::default()
        })
        .unwrap_or_else(|error| panic!("{backend:?}/{elements}: {error}"));
        assert_eq!(report.backend, backend);
        assert_eq!(report.elements, elements);
        assert_eq!(report.view_bytes, elements * 4);
        assert_eq!(report.output_owner_bytes, (elements + 32) * 4);
        assert_eq!(report.completed_submissions, 6);
        assert_eq!(report.failed_submissions, 1);
        assert_eq!(report.rejected_requests, 16);
        assert_eq!(report.graph_replays, 1);
        assert_eq!(
            report.compiler_revision,
            if backend == ProbeBackend::NativePtx {
                None
            } else {
                Some("1f73b9f63de50a17398c1d5278e2a5f11612c7e1")
            }
        );
    }
}

#[test]
#[ignore = "需要NVIDIA GPU；不回退CPU"]
fn native_preserves_owners_views_and_completion() {
    check(ProbeBackend::NativePtx);
}

#[cfg(feature = "cubecl-cpp-probe")]
#[test]
#[ignore = "需要NVIDIA GPU与NVRTC；不回退CPU"]
fn cpp_preserves_owners_views_and_completion() {
    check(ProbeBackend::CubeClCpp);
}

#[cfg(feature = "cubecl-llvm-probe")]
#[test]
#[ignore = "需要NVIDIA GPU与LLVM；不回退CPU"]
fn llvm_preserves_owners_views_and_completion() {
    check(ProbeBackend::CubeClLlvm);
}

#[test]
#[ignore = "需要NVIDIA驱动；不回退CPU"]
fn rejects_missing_resource_device_without_fallback() {
    assert!(matches!(
        run_resource_probe(ResourceProbeConfig {
            device: usize::MAX,
            ..Default::default()
        }),
        Err(ProbeError::InvalidDevice { .. })
    ));
}
