//! G10@P5 primitive 设备入口验收的输入/证据 helper（`g10_p5_primitive_cuda.rs` 以
//! `#[path]` 引入，测试专用；不进入产品路径）。
//!
//! 只读 G5 冻结 case bins 与 QA 已提交的上游参考 JSON；负责 23 项输入 sha256 守卫、
//! `meta.txt` 标量解析与 `CollisionGpuInputs` 组装。
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use mjwarp_rs::runtime::collision_gpu::{
    CollisionGpuInputs, CollisionGpuMeta, CollisionGpuOutput, CollisionGpuProgram, CollisionKernel,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const INPUT_BINS: [&str; 23] = [
    "geom_xpos",
    "geom_xmat",
    "geom_size",
    "geom_margin",
    "geom_gap",
    "geom_rbound",
    "geom_aabb",
    "geom_type",
    "geom_dataid",
    "mesh_vertadr",
    "mesh_vertnum",
    "mesh_graphadr",
    "mesh_octadr",
    "mesh_vert",
    "mesh_graph",
    "hfield_size",
    "hfield_nrow",
    "hfield_ncol",
    "hfield_adr",
    "hfield_data",
    "oct_aabb",
    "oct_child",
    "oct_coeff",
];

pub(crate) fn cases_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases")
}

pub(crate) fn reference_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json")
}

pub(crate) fn load_reference() -> Value {
    let text = std::fs::read(reference_path()).expect("读取 QA 上游 GPU 参考");
    serde_json::from_slice(&text).expect("解析 qa-upstream-gpu.json")
}

fn read_bytes(directory: &Path, name: &str) -> Vec<u8> {
    let path = directory.join(format!("{name}.bin"));
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("读取 {} 失败：{error}", path.display()));
    assert!(
        bytes.len().is_multiple_of(4),
        "{name}.bin 长度不是 4 的倍数"
    );
    bytes
}

macro_rules! read_values {
    ($name:ident, $type:ty) => {
        fn $name(directory: &Path, name: &str) -> Vec<$type> {
            read_bytes(directory, name)
                .as_chunks::<4>()
                .0
                .iter()
                .map(|chunk| <$type>::from_le_bytes(*chunk))
                .collect()
        }
    };
}
read_values!(read_f32, f32);
read_values!(read_i32, i32);

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// 输入一致性守卫：重算 23 个 case 输入 bin 的 sha256，必须与 QA 参考 `input_bins` 全同。
fn guard_inputs(case: &str, case_reference: &Value, directory: &Path) {
    let bins = &case_reference["input_bins"];
    for name in INPUT_BINS {
        let bytes = read_bytes(directory, name);
        let expected = bins[name]["sha256"]
            .as_str()
            .unwrap_or_else(|| panic!("{case}: QA 参考缺少 input_bins.{name}.sha256"));
        assert_eq!(
            sha256_hex(&bytes),
            expected,
            "{case}: 输入不一致 {name}.bin sha256 与 QA 参考不同"
        );
        let count = bins[name]["count"]
            .as_u64()
            .unwrap_or_else(|| panic!("{case}: QA 参考缺少 input_bins.{name}.count"));
        assert_eq!(
            bytes.len() as u64 / 4,
            count,
            "{case}: {name}.bin 元素数与 QA 参考不同"
        );
    }
}

fn read_meta(directory: &Path) -> BTreeMap<String, String> {
    let path = directory.join("meta.txt");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("读取 {} 失败：{error}", path.display()))
        .lines()
        .filter_map(|line| line.split_once(' '))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}

fn meta_value<T: FromStr>(meta: &BTreeMap<String, String>, key: &str) -> T
where
    T::Err: std::fmt::Debug,
{
    meta.get(key)
        .unwrap_or_else(|| panic!("meta 缺少 {key}"))
        .trim()
        .parse()
        .unwrap_or_else(|error| panic!("meta {key} 解析失败：{error:?}"))
}

fn meta_f32s<const N: usize>(meta: &BTreeMap<String, String>, key: &str) -> [f32; N] {
    let values: Vec<f32> = meta
        .get(key)
        .unwrap_or_else(|| panic!("meta 缺少 {key}"))
        .split_whitespace()
        .map(|value| {
            value
                .parse()
                .unwrap_or_else(|error| panic!("meta {key} 分量非 f32：{error}"))
        })
        .collect();
    assert_eq!(values.len(), N, "meta {key} 分量数不符");
    let mut array = [0.0; N];
    array.copy_from_slice(&values);
    array
}

