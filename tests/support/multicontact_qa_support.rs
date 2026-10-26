//! G10@P5 multicontact 片验收 helper（`g10_p5_multicontact_cuda.rs` 以 `#[path]` 引入，
//! 测试专用；不进入产品路径）。
//!
//! 只读 G5 冻结 case bins 与 QA 已提交的上游参考 JSON；负责 23 项输入 sha256 守卫、
//! `meta.txt` 标量解析与 `CollisionGpuInputs` 组装（与 primitive 片同一套输入）。

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use mjwarp_rs::runtime::collision_gpu::{
    CollisionGpuInputs, CollisionGpuMeta, CollisionGpuOutput, CollisionGpuProgram, CollisionKernel,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// 32 项输入口径（M0b 起）：前 23 项为冻结 G5 case 的既有 bins，后 9 项为可选 mesh polygon
/// bins（M0a sidecar）。旧 23 项 case 目录缺 9 个 bin → 空 slice + null sidecar，不报错；
/// sha 守卫按 case 参考实际声明的数组集合执行（见 `guard_inputs`）。
const INPUT_BINS: [&str; 32] = [
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
    "mesh_polynum",
    "mesh_polyadr",
    "mesh_polynormal",
    "mesh_polyvertadr",
    "mesh_polyvertnum",
    "mesh_polyvert",
    "mesh_polymapadr",
    "mesh_polymapnum",
    "mesh_polymap",
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

/// 每个 case 使用的产品入口（与上游 dispatch + fixture flag 对应）：
/// - `cylinder-box`（multiccd=enable）→ `ContactMulti`（<=4 接触）；
/// - `box-box-margin`（nativeccd=disable → `(BOX,BOX)` primitive）→ `BoxBox`；
/// - `mesh-mesh`（multiccd 未启用）→ 既有 `Contact`（单接触）。
pub(crate) fn kernel_for(case: &str) -> CollisionKernel {
    match case {
        "cylinder-box" => CollisionKernel::ContactMulti,
        "box-box-margin" => CollisionKernel::BoxBox,
        "mesh-mesh" => CollisionKernel::Contact,
        other => panic!("未知 case {other}"),
    }
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

/// 可选 bin：文件缺失返回 `None`（旧 23 项 case 目录没有 9 个 mesh_poly bin）。
fn read_optional_bytes(directory: &Path, name: &str) -> Option<Vec<u8>> {
    let path = directory.join(format!("{name}.bin"));
    path.exists().then(|| read_bytes(directory, name))
}

fn read_optional_f32(directory: &Path, name: &str) -> Vec<f32> {
    read_optional_bytes(directory, name)
        .map(|bytes| {
            bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|chunk| f32::from_le_bytes(*chunk))
                .collect()
        })
        .unwrap_or_default()
}

fn read_optional_i32(directory: &Path, name: &str) -> Vec<i32> {
    read_optional_bytes(directory, name)
        .map(|bytes| {
            bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|chunk| i32::from_le_bytes(*chunk))
                .collect()
        })
        .unwrap_or_default()
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// 输入一致性守卫：按 QA 参考**实际声明**的 `input_bins` 集合逐项重算 sha256/count
/// （旧 23 项 case 只声明 23 项；声明集合中的名字必须在 `INPUT_BINS` 内）。
fn guard_inputs(case: &str, case_reference: &Value, directory: &Path) {
    let bins = &case_reference["input_bins"];
    let declared = bins
        .as_object()
        .unwrap_or_else(|| panic!("{case}: QA 参考 input_bins 不是对象"));
    for (name, bin_reference) in declared {
        assert!(
            INPUT_BINS.contains(&name.as_str()),
            "{case}: QA 参考声明了未知输入 {name}（不在 32 项 INPUT_BINS 内）"
        );
        let bytes = read_bytes(directory, name);
        let expected = bin_reference["sha256"]
            .as_str()
            .unwrap_or_else(|| panic!("{case}: QA 参考缺少 input_bins.{name}.sha256"));
        assert_eq!(
            sha256_hex(&bytes),
            expected,
            "{case}: 输入不一致 {name}.bin sha256 与 QA 参考不同"
        );
        let count = bin_reference["count"]
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
                .unwrap_or_else(|error| panic!("meta {key} 分量非 f32：{error:?}"))
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

/// 读入一个冻结 case 的 23 项输入，按上游 dispatch 选择的产品入口运行，返回回读结果。
pub(crate) fn run_case(
    program: &CollisionGpuProgram,
    case: &str,
    reference: &Value,
) -> (CollisionKernel, CollisionGpuOutput) {
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
    // M0a/M0b：9 个 mesh_poly bin 可选。旧 23 项 case 缺文件 → 空 slice + null sidecar；
    // 新 32 项 case 齐 9 个 → 正常装配。这里显式断言「9 个同时存在或同时缺失」。
    let mesh_polynum = read_optional_i32(&directory, "mesh_polynum");
    let mesh_polyadr = read_optional_i32(&directory, "mesh_polyadr");
    let mesh_polynormal = read_optional_f32(&directory, "mesh_polynormal");
    let mesh_polyvertadr = read_optional_i32(&directory, "mesh_polyvertadr");
    let mesh_polyvertnum = read_optional_i32(&directory, "mesh_polyvertnum");
    let mesh_polyvert = read_optional_i32(&directory, "mesh_polyvert");
    let mesh_polymapadr = read_optional_i32(&directory, "mesh_polymapadr");
    let mesh_polymapnum = read_optional_i32(&directory, "mesh_polymapnum");
    let mesh_polymap = read_optional_i32(&directory, "mesh_polymap");
    let mesh_poly_present = [
        mesh_polynum.is_empty(),
        mesh_polyadr.is_empty(),
        mesh_polynormal.is_empty(),
        mesh_polyvertadr.is_empty(),
        mesh_polyvertnum.is_empty(),
        mesh_polyvert.is_empty(),
        mesh_polymapadr.is_empty(),
        mesh_polymapnum.is_empty(),
        mesh_polymap.is_empty(),
    ]
    .iter()
    .filter(|empty| !**empty)
    .count();
    assert!(
        mesh_poly_present == 0 || mesh_poly_present == 9,
        "{case}: mesh_poly bin 应 9 个同时存在或同时缺失，实际 {mesh_poly_present} 个"
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
        // M0a/M0b：旧 case 的 9 项为空（null sidecar），新 case 从 bins 装配。
        mesh_polynum: &mesh_polynum,
        mesh_polyadr: &mesh_polyadr,
        mesh_polynormal: &mesh_polynormal,
        mesh_polyvertadr: &mesh_polyvertadr,
        mesh_polyvertnum: &mesh_polyvertnum,
        mesh_polyvert: &mesh_polyvert,
        mesh_polymapadr: &mesh_polymapadr,
        mesh_polymapnum: &mesh_polymapnum,
        mesh_polymap: &mesh_polymap,
        meta,
    };
    let kernel = kernel_for(case);
    let output = program
        .run(kernel, &inputs)
        .unwrap_or_else(|error| panic!("{case} 运行失败：{error}"));
    (kernel, output)
}
