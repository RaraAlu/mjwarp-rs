#![cfg(feature = "cuda-probe")]

use mjwarp_rs::{
    diagnostics::ProbeError,
    runtime::{ProbeBackend, ResourceProbeConfig, run_external_resource_probe},
};

fn check(backend: ProbeBackend) {
    for elements in [1, 129, 257, 4097, 1048576] {
        let report = run_external_resource_probe(ResourceProbeConfig {
            backend,
            elements,
            ..Default::default()
        })
        .unwrap_or_else(|error| panic!("{backend:?}/{elements}: {error}"));
        let resources = report.resources;
        assert_eq!(resources.backend, backend);
        assert_eq!(resources.elements, elements);
        assert_eq!(resources.view_bytes, elements * 4);
        assert_eq!(resources.output_owner_bytes, (elements + 32) * 4);
        assert_eq!(resources.completed_submissions, 6);
        assert_eq!(resources.failed_submissions, 1);
        assert_eq!(resources.rejected_requests, 17);
        assert_eq!(resources.graph_replays, 1);
        assert_eq!(report.imported_allocations, 15);
        assert_eq!(report.released_owners, 15);
        assert_eq!(report.producer_dependencies, 15);
        assert_eq!(
            resources.compiler_revision,
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
fn native_checks_external_allocations_and_owner_handoff() {
    check(ProbeBackend::NativePtx);
}

#[cfg(feature = "cubecl-cpp-probe")]
#[test]
#[ignore = "需要NVIDIA GPU与NVRTC；不回退CPU"]
fn cpp_checks_external_allocations_and_owner_handoff() {
    check(ProbeBackend::CubeClCpp);
}

#[cfg(feature = "cubecl-llvm-probe")]
#[test]
#[ignore = "需要NVIDIA GPU与LLVM；不回退CPU"]
fn llvm_checks_external_allocations_and_owner_handoff() {
    check(ProbeBackend::CubeClLlvm);
}

#[test]
#[ignore = "需要NVIDIA驱动；不回退CPU"]
fn rejects_missing_external_device_without_fallback() {
    assert!(matches!(
        run_external_resource_probe(ResourceProbeConfig {
            device: usize::MAX,
            ..Default::default()
        }),
        Err(ProbeError::InvalidDevice { .. })
    ));
}