fn build_meta(meta: &BTreeMap<String, String>) -> CollisionGpuMeta {
    CollisionGpuMeta {
        nworld: meta_value(meta, "nworld"),
        ngeom: meta_value(meta, "ngeom"),
        g1: meta_value(meta, "g1"),
        g2: meta_value(meta, "g2"),
        type1: meta_value(meta, "type1"),
        type2: meta_value(meta, "type2"),
        geomgeomid: 0,
        naccdmax: meta_value(meta, "naccdmax"),
        hfield_id: meta_value(meta, "hfield_id"),
        oct_root: meta_value(meta, "oct_root"),
        noct: meta_value(meta, "noct"),
        tolerance: meta_value(meta, "tolerance"),
        margin: meta_value(meta, "margin"),
        gap: meta_value(meta, "gap"),
        gjk_iterations: meta_value(meta, "gjk_iterations"),
        epa_iterations: meta_value(meta, "epa_iterations"),
        condim: meta_value(meta, "condim"),
        pairid0: meta_value(meta, "pairid0"),
        pairid1: meta_value(meta, "pairid1"),
        friction: meta_f32s(meta, "friction"),
        solref: meta_f32s(meta, "solref"),
        solreffriction: meta_f32s(meta, "solreffriction"),
        solimp: meta_f32s(meta, "solimp"),
        adhesion: meta_value(meta, "adhesion"),
        sdf_initpoints: meta_value(meta, "sdf_initpoints"),
        sdf_iterations: meta_value(meta, "sdf_iterations"),
        naconmax: meta_value(meta, "naconmax"),
        efc_width: meta_value(meta, "efc_width"),
    }
}

/// 读入一个冻结 case 的 23 项输入，跑 `CollisionKernel::Primitive`，返回回读结果。
pub(crate) fn run_case(
    program: &CollisionGpuProgram,
    case: &str,
    reference: &Value,
) -> CollisionGpuOutput {
    let directory = cases_root().join(case);
    let case_reference = &reference["cases"][case];
    guard_inputs(case, case_reference, &directory);
    let meta = build_meta(&read_meta(&directory));
    let worlds = case_reference["worlds"].as_u64().expect("QA worlds") as i32;
    assert_eq!(meta.nworld, worlds, "{case}: meta nworld 与 QA worlds 不符");

    macro_rules! load {
        ($($name:ident : $reader:ident),* $(,)?) => {
            $(let $name = $reader(&directory, stringify!($name));)*
        };
    }
    load!(
        geom_xpos: read_f32, geom_xmat: read_f32, geom_size: read_f32, geom_margin: read_f32,
        geom_gap: read_f32, geom_rbound: read_f32, geom_aabb: read_f32, geom_type: read_i32,
        geom_dataid: read_i32, mesh_vertadr: read_i32, mesh_vertnum: read_i32,
        mesh_graphadr: read_i32, mesh_octadr: read_i32, mesh_vert: read_f32,
        mesh_graph: read_i32, hfield_size: read_f32, hfield_nrow: read_i32,
        hfield_ncol: read_i32, hfield_adr: read_i32, hfield_data: read_f32,
        oct_aabb: read_f32, oct_child: read_i32, oct_coeff: read_f32,
    );
    let inputs = CollisionGpuInputs {
        geom_xpos: &geom_xpos,
        geom_xmat: &geom_xmat,
        geom_size: &geom_size,
        geom_margin: &geom_margin,
        geom_gap: &geom_gap,
        geom_rbound: &geom_rbound,
        geom_aabb: &geom_aabb,
        geom_type: &geom_type,
        geom_dataid: &geom_dataid,
        mesh_vertadr: &mesh_vertadr,
        mesh_vertnum: &mesh_vertnum,
        mesh_graphadr: &mesh_graphadr,
        mesh_octadr: &mesh_octadr,
        mesh_vert: &mesh_vert,
        mesh_graph: &mesh_graph,
        hfield_size: &hfield_size,
        hfield_nrow: &hfield_nrow,
        hfield_ncol: &hfield_ncol,
        hfield_adr: &hfield_adr,
        hfield_data: &hfield_data,
        oct_aabb: &oct_aabb,
        oct_child: &oct_child,
        oct_coeff: &oct_coeff,
        meta,
    };
    program
        .run(CollisionKernel::Primitive, &inputs)
        .unwrap_or_else(|error| panic!("{case} 运行失败：{error}"))
}
