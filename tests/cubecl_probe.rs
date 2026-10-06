#![cfg(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe"))]

use mjwarp_rs::runtime::{ProbeBackend, ProbeConfig, run_probe};

fn check_backend(backend: ProbeBackend) {
    for elements in [1, 127, 128, 129, 257, 4097] {
        let report = run_probe(ProbeConfig {
            backend,
            elements,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(report.backend, backend);
        assert_eq!(report.graph_replays, 3);
        assert_eq!(
            report.compiler_revision,
            Some("1f73b9f63de50a17398c1d5278e2a5f11612c7e1")
        );
    }
}

#[cfg(feature = "cubecl-cpp-probe")]
#[test]
#[ignore = "需要NVIDIA GPU与NVRTC；不回退CPU"]
fn cpp_runs_rust_kernel_with_guard_and_changed_graph_inputs() {
    check_backend(ProbeBackend::CubeClCpp);
}

#[cfg(feature = "cubecl-llvm-probe")]
#[test]
#[ignore = "需要NVIDIA GPU与LLVM构建链；不回退CPU"]
fn llvm_runs_same_rust_kernel_with_guard_and_changed_graph_inputs() {
    check_backend(ProbeBackend::CubeClLlvm);
}
