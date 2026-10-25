//! G3 参考夹具结构校验：上游 f32 CPU 参考（warp CPU 后端）与 G0b 原生参考的
//! 结构一致性；二进制 octree 数组按 JSON 的 sha256/长度校验。
//! 数值判据在 lib 内 `physics::collision::sdf::tests`（模块尚未公开到 crate 外）。

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_json(relative: &str) -> Value {
    let path = root().join(relative);
    let text = std::fs::read_to_string(&path).expect("读取 JSON 参考");
    serde_json::from_str(&text).expect("解析 JSON 参考")
}

fn read_bin(relative: &str) -> Vec<u8> {
    std::fs::read(root().join(relative)).expect("读取二进制参考")
}

fn scalar(value: &Value) -> f64 {
    value.as_f64().expect("数值")
}

#[test]
fn sdf_cpu_reference_is_well_formed_and_matches_g0b_structure() {
    let reference = read_json("fixtures/g10-p5/sdf-f32/sdf-box-cpu.json");
    assert_eq!(reference["schema"], 1);
    assert_eq!(
        reference["upstream_revision"],
        "71da24d956378a87a703b6e1442b13aec0c4ac29"
    );
    assert_eq!(reference["provenance"]["device"], "cpu");
    assert_eq!(reference["provenance"]["authoring_only"], true);
    assert_eq!(reference["case"]["two_captures_bitwise_identical"], true);
    assert_eq!(reference["case"]["sdf_iterations"], 10);
    assert_eq!(reference["case"]["sdf_initpoints"], 40);

    let worlds = scalar(&reference["case"]["worlds"]) as usize;
    let initpoints = scalar(&reference["case"]["sdf_initpoints"]) as usize;
    let count = scalar(&reference["case"]["output"]["count"]) as usize;
    assert_eq!(count, worlds * initpoints);
    assert_eq!(
        reference["case"]["qpos_bits"].as_array().unwrap().len(),
        worlds * 7
    );

    // 二进制 octree 数组：长度、形状、sha256 与 JSON 声明一致。
    let mut total = 0usize;
    for name in ["oct_aabb", "oct_child", "oct_coeff"] {
        let file = &reference["model"]["files"][name];
        let path = file["path"].as_str().expect("path");
        let bytes = read_bin(path);
        let declared = scalar(&file["count"]) as usize;
        assert_eq!(bytes.len(), declared * 4, "{name} 字节数");
        let shape: usize = file["shape"]
            .as_array()
            .unwrap()
            .iter()
            .map(|dim| scalar(dim) as usize)
            .product();
        assert_eq!(shape, declared, "{name} 形状");
        let digest = format!("{:x}", Sha256::digest(&bytes));
        assert_eq!(digest, file["sha256"].as_str().expect("sha256"), "{name}");
        total += declared;
    }
    assert!(total > 1_000_000, "octree 标量总数 {total}");

    // 结构对齐 G0b 原生参考（同模型、同 qpos 的 GPU warp 捕获）：接触数与几何对一致。
    let native = read_json("fixtures/g10-p5/references/sdf-box.json");
    let native_count = scalar(&native["output"]["count"]) as usize;
    assert_eq!(native_count, count);
    let native_geom = native["output"]["fields"]["geom"]["values"]
        .as_array()
        .expect("G0b geom");
    let reference_geom = reference["case"]["output"]["fields"]["geom"]
        .as_array()
        .expect("G3 geom");
    assert_eq!(native_geom.len(), count * 2);
    assert_eq!(reference_geom.len(), count);
    let flatten = |values: &[Value]| -> Vec<i32> {
        let mut out = Vec::new();
        for value in values {
            match value {
                Value::Array(row) => out.extend(row.iter().map(|item| scalar(item) as i32)),
                other => out.push(scalar(other) as i32),
            }
        }
        out
    };
    assert_eq!(flatten(native_geom), flatten(reference_geom), "几何对结构");
    let native_world = native["output"]["fields"]["worldid"]["values"]
        .as_array()
        .expect("G0b worldid");
    let reference_world = reference["case"]["output"]["fields"]["worldid"]
        .as_array()
        .expect("G3 worldid");
    assert_eq!(
        flatten(native_world),
        flatten(reference_world),
        "worldid 结构"
    );
}
