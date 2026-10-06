#![cfg(feature = "cuda-probe")]

use mjwarp_rs::diagnostics::ProbeError;
use mjwarp_rs::runtime::{
    ArtifactStatus, ProbeBackend, ProbeConfig, ProbeKernel, build_probe_artifact, run_cached_probe,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static ID: AtomicU64 = AtomicU64::new(0);
struct Cache(PathBuf);
impl Cache {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "mjwarp-gpu-artifacts-{}-{}",
            std::process::id(),
            ID.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Cache {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn check(backend: ProbeBackend) {
    let directory = Cache::new();
    for kernel in ProbeKernel::ALL {
        if backend == ProbeBackend::NativePtx && kernel != ProbeKernel::Affine {
            continue;
        }
        let config = ProbeConfig {
            backend,
            kernel,
            ..Default::default()
        };
        let built = build_probe_artifact(config, &directory.0, false).unwrap();
        assert_eq!(built.status, ArtifactStatus::Compiled);
        assert_eq!(
            built.stages,
            if kernel == ProbeKernel::GlobalScan {
                3
            } else {
                1
            }
        );
        let hit = build_probe_artifact(config, &directory.0, false).unwrap();
        assert_eq!(hit.status, ArtifactStatus::Hit);
        for elements in [1, 127, 128, 129, 257, 4097, 16384, 16385, 1048576] {
            let current = ProbeConfig { elements, ..config };
            // SAFETY: 本测试独占临时目录。
            // 本测试刚用可信编译器写入PTX。
            let report = unsafe { run_cached_probe(current, &directory.0, false) }.unwrap();
            assert_eq!(report.artifact.status, ArtifactStatus::Hit);
            assert_eq!(report.artifact.path, built.path);
            assert_eq!(report.probe.backend, backend);
            assert_eq!(report.probe.kernel, kernel);
            assert_eq!(report.probe.graph_replays, 3);
            let mut levels = 0;
            let mut count = elements;
            if kernel == ProbeKernel::GlobalScan {
                while count > 128 {
                    count = count.div_ceil(128);
                    levels += 1;
                }
            }
            assert_eq!(report.probe.graph_kernel_nodes, 1 + 3 * levels);
            assert_eq!(
                report.probe.graph_node_updates,
                if levels == 0 { 3 } else { 9 }
            );
        }
        if matches!(
            kernel,
            ProbeKernel::FloatAtomicSum | ProbeKernel::GlobalScan
        ) {
            // SAFETY: 同上，目录与产物仍可信。
            let report = unsafe {
                run_cached_probe(
                    ProbeConfig {
                        elements: 16385,
                        replays: 10,
                        ..config
                    },
                    &directory.0,
                    false,
                )
            }
            .unwrap();
            assert_eq!(report.probe.graph_replays, 10);
            assert_eq!(report.artifact.path, built.path);
        }
    }
}

#[test]
#[ignore = "需要NVIDIA GPU；不回退CPU"]
fn native_reuses_trusted_ptx_and_updates_graphs() {
    check(ProbeBackend::NativePtx);
}

#[cfg(feature = "cubecl-cpp-probe")]
#[test]
#[ignore = "需要NVIDIA GPU与NVRTC12.8"]
fn cpp_reuses_all_artifacts_and_scan_stages() {
    check(ProbeBackend::CubeClCpp);
}

#[cfg(feature = "cubecl-llvm-probe")]
#[test]
#[ignore = "需要NVIDIA GPU与LLVM构建链"]
fn llvm_reuses_all_artifacts_and_scan_stages() {
    check(ProbeBackend::CubeClLlvm);
}

#[test]
#[ignore = "需要NVIDIA驱动；不编译缺失产物"]
fn missing_and_corrupt_artifacts_fail_without_compiling() {
    let directory = Cache::new();
    let config = ProbeConfig::default();
    // SAFETY: 空目录不含外部代码。
    let missing = unsafe { run_cached_probe(config, &directory.0, false) };
    assert!(matches!(
        missing,
        Err(ProbeError::Artifact {
            stage: "missing",
            ..
        })
    ));
    let report = build_probe_artifact(config, &directory.0, false).unwrap();
    std::fs::write(&report.path, "corrupted").unwrap();
    // SAFETY: 错误JSON不包含可加载代码。
    let corrupt = unsafe { run_cached_probe(config, &directory.0, false) };
    assert!(matches!(
        corrupt,
        Err(ProbeError::Artifact {
            stage: "archive-json",
            ..
        })
    ));
    assert!(build_probe_artifact(config, &directory.0, false).is_err());
    assert_eq!(std::fs::read_to_string(report.path).unwrap(), "corrupted");
}
