//! 检查新修复的源码与历史边界。
//! 不编译CUDA，不重放物理样本。
#![cfg(feature = "cuda-probe")]

#[path = "support/t4_face_lineage.rs"]
mod lineage;

use serde_json::Value;
use sha2::{Digest, Sha256};

const FACE: &str = include_str!("../src/physics/flex_face.rs");
// 保留修复当时的失效断言；不让后续真实取证改写历史。
const INVENTORY: &str =
    include_str!("../docs/evidence/t4-g03-retest/face-current-compile/previous-kernels.json");
const STATUS: &str =
    include_str!("../docs/evidence/t4-g03-retest/face-current-compile/previous-status.json");
// 固定本次真实编译点；后续机器码或数值取证不能改写此点。
const CURRENT_INVENTORY: &str =
    include_str!("../docs/evidence/t4-g03-retest/face-current-compile/kernels.json");
const CURRENT_STATUS: &str =
    include_str!("../docs/evidence/t4-g03-retest/face-current-compile/status.json");
const FUSION: &str = include_str!("../docs/evidence/t4-g03-retest/face-machine/fusion.json");

fn digest(text: &str) -> String {
    format!(
        "{:x}",
        Sha256::digest(text.replace("\r\n", "\n").as_bytes())
    )
}

fn previous_source(text: &str) -> Result<String, &'static str> {
    let text = text.replace("\r\n", "\n");
    if text.matches(lineage::FUSED_M).count() != 1 || text.contains(lineage::FIX_M) {
        return Err("unknown fusion source");
    }
    let old = text.replace(lineage::FUSED_M, lineage::FIX_M);
    if digest(&old) != "42c15f37ab69d01e0252993c6039a36287e2a1936febeb253e5fe06c8fa2bed9" {
        return Err("unrelated source drift");
    }
    Ok(old)
}

fn validates_stale_status(status: &Value) -> Result<(), &'static str> {
    if status["status"] != "stale-after-face-final-fusion-source-repair"
        || status["current_source"]["sha256"] != digest(FACE)
        || status["inventory_sha256"] != digest(INVENTORY)
        || status["affected_entries"] != serde_json::json!(["flex_faces"])
        || status["current_bit_differences"] != Value::Null
        || status["previous_strict_bit_differences"] != 2078
        || status["pending"].as_array().map(Vec::len) != Some(3)
    {
        return Err("stale source/inventory boundary");
    }
    for key in [
        "current_cuda_compiled",
        "current_ptx_captured",
        "current_machine_code_captured",
        "current_numerical_tested",
        "full_acceptance",
    ] {
        if status[key] != false {
            return Err("unearned current evidence");
        }
    }
    let source = FACE.replace("\r\n", "\n");
    let cuda = source
        .split_once("const FLEX_FACE_CUDA: &str = r#\"")
        .ok_or("template")?
        .1
        .split_once("\"#;")
        .ok_or("template end")?
        .0;
    if status["current_source"]["cuda_sha256"] != digest(cuda) {
        return Err("current CUDA hash");
    }
    let inventory: Value = serde_json::from_str(INVENTORY).unwrap();
    let stamps: Vec<_> = inventory["product_sources"]
        .as_array()
        .ok_or("source stamps")?
        .iter()
        .filter(|stamp| stamp["path"] == "src/physics/flex_face.rs")
        .collect();
    if stamps.len() != 1
        || stamps[0]["sha256"] != status["old_capture"]["source_sha256"]
        || stamps[0]["sha256"] == status["current_source"]["sha256"]
    {
        return Err("old source stamp cannot represent current source");
    }
    let rows = inventory["artifacts"].as_array().ok_or("artifacts")?;
    let face: Vec<_> = rows
        .iter()
        .filter(|row| row["entry"] == "flex_faces")
        .collect();
    if rows.len() != 30 || face.len() != 1 {
        return Err("capture denominator");
    }
    if face[0]["source_sha256"] != status["old_capture"]["cuda_sha256"]
        || face[0]["ptx_sha256"] != status["old_capture"]["ptx_sha256"]
        || face[0]["source_sha256"] == status["current_source"]["cuda_sha256"]
    {
        return Err("old capture cannot represent current source");
    }
    Ok(())
}

