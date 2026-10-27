//! S3 端到端 target 支持：pinned MJB fixtures、M1/M3 冻结参考与 qpos 读取。
//!
//! 只读仓库内文件，不写、不改；不启动 Python、不使用 GPU（GPU 由调用方 target 负责）。
//! MJB 生成与 32 数组对账见 `docs/development-evidence/product-collision-inputs-wiring/scripts/
//! generate_collision_inputs_mjb.py` 与 `…/fixtures/mjb-312/provenance.json`。
#![allow(dead_code)]

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub const ROOT: &str = env!("CARGO_MANIFEST_DIR");

/// 一个端到端 case 的冻结输入（MJB + 参考 + 32 项 bins 目录）。
#[derive(Clone, Copy, Debug)]
pub struct CaseFixture {
    pub name: &'static str,
    /// pinned MuJoCo 3.12.0 从冻结 XML 编译出的 MJB（evidence fixtures，sha 写死）。
    pub mjb: &'static str,
    pub mjb_sha256: &'static str,
    pub mjb_bytes: usize,
    /// 冻结上游 GPU 参考（M1 覆盖 mesh-box/mesh-cylinder，M3 覆盖 mesh-mesh）。
    pub reference: &'static str,
    pub reference_sha256: &'static str,
    /// 32 项 bins（M0b / M3 只读证据目录）：S1 快照的宿主侧对账来源。
    pub bins: &'static str,
    /// `contact.geom` 的 pinned 类型序（BOX/CYLINDER 6/5 < MESH 7 → `[1,0]`；MESH-MESH → `[0,1]`）。
    pub geom_order: &'static str,
}

pub const CASES: [CaseFixture; 3] = [
    CaseFixture {
        name: "mesh-box",
        mjb: "docs/development-evidence/product-collision-inputs-wiring/fixtures/mjb-312/mesh-box-312.mjb",
        mjb_sha256: "70b2fdc36bc8c342bcdbce7c522085a33c5286855c5e613f351f920387261e3e",
        mjb_bytes: 8800,
        reference: "docs/development-evidence/mesh-multicontact-m1/upstream-mesh-multicontact-gpu.json",
        reference_sha256: "c83baffe6de695fdefda6852c574d5d0d7312ebb591c1b487dfc59ec126c206a",
        bins: "docs/development-evidence/mesh-multicontact-m0b/cases/mesh-box",
        geom_order: "[1,0]",
    },
    CaseFixture {
        name: "mesh-cylinder",
        mjb: "docs/development-evidence/product-collision-inputs-wiring/fixtures/mjb-312/mesh-cylinder-312.mjb",
        mjb_sha256: "ceef075c590710224f5e462c98bebf6cd3859e28dad0456804ff3165439505bc",
        mjb_bytes: 8815,
        reference: "docs/development-evidence/mesh-multicontact-m1/upstream-mesh-multicontact-gpu.json",
        reference_sha256: "c83baffe6de695fdefda6852c574d5d0d7312ebb591c1b487dfc59ec126c206a",
        bins: "docs/development-evidence/mesh-multicontact-m0b/cases/mesh-cylinder",
        geom_order: "[1,0]",
    },
    CaseFixture {
        name: "mesh-mesh",
        mjb: "docs/development-evidence/product-collision-inputs-wiring/fixtures/mjb-312/mesh-mesh-312.mjb",
        mjb_sha256: "298b8089345402bc2afa3bbf0c8cde1732341460cc6a0e4e1fd0bad4067f7734",
        mjb_bytes: 12032,
        reference: "docs/development-evidence/mesh-multicontact-m3/upstream-mesh-multicontact-mesh-mesh-gpu.json",
        reference_sha256: "96e7895d13c3ba23475a013fcaa586485e93f95cced5351e2874973917e149a1",
        bins: "docs/development-evidence/mesh-multicontact-m3/cases/mesh-mesh",
        geom_order: "[0,1]",
    },
];

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// 仓库相对路径 → 绝对路径。
pub fn repo_path(relative: &str) -> PathBuf {
    Path::new(ROOT).join(relative)
}

/// 读取并校验 pinned MJB（字节数与 sha256 都必须等于冻结值）。
pub fn mjb_bytes(case: &CaseFixture) -> Vec<u8> {
    let path = repo_path(case.mjb);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("读取 {} 失败：{error}", path.display()));
    assert_eq!(
        bytes.len(),
        case.mjb_bytes,
        "{}: MJB 字节数须与冻结值一致",
        case.name
    );
    assert_eq!(
        sha256_hex(&bytes),
        case.mjb_sha256,
        "{}: MJB sha256 须与冻结值一致",
        case.name
    );
    bytes
}

/// 读取并校验冻结参考（M1 / M3；参考只读、不可改动）。
pub fn reference_value(case: &CaseFixture) -> Value {
    let path = repo_path(case.reference);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("读取 {} 失败：{error}", path.display()));
    assert_eq!(
        sha256_hex(&bytes),
        case.reference_sha256,
        "{}: 参考 sha256 与冻结值不符（参考不可被改动）",
        case.name
    );
    serde_json::from_slice(&bytes).expect("解析参考 JSON")
}

/// 参考记录的 qpos（f64；捕获时使用同一组 qpos 值）。
pub fn qpos_f64(case_reference: &Value) -> Vec<f64> {
    case_reference["qpos"]
        .as_array()
        .unwrap_or_else(|| panic!("参考缺少 qpos"))
        .iter()
        .map(|value| value.as_f64().expect("qpos 分量非数值"))
        .collect()
}

/// 可信 MuJoCo DLL 路径（窗口/本地按 pinned 3.12.0 包提供）。
pub fn dll_path() -> PathBuf {
    std::env::var_os("MJWARP_MUJOCO_DLL")
        .map(PathBuf::from)
        .expect("需要 MJWARP_MUJOCO_DLL 指向 pinned MuJoCo 3.12.0 DLL")
}
