//! S1碰撞输入快照测试的oracle辅助：pinned MJB、oracle JSON解析与位型工具。
//! 只被`tests/g10_p5_collision_inputs_snapshot.rs`包含；不进入产品路径。

use mjwarp_rs::{io::NativeModelProbe, model::collision_inputs::CollisionInputsCounts};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

/// 三份冻结MJB；覆盖mesh（scene-312）、hfield（hfield-box）与octree（sdf-cylinder）。
pub const MODELS: [(&str, &[u8]); 3] = [
    (
        "hfield-box",
        include_bytes!("../../fixtures/g10-p5/mjb-312/hfield-box.mjb"),
    ),
    (
        "scene-312",
        include_bytes!("../../fixtures/render-bvh/scene-a/scene-312.mjb"),
    ),
    (
        "sdf-cylinder",
        include_bytes!("../../fixtures/g10-p5/mjb-312/sdf-cylinder.mjb"),
    ),
];

pub fn dll() -> PathBuf {
    std::env::var_os("MJWARP_MUJOCO_DLL").unwrap().into()
}

pub fn load(bytes: &[u8]) -> NativeModelProbe {
    // SAFETY: this opt-in target uses the verified 3.12 library and frozen MJBs.
    unsafe { NativeModelProbe::load_trusted(&dll(), bytes) }.unwrap()
}

pub fn oracle(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

pub fn model<'a>(oracle: &'a Value, name: &str) -> &'a Value {
    oracle["models"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("oracle model {name}"))
}

pub fn counts_of(value: &Value) -> CollisionInputsCounts {
    let counts = &value["counts"];
    let read = |name: &str| counts[name].as_i64().unwrap();
    CollisionInputsCounts {
        nq: read("nq"),
        ngeom: read("ngeom"),
        nmesh: read("nmesh"),
        nmeshvert: read("nmeshvert"),
        nmeshgraph: read("nmeshgraph"),
        nmeshpoly: read("nmeshpoly"),
        nmeshpolyvert: read("nmeshpolyvert"),
        nmeshpolymap: read("nmeshpolymap"),
        nhfield: read("nhfield"),
        nhfielddata: read("nhfielddata"),
        noct: read("noct"),
    }
}

pub fn i32s(value: &Value) -> Vec<i32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| i32::try_from(item.as_i64().unwrap()).unwrap())
        .collect()
}

/// f64 期望值以 u64 位型存储，避免 JSON 浮点解析丢 1 ULP（serde_json 无 float_roundtrip）。
pub fn f64s(value: &Value) -> Vec<f64> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| f64::from_bits(item.as_u64().unwrap()))
        .collect()
}

pub fn u32s(value: &Value) -> Vec<u32> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| u32::try_from(item.as_u64().unwrap()).unwrap())
        .collect()
}

pub fn bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

pub fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
