+++
uid = "7295f976"
id = "mjwarp"
tags = ["planned", "engine", "nvidia", "windows", "linux"]
fingerprint = "e96ed8e2ae6adabc1481dec7108595120c7c0910a3181bc5b28b8923c8e11cb3"

[name]
zh = "MJWarp Rust引擎"
en = "MJWarp Rust engine"

[description]
zh = """
规划七模块GPU引擎。历史完整数值复验双模式各483过。已核对78项模型入口、15项状态与10个打包器存储。派生质量与布尔编码单列。全链精度及真实性能未齐，T4继续开放。
"""
en = """
Planned seven-module GPU engine. Historical numerical revalidation passes 483 tests per mode. Evidence now covers 78 model inputs, 15 state declarations and storage types in 10 packers. Derived mass and Boolean encodings remain explicit boundaries. Full precision and measured performance evidence remain incomplete.
"""

[[source]]
path = "src/lib.rs"

[[source]]
path = "Cargo.toml"

[[source]]
path = "Cargo.lock"

[[source]]
path = ".cargo/config.toml"

[[source]]
path = "THIRD_PARTY_NOTICES.md"

[[source]]
path = "LICENSES/Apache-2.0.txt"

[[source]]
path = "build.rs"

[[source]]
path = "include/mjwarp_native_probe.h"

[[source]]
path = "native/model_probe.cpp"

[[source]]
path = "tests/native_model_probe.rs"

[[source]]
path = "tests/native/mock_mujoco.cpp"

[[source]]
path = "scripts/prepare-native-probe.ps1"

[[source]]
path = "scripts/test-windows-native.ps1"

[[source]]
path = "fixtures/native-probe/two-joints.xml"

[[source]]
path = "fixtures/native-probe/two-joints.mjb"

[[source]]
path = "fixtures/native-probe/zero-dof.xml"

[[source]]
path = "fixtures/native-probe/zero-dof.mjb"

[[source]]
path = "fixtures/native-probe/manifest.json"

[[source]]
path = "fixtures/native-probe/README.md"

[[source]]
path = ".gitattributes"

[[source]]
path = ".gitignore"

[[source]]
path = "AGENTS.md"

[[source]]
path = "docs/README.md"

[[source]]
path = "docs/implementation-plan.md"

[[source]]
path = "docs/module-contracts.md"

[[source]]
path = "docs/windows-native-model-probe.md"

[[source]]
path = "fixtures/native-probe/mixed-joints.xml"

[[source]]
path = "fixtures/native-probe/mixed-joints.mjb"

[[source]]
path = "fixtures/native-probe/mixed-joints-reference.txt"

[[source]]
path = "tests/native/kinematic_preflight.rs"

[[source]]
path = "docs/windows-kinematic-fields.md"

[[source]]
path = "fixtures/native-probe/inertial-tree.xml"

[[source]]
path = "fixtures/native-probe/inertial-tree.mjb"

[[source]]
path = "fixtures/native-probe/inertial-tree-reference.txt"

[[source]]
path = "tests/native/inertial_fields.rs"

[[source]]
path = "tests/native/inertial_preflight.rs"

[[source]]
path = "docs/windows-inertial-fields.md"

[[source]]
path = "docs/project-positioning.md"

[[source]]
path = "tests/kinematics_probe.rs"

[[source]]
path = "scripts/test-windows-kinematics.ps1"

[[source]]
path = "docs/windows-kinematics-probe.md"

[[source]]
path = "fixtures/kinematics/reference.cpp"

[[source]]
path = "fixtures/kinematics/rotated-tree.xml"

[[source]]
path = "fixtures/kinematics/rotated-tree.mjb"

[[source]]
path = "fixtures/kinematics/rotated-tree.json"

[[source]]
path = "fixtures/kinematics/mixed-joints.json"

[[source]]
path = "fixtures/kinematics/inertial-tree.json"

[[source]]
path = "fixtures/kinematics/zero-dof.json"

[[source]]
path = "fixtures/kinematics/manifest.json"

[[source]]
path = "fixtures/kinematics/README.md"

[[source]]
path = "docs/windows-com-position-probe.md"

[[source]]
path = "tests/com_position_probe.rs"

[[source]]
path = "tests/support/rigid_reference.rs"

[[source]]
path = "scripts/test-windows-com-position.ps1"

[[source]]
path = "fixtures/com-position/README.md"

[[source]]
path = "fixtures/com-position/manifest.json"

[[source]]
path = "fixtures/com-position/reference.cpp"

[[source]]
path = "fixtures/com-position/massless-tree.xml"

[[source]]
path = "fixtures/com-position/massless-tree.mjb"

[[source]]
path = "fixtures/com-position/massless-tree.json"

[[source]]
path = "fixtures/com-position/mixed-joints.json"

[[source]]
path = "fixtures/com-position/inertial-tree.json"

[[source]]
path = "fixtures/com-position/rotated-tree.json"

[[source]]
path = "fixtures/com-position/zero-dof.json"

[[source]]
path = "docs/windows-mass-matrix-probe.md"

[[source]]
path = "tests/mass_matrix_probe.rs"

[[source]]
path = "scripts/test-windows-mass-matrix.ps1"

[[source]]
path = "fixtures/mass-matrix/README.md"

[[source]]
path = "fixtures/mass-matrix/manifest.json"

[[source]]
path = "fixtures/mass-matrix/reference.cpp"

[[source]]
path = "fixtures/mass-matrix/armature-chain.xml"

[[source]]
path = "fixtures/mass-matrix/armature-chain.mjb"

[[source]]
path = "fixtures/mass-matrix/mixed-joints.json"

[[source]]
path = "fixtures/mass-matrix/inertial-tree.json"

[[source]]
path = "fixtures/mass-matrix/rotated-tree.json"

[[source]]
path = "fixtures/mass-matrix/zero-dof.json"

[[source]]
path = "fixtures/mass-matrix/massless-tree.json"

[[source]]
path = "fixtures/mass-matrix/armature-chain.json"

[[source]]
path = "docs/windows-mass-solve-probe.md"

[[source]]
path = "tests/mass_solve_probe.rs"

[[source]]
path = "scripts/test-windows-mass-solve.ps1"

[[source]]
path = "fixtures/mass-solve/README.md"

[[source]]
path = "fixtures/mass-solve/manifest.json"

[[source]]
path = "fixtures/mass-solve/reference.cpp"

[[source]]
path = "fixtures/mass-solve/mixed-joints.json"

[[source]]
path = "fixtures/mass-solve/inertial-tree.json"

[[source]]
path = "fixtures/mass-solve/rotated-tree.json"

[[source]]
path = "fixtures/mass-solve/zero-dof.json"

[[source]]
path = "fixtures/mass-solve/massless-tree.json"

[[source]]
path = "fixtures/mass-solve/armature-chain.json"

[[source]]
path = "docs/windows-attached-kinematics-probe.md"

[[source]]
path = "tests/attached_kinematics_probe.rs"

[[source]]
path = "scripts/test-windows-attached-kinematics.ps1"

[[source]]
path = "fixtures/attached-kinematics/README.md"

[[source]]
path = "fixtures/attached-kinematics/manifest.json"

[[source]]
path = "fixtures/attached-kinematics/reference.cpp"

[[source]]
path = "fixtures/attached-kinematics/attached-tree.xml"

[[source]]
path = "fixtures/attached-kinematics/attached-tree.mjb"

[[source]]
path = "fixtures/attached-kinematics/attached-tree.json"

[[source]]
path = "fixtures/attached-kinematics/static-frames.xml"

[[source]]
path = "fixtures/attached-kinematics/static-frames.mjb"

[[source]]
path = "fixtures/attached-kinematics/static-frames.json"

[[source]]
path = "fixtures/attached-kinematics/sites-only.xml"

[[source]]
path = "fixtures/attached-kinematics/sites-only.mjb"

[[source]]
path = "fixtures/attached-kinematics/sites-only.json"

[[source]]
path = "fixtures/attached-kinematics/geoms-only.json"

[[source]]
path = "fixtures/attached-kinematics/empty-frames.json"

[[source]]
path = "docs/windows-resident-kinematics.md"

[[source]]
path = "tests/resident_kinematics.rs"

[[source]]
path = "scripts/test-windows-resident-kinematics.ps1"

[[source]]
path = "tests/resident_parameters.rs"

[[source]]
path = "docs/windows-resident-parameters.md"

[[source]]
path = "src/model/mocap.rs"

[[source]]
path = "tests/resident_mocap.rs"

[[source]]
path = "fixtures/mocap-kinematics/README.md"

[[source]]
path = "fixtures/mocap-kinematics/manifest.json"

[[source]]
path = "fixtures/mocap-kinematics/mixed-tree.xml"

[[source]]
path = "fixtures/mocap-kinematics/mixed-tree.mjb"

[[source]]
path = "fixtures/mocap-kinematics/mixed-tree.json"

[[source]]
path = "fixtures/mocap-kinematics/reference.cpp"

[[source]]
path = "docs/windows-resident-mocap.md"

[[source]]
path = "docs/architecture.md"

[[source]]
path = "docs/compatibility-matrix.md"

[[source]]
path = "tests/resident_camlight.rs"

[[source]]
path = "fixtures/camlight/manifest.json"

[[source]]
path = "fixtures/camlight/mixed-tree.json"

[[source]]
path = "fixtures/camlight/mixed-tree.mjb"

[[source]]
path = "fixtures/camlight/mixed-tree.xml"

[[source]]
path = "fixtures/camlight/reference.cpp"

[[source]]
path = "fixtures/camlight/README.md"

[[source]]
path = "docs/windows-resident-camlight.md"

[[source]]
path = "tests/resident_fixed_tendon.rs"

[[source]]
path = "fixtures/fixed-tendon/README.md"

[[source]]
path = "fixtures/fixed-tendon/manifest.json"

[[source]]
path = "fixtures/fixed-tendon/mixed-tree.json"

[[source]]
path = "fixtures/fixed-tendon/mixed-tree.mjb"

[[source]]
path = "fixtures/fixed-tendon/mixed-tree.xml"

[[source]]
path = "fixtures/fixed-tendon/reference.cpp"

[[source]]
path = "docs/windows-resident-fixed-tendon.md"

