//! 面算子修复归档（仅谱系，不重跑编译或 GPU）。

use super::lineage::lineage;
use super::*;

#[test]
#[ignore = "archive already measured face artifact; no compiler or GPU replay"]
fn archives_face_operator_repair_with_failed_numeric_gate() {
    let out = PathBuf::from(std::env::var_os("MJWARP_PHYSICS_PTX_DIR").unwrap());
    assert!(out.is_absolute() && !out.exists());
    assert!(
        out.parent()
            .unwrap()
            .canonicalize()
            .unwrap()
            .starts_with(root().join("target").canonicalize().unwrap())
    );
    let previous = root().join("target/t4-rigid-recapture/with-dependents/current");
    let mut value: Inventory = serde_json::from_str(include_str!(
        "../../../docs/evidence/t4-g03-retest/face-operators/previous-kernels.json"
    ))
    .unwrap();
    verify_capture(&value, &previous).unwrap();
    let source = fs::read_to_string(root().join("target/t4-face-operators/product.cu")).unwrap();
    let ptx = fs::read_to_string(root().join("target/t4-face-operators/product.ptx")).unwrap();
    assert_eq!(source, template(&normalized(FACE), "FLEX_FACE_CUDA"));
    let result: serde_json::Value = serde_json::from_slice(
        &fs::read(root().join("target/t4-face-operators/factorial.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(result["product_cuda_sha256"], digest(source.as_bytes()));
    assert_eq!(result["product_ptx_sha256"], digest(ptx.as_bytes()));
    // 保留失败，不把编译证据转成数值准入。
    assert_eq!(result["repaired_product"]["bit_differences"], 2078);
    assert_eq!(result["full_acceptance"], false);
    let historic = face_lineage::historical(FACE).unwrap();
    let old = base();
    let old_stamp = old
        .product_sources
        .iter()
        .find(|s| s.path == "src/physics/flex_face.rs")
        .unwrap();
    assert_eq!(digest(historic.as_bytes()), old_stamp.sha256);
    for block in [
        face_lineage::NEW_R,
        face_lineage::FIX_M,
        face_lineage::NEW_N,
    ] {
        assert!(face_lineage::historical(&normalized(FACE).replace(block, "changed")).is_err());
    }
    fs::create_dir(&out).unwrap();
    let mut retained = 0;
    for row in &value.artifacts {
        if row.entry != "flex_faces" {
            for ext in ["cu", "ptx", "json"] {
                let name = format!("{}-{}.{ext}", row.entry, row.ptx_sha256);
                fs::copy(previous.join(&name), out.join(name)).unwrap();
            }
            retained += 1;
        }
    }
    assert_eq!(retained, 29);
    check_physics_ptx(&ptx, "flex_faces").unwrap();
    record_physics_ptx(&source, &ptx, "flex_faces", true).unwrap();
    let row = value
        .artifacts
        .iter_mut()
        .find(|a| a.entry == "flex_faces")
        .unwrap();
    row.source_sha256 = digest(source.as_bytes());
    row.ptx_sha256 = digest(ptx.as_bytes());
    row.receipt_sha256 =
        digest(&fs::read(out.join(format!("flex_faces-{}.json", row.ptx_sha256))).unwrap());
    value
        .product_sources
        .iter_mut()
        .find(|s| s.path == "src/physics/flex_face.rs")
        .unwrap()
        .sha256 = digest(normalized(FACE).as_bytes());
    value.source_revision = REVISION.into();
    lineage(&value).unwrap();
    verify_sources(
        &value,
        &root(),
        &root().join("target/mjwarp-parallel/upstream/mujoco_warp"),
    )
    .unwrap();
    verify_capture(&value, &out).unwrap();
    for artifact in &value.artifacts {
        let ptx =
            fs::read_to_string(out.join(format!("{}-{}.ptx", artifact.entry, artifact.ptx_sha256)))
                .unwrap();
        check_physics_ptx(&ptx, "face-current-capture").unwrap();
    }
    let bytes = serde_json::to_vec_pretty(&value).unwrap();
    fs::write(
        out.parent().unwrap().join("t4-current-kernels.json"),
        &bytes,
    )
    .unwrap();
    let audit = serde_json::json!({"scope":"face-repair-artifact-lineage-only","source_revision":REVISION,"current_inventory_sha256":digest(&bytes),"entries":24,"variants":30,"retained_variants":29,"updated_face_variants":1,"ptx_precision_gate_passes":30,"lineage_mutation_rejections":3,"new_compiler_calls":0,"gpu_executions":0,"numerical_gate":"failed-2078-bit-differences","full_acceptance":false});
    fs::write(
        out.parent().unwrap().join("capture.json"),
        serde_json::to_string_pretty(&audit).unwrap() + "\n",
    )
    .unwrap();
}
