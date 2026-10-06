#![cfg(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe"))]

use mjwarp_rs::runtime::{ProbeBackend, ProbeConfig, ProbeKernel, run_probe};

fn check_backend(backend: ProbeBackend) {
    for kernel in ProbeKernel::ALL {
        for elements in [1, 127, 128, 129, 257, 4097, 16384, 16385, 1048576] {
            let report = run_probe(ProbeConfig {
                backend,
                kernel,
                elements,
                ..Default::default()
            })
            .unwrap_or_else(|error| panic!("{backend:?}/{kernel:?}/{elements}: {error}"));
            assert_eq!(report.backend, backend);
            assert_eq!(report.kernel, kernel);
            assert_eq!(report.graph_replays, 3);
            let outputs = match kernel {
                ProbeKernel::AtomicSum | ProbeKernel::FloatAtomicSum => 1,
                ProbeKernel::BlockReduce => elements.div_ceil(128),
                ProbeKernel::SmallSolve => elements * 2,
                _ => elements,
            };
            assert_eq!(report.buffer_bytes, (outputs + 16) * 4);
            let mut levels = 0;
            let mut count = elements;
            if kernel == ProbeKernel::GlobalScan {
                while count > 128 {
                    count = count.div_ceil(128);
                    levels += 1;
                }
            }
            assert_eq!(report.graph_kernel_nodes, 1 + 3 * levels);
            assert_eq!(report.graph_node_updates, if levels == 0 { 3 } else { 9 });
            assert_eq!(
                report.compiler_revision,
                Some("1f73b9f63de50a17398c1d5278e2a5f11612c7e1")
            );
        }
    }
    // 覆盖浮点样本轮次回绕。
    // 全局扫描跨越两层工作区。
    for kernel in [ProbeKernel::FloatAtomicSum, ProbeKernel::GlobalScan] {
        let report = run_probe(ProbeConfig {
            backend,
            kernel,
            elements: 16385,
            replays: 10,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(report.graph_replays, 10);
        assert_eq!(
            report.graph_node_updates,
            if kernel == ProbeKernel::GlobalScan {
                30
            } else {
                10
            }
        );
    }
}

#[cfg(feature = "cubecl-cpp-probe")]
#[test]
#[ignore = "需要NVIDIA GPU与NVRTC；不回退CPU"]
fn cpp_runs_all_kernels_with_guard_and_changed_graph_inputs() {
    check_backend(ProbeBackend::CubeClCpp);
}

#[cfg(feature = "cubecl-llvm-probe")]
#[test]
#[ignore = "需要NVIDIA GPU与LLVM构建链；不回退CPU"]
fn llvm_runs_all_kernels_with_guard_and_changed_graph_inputs() {
    check_backend(ProbeBackend::CubeClLlvm);
}