[[source]]
path = "tests/resident_spatial_tendon.rs"

[[source]]
path = "docs/windows-resident-spatial-tendon.md"

[[source]]
path = "fixtures/spatial-tendon/README.md"

[[source]]
path = "fixtures/spatial-tendon/manifest.json"

[[source]]
path = "fixtures/spatial-tendon/mixed-tree.xml"

[[source]]
path = "fixtures/spatial-tendon/mixed-tree.mjb"

[[source]]
path = "fixtures/spatial-tendon/mixed-tree.json"

[[source]]
path = "fixtures/spatial-tendon/reference.cpp"

[[source]]
path = "tests/resident_geom_tendon.rs"

[[source]]
path = "docs/windows-resident-geom-tendon.md"

[[source]]
path = "fixtures/geom-tendon/README.md"

[[source]]
path = "fixtures/geom-tendon/manifest.json"

[[source]]
path = "fixtures/geom-tendon/reference.cpp"

[[source]]
path = "fixtures/geom-tendon/wrap-tree.xml"

[[source]]
path = "fixtures/geom-tendon/wrap-tree.json"

[[source]]
path = "fixtures/geom-tendon/wrap-tree.mjb"

[[source]]
path = "fixtures/geom-tendon/motion-reference.cpp"

[[source]]
path = "fixtures/geom-tendon/inside-cylinder-motion-inputs.txt"

[[source]]
path = "fixtures/geom-tendon/inside-cylinder-motion.json"

[[source]]
path = "docs/windows-geom-tendon-frame-review.md"

[[source]]
path = "tests/resident_tendon.rs"

[[source]]
path = "fixtures/mixed-tendon/README.md"

[[source]]
path = "fixtures/mixed-tendon/manifest.json"

[[source]]
path = "fixtures/mixed-tendon/mixed-tree.xml"

[[source]]
path = "fixtures/mixed-tendon/mixed-tree.json"

[[source]]
path = "fixtures/mixed-tendon/mixed-tree.mjb"

[[source]]
path = "fixtures/mixed-tendon/reference.cpp"

[[source]]
path = "docs/windows-resident-tendon.md"

[[source]]
path = "tests/resident_flex_position.rs"

[[source]]
path = "fixtures/flex-position/README.md"

[[source]]
path = "fixtures/flex-position/manifest.json"

[[source]]
path = "fixtures/flex-position/reference.cpp"

[[source]]
path = "fixtures/flex-position/flex-tree.xml"

[[source]]
path = "fixtures/flex-position/flex-tree.mjb"

[[source]]
path = "fixtures/flex-position/flex-tree.json"

[[source]]
path = "docs/windows-resident-flex-position.md"

[[source]]
path = "tests/resident_tendon_wake.rs"

[[source]]
path = "fixtures/tendon-wake/README.md"

[[source]]
path = "fixtures/tendon-wake/manifest.json"

[[source]]
path = "fixtures/tendon-wake/reference.cpp"

[[source]]
path = "fixtures/tendon-wake/tree-state.json"

[[source]]
path = "docs/windows-resident-tendon-wake.md"

[[source]]
path = "tests/resident_flex_edge.rs"

[[source]]
path = "fixtures/flex-edge/README.md"

[[source]]
path = "fixtures/flex-edge/manifest.json"

[[source]]
path = "fixtures/flex-edge/reference.cpp"

[[source]]
path = "fixtures/flex-edge/edge-state.json"

[[source]]
path = "docs/windows-resident-flex-edge.md"

[[source]]
path = "tests/resident_flex_face.rs"

[[source]]
path = "fixtures/flex-face/reference.cpp"

[[source]]
path = "fixtures/flex-face/README.md"

[[source]]
path = "fixtures/flex-face/face-state.json"

[[source]]
path = "fixtures/flex-face/manifest.json"

[[source]]
path = "docs/windows-resident-flex-face.md"

[[source]]
path = "docs/windows-resident-flex-hessian.md"

[[source]]
path = "tests/resident_flex_stretch.rs"

[[source]]
path = "tests/support/flex_stretch_reference.rs"

[[source]]
path = "fixtures/flex-stretch/manifest.json"

[[source]]
path = "fixtures/flex-stretch/reference.cpp"

[[source]]
path = "fixtures/flex-stretch/stretch.json"

[[source]]
path = "fixtures/flex-stretch/stretch.mjb"

[[source]]
path = "fixtures/flex-stretch/stretch.xml"

[[source]]
path = "fixtures/flex-stretch/README.md"

[[source]]
path = "docs/windows-resident-flex-stretch.md"

[[source]]
path = "src/io/flex.rs"

[[source]]
path = "tests/native/flex_position.rs"

[[source]]
path = "tests/native/flex_position_preflight.rs"

[[source]]
path = "tests/native_flex_position.rs"

[[source]]
path = "tests/support/flex_position_reference.rs"

[[source]]
path = "docs/windows-native-flex-position.md"

[[source]]
path = "native/g01_fields.inc"

[[source]]
path = "scripts/test-windows-g01.ps1"

[[source]]
path = "tests/g01.rs"

[[source]]
path = "tests/native_g01.rs"

[[source]]
path = "tests/native/g01_preflight.rs"

[[source]]
path = "fixtures/g01/scene.xml"

[[source]]
path = "fixtures/g01/scene.mjb"

[[source]]
path = "fixtures/g01/reference.cpp"

[[source]]
path = "fixtures/g01/reference.json"

[[source]]
path = "fixtures/g01/manifest.json"

[[source]]
path = "fixtures/g01/README.md"

[[source]]
path = "docs/windows-g01-acceptance.md"

[[source]]
path = "docs/physics-stage-contracts.md"

[[source]]
path = "docs/upstream-api-map.md"

[[source]]
path = "tests/mjwarp_geom_tendon.rs"

[[source]]
path = "tests/support/geom_tendon_input.rs"

[[source]]
path = "fixtures/mjwarp-geom-tendon/manifest.json"

[[source]]
path = "fixtures/mjwarp-geom-tendon/reference.json"

[[source]]
path = "fixtures/mjwarp-geom-tendon/provenance.json"

[[source]]
path = "fixtures/mjwarp-geom-tendon/reference.py"

[[source]]
path = "docs/windows-mjwarp-geom-tendon-comparison.md"

[[source]]
path = "docs/numerical-precision-contract.md"

[[source]]
path = "docs/linux-f32-precision-restoration.md"

[[source]]
path = "docs/foundation-architecture.md"

[[source]]
path = "docs/gpu-probe.md"

[[source]]
path = "docs/linux-g01-acceptance.md"

[[source]]
path = "docs/windows-artifact-probe.md"

[[source]]
path = "docs/windows-t4-recheck.md"

[[source]]
path = "fixtures/com-velocity/README.md"

[[source]]
path = "fixtures/rne-bias/README.md"

[[source]]
path = "tests/support/two_reference.rs"

[[source]]
path = "tests/support/mjwarp_geom_reference.rs"

[[source]]
path = "tests/two_reference_policy.rs"

[[source]]
path = "docs/windows-two-reference-acceptance.md"

[[source]]
path = "docs/native-error-redline-acceptance.md"

[[source]]
path = "docs/windows-t4-redline-retest.md"

[[source]]
path = "fixtures/mjwarp-mass-solve/reference.py"

[[source]]
path = "fixtures/mjwarp-mass-solve/reference.json"

[[source]]
path = "fixtures/mjwarp-mass-solve/manifest.json"

[[source]]
path = "fixtures/mjwarp-mass-solve/acceptance.json"

[[source]]
path = "fixtures/mjwarp-mass-solve/README.md"

[[source]]
path = "tests/support/mass_solve_reference.rs"

[[source]]
path = "docs/windows-g03-block-repair.md"

[[source]]
path = "docs/windows-t4-g03-retest.md"

[[source]]
path = "docs/evidence/t4-g03-retest/.gitattributes"

[[source]]
path = "docs/evidence/t4-g03-retest/environment.json"

[[source]]
path = "docs/evidence/t4-g03-retest/calls.json"

[[source]]
path = "docs/evidence/t4-g03-retest/report.json"

[[source]]
path = "docs/evidence/t4-g03-retest/manifest.json"

[[source]]
path = "docs/evidence/t4-g03-retest/cargo-metadata.json"

[[source]]
path = "docs/evidence/t4-g03-retest/debug-native-report.json"

[[source]]
path = "docs/evidence/t4-g03-retest/release-native-report.json"

[[source]]
path = "docs/evidence/t4-g03-retest/debug-products-np03.json"

[[source]]
path = "docs/evidence/t4-g03-retest/release-products-np03.json"

[[source]]
path = "docs/evidence/t4-g03-retest/run.ps1"

[[source]]
path = "docs/evidence/t4-g03-retest/summarize.ps1"

[[source]]
path = "tests/t4_evidence.rs"

[[source]]
path = "tests/support/t4_evidence.rs"

[[source]]
path = "fixtures/precision/t4-kernels.json"

[[source]]
path = "fixtures/precision/audit.json"

[[source]]
path = "fixtures/precision/.gitattributes"

[[source]]
path = "docs/t4-evidence-gates.md"

[[source]]
path = "tests/t4_input_precision.rs"

[[source]]
path = "fixtures/precision/g01-inputs.json"

[[source]]
path = "fixtures/precision/g01-input-audit.json"

[[source]]
path = "docs/t4-input-precision.md"

[[source]]
path = "tests/support/t4_state_packing.rs"

[[source]]
path = "fixtures/precision/g01-state-packing.json"

[[source]]
path = "fixtures/precision/g01-state-audit.json"

[[source]]
path = "docs/t4-state-packing.md"

[[source]]
path = "tests/support/t4_field_internal.rs"

[[source]]
path = "fixtures/precision/g01-field-internal.json"

[[source]]
path = "fixtures/precision/g01-field-internal-audit.json"

[[source]]
path = "docs/t4-field-internal.md"

[[source]]
path = "tests/support/t4_packing_routes.rs"

[[source]]
path = "fixtures/precision/g01-packing-routes.json"

[[source]]
path = "fixtures/precision/g01-packing-routes-audit.json"

[[source]]
path = "docs/t4-packing-routes.md"

[[source]]
path = "tests/support/t4_topology_frames.rs"

