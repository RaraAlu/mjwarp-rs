//! 冻结工件导入与启动测试。

use super::package_gate::*;
use super::support::*;
use super::*;

#[cfg(windows)]
#[test]
#[ignore = "requires an SM89 GPU and the frozen generated Warp 1.15 artifact"]
pub(super) fn frozen_generated_artifact_imports_and_launches() {
    use std::time::Instant;

    let package = package().unwrap();
    validate_package(&package).unwrap();
    let directory = root().join(
        package["generation_pipeline"]["artifact_directory"]
            .as_str()
            .unwrap(),
    );
    let identity = admit_artifact(&package, &directory).unwrap();
    let load_started = Instant::now();
    let artifact = FrozenWarpGravityArtifact::load(&directory, 0).unwrap();
    let load_micros = load_started.elapsed().as_micros();
    assert_eq!(artifact.receipt().entry, identity.entry);

    let input = frozen_physical_input(&package);
    let launch_started = Instant::now();
    let readback = artifact.launch_diagnostic(&input).unwrap();
    let launch_micros = launch_started.elapsed().as_micros();
    let raw_bits = readback
        .bits()
        .iter()
        .map(|value| format!("{value:08x}"))
        .collect::<Vec<_>>();
    let evidence_directory =
        std::env::var_os("MJWARP_GRAVITY_UPSTREAM_EVIDENCE_DIR").map(PathBuf::from);
    if let Some(evidence) = &evidence_directory {
        fs::create_dir_all(evidence).unwrap();
        let raw = serde_json::json!({
            "schema": 1,
            "provenance": "direct_generated_kernel_readback_before_finite_validation",
            "upstream_revision": REVISION,
            "entry": identity.entry,
            "artifact_cu_sha256": identity.cu_sha256,
            "artifact_ptx_sha256": identity.ptx_sha256,
            "actual_bits": &raw_bits,
            "count": readback.bits().len()
        });
        fs::write(
            evidence.join("raw.json"),
            serde_json::to_vec_pretty(&raw).unwrap(),
        )
        .unwrap();
    }

    let actual = readback.into_finite().unwrap();
    let expected = analytic_expected(&package);
    let max_absolute = max_absolute_finite(&actual, &expected).unwrap();
    let bit_differences = actual
        .iter()
        .zip(&expected)
        .filter(|(actual, expected)| actual.to_bits() != expected.to_bits())
        .count();
    if let Some(evidence) = &evidence_directory {
        let report = serde_json::json!({
            "schema": 1,
            "evidence_kind": "frozen-generated-upstream-device-launch",
            "upstream_revision": REVISION,
            "warp_version": WARP_VERSION,
            "entry": artifact.receipt().entry,
            "artifact_cu_sha256": artifact.receipt().cu_sha256,
            "artifact_ptx_sha256": artifact.receipt().ptx_sha256,
            "package_sha256": PACKAGE_SHA256,
            "launch_shape": [input.worlds, input.nbody - 1, input.nv],
            "output_count": actual.len(),
            "actual": &actual,
            "actual_bits": &raw_bits,
            "expected": &expected,
            "expected_kind": "analytic_certificate_not_upstream_device_output",
            "tolerance": 0.000002,
            "max_absolute": max_absolute,
            "bit_differences": bit_differences,
            "artifact_load_micros": load_micros,
            "launch_and_readback_micros": launch_micros,
            "upstream_generated_gravity_kernel": true,
            "candidate_derived_inputs": true,
            "full_chain_independent": false,
            "artifact_equivalence_eligible": true,
            "full_chain_equivalence_eligible": false,
            "product_python_dependency": false
        });
        fs::write(
            evidence.join("run.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
    assert_eq!(actual.len(), expected.len());
    assert!(max_absolute <= 0.000002);
}
