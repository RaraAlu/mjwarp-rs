#![cfg(feature = "cuda-probe")]

use mjwarp_rs::diagnostics::ProbeError;
use mjwarp_rs::runtime::{ProbeConfig, run_probe};

#[test]
#[ignore = "需要NVIDIA GPU与驱动"]
fn validates_buffers_events_and_graph_replays_on_gpu() {
    for elements in [1, 127, 128, 129, 257, 4097] {
        let report = run_probe(ProbeConfig {
            elements,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(report.elements, elements);
        assert_eq!(report.graph_replays, 3);
        assert_eq!(report.graph_kernel_nodes, 1);
        assert_eq!(report.graph_node_updates, 3);
    }
}

#[test]
#[ignore = "需要NVIDIA GPU与驱动"]
fn rejects_missing_device_without_cpu_fallback() {
    assert!(matches!(
        run_probe(ProbeConfig {
            device: usize::MAX,
            ..Default::default()
        }),
        Err(ProbeError::InvalidDevice { .. })
    ));
}