[[source]]
path = "fixtures/precision/g01-topology-frames.json"

[[source]]
path = "fixtures/precision/g01-topology-frames-audit.json"

[[source]]
path = "docs/t4-topology-frames.md"

[[source]]
path = "tests/support/t4_rigid_internal.rs"

[[source]]
path = "fixtures/precision/g01-rigid-internal.json"

[[source]]
path = "fixtures/precision/g01-rigid-internal-audit.json"

[[source]]
path = "docs/t4-rigid-internal.md"

[[source]]
path = "tests/support/t4_zero_quaternion.rs"

[[source]]
path = "fixtures/precision/g01-zero-quaternion.json"

[[source]]
path = "fixtures/precision/g01-zero-quaternion-result.json"

[[source]]
path = "docs/t4-zero-quaternion.md"

[[source]]
path = "tests/support/t4_rigid_capture.rs"

[[source]]
path = "fixtures/precision/t4-current-kernels.json"

[[source]]
path = "fixtures/precision/t4-rigid-recapture-audit.json"

[[source]]
path = "docs/t4-rigid-recapture.md"

[[source]]
path = "docs/evidence/t4-g03-retest/flex-internal/map.json"

[[source]]
path = "docs/evidence/t4-g03-retest/flex-internal/audit.json"

[[source]]
path = "docs/evidence/t4-g03-retest/flex-matrices/map.json"

[[source]]
path = "docs/evidence/t4-g03-retest/flex-matrices/audit.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-polar/result.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-polar/audit.json"

[[source]]
path = "tests/support/t4_face_operators.rs"

[[source]]
path = "tests/support/t4_face_lineage.rs"

[[source]]
path = "docs/evidence/t4-g03-retest/face-operators/audit.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-operators/before.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-operators/capture.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-operators/factorial.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-operators/headers.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-operators/previous-kernels.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-operators/trace.json"

[[source]]
path = "tests/t4_hessian_generation.rs"

[[source]]
path = "docs/evidence/t4-g03-retest/hessian-generation/map.json"

[[source]]
path = "docs/evidence/t4-g03-retest/hessian-generation/audit.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-static/run.cjs"

[[source]]
path = "docs/evidence/t4-g03-retest/face-static/report.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-static/audit.json"

[[source]]
path = "tests/t4_machine_code.rs"

[[source]]
path = "docs/evidence/t4-g03-retest/face-machine/capture.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-machine/audit.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-machine/fusion.json"

[[source]]
path = "tests/t4_face_fusion_repair.rs"

[[source]]
path = "fixtures/precision/t4-current-capture-status.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-fusion-repair/audit.json"

[[source]]
path = "tests/t4_face_geometry_domain.rs"

[[source]]
path = "docs/evidence/t4-g03-retest/face-geometry/audit.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-current-compile/previous-kernels.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-current-compile/previous-status.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-current-compile/compile.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-current-compile/audit.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-current-compile/kernels.json"

[[source]]
path = "docs/evidence/t4-g03-retest/face-current-compile/status.json"

[[source]]
path = "tests/t4_trig_control_domain.rs"

[[source]]
path = "docs/t4-face-trig-control.md"

[[source]]
path = "docs/evidence/t4-g03-retest/face-trig/audit.json"

[[source]]
path = "tests/t4_camlight_precision.rs"

[[source]]
path = "docs/t4-camlight-internal.md"

[[source]]
path = "docs/evidence/t4-g03-retest/camlight-internal/map.json"

[[source]]
path = "tests/t4_tendon_wake_precision.rs"

[[source]]
path = "docs/t4-tendon-wake-internal.md"

[[source]]
path = "docs/evidence/t4-g03-retest/tendon-wake-internal/map.json"

[[source]]
path = "tests/t4_spatial_tendon_precision.rs"

[[source]]
path = "docs/t4-spatial-tendon-internal.md"

[[source]]
path = "docs/evidence/t4-g03-retest/spatial-tendon-internal/map.json"

[[source]]
path = "scripts/verify-static-gpu-references.ps1"

[[source]]
path = "docs/evidence/t4-g03-retest/d04-reference-inventory.json"

[[source]]
path = "scripts/prepare-g08-reference-input.ps1"

[[source]]
path = "fixtures/collision-reference-input/plane-sphere.xml"

[[source]]
path = "fixtures/collision-reference-input/plane-sphere-states.json"

[[source]]
path = "docs/evidence/t4-g03-retest/g08-reference-input.json"

[[source]]
path = "fixtures/collision-reference-input/plane-sphere.mjb"

[[source]]
path = "fixtures/collision-reference-input/plane-sphere-native.txt"

[[source]]
path = "fixtures/collision-reference-input/states-f32.json"

[[source]]
path = "fixtures/collision-reference-input/native-input-receipt.json"

[[source]]
path = "fixtures/collision-reference-input/.gitattributes"

[[source]]
path = "fixtures/collision-reference-input/native-contacts.cpp"

[[source]]
path = "scripts/collect-g08-native-contacts.ps1"

[[source]]
path = "fixtures/collision-reference-input/native-contacts.json"

[[source]]
path = "fixtures/collision-reference-input/native-contact-receipt.json"

[[source]]
path = "docs/evidence/t4-g03-retest/g08-native-contacts.json"

[[source]]
path = "src/model/collision.rs"

[[source]]
path = "src/io/collision.rs"

[[source]]
path = "tests/collision_input.rs"

[[source]]
path = "docs/evidence/t4-g03-retest/g08-host-input.json"

[[source]]
path = "docs/evidence/t4-g03-retest/g08-cache-discovery.json"

[[source]]
path = "include/mjwarp_collision_probe.h"

[[source]]
path = "native/model_probe_collision.cpp"

[[source]]
path = "src/io/collision_native.rs"

[[source]]
path = "tests/native/mock_collision.cpp"

[[source]]
path = "tests/native_collision_input.rs"

[[source]]
path = "scripts/test-windows-collision-input.ps1"

[[source]]
path = "docs/evidence/t4-g03-retest/g08-native-capture.json"

[[source]]
path = "src/model/collision_parameters.rs"

[[source]]
path = "src/io/collision_parameters.rs"

[[source]]
path = "tests/collision_parameters.rs"

[[source]]
path = "docs/evidence/t4-g03-retest/g08-parameter-batches.json"

[[source]]
path = "docs/normify-delivery-workflow.md"

[[source]]
path = "docs/development-evidence/schedule-reconciliation.json"
+++

2026-10-14 原编译选择最后额度已合法使用：核查原日志明确显式编译先前2次，3次宿主选择均ignored该用例；此前将宿主满额泛化为全部编译满额不准确。本轮执行原recompile_rigid_variants_without_launching_physics第3次，不改名不拆分不减覆盖，1过0败238过滤1.92秒；11变体重编译、19保留、30PTX类型门禁与3文件破坏拒绝通过，现编译选择满额禁止继续。当前面CUDA2548aee5…与PTX8057e350…真实绑定，NVRTC12.8.93 compute70 fmad=true fast-math=false，实际末项PTX三条fma.rn.f32已核。当前完整清单bafa1f32…共24入口30变体，对比前轮29条产物记录完全不变，仅面产物更新。旧清单eb16a66b…和旧状态71ad4173…原字节归档，旧CUBIN与PTX目录不改。当前状态只更新CUDA编译/PTX捕获为true，当前机器码/数值/full仍false，位差unknown、前轮2078失败保留。旧三轴宿主断言指向明确历史快照，未知漂移拒绝保留；另核新编译点数据，不运行老满额宿主谱系。t4_face_fusion_repair用第3次剩余额度4过0败0.01秒，现也满额。最后只把新编译点输入冻结到字节相同快照，Clippy通过，不追加第四次。原GPU/物理/CUPTI/Hessian/静态图满额目标均未执行，无GPU会话/内核/Python，未Linux发布原生数值复验。4来源5快照4日志90捕获文件哈希核实，格式/差异/文档审计通过。未改产品物理源码、原容差、全覆盖或21系数范围。NP01当前编译f32门禁通过但上游完整映射未齐；NP02新PTX齐机器码/Hessian缺；NP03数值未测；NP04公平性能显存未测。主排程原前置资源UID状态保持，下一独立候选三角控制域或其他既定就绪阶段。证据docs/evidence/t4-g03-retest/face-current-compile/audit.json。

2026-10-14 线性面可达几何域增量：排程未对子切片编码先后，不能称全局最早；几何域属于既列候选，原前置verified、资源锁保持。新增公开Model宿主检查，独立按六侧二维单元枚举64种1至4格三轴网格，2400面、9600角引用、12000负一尾槽，非零节点偏移8、面所属柔体1。实际公开接口全部接纳规范映射，46个非法状态拒绝（高阶、尾槽、缺角、外法向翻转、负二哨兵）。结合原源码精确槽位比较，关闭当前线性域中其他负值或高阶分支可达性的疑点；不覆盖原始私有缓冲或浮点几何/极分解/三角求值。未发现必要几何公式变更依据，产品源码不改。新t4_face_geometry_domain仅1次3过0败0.00秒；首次Clippy分块写法警告后只改as_chunks，最终Clippy通过，不重复测试。6来源及1日志哈希核实，文档审计0偏差。未运行任何旧满额目标、GPU、物理编译器或Python。当前三轴新源码仍未编译/捕获/数值执行，捕获失效状态字节未变，前轮2078失败保留。构建、捕获、物理测试操作不同，但不由源码变更推定新验证额度。21系数子集和完整G22界限、容差及覆盖保持。NP01只补宿主域，NP02/03/04仍开放。下一就绪为独立三角控制边界核对或明确合法构建取证边界。根计划态、相关叶活动态保持。见docs/evidence/t4-g03-retest/face-geometry/audit.json。