#[test]
fn matches_confirmed_three_axis_machine_formula() {
    let text = FACE.replace("\r\n", "\n");
    previous_source(&text).unwrap();
    for (axis, expression) in [
        "fmaf(-q[1],rot[2],fmaf(q[2],rot[1],fmaf(q[3],rot[0],__fmul_rn(q[0],rot[3]))))",
        "fmaf(-q[2],rot[0],fmaf(q[0],rot[2],fmaf(q[3],rot[1],__fmul_rn(q[1],rot[3]))))",
        "fmaf(-q[0],rot[1],fmaf(q[1],rot[0],fmaf(q[3],rot[2],__fmul_rn(q[2],rot[3]))))",
    ]
    .iter()
    .enumerate()
    {
        assert!(text.contains(&format!("next[{axis}]={expression};")));
    }
    assert!(!text.contains("__fsub_rn"));
    let fusion: Value = serde_json::from_str(FUSION).unwrap();
    assert_eq!(
        digest(FUSION),
        "6561502f1c59dd31919f2868495b5e0f43842c4c4cbe513b481ceffe48d72916"
    );
    assert_eq!(fusion["axes"].as_array().unwrap().len(), 3);
    for row in fusion["axes"].as_array().unwrap() {
        assert!(
            row["upstream_final"]["instruction"]
                .as_str()
                .unwrap()
                .starts_with("FFMA ")
        );
    }
    assert_eq!(fusion["full_acceptance"], false);
}

#[test]
fn preserves_exact_historical_lineage_and_rejects_drift() {
    let text = FACE.replace("\r\n", "\n");
    let old = previous_source(&text).unwrap();
    assert_eq!(old.replace(lineage::FIX_M, lineage::FUSED_M), text);
    assert_eq!(
        lineage::historical(&old).unwrap(),
        lineage::historical(&text).unwrap()
    );
    assert_eq!(
        digest(&lineage::historical(&text).unwrap()),
        "47a17fc2f053e93afaaab408b2c185f0bbef0cce6687deb4120abc94d478c1e6"
    );
    for changed in [
        text.replace("fmaf(-q[1],rot[2]", "fmaf(-q[2],rot[2]"),
        format!("{text}\n{}", lineage::FUSED_M),
        format!("{text}\n{}", lineage::FIX_M),
        text.replace("next[3]=rot[3]*q[3]", "next[3]=rot[2]*q[3]"),
    ] {
        assert!(previous_source(&changed).is_err());
        assert!(lineage::historical(&changed).is_err());
    }
    // 逆变换只识别已知块；全文件哈希继续保护块外源码。
    let changed = text.replace("iter<50", "iter<51");
    assert!(lineage::historical(&changed).is_ok());
    assert!(previous_source(&changed).is_err());
}

#[test]
fn keeps_old_capture_separate_from_uncompiled_product() {
    let status: Value = serde_json::from_str(STATUS).unwrap();
    validates_stale_status(&status).unwrap();
    for key in [
        "current_cuda_compiled",
        "current_numerical_tested",
        "full_acceptance",
    ] {
        let mut changed = status.clone();
        changed[key] = true.into();
        assert!(validates_stale_status(&changed).is_err());
    }
}

#[test]
fn binds_current_ptx_without_admitting_machine_or_numerical_acceptance() {
    let status: Value = serde_json::from_str(CURRENT_STATUS).unwrap();
    let inventory: Value = serde_json::from_str(CURRENT_INVENTORY).unwrap();
    assert_eq!(
        status["status"],
        "compiled-ptx-only-machine-and-numerical-pending"
    );
    assert_eq!(status["inventory_sha256"], digest(CURRENT_INVENTORY));
    assert_eq!(status["current_source"]["sha256"], digest(FACE));
    assert_eq!(status["current_cuda_compiled"], true);
    assert_eq!(status["current_ptx_captured"], true);
    for key in [
        "current_machine_code_captured",
        "current_numerical_tested",
        "full_acceptance",
    ] {
        assert_eq!(status[key], false);
    }
    assert_eq!(status["current_bit_differences"], Value::Null);
    assert_eq!(status["previous_strict_bit_differences"], 2078);
    assert_eq!(status["pending"].as_array().unwrap().len(), 2);
    assert_eq!(inventory["artifacts"].as_array().unwrap().len(), 30);
    let face = inventory["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["entry"] == "flex_faces")
        .unwrap();
    assert_eq!(face, &status["current_capture"]);
    assert_eq!(
        face["source_sha256"],
        status["current_source"]["cuda_sha256"]
    );
    assert_ne!(face["source_sha256"], status["old_capture"]["cuda_sha256"]);
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = root.join(status["current_capture_directory"].as_str().unwrap());
    for (extension, key) in [
        ("cu", "source_sha256"),
        ("ptx", "ptx_sha256"),
        ("json", "receipt_sha256"),
    ] {
        let path = dir.join(format!(
            "flex_faces-{}.{extension}",
            face["ptx_sha256"].as_str().unwrap()
        ));
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            face[key].as_str().unwrap()
        );
    }
}
