#![cfg(feature = "cuda-probe")]
//! G03分块因子与多右端项辅助。
use mjwarp_rs::{
    diagnostics::{InputError, TransferError},
    model::InertialModelInput,
    physics::{MassFactorKind, MassSolveOutput, probe_mass_solve},
    runtime::TransferSession,
};
use serde_json::Value;
#[path = "support/rigid_reference.rs"]
mod reference;
use reference::{floats, model};
#[path = "support/mass_solve_reference.rs"]
mod block_reference;
#[path = "support/two_reference.rs"]
mod two_reference;
use sha2::{Digest, Sha256};
use two_reference::{Audit, MJWARP_RELATIVE_SLACK, SecondaryReference, evaluate, native_reference};

const MODELS: [&str; 6] = [
    include_str!("../fixtures/mass-matrix/mixed-joints.json"),
    include_str!("../fixtures/mass-matrix/inertial-tree.json"),
    include_str!("../fixtures/mass-matrix/rotated-tree.json"),
    include_str!("../fixtures/mass-matrix/zero-dof.json"),
    include_str!("../fixtures/mass-matrix/massless-tree.json"),
    include_str!("../fixtures/mass-matrix/armature-chain.json"),
];
const SOLVES: [&str; 6] = [
    include_str!("../fixtures/mass-solve/mixed-joints.json"),
    include_str!("../fixtures/mass-solve/inertial-tree.json"),
    include_str!("../fixtures/mass-solve/rotated-tree.json"),
    include_str!("../fixtures/mass-solve/zero-dof.json"),
    include_str!("../fixtures/mass-solve/massless-tree.json"),
    include_str!("../fixtures/mass-solve/armature-chain.json"),
];
// Freeze these before the first solve GPU run. Matrix probe tolerances stay
// unchanged; inversion can amplify small float32 matrix differences.
const ABS: f64 = 2e-4;
const REL: f64 = 2e-4;
const RESIDUAL: f64 = 2e-5;
const RECONSTRUCTION: f64 = 2e-6;

#[path = "mass_solve_probe/compare.rs"]
mod compare;
#[path = "mass_solve_probe/gpu.rs"]
mod gpu;
#[cfg(feature = "native-model-probe")]
#[path = "mass_solve_probe/native.rs"]
mod native;
#[path = "mass_solve_probe/solves.rs"]
mod solves;

fn verify_native_references() {
    static VERIFIED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    VERIFIED.get_or_init(|| {
        let manifest: Value =
            serde_json::from_str(include_str!("../fixtures/mass-solve/manifest.json")).unwrap();
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let files = manifest["files"].as_array().unwrap();
        for file in files {
            let path = root.join(file["path"].as_str().unwrap());
            assert_eq!(
                format!("{:x}", Sha256::digest(std::fs::read(path).unwrap())),
                file["sha256"]
            );
        }
        for source in SOLVES {
            let digest = format!("{:x}", Sha256::digest(source.as_bytes()));
            assert!(files.iter().any(|file| file["sha256"] == digest));
        }
    });
}

fn rhs(case: &Value, n: usize, count: usize) -> Vec<f32> {
    let base = floats(case, "rhs");
    (0..count)
        .flat_map(|r| base[r % 3 * n..(r % 3 + 1) * n].iter().copied())
        .collect()
}