2026-10-14 三轴末项融合已落实：仅将xyz末项__fsub_rn(head,__fmul_rn(term))改为fmaf(-q,rot,head)。保留首项舍入乘积、此前两级FMA、w分量、矩阵/归一化、f32、迭代阈值、编译选项与ABI。全文件精确逆变换证明除此块与说明外无其他物理改动；FUSED_M→FIX_M→原始历史块只服务历史核验。旧t4-current-kernels.json及旧PTX/CUBIN回执字节未改，24入口30变体快照继续保留；新t4-current-capture-status.json明确flex_faces已陈旧，新源码c7670316…/CUDA2548aee5…尚未编译、捕获或数值执行。旧当前来源绑定门禁将拒绝新源码，不重跑或削弱该门禁。前轮2078失败保留，修复后位差记未知，不宣称归零。新宿主目标t4_face_fusion_repair累计2次，均3过0败（框架0.00秒）；8项负向拒绝通过，核对新表达、精确谱系和旧证据拒绝。未调用任何旧满额选择、GPU、物理编译器或Python。最终Clippy、格式、4来源/2历史证据/2日志哈希及差异检查通过，文档审计0偏差。21系数子集不扩展，完整G22另行计划。主排程/前置/资源锁、根计划态与面活动态保持。NP01新生成类型、NP02当前面PTX/机器码、NP03修复后数值、NP04公平性能均未验收。下步就绪为其他既定源码路径核对，或明确未满额的独立产物取证边界；不得重跑已满额面实验与捕获。证据docs/evidence/t4-g03-retest/face-fusion-repair/audit.json。

2026-10-14 实际加载机器码增量：新增Windows CUDA可选捕获目标，校验官方CUPTI/nvdisasm12.8.90包与本地库摘要。CUPTI模块加载回调直接复制本轮cuModuleLoadData原加载路径的CUBIN；未用cuLink，未覆盖JIT选项，未NVRTC重编译。最终产品19320字节/27df8f4f5026…，上游759904字节/2f82753498f1…；当前RTX4070TiSUPER/596.36，sm89，driverAPI13020。实际SASS确认：上游xyz末项0x7310/0x72f0/0x7340为FFMA；产品对应0x2a00/0x29f0/0x2a50为FADD且末项先FMUL。此前融合许可假设已在本轮实际加载路径证实。保留标准ELF格式警告；自动数据流关闭，保留原编码并人工追踪。此产物不是历史数值会话回收件；未证明全部2078位差归因或逐样本首差。独立捕获累计3次：头文件摘要文字错误0.14秒失败（无上下文），目标格式断言0.44秒失败（产品CUBIN已取得），最终1过0败1.26秒；停止重跑。产品反汇编累计3次（含手工预览），上游1次，版本查询1次。累计两个上下文、三个目标模块加载；物理样本/内核启动/物理缓冲分配/Python均0。旧满额测试完全不重跑。格式、Clippy、3份源码/回执、4份产物、3日志与当前清单绑定通过；文档审计0偏差。产品源码、容差与覆盖不改。完整NP01、2078数值归因及Hessian产物、NP03数值和NP04公平性能仍开放。主排程/前置/资源与模块状态不变。下一就绪项为末项融合修复及准确历史谱系设计，预先明确不重跑满额实验的验证边界。见docs/evidence/t4-g03-retest/face-machine/audit.json。

2026-10-14 既有面PTX静态核查：固定两份实际PTX原哈希，在相同q/F寄存器输入假设下核对19项名义运算图，均一致。该图未覆盖几何拼装、三角控制流或归一化分支，也不证明驱动机器码等价。四元数xyz末项，产品mul.rn/sub.rn而冻结上游mul/sub；NVIDIA PTX8.7明确省略限定允许乘减融合。已证实的是优化许可差异，未证实实际后端融合或2078项残余首差。此前掩码7轨迹不能冒充当前产品轨迹。本轮不盲改产品。静态工具累计3次（含探索14项一次、跨入口寄存器失败一次、最终19项及3差异与2负向通过一次），不再追加执行；旧满额面及Hessian目标没有重跑。另查4个指定柔体/研究证据目录，未找到CU/PTX/CUBIN；旧passive源码统一换行后匹配冻结源码，不是生成证据。原生wp_cuda_compile_program需要cuda_src；现有ModuleBuilder生成器依赖Python AST，本轮未执行。缺少实际passive模块CUDA/PTX及版本/选项回执；缺少已加载机器指令与本地反汇编工具。原容差、全部覆盖、产品源码及精度策略不变。无物理编译/GPU/Python执行，NP01–NP04继续开放。主排程与依赖资源保持，根计划态、面活动态。下一就绪项为几何/三角控制流静态核对和独立机器码获取条件。证据见docs/evidence/t4-g03-retest/face-static/audit.json。

2026-10-14 Hessian生成前置静态切片：固定11份来源、九组前置、七组累加路线、三份缓存文件。清零→单元→对角→有效标志顺序核实。当前运动学缓存不含Hessian符号，不能证明其他目录不存在。当前元数据3.15.0按源码推断选择24系数，但未实测导入值，也不证明历史编译环境；产品仍只支持21。冻结Warp矩阵/向量按分量转发设备atomicAdd；产品串行累加。宿主f32例证求和重排及零符号风险，不提供设备物理准入。新增目标t4_hessian_generation累计3次：首次默认特性编译失败，后两次cuda-probe各3过0败0.06秒；四项负向拒绝通过。保守累计已满额，不再重跑。格式、Clippy与来源/回执哈希通过。无物理编译、GPU会话、Python产品执行、Linux/发布验收。产品源码、原容差与旧全部覆盖未改。面2078严格位差阻断仍保留。NP01生成类型/24映射、NP02实际矩阵产物与原子PTX、NP03物理数值、NP04公平性能均未验收。主任务继续进行；根计划态、矩阵活动态、原依赖资源锁不变。下一项就绪为既有面PTX静态残差核对及独立矩阵产物获取契约。证据见docs/evidence/t4-g03-retest/hessian-generation/audit.json与docs/t4-field-internal.md。

2026-10-14 面算子隔离与局部修复：保留12样本、16面、1/2/5/513世界及四轮。五组编译器/目标/选项在当前样本上无跨组位差。生成极分解替换和完整上游重编译均零位差。轨迹33344面把首差定位至乘积xyz。产品修复基向量旋转、四元数乘积分组及倒数归一化；显式固定已证实的f32融合次序。独立产品从98625降至2078位差，最大5.960464477539063e-8；原容差外0。严格零位差断言仍失败，保留失败，不批准等价验收。新因子、轨迹、归档各累计3次，均满额；不得改名或拆分重跑。归档保留29份并更新1份，30份PTX类型检查通过，不提供数值准入。无旧满额重跑、无Python、无Linux/发布/完整原生复验。格式与Clippy通过；8份源码、6份证据及当前清单哈希核对通过。诊断显存与耗时缺口见伴随记录。NP01完整映射、NP02剩余位差、NP03当前红线、NP04公平性能仍开放。主任务保持进行态，根计划态、面活动态，原前置与资源锁不变。下一步仅静态核对残余PTX；另补Hessian生成条件与原子累加证据。详见docs/t4-field-internal.md与docs/evidence/t4-g03-retest/face-operators/audit.json。

2026-10-13 面产物实测：Rust直接运行冻结 face PTX，与当前完整面模板比较。原12样本、16面、1/2/5/513世界、每规模4轮，共2084世界及1033664分量。位置无位差；四元数98625项位差，最大4.76837158203125e-7，原容差外0。新诊断前两次因1024线程默认配置超出Warp256共享槽而终止；修复为256后第三次1过0败45过滤，0.33秒。选择累计3次已满额，不得重跑。产品公式、原容差、旧测试完整覆盖不变。NP03仅并列诊断，不提供原生红线准入；NP01/NP02全链与NP04公平性能未验收。自有分配峰值2160220字节，不含模块与驱动。GPU RTX4070TiSUPER/596.36；上游12.9.86 sm89，产品12.8.93 compute70。继续隔离算子与编译器贡献，再定产品修复和捕获；矩阵生成与原子累加仍缺证据。

2026-10-13面与矩阵切片已提交616555bd823b0f4fecdcb383a2e12a9c1ae4e863。仅六份目标文件。工作区干净。21组映射、四项输出、一个发布标志。定向四过，旧41项未运行。主任务继续，完整精度未验收。

## 2026-10-13 柔体面与矩阵内部量

新增21组映射：19组f32、一组i32缓存标志、一组宿主bool门控。
四项浮点输出追踪至宿主，另核缓存有效标志发布。
六份产品及七份冻结外部源码核对通过。
面、矩阵及发布三入口：六份历史、三份当前沿用捕获均实核。
复用冻结smooth缓存，核对CU/PTX原哈希及面入口。
极分解生成源码明确采用f32，调用mul后normalize。
上游生成Hessian产物仍缺，未生成或执行任何内核。
面旋转矩阵表达、乘积分组、归一化倒数乘法均与产品不同。
矩阵串行/原子累加不同；零符号中间量风险未量化。
同型不证明算子或数值等价，不盲改产品公式。
原21系数及线性面边界保持，不冒充24系数或高阶面。
新增选择三次：首次缓存后缀错误，3过1败；修正sm89后4过；补旋转表达后最终4过0败。
41项旧用例始终过滤，此新增选择已满三次。
旧测试正文逐字保留，容差、参考、生产源码及覆盖不变。
Clippy、格式、差异、清单回执哈希及133条本地链接通过。
文档工具报告284份源码、92份文档、零偏差。
最终宿主测试0.58秒，不是物理性能。
无GPU会话、Python执行、重新编译、Linux或发布验收。
NP01/NP02全链未完，NP03沿用历史，NP04未测。
保留主任务及模块状态，沿用根模块有效占用和已登记证据目录。
原依赖与独占资源保持，不启动并行任务。
后续优先极分解实际算子，再补矩阵生成及累加证据。
证据：docs/t4-field-internal.md#柔体面与矩阵切片。

2026-10-13柔体切片已提交bdcc9d97482d629c40bcabfa52c0c159590bcd4d。六份目标文件。工作区干净。校验零错误、两条历史警告。主任务继续进行；满额验证未重跑。

## 2026-10-13 柔体位置与边精度切片

