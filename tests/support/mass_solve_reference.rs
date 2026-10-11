//! 分块求解的冻结双参考；测试不生成参考。
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

const REVISION: &str = "71da24d956378a87a703b6e1442b13aec0c4ac29";

pub fn reference() -> &'static Value {
    static DATA: OnceLock<Value> = OnceLock::new();
    DATA.get_or_init(|| {
        let manifest: Value = serde_json::from_str(include_str!(
            "../../fixtures/mjwarp-mass-solve/manifest.json"
        ))
        .unwrap();
        let bytes = include_bytes!("../../fixtures/mjwarp-mass-solve/reference.json");
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            manifest["reference_sha256"]
        );
        assert_eq!(manifest["revision"], REVISION);
        assert_eq!(manifest["module_options"]["fuse_fp"], true);
        assert_eq!(manifest["module_options"]["fast_math"], false);
        assert_eq!(
            manifest["stages"],
            serde_json::json!(["kinematics", "com_pos", "crb", "factor_m", "solve_m"])
        );
        assert_eq!(manifest["packages"]["warp-lang"], "1.15.0");
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        for (path, hash) in manifest["inputs"].as_object().unwrap() {
            assert_eq!(
                format!(
                    "{:x}",
                    Sha256::digest(std::fs::read(root.join(path)).unwrap())
                ),
                *hash,
                "{path}"
            );
        }
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(include_bytes!(
                    "../../fixtures/mjwarp-mass-solve/reference.py"
                ))
            ),
            manifest["authoring_sha256"]
        );
        let data: Value = serde_json::from_slice(bytes).unwrap();
        assert_eq!(data["revision"], REVISION);
        assert_eq!(data["scalar"], "f32");
        assert_eq!(data["absolute_tolerance"], 2e-4);
        assert_eq!(data["relative_tolerance"], 2e-4);
        data
    })
}

pub fn row(
    native_source: &str,
    worlds: usize,
    rhs_count: usize,
    world: usize,
    case: &Value,
) -> (&'static Value, &'static Value) {
    let hash = format!("{:x}", Sha256::digest(native_source.as_bytes()));
    let model = reference()["models"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["native_sha256"] == hash)
        .expect("matched native model hash");
    let row = model["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["worlds"] == worlds && r["rhs_count"] == rhs_count && r["world"] == world)
        .expect("matched world and RHS configuration");
    assert_eq!(row["case"], case["id"]);
    let quantized = |v: &Value| {
        v.as_array()
            .unwrap()
            .iter()
            .map(|x| (x.as_f64().unwrap() as f32).to_bits())
            .collect::<Vec<_>>()
    };
    assert_eq!(quantized(&row["qpos"]), quantized(&case["qpos"]));
    let rhs = quantized(&case["rhs"]);
    let n = rhs.len() / 3;
    let expected: Vec<_> = (0..rhs_count)
        .flat_map(|r| rhs[r % 3 * n..(r % 3 + 1) * n].iter().copied())
        .collect();
    assert_eq!(quantized(&row["rhs"]), expected);
    (model, row)
}
