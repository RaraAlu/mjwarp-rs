//! New native CSR capture/conversion only; no physics/GPU/reference author.
#![cfg(feature = "native-model-probe")]
use mjwarp_rs::{io::NativeModelProbe, model::SparseMassRoute};
use serde_json::Value;
use std::path::PathBuf;
const FIXTURES: [(&str, &[u8]); 6] = [
    (
        "mixed-joints",
        include_bytes!("../fixtures/native-probe/mixed-joints.mjb"),
    ),
    (
        "inertial-tree",
        include_bytes!("../fixtures/native-probe/inertial-tree.mjb"),
    ),
    (
        "rotated-tree",
        include_bytes!("../fixtures/kinematics/rotated-tree.mjb"),
    ),
    (
        "zero-dof",
        include_bytes!("../fixtures/native-probe/zero-dof.mjb"),
    ),
    (
        "massless-tree",
        include_bytes!("../fixtures/com-position/massless-tree.mjb"),
    ),
    (
        "armature-chain",
        include_bytes!("../fixtures/mass-matrix/armature-chain.mjb"),
    ),
];
fn load(bytes: &[u8]) -> NativeModelProbe {
    let dll = PathBuf::from(std::env::var_os("MJWARP_MUJOCO_DLL").unwrap());
    // SAFETY: Dedicated test script checks trusted3.12 DLL and frozen MJB hashes.
    // Product tests only load existing compiled assets, without MJCF or physics.
    unsafe { NativeModelProbe::load_trusted(&dll, bytes).unwrap() }
}
#[test]
#[ignore = "requires verified native3.12 DLL"]
fn captures_six_fields_from_six_existing_models_and_outlives_owner() {
    let refs: Value =
        serde_json::from_str(include_str!("../fixtures/mjwarp-mass-solve/reference.json")).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut models = vec![];
    for (name, bytes) in FIXTURES {
        let native = load(bytes);
        let snapshot = native.sparse_mass_snapshot().unwrap();
        assert_eq!(
            snapshot.fields.dof_parentid,
            native.inertial_snapshot().unwrap().dof_parentid
        );
        let native_base: Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(format!("fixtures/mass-matrix/{name}.json")))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            snapshot.fields.dof_parentid,
            native_base["model"]["dof_parentid"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_i64().unwrap() as i32)
                .collect::<Vec<_>>()
        );
        let again = native.sparse_mass_snapshot().unwrap();
        drop(native);
        assert_eq!(snapshot, again);
        let nc = snapshot.nc;
        let ntree = snapshot.ntree;
        let input = snapshot.into_input().unwrap();
        let frozen = refs["models"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["name"] == name)
            .unwrap();
        let blocks = frozen["blocks"].as_array().unwrap();
        assert_eq!(input.blocks().len(), blocks.len());
        let nonempty: Vec<_> = input
            .fields()
            .tree_dofadr
            .iter()
            .zip(&input.fields().tree_dofnum)
            .filter(|(_, num)| **num > 0)
            .map(|(&a, &n)| (a as usize, n as usize))
            .collect();
        assert_eq!(nonempty.len(), blocks.len());
        for ((block, r), &(start, size)) in input.blocks().iter().zip(blocks).zip(&nonempty) {
            assert_eq!((block.start, block.size), (start, size));
            assert_eq!(r["start"], start);
            assert_eq!(r["size"], size);
            let kind = match r["route"].as_str().unwrap() {
                "compact" => SparseMassRoute::Compact,
                "small_cholesky" => SparseMassRoute::SmallCholesky,
                "tile_cholesky" => SparseMassRoute::TileCholesky,
                "sparse_ldl" => SparseMassRoute::SparseLdl,
                _ => panic!("route"),
            };
            assert_eq!(block.route, kind);
            for dof in start..start + size {
                assert_eq!(
                    input.qld_block_adr()[dof],
                    r["upstream_address"].as_i64().unwrap() as i32
                );
            }
        }
        models.push(serde_json::json!({"name":name,"nv":input.nv(),"nc":nc,"ntree":ntree,"block_count":input.blocks().len(),"qld_block_total":input.qld_block_total()}));
    }
    let report = serde_json::json!({"models":models,"fields":6,"CPU_physics_calls":0,"GPU_sessions":0,"Python_calls":0,
        "precision_scope":"integer field capture and old block metadata only, not new numerical acceptance"});
    if let Some(dir) = std::env::var_os("MJWARP_NATIVE_SPARSE_OUTPUT") {
        std::fs::write(
            PathBuf::from(dir).join("results.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
    println!("{report}");
}
#[test]
#[ignore = "requires verified native3.12 DLL"]
fn rejects_modified_snapshot_counts_lengths_and_topology_without_touching_owner() {
    let native = load(FIXTURES[1].1);
    let base = native.sparse_mass_snapshot().unwrap();
    for case in 0..12 {
        let mut bad = base.clone();
        match case {
            0 => bad.info.schema = 2,
            1 => bad.info.pointer_bytes = 4,
            2 => bad.nc = usize::MAX,
            3 => bad.ntree = usize::MAX,
            4 => {
                bad.fields.dof_parentid.pop();
            }
            5 => {
                bad.fields.m_rownnz.pop();
            }
            6 => {
                bad.fields.m_rowadr.pop();
            }
            7 => {
                bad.fields.m_colind.pop();
            }
            8 => {
                bad.fields.tree_dofadr.pop();
            }
            9 => {
                bad.fields.tree_dofnum.pop();
            }
            10 => bad.fields.dof_parentid[0] = 0,
            _ => bad.fields.m_rownnz[0] = 0,
        };
        assert!(bad.into_input().is_err(), "case={case}");
    }
    assert_eq!(native.sparse_mass_snapshot().unwrap(), base);
}
#[test]
#[ignore = "requires separately compiled new sparse ABI preflight program"]
fn checks_new_abi_preflight_without_native_physics() {
    let exe = std::env::var_os("MJWARP_SPARSE_PREFLIGHT_EXE").unwrap();
    let result = std::process::Command::new(exe).output().unwrap();
    assert!(result.status.success(), "{:?}", result.status);
    let text = String::from_utf8(result.stdout).unwrap();
    assert!(text.contains("SPARSE_ABI checks="));
    println!("{text}");
}