16组内部量：15组f32，一组私有i32标志。
六项输出：五项f32，一项bool。
节点、直接顶点、插值、归一化、雅可比与边速度均已映射。
六维速度确认f32别名；标志解码仅接受零一。
五份产品与五份冻结外部源码实核。
核实六份历史捕获与三份当前沿用捕获。
产品全池清零，上游边核只写活动行。
未用尾槽副作用仍待初始化与可达状态证据。
求值顺序、上游生成语义仍待审。
本轮未修改任何生产源码、容差、参考及旧测试正文。
新增选择三次：首次缺环境变量，四过一败；后两次五过零败。
36项旧用例始终过滤。此新增选择已满三次。
最后宿主测试0.52秒，不算物理性能。
Clippy、格式、差异检查及清单回执哈希通过。
132条本地文档链接核对通过。
文档工具报告282份源码、92份文档、零偏差。
没有Python、GPU会话、物理执行或新内核编译。
未运行Linux、发布与满额验收。
NP01/NP02全路径未完，NP03历史，NP04未测。
工具拒绝扩展进行态占用，故沿用登记文件和目录前缀。
保留原UID及模块状态，不关闭主任务。
原资源acceptance:windows-product及gpu:windows-t4保持登记。
后续：柔体面与Hessian映射、尾槽副作用核对。
证据：docs/t4-packing-routes.md#柔体后续切片。

## 2026-10-13 编译切片提交

提交：`e272b1474391c4b875ad89b70a7e865a33eec914`。
本轮仅新增测试与编译证据。
十份重新编译，二十份核实沿用。
三十份均通过原精度门禁。
原容差与全部覆盖不变。
主排程继续进行，完整T4未验收。

## 2026-10-13当前产物增量编译

保留旧清单与历史回执。
另建当前编译谱系清单。
首轮仅补三种直接刚体变体。
复查发现附着及相机模板复用。
第二轮补齐全部十个受影响变体。
当前提交只采纳第二轮集合。
三种刚体、五种附着、两种相机。
其余20份产物核实后原样沿用。
当前集合覆盖24入口、30变体。
三十份PTX均再次通过原精度门禁。
三类文件破坏均触发拒绝。
检查器拒绝遗留旧附着产物。
谱系保留全部映射及未验收边界。
完整刚体PTX匹配前轮608项实测。
两种严格刚体PTX保持原字节。
附着完整及其初始化PTX同步变化。
本轮不重新执行物理或GPU工作。
NVRTC捕获两次各1过0败。
新宿主选择三次均2过1忽略。
忽略项已经另行显式运行。
该宿主选择已满三次。
旧满额配置、容差和覆盖不变。
新增源码仅为测试挂接与检查器。
生产公式、ABI及公开接口不变。
格式、Clippy及差异检查通过。
两份清单回执哈希核对通过。
133条本地链接通过。
文档工具280源码、92文档零提示。
最终1410毫秒属于编译核验耗时。
未测物理性能及显存峰值。
NP01/NP02整体及G01集成仍未验收。
NP03保留历史，NP04仍待实测。
本轮不激活计划模块。
主T4继续进行，不关闭历史变更。
仅同步本机伴随目录。
证据：C:/Rust/mjwarp-rs/docs/t4-rigid-recapture.md。

## 2026-10-12零四元数提交收尾

提交：af7f8871c7377d016ed04e99440e5a1486dcc318。
本轮仅提交18份目标文件。
提交后工作区干净，未推送。
608项定向对照全部一致。
六项刚体静态检查全部通过。
两组定向选择均已满三次。
原容差与全部覆盖保持不变。
21项来源戳与两份结果哈希通过。
130条本地文档链接通过。
完整精度与性能仍未验收。
旧完整产物不能放行当前产品。
架构校验零错误、两项历史警告。
主T4继续保持进行态。
仅同步本机伴随记录。

## 2026-10-12零四元数产物核验

Rust直启冻结Warp缓存PTX。
本轮不启动产品Python链。
三处实际归一化采用f32四元数。
缓存实测返回[0,0,0,1]。
注册文档的全零说法不符。
完整入口仅修改一个零值分支。
严格入口与正长度公式不变。
四世界、五体、两关节、七输出。
覆盖自由、球、mocap与自由子体。
608项比较由129差异降为零。
新选择累计三次：适配失败、旧产品失败、修复后2过。
刚体静态选择累计第三次6过。
两组均不可原配置强制重试。
旧满额配置、原容差与覆盖不变。
只重编译G01测试，不重跑旧物理配置。
Clippy、格式与差异检查通过。
定向耗时290毫秒，显式缓冲5872字节。
这些值不代表稳态性能或进程峰值。
四份静态清单刷新源码戳。
旧捕获、旧回执保持历史字节。
总产物门禁仍阻止当前源码漂移。
完整当前捕获及G01集成待补证。
NP01/NP02不整体放行。
NP03沿历史，NP04仍未验收。
调度门禁拒绝重叠子任务。
主任务不能回退候选态。
沿现有主任务串行登记本切片。
不关闭主验收，不激活计划模块。
仅同步本机伴随目录。
文档工具277源码、91文档零提示。
证据：C:/Rust/mjwarp-rs/docs/t4-zero-quaternion.md。

## 2026-10-12刚体内部量提交收尾

提交：dcb1fae771c45ca5df823d0b2517f04724f625b2。
只提交本轮八份文件。
工作区干净，未推送。
17组内部量与七项输出已映射。
新增六项两次全部通过。
旧28项及满额配置未重跑。
原容差、公式、参考和覆盖不变。
Clippy、格式和暂存检查通过。
128条本地链接及归档哈希通过。
文档工具274源码、90文档零提示。
架构校验零错误。
保留两项历史粗粒度警告。
架构构建通过。
零四元数源码分歧仍阻止等价放行。
上游实际生成产物仍未核验。
完整精度与真实性能仍待验收。
后续先核对零四元数实际产物。
柔体与空间肌腱映射仍待补齐。
本轮只同步本机伴随目录。
主T4保持进行态。
原模块UID与计划状态不变。

## 2026-10-12刚体内部量与输出映射

本轮沿原排程串行落实。
登记17组刚体内部量映射。
七项输出追踪到宿主视图。
覆盖四种关节及mocap路径。
局部量、累加和输出保持f32。
只读f64诊断不回写物理结果。
保留世界体、串行分支调度边界。
三份产品及八份外部来源绑定哈希。
双模式核对六份既有入口产物。
每模式含普通、常驻严格和完整入口。
分别核对CUDA、PTX与精度回执。
实际宏定义不与注释混淆。
冻结源码显示零四元数分歧。
本库完整入口宏返回四分量全零。
Warp头文件返回四分量末位一。
Warp注册文档却宣称返回零。
完整状态入口不排除零四元数。
本轮只登记源码阻断，不改公式。
上游生成产物仍未核验。
对应等价资格保持为假。
新增rigid_internal选择执行两次。
两次均6过、0败、0忽略。
每次耗时0.16秒。
第二次补齐CRLF兼容检查。
旧28项用例保持过滤。
满额配置调用次数保持不变。
产品公式、参考、容差和覆盖不变。
不运行产品GPU或Python链。
Linux及真实性能本轮未测。
格式、Clippy及差异检查通过。
128条本地文档链接通过。
清单哈希与归档回执匹配。
文档工具274源码、90文档零不匹配。
修正一处外部路径识别提示。
人工核对本轮限定范围。
NP01及NP02仍未完整验收。
NP03沿用历史，NP04本轮未测。
后续先核对零四元数实际产物。
再继续柔体及空间肌腱映射。
主T4继续保持进行态。
不激活计划模块，不关闭历史变更。
只同步本机根模块与来源映射。
证据：C:/Rust/mjwarp-rs/docs/t4-rigid-internal.md。

## 2026-10-12拓扑与位姿提交收尾

提交：9753037e7b3a783fa6f8a05a77cfea854923f83a。
只提交本轮八份文件。
提交后工作区干净，未推送。
新增五项用例一次全过。
旧23项用例未重跑。
原容差、参考及覆盖不变。
Clippy、格式及暂存检查通过。
126条本地文档链接通过。
清单哈希与归档回执匹配。
文档工具271源码、89文档零提示。
架构校验零错误。
保留两项历史粗粒度警告。
架构构建通过。
完整精度与性能仍待验收。
下一步展开刚体内部量及输出。
随后补柔体及空间肌腱链。
保留上游生成产物与性能缺口。
只同步本机伴随目录。
主T4保持进行态。
原模块UID与计划状态不变。

## 2026-10-12拓扑与附着相机精度映射

沿原排程串行落实本轮切片。
四组拓扑登记23条路由。
它们引用19项去重上游字段。
十八条整数直接复制。
三条保存私有步长。
两条保存数量与静态标记派生。
附着相机内部量映射16组。
八项输出追踪到宿主视图。
单列旋转求值顺序差异。
单列静态缓存和两次启动边界。
无效光源目标保留跳过归一化。
五份产品源码绑定LF哈希。
六份外部来源冻结实核通过。
双模式核对24份既有入口产物。
其中18份启用对应常驻宏。
源码、PTX与精度回执均核对。
本轮不编译或运行GPU内核。
产品测试不启动Python。
新增topology_frames选择执行一次。
5过、0败、0忽略，耗时0.49秒。
旧23项用例保持过滤。
旧满额验证次数保持不变。
原容差、公式、参考和覆盖不变。
格式、Clippy及差异检查通过。
126条本地文档链接通过。
清单哈希与归档回执匹配。
文档工具检查271源码、89文档。
工具报告零不匹配。
人工核对本轮限定范围。
NP01仍缺其他内部量与输出。
算子顺序及预处理语义仍待审。
NP02仍缺上游生成产物。
NP03保留历史，NP04本轮未测。
下一步展开刚体内部量及输出。
随后补柔体与空间肌腱链。
完整T4继续保持进行态。
不激活模块，不关闭无关变更。
本轮只同步本机伴随记录。
证据：C:/Rust/mjwarp-rs/docs/t4-topology-frames.md。

## 2026-10-12 其余打包路由提交收尾

提交：20df13fa6ddae17794b334d569e198e938ace6cd。
提交后工作区干净。
本轮提交八份目标文件。
八个打包器登记82条路由。
关联56项冻结上游字段。
固定肌腱内部量映射四组。
双模式既有产物核对16份。
新五项用例两次全部通过。
两次均过滤旧18项用例。
原容差、参考和覆盖不变。
满额验证调用次数不变。
Clippy、格式及暂存检查通过。
123条本地链接检查通过。
文档审计零不一致。
Normify校验零错误。
保留两项历史叶子警告。
架构构建通过。
本轮不运行产品GPU内核。
产品测试不启动Python。
完整精度与性能仍待验收。
主变更继续保持进行态。
原模块UID与计划状态不变。
下一步映射刚体相机整数拓扑。
随后展开其他内部量及输出。
上游生成产物仍需补证。
真实性能仍需独立实测。
本轮只同步本机伴随目录。

