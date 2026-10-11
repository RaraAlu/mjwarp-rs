//! 校验冻结参考的模型与批量状态。
use super::input::{fixture, floats, ids};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const REVISION: &str = "71da24d956378a87a703b6e1442b13aec0c4ac29";
pub fn upstream() -> Value {
    serde_json::from_str(include_str!(
        "../../fixtures/mjwarp-geom-tendon/reference.json"
    ))
    .unwrap()
}
pub fn provenance() -> Value {
    serde_json::from_str(include_str!(
        "../../fixtures/mjwarp-geom-tendon/provenance.json"
    ))
    .unwrap()
}
pub fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn float_bytes(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}
pub fn input_hashes(inputs: &[&Value]) -> Value {
    ["qpos", "mocap_pos", "mocap_quat"]
        .into_iter()
        .map(|key| {
            let values: Vec<_> = inputs.iter().flat_map(|c| floats(c, key)).collect();
            (key.to_owned(), json!(sha(&float_bytes(&values))))
        })
        .collect()
}
pub fn validate() {
    let manifest: Value = serde_json::from_str(include_str!(
        "../../fixtures/mjwarp-geom-tendon/manifest.json"
    ))
    .unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in manifest["files"].as_array().unwrap() {
        let path = file["path"].as_str().unwrap();
        assert_eq!(
            sha(&std::fs::read(root.join(path)).unwrap()),
            file["sha256"],
            "{path}"
        );
    }
    let u = upstream();
    let p = provenance();
    for value in [&manifest, &u, &p] {
        assert_eq!(value["upstream_revision"], REVISION);
    }
    for (path, bytes) in [
        (
            "fixtures/mjwarp-geom-tendon/reference.json",
            include_bytes!("../../fixtures/mjwarp-geom-tendon/reference.json").as_slice(),
        ),
        (
            "fixtures/mjwarp-geom-tendon/provenance.json",
            include_bytes!("../../fixtures/mjwarp-geom-tendon/provenance.json").as_slice(),
        ),
    ] {
        let entry = manifest["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["path"] == path)
            .unwrap();
        assert_eq!(sha(bytes), entry["sha256"]);
    }
    assert_eq!(u["rows"].as_array().unwrap().len(), 37);
    assert_eq!(u["float_fields"].as_array().unwrap().len(), 16);
    assert_eq!(u["int_fields"].as_array().unwrap().len(), 3);
    assert_eq!(p["packages"]["warp-lang"], "1.15.0");
    assert_eq!(p["packages"]["mujoco"], "3.15.0");
    assert_eq!(p["module_options"]["fuse_fp"], true);
    assert_eq!(p["module_options"]["fast_math"], false);
    let v = fixture();
    for (key, mapping) in p["device_model_fields"].as_object().unwrap() {
        let bytes = if key == "geom_size" {
            float_bytes(
                &(0..3)
                    .flat_map(|i| floats(&v["cases"][i], key))
                    .collect::<Vec<_>>(),
            )
        } else if mapping["dtype"] == "float32" {
            float_bytes(&floats(&v["model"], key))
        } else {
            assert_eq!(mapping["dtype"], "int32");
            ids(&v["model"], key)
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect()
        };
        assert_eq!(sha(&bytes), mapping["sha256"], "model field={key}");
    }
}

/// 精确匹配批量规模、世界与输入哈希。
/// 不把其他世界或帧的参考借来使用。
pub fn motion_row(
    worlds: usize,
    world: usize,
    frame: u64,
    hashes: &Value,
) -> Result<Option<Value>, String> {
    static VALIDATED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    VALIDATED.get_or_init(validate);
    let u = upstream();
    let rows: Vec<_> = u["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| {
            r["worlds"] == worlds
                && r["world"] == world
                && r["kind"] == "motion"
                && r["case"] == frame
        })
        .collect();
    match rows.as_slice() {
        [] => Ok(None),
        [row] if row["input_hashes"] == *hashes => Ok(Some((*row).clone())),
        [_] => Err("batch_input_hash_mismatch".into()),
        _ => Err("duplicate_reference_row".into()),
    }
}