## 2026-10-12其余打包器与固定肌腱内部量

本轮沿原任务串行推进。
展开其余八个打包器。
登记82条字段及派生段路由。
它们引用56项去重上游字段。
包括八组头部、41条整数复制。
另有九条浮点复制与滑轮预计算。
单列17条派生、两条布尔编码。
保留三条浮点占位及一条未用整数。
未用槽为Hessian的flex_elemadr。
检查器区分r[6]与upper[6]。
固定肌腱另映射四组内部量。
产品与上游采用f32，累加调度不同。
本轮不宣称逐算子等价。
15份产品来源绑定LF哈希。
冻结上游类型与计算源码实核通过。
双模式合计16份既有产物匹配。
源码、PTX与回执分别核对哈希。
本轮没有编译或执行GPU内核。
新增packing_routes选择执行两次。
两次均5过0败0忽略。
耗时分别0.88秒与0.83秒。
第二次补全派生引用缺项负例。
旧18项保持过滤，不算重新通过。
完整T4与旧满额目标次数不变。
不启动Python或原生动态库。
产品源码、参考、容差与覆盖不变。
格式与Clippy检查通过。
NP01仍缺刚体相机整数拓扑。
其他物理内部量与输出也待审。
NP02仍缺上游生成产物全证据。
NP03沿用历史，NP04本轮未测。
主T4继续保持进行态。
不关闭无关变更，不激活计划模块。
仅更新本机根模块及源码映射。
证据：C:/Rust/mjwarp-rs/docs/t4-packing-routes.md。

## 2026-10-12逐字段内部量提交

提交：9db581d509adbaf487cbe0498e0b7c2b9a5caba1。
仅提交本轮八份文件。
提交后工作区干净，未推送。
新增五项用例通过，旧十三项未运行。
新增选择累计一次，旧满额次数不变。
Clippy、格式与暂存检查通过。
121处本地链接与归档哈希通过。
文档工具265源码、87文档，零提示。
修正的路径提示只涉及上游路径说明。
架构校验零错误、两项历史警告。
警告仍涉及柔体与绕行粗粒度。
架构构建通过，不替代产品验收。
后续继续展开其余八个打包器。
同时补其余物理内核内部量映射。
随后核对上游产物及真实性能。
本轮只同步本机伴随记录。
主T4保持进行态，不关闭或激活模块。

## 2026-10-12逐字段与质心内部量

本轮沿主任务串行落实静态切片。
展开23项独立批量参数打包。
逐项绑定共享字段、顺序、描述槽与设备读取。
记录pack_model和pack_camlight两条复制路径。
质心内部量登记15组计算映射。
产品参数、局部量与累加保持f32。
描述符与索引保持整数。
单列派生子树质量与不同累加顺序。
类型相同不等于逐算子等价。
六份产品来源与上游smooth.py绑定哈希。
读取双模式20份既有产物。
其中14份启用批量宏。
每模式覆盖全部23项读取。
分别核对CUDA、PTX与精度回执哈希。
核对冻结上游与Warp类型文本。
不运行Python、GPU或原生动态库。
新增field_internal命名空间五项全过。
本轮只执行该新增选择一次，耗时0.99秒。
过滤13项旧用例，不声称重新通过。
旧宿主完整目标与T4次数保持不变。
Clippy、格式检查通过。
产品源码、原参考、容差与物理覆盖不变。
NP03沿用既有历史数值证据。
NP01仍缺其余打包、拓扑和内部量。
NP02仍缺上游生成产物全证据。
NP04仍缺公平稳态与进程分配峰值。
完整T4继续in_progress。
不调用主验收关闭工具。
不激活计划模块，不关闭历史变更。
本轮仅更新本机根节点及源码映射。
证据：C:/Rust/mjwarp-rs/docs/t4-field-internal.md。

## 2026-10-12状态打包提交

提交：78601790d2d09d8d08a93ba6876b388f1ca43174。
只提交本轮八份文件。
工作区干净，未推送。
最终13项宿主验证通过。
该目标累计三次，停止追加。
架构校验零错误、两项历史警告。
主T4保持进行态。
下一步补逐字段打包及内部量映射。
随后核对上游产物与真实性能。

## 2026-10-12状态与设备打包追踪

本轮串行落实NP01状态与存储切片。
核对G01State十项及休眠五项声明。
映射25项去重上游字段。
核对十个打包器的i32元数据与f32参数。
柔体边打包器仅返回整数。
记录空间肌腱f32滑轮预计算。
单列限位、门控、居中和有效标志的编码。
子树质量属于GPU派生缓存，不冒充上游模型直接复制。
七份产品来源绑定LF哈希。
双模式各四份既有CUDA读取产物实核通过。
同时核对配套PTX与原回执哈希。
只读取冻结Python文本，不运行解释器。
本轮无GPU、原生动态库、Linux或真实性能执行。
产品源码、物理参考、容差和覆盖保持不变。
旧完整T4调用次数不变。
扩展原宿主目标，不另建规避次数的测试目标。
本轮先12过1败，检查器误用源码哈希定位文件。
定位规则实际采用PTX哈希。
修正检查器后13过0败0忽略，耗时4.74秒。
原始失败日志和修复日志均保留。
该宿主配置累计三次，不再追加。
Clippy、格式、链接和回执哈希检查通过。
文档工具262源码、86文档，0处不匹配。
全模型逐字段打包、局部量、累加与完整输出仍待审。
其他物理状态、上游生成产物和真实性能仍缺证据。
NP03沿用历史数值证据，不撤销也不虚构新验收。
主T4继续in_progress。
本轮只同步本机根模块。
不激活计划模块，不关闭无关历史变更。
证据：C:/Rust/mjwarp-rs/docs/t4-state-packing.md。

## 2026-10-12入口映射提交

提交：248a28a7242525d5f86871d4b588e50c1a27693b。
只提交本轮七份文件。
提交后工作区干净，未推送。
新增宿主目标8过0败0忽略。
该配置累计一次。
架构校验零错误、两项历史警告。
完整T4仍保持进行态。
下一步追踪状态与设备侧精度。
随后补上游产物和真实性能。

## 2026-10-12原生入口逐字段映射

落实78项模型入口清单。
包含12运动学、9惯性、12柔体、45附加字段。
逐项核对原生ABI、暂存、宿主转换和上游声明。
18份产品源码绑定LF哈希。
32项原生f64均转换为f32。
75项宿主标量匹配，3项布尔适配单列。
适配不豁免硬指标，也不冒充完整语义等价。
冻结上游与Warp 1.15.0类型源码实核通过。
新增独立宿主目标8过0败0忽略，耗时4.19秒。
该新配置只执行一次。
原产品目标与完整T4调用次数不变。
Clippy和格式检查通过。
文档工具259源码、85文档、0处不匹配。
产品算法、原生参考、容差和物理覆盖未改。
测试不启动Python、GPU或原生动态库。
本轮未测Linux，也未采集真实性能。
NP01仍缺状态、打包、中间量、累加及输出。
NP02仍缺上游生成内核与编译库全证据。
NP04仍缺双方公平稳态与进程峰值。
主T4继续in_progress。
原UID、计划状态与历史变更保持不变。
证据：C:/Rust/mjwarp-rs/docs/t4-input-precision.md。
清单：fixtures/precision/g01-inputs.json。
回执：fixtures/precision/g01-input-audit.json。

## 2026-10-12证据门禁提交

提交：62b92e88100d20939c1a5c0a96f5f5f20626e3d2。
工作区干净，尚未推送。
本轮只新增宿主证据检查。
最终11项全部通过。
该配置累计三次，不再重复。
本轮未重跑完整产品或Linux。
性能正例仅为合成真值表。
完整逐字段精度与真实性能仍待补齐。
主T4继续保持进行态。
不以宿主测试通过关闭完整门禁。

## 2026-10-12精度来源与性能条件门禁

新增独立Rust宿主证据目标。
冻结24个同步物理入口。
核对双模式各30份变体。
登记20份本库来源与7份上游来源。
分别验证源码、PTX与回执哈希。
严格拒绝缺件、重复件及覆盖缩减。
四个整数入口另记私有标记边界。
性能条件检查绑定计划、源码和产物。
它拒绝输入、设备、精度不一致。
它拒绝冷启动与板卡显存快照。
固定G01四组世界数和四次更新。
真实性能采集器本轮仍未实现。
性能真值表均为合成数据。
不将合成正例登记为实测成绩。
新增宿主目标先10过，后两次各11过。
该配置累计三次，不再重复。
Clippy与格式通过。
文档工具256源码、84文档，0提示。
本轮没有产品GPU或Python测试。
完整产品验证账本次数保持不变。
产品算法、参考、容差保持不变。
NP01仍缺完整逐字段映射。
NP02仍缺上游生成内核全证据。
NP04仍缺真实双方性能记录。
因此主T4继续in_progress。
重复专项提案未启动，已撤销。
原任务与模块UID均保持不变。

证据：C:/Rust/mjwarp-rs/docs/t4-evidence-gates.md。
清单：C:/Rust/mjwarp-rs/fixtures/precision/t4-kernels.json。
回执：C:/Rust/mjwarp-rs/fixtures/precision/audit.json。

## 本轮提交与收尾

提交：8c0f8147dac3422724f00abe2d1916bac93e4e14。
本轮只提交文档与验收证据。
本轮没有推送提交。
提交后工作区干净。
文档工具扫描252源码、83文档。
工具报告0处不匹配。
归档哈希与本地链接通过。
暂存差异检查通过。
架构校验0错误、2项旧警告。
本轮完整数值复测已完成。
完整T4验收仍缺独立硬指标。
因此不调用主验收关闭工具。
后续先补精度全路径证据。
随后补公平稳态和峰值显存。
本轮不追加任何产品测试。
保留主任务与所有历史状态。

## 2026-10-12完整T4追加验证

本节定义当前进度口径。
以下旧节保留历史状态。
源码基线4dc8893。
用户允许双模式各追加一次。
保留32个产品目标及全部物理覆盖。
双模式各483过0败0忽略。
库另过滤一项满额首差诊断。
原393两项独立诊断不重跑。
不宣称全部仓库诊断重跑。
原生受限另各27过。
仅原生配置另各1过。
GNU宿主与文档另226过。
每模式65733块原生放行。
另29块采用上游对照放行。
失败与待验收均为零。
双模式NP03摘要逐字节一致。
原容差、参考与产品源码不变。
30份物理PTX变体通过类型检查。
测试负例按预期拒绝f64。
NP01与NP02仍缺全路径证据。
NP04仍缺公平稳态性能对照。
板卡快照不等于进程峰值。
Linux本轮未测。
完整G03仍有路由和调度缺口。
完整T4主变更继续开放。
后续先补逐算子精度映射。
随后补同条件稳态耗时及显存。
追加完整验证额度已用尽。
不激活计划模块。
不关闭无关历史变更。

证据：C:/Rust/mjwarp-rs/docs/windows-t4-g03-retest.md。
归档：C:/Rust/mjwarp-rs/docs/evidence/t4-g03-retest/。

## 当前G03辅助首差修复（2026-10-12）

本轮解除G03辅助数值阻断。
沿用原任务与模块UID。
失败状态的矩阵与上游逐位一致。
首差来自分解首项的平方根路径。
产品对齐rsqrt.approx及融合许可。
回代算法与既有容差保持不变。
双模式集成各8过0败。
双模式单元各10过0败。
每模式1411次世界比较。
7027块原生放行、20块对照放行。
失败与待验收字段块均归零。
三次诊断与唯一追加专项分别记账。
调试专项累计5次，发布累计4次。
本轮没有重跑G01、393或矩阵专项。
默认宿主测试226项通过。
格式与全特性Clippy通过。
四类生成内核通过f32检查。
完整逐算子等价仍缺证据。
NP04仍缺稳态和进程峰值对照。
Linux与完整T4本轮未测。
主T4变更保持开放。
完整G03仍有路由和调度缺口。
不激活计划模块。
不关闭无关历史变更。
以下旧节只保留历史状态。

证据：C:/Rust/mjwarp-rs/docs/windows-g03-first-difference.md。

## 2026-10-12 G03接续

分块格式与双参考已实现。
每模式7017块原生放行。
每模式20块通过上游对照。
每模式10块越过误差红线。
缺参考待验收降为零。
所有失败定位到mixed-joints状态1。
重复右端项去重后涉及两组解。
没有确认唯一误差根因。
本轮没有重复失败测试。
追加专项额度已经用尽。
NP04仍未验收。
G03修复任务保持进行中。
完整G03与T4仍不放行。
证据见docs/windows-g03-block-repair.md。

## 当前阶段与排程（2026-10-12）

本段定义当前进度口径。
以下旧段保留历史证据。
正式准入不等于子集交付。

| 阶段 | 实现证据 | 正式准入 |
| --- | --- | --- |
| P0 | 范围与接口已有盘点 | 契约与环境仍待收口 |
| P1 | 执行路线已有探针 | 双平台路线仍待冻结 |
| P2 | G01已有78项读字段 | 完整模型与资产待续 |
| P3 | G01常驻底座已落地 | 完整状态交换待续 |
| P4/G01 | 完整入口与局部NP03通过 | 完整NP证据仍待验收 |
| P4/G02 | 速度与偏置力子集已落地 | 完整RNE与调度待续 |
| P4/P6/G03 | 反向LDL辅助已落地 | 默认分块与完整族待续 |

当前只推进G03辅助修复。
完整T4主验收等待该修复。
两项共用本机验收资源锁。
历史三项回队待续。
它们须先完成人工范围复审。
复审提案不阻断正式产品路线。
它们不撤销旧交付证据。
本次不关闭历史变更。
本次不激活计划模块。

NP01与NP02按批次先行。
P11条目只做最终汇总。
NP03仍缺7组同条件参考。
它们产生38次待验收。
NP04仍缺耗时与显存对照。
正式P0至P4不宣称完成。

源码快照：fa2037babb549af6d477b9b28b65133dfcd9d9b2。
工作区保留两处未提交修改。
矩阵修改只调整累加顺序。
脚本只修正内部精度报告。
本次仅审查实际Git差异。
新GPU回归仍未执行。
当前指纹只标记源码快照。
它不证明产品验收通过。
产品全链f32口径保持不变。
本次未运行Rust或GPU测试。
Linux本轮尚未补验。
远端结构本次尚未同步。

证据：C:/Rust/mjwarp-rs/docs/windows-t4-redline-retest.md。
排程：changes/2026-10-11-g03-mjwarp-reference.json。
准入：changes/2026-10-11-mjwarp-parallel-acceptance.json。

## 本轮T4完整产品复测（2026-10-11）

源码基线76e9118028ad3418a6aa5573a88e1c2ce8817ded。
沿用native_error_v2与5%红线。
本轮不改源码、测试或容差。
只新增复测证据与文档记录。
Windows双模式各474过3败。
各模式没有忽略用例。
合计含主体、原生输入和文档测试。
宿主测试不能全部算作物理证据。
受限原生另各27过。
仅原生feature另各1过。
GNU宿主与文档共225过。

主体包含库、程序和32个目标。
原生脚本另覆盖27项输入测试。
诊断frame393_trace本轮未执行。
它含两项诊断，已累计调用三次。
历史诊断不冒充本轮新证据。
本轮不宣称全部仓库用例重跑。
G01与运动肌腱本轮各一次。
它们自上限指令后累计三次。
未重试失败测试。
各命令记录在calls.json。

三项失败均来自质量求解。
它们报告38次待验收。
原因均为missing_mjwarp_reference。
这些记录对应7种原生字段参考。
原生误差最高为容限的2.744052倍。
缺少同条件上游误差，不能计算红线。
残差与重构不变量通过。
这不替代第二参考。
完整T4继续不放行。

G01主链与低层全部原生放行。
运动肌腱18块原生放行。
另9块对照放行。
393帧仍原生超差。
双模式NP03摘要逐字节一致。
累计65715块原生放行。
另9块二级放行、38块待验收。
二级均属对照放行。
没有实际字段使用5%余量放行。
并列诊断仍不作准入依据。

全套Clippy、格式与Rustdoc通过。
文档工具扫描230源码、79文档。
工具报告零处不匹配。
四份变更文档本地链接齐全。
本轮未测Linux或NP04性能。
不激活计划架构。
布局、API和模块依赖均不变。
只关闭本轮复测记录变更。
主T4变更继续保持开放。

报告：C:/Rust/mjwarp-rs/docs/windows-t4-redline-retest.md
日志：C:/Rust/mjwarp-rs/target/t4-redline-retest/
变更：changes/2026-10-11-t4-redline-retest.json

## 现行误差规则与前轮专项证据（2026-10-11）

用户在本轮测试前批准5%余量。
当前规则为native_error_v2。
第一参考始终为原生MuJoCo。
原生达标直接放行该项NP03。
仅原生超差后启用同条件MJWarp。
两路误差都相对同一原生参考。
每字段块取最大归一化误差。
分母沿用原生绝对加相对容差。
本库不劣于MJWarp时对照放行。
误差略高但未越5%红线也放行。
红线等于MJWarp误差乘1.05。
等于边界允许通过。
5%不作用于物理输出值。
本轮没有放宽原生容差。
本轮也没有修改产品物理内核。

越过红线严格禁行。
缺少必要参考仍待验收。
身份、条件或非有限异常禁行。
零数值证据不能通过汇总。
独立诊断明确不作准入依据。
NP01、NP02与NP04继续独立。
报告升级schema 2。
它明确记录准入标志与原因。
mjwarp误差现在也指向原生基准。
旧值差报告不能沿用新语义。

GNU宿主规则20项通过。
MSVC双模式专项各35过0败。
其中G01各7过，几何肌腱各8过。
宿主规则各20过，不算物理证据。
各配置本轮只调用一次。
运动肌腱18块原生放行。
另9块对照放行，没有红线内放行。
393帧本库分数12.236763。
MJWarp分数12.240589。
所以该字段放行，但仍原生超差。
本轮改变规则，不改变物理结果。

完整T4、Linux与性能未重测。
旧393诊断已满三次，不再运行。
质量求解仍缺同条件第二参考。
完整T4主变更继续开放。
产品测试没有启动Python。
全特性全目标Clippy与格式通过。
文档工具检查230源码、78文档。
工具报告0处不匹配。
本轮不激活计划架构。
布局层级、API和依赖保持原样。

证据：C:/Rust/mjwarp-rs/docs/native-error-redline-acceptance.md
契约：C:/Rust/mjwarp-rs/docs/numerical-precision-contract.md
变更：changes/2026-10-11-acceptance-verdict-labels.json

## 以下保留历史阶段证据

历史数值与失败记录保持原样。
若历史门禁冲突，以顶部规则为准。

## 两处初始长度修复（2026-10-11）

产品新增inside_norm。
它先舍入x平方，再融合y平方。
inside仅用它计算l0与l1。
通用n2、f32与迭代阈值不变。
本轮没有改动两级容差。
第393帧六个接触坐标逐位一致。
其MJWarp力臂差降至8.94e-8。
原生力臂差增至2.8593e-4。
基础样本1新增两坐标1 ULP差。
因此不宣称原生误差修复。
也不宣称全样本逐位一致。

双模式局部回归各35过1败。
G01相关四目标各22项通过。
运动肌腱仍有9次字段块待验收。
缺批准二级容差仍阻止准入。
所有配置本轮只调用一次。
旧393诊断此前已调用三次。
本轮没有再次运行该诊断。
完整T4、Linux与性能本轮未测。
产品测试没有启动Python。
Clippy全目标全特性与格式通过。
文档工具检查230源码与77文档。
工具报告0处不匹配。

证据：C:/Rust/mjwarp-rs/docs/inside-norm-order-fix.md
变更：changes/2026-10-11-inside-norm-order-fix.json
完整T4主变更继续开放。

## 前轮两级机制与完整T4补测

用户选择先接入机制。
缺少二级容差时记录待验收。
本轮不采用上游通用容差。
原生容差与物理内核保持不变。
G01主链与低层已接入判定。
运动肌腱与G03辅助也已接入。
其他辅助仍保留原有门禁。

完整回归双模式各463过4败。
两种模式均无忽略用例。
4项因待验收而拒绝准入。
每种模式47次字段块待验收。
肌腱9次缺批准容差。
质量求解38次缺同状态参考。
重复世界与调用分别计数。
两种模式的判定摘要完全相同。
G01专项各7项通过。
GNU宿主216项通过。
格式、Clippy与Rustdoc均通过。
文档工具227源码、75文档。
工具报告0处不匹配。
结构校验0错误、2警告。
仅本轮机制与补测工作收尾。
完整T4主验收继续开放。
本轮未测Linux与NP04性能。

证据：C:/Rust/mjwarp-rs/docs/windows-two-reference-acceptance.md

## 前轮漂移修复与完整T4补测

已逐项复审八个过期模块。
复审覆盖Linux桥接与f32恢复。
也覆盖常驻速度前置链。
保留计划API与原有模块状态。
12处文档路径提示已清零。
结构校验为0错误、2警告。
两组G02样本固定LF检出。
12个文件匹配原清单哈希。
本轮不修改物理公式或容差。

双模式完整回归各451过4败。
质量求解3败，运动肌腱1败。
G01专项双模式各7项通过。
G02速度各7过，RNE各8过。
上游并列诊断各4项通过。
诊断不代表二级数值门禁通过。
完整T4保持未验收。
仅本轮修复与补测工作已完成。
本轮未验收Linux或NP04性能。

证据：C:/Rust/mjwarp-rs/docs/windows-t4-recheck.md。

## 历史值差门禁（现行规则见顶部）

第一参考标准为原生MuJoCo。
原生通过即通过该项数值门禁。
此时不要求MJWarp再次通过。
仅原生超差后启用冻结MJWarp。
第二参考通过也通过数值门禁。
两路均超差则判失败。
缺少所需参考或容差则待验收。
本规则不要求双路同时通过。
两类通过具有相同准入效力。
报告另记实际采用的参考。
可选上游诊断不追加否决权。

两路按字段分别冻结容差。
不得根据失败结果放宽容差。
三方核对有效模型与状态。
版本与量化差异须单独登记。
结构错误不进入容差回退。
非有限值直接判失败。
内部精度仍跟随冻结上游。
NP01、NP02与NP04继续独立检查。
NP03改为两级数值判定全覆盖。
失败与待验收数量均须归零。

T4表示Windows本机GPU验收。
它覆盖完整G01与低层入口。
它不表示显卡型号或肌腱编号。
双模式均须记录实际测试证据。
产品测试不启动Python。
本轮已重跑完整T4范围。
两级机制已接入G01主链与低层。
运动肌腱与G03辅助也已接入。
其余辅助仍保留原有门禁。
用户选择缺容差时记待验收。
本轮不采用上游通用容差。
待验收继续阻止完整T4通过。
本轮不改写历史测试结果。
完整T4仍保持未验收。
未审查源码不刷新历史指纹。

Native MuJoCo is the first numerical reference. A native-tolerance pass is sufficient for the corresponding NP03 check; no MJWarp numerical pass is additionally required. Only a native-tolerance failure invokes frozen MJWarp as the second reference. A second-reference pass also passes NP03. Both failing means failure; missing required references or frozen tolerances means unverified. NP01, NP02 and NP04 remain independent. Automated tests now cover G01 main/raw fields, motion tendon blocks and G03 mass-solve references. Other auxiliary gates remain unchanged. Missing second-reference evidence or approved tolerances stays pending and blocks T4; the user explicitly chose not to freeze secondary tolerances in this change.

契约：C:/Rust/mjwarp-rs/docs/numerical-precision-contract.md
变更：changes/2026-10-11-mjwarp-parallel-acceptance.json

## 本机并列诊断进展（2026-10-11）

已补37组冻结MJWarp参考。
本机对照覆盖几何肌腱子集。
FMA设置现已跟随上游。
第310帧力臂已逐位对齐。
第393帧差异仍待收口。
完整G01原生回归双模式各7通过。
上述原生证据需按新规核验。
完整T4仍保持未验收。
相关测试不启动Python。
两级补测另见最新报告。
C:/Rust/mjwarp-rs/docs/windows-two-reference-acceptance.md
此前记录8处源码漂移。
本轮逐项复审后刷新指纹。
本节更新下面的历史操作状态。

证据：C:/Rust/mjwarp-rs/docs/windows-mjwarp-geom-tendon-comparison.md。

## 历史阶段记录

以下描述对应当时实现。
当前精度遵循首部现行规则。

## 本轮实现证据

库入口导出基础接口与探针。
Cargo锁定CubeCL与pliron。
GNU配置保留编译器注册项。
本模块仍表示计划架构。
完整物理与渲染仍待实现。

## P1缓存依赖

cuda-probe可选接入serde与JSON。
SHA256使用固定版本sha2。
默认构建不接入缓存依赖。
冻结CubeCL与上游提交不变。
部署烟测仍不代表正式准入。
七个产品模块继续保持计划。

证据：C:/Rust/mjwarp-rs/docs/windows-artifact-probe.md。

## 基础源码落地

2026-10-07：导出七个顶层模块。
五个新模块各有真实基础行为。
新增接口不替代原141项契约。
八项数学公式取得宿主测试证据。
其余正式目标继续保持计划。
运行时拆分七个职责叶子。
25项旧映射全部保留。

证据：C:/Rust/mjwarp-rs/docs/foundation-architecture.md。

## Windows字段交换增量

连续字段已有真实GPU交换。
运行时新增同步传输叶子。
原轮支持三类元素与世界复制。
API继续隐藏设备地址。
未知完成时保留实际资源。
本轮不冻结完整Model与Data。
生产模块继续保持计划态。

证据：C:/Rust/mjwarp-rs/docs/windows-transfer-probe.md。

## 原生输入跨语言集成

根层登记构建与FFI集成。
IO负责原生所有者与字段快照。
model负责小型DTO与容量。
诊断层定义准入错误。
根映射保留测试与静态样本。
测试不调用模型编译器。
默认构建不接入C++。
探针仅用Windows x64 MSVC。
本轮不冻结完整模型ABI。

证据：C:/Rust/mjwarp-rs/docs/windows-native-model-probe.md。

原生探针文档同步记录真实边界。
样本属性固定XML换行与MJB字节。
MuJoCo诊断日志不进入Git。

## 运动学字段增量

内部源码归属model与io子树。
保留原十六节点与API路径。
本轮新增四个职责叶子。
新增四项字段辅助接口。
混合样本覆盖四种关节。
静态打印件提供独立结构证据。
产品测试只读取固定MJB。
完整模型与动力学仍待实现。
本轮继续只验收Windows。
生产模块继续保持计划态。

证据：C:/Rust/mjwarp-rs/docs/windows-kinematic-fields.md。

## 惯性字段增量

九字段组合十二运动学字段。
原生快照与GPU组各自独占。
本轮只验收Windows辅助子集。
完整模型与物理继续保持计划。
旧运动学变更继续保持开放。

证据：C:/Rust/mjwarp-rs/docs/windows-inertial-fields.md。

## 字段底层拆分

连续字段承接已有真实行为。
模型组不再回依赖原生所有者。
本轮保留全部公开路径。
内部暂存另记私有契约。
本轮不放宽无环规则。

## Windows刚体与质心GPU子集

根层登记静态参考与脚本。
物理层计算七项体关节字段。
物理层新增四项质心结果。
输入仍采用二十一字段子集。
产品链不运行原生物理参考。
零质量采用冻结Warp语义。
独立原生结果保留实际差异。
完整G01与U057仍保持计划。
本轮继续只验收Windows。

证据：C:/Rust/mjwarp-rs/docs/windows-com-position-probe.md。

## Windows质量矩阵GPU子集

根层登记六组独立静态参考。
物理层复用设备质心结果。
GPU输出CRB与对称稠密矩阵。
非零armature与解析动能均有测试。
探针允许半正定矩阵。
矩阵入口不提供分解与求解。
完整U059、G02与G03继续计划。
保留旧契约与真实源码证据。
本轮继续只验收Windows。

证据：C:/Rust/mjwarp-rs/docs/windows-mass-matrix-probe.md。

## Windows质量矩阵求解子集

GPU执行反向LDL与多右端项求解。
内部GPU链采用f64。
公开物理输入与结果仍为f32。
矩阵不经过宿主重新上传。
连续字段新增原样f64复制。
严格窄化辅助保持原语义。
六组独立原生参考固定容限。
解析链与重构残差另作交叉检查。
坏主元不触发正则化或CPU回退。
完整G01、G02与G03继续计划。
Linux继续暂缓。

证据：C:/Rust/mjwarp-rs/docs/windows-mass-solve-probe.md。

## 常驻字段批量增量

十三项浮点参数独立按世界取模。
默认字段保持共享行。
拓扑仍共享，参数计划保持只读。
完整G01与Linux验收继续计划。

证据：C:/Rust/mjwarp-rs/docs/windows-resident-parameters.md。

## 内侧圆柱精度修复（历史记录）

内侧GPU计算采用f64。
公开物理结果仍为f32。
方向缓存先于接点舍入。
三组运动状态锁定原生参考。
480源帧通过原容限复核。
119项探针通过双模式验收。
完整G01仍保持计划。

证据：C:/Rust/mjwarp-rs/docs/windows-geom-tendon-frame-review.md。

## G01本轮实现

G01模型输入覆盖全部阶段读字段。
G01常驻入口保留等价调用范围。
Windows验收另见本轮报告。
本模块不因此完成其他契约。
其余计划API保持原有路径。

证据：C:/Rust/mjwarp-rs/docs/windows-g01-acceptance.md。

## 2026-10-14 常驻混合桥接增量

导出MixedResident Plan/Data，qpos→CRB/M→CSR→factor/solve。
原f32计算、ABI与编译门禁保持。
新host2过，GPU1过1败，private1过，组合3/3冻结。
数值12批原容差通过，状态测试修后未执行。
不激活完整G03/G02或正式compiler。
原UID/计划态/正式前置资源锁保持。
旧柔体面覆盖与容差不改，不重放满额目标。
证据：C:/Rust/mjwarp-rs/docs/resident-mixed-mass.md。
