//! M0b：mesh multicontact 32 项 case bins 的 CPU 装配 helper（`g10_p5_mesh_poly_case_assembly.rs`
//! 以 `#[path]` 引入，测试专用；不进入产品路径）。
//!
//! 读入 `docs/development-evidence/mesh-multicontact-m0b/cases/<case>/` 的 32 个 bin 与
//! `meta.txt`，装配 `CollisionGpuInputs`（32 项）。旧 G5 23 项 case 目录缺 9 个 mesh_poly bin
//! → 空 slice + null sidecar（与 M0a 语义一致）。
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use sha2::{Digest, Sha256};

use mjwarp_rs::runtime::collision_gpu::{CollisionGpuInputs, CollisionGpuMeta};
use serde_json::Value;

/// 32 项输入名（顺序即 M0b manifest 的 `arrays_order`；前 23 项为 G5 既有口径）。
pub(crate) const INPUT_BINS: [&str; 32] = [
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
    "mesh_polynum",
    "mesh_polyadr",
    "mesh_polynormal",
    "mesh_polyvertadr",
    "mesh_polyvertnum",
    "mesh_polyvert",
    "mesh_polymapadr",
    "mesh_polymapnum",
    "mesh_polymap",
    "hfield_size",
    "hfield_nrow",
    "hfield_ncol",
    "hfield_adr",
    "hfield_data",
    "oct_aabb",
    "oct_child",
    "oct_coeff",
];

/// M0a sidecar 的 9 个可选数组（与 `MjwarpMeshPolyArgs` 顺序一致）。
pub(crate) const MESH_POLY_BINS: [&str; 9] = [
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

pub(crate) fn m0b_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/development-evidence/mesh-multicontact-m0b")
}

pub(crate) fn m0b_cases_root() -> PathBuf {
    m0b_root().join("cases")
}

pub(crate) fn m0b_manifest() -> Value {
    let path = m0b_root().join("mesh-cases-manifest.json");
    let text = std::fs::read(&path).expect("读取 M0b manifest");
    serde_json::from_slice(&text).expect("解析 mesh-cases-manifest.json")
}

/// M3 mesh-mesh 证据根目录（bins / manifest / 新捕获参考）。
pub(crate) fn m3_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/development-evidence/mesh-multicontact-m3")
}

pub(crate) fn m3_cases_root() -> PathBuf {
    m3_root().join("cases")
}

pub(crate) fn m3_manifest() -> Value {
    let path = m3_root().join("mesh-case-manifest.json");
    let text = std::fs::read(&path).expect("读取 M3 manifest");
    serde_json::from_slice(&text).expect("解析 M3 mesh-case-manifest.json")
}

pub(crate) fn m3_reference() -> Value {
    let path = m3_root().join("upstream-mesh-multicontact-mesh-mesh-gpu.json");
    let text = std::fs::read(&path).expect("读取 M3 GPU 参考");
    serde_json::from_slice(&text).expect("解析 M3 参考 JSON")
}

pub(crate) fn m1_reference() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs/development-evidence/mesh-multicontact-m1/upstream-mesh-multicontact-gpu.json");
    let text = std::fs::read(&path).expect("读取 M1 GPU 参考");
    serde_json::from_slice(&text).expect("解析 M1 参考 JSON")
}

pub(crate) fn g5_cases_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases")
}

pub(crate) fn qa_reference() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "docs/development-evidence/g10-p5-gpu-kernels-independent-check/qa-upstream-gpu.json",
    );
    let text = std::fs::read(&path).expect("读取 QA 上游 GPU 参考");
    serde_json::from_slice(&text).expect("解析 qa-upstream-gpu.json")
}

fn read_bytes(directory: &Path, name: &str) -> Vec<u8> {
    let path = directory.join(format!("{name}.bin"));
    std::fs::read(&path).unwrap_or_else(|error| panic!("读取 {} 失败：{error}", path.display()))
}

fn read_optional_bytes(directory: &Path, name: &str) -> Option<Vec<u8>> {
    let path = directory.join(format!("{name}.bin"));
    path.exists().then(|| read_bytes(directory, name))
}

macro_rules! read_values {
    ($name:ident, $type:ty) => {
        fn $name(directory: &Path, name: &str) -> Vec<$type> {
            let bytes = read_bytes(directory, name);
            assert!(
                bytes.len().is_multiple_of(4),
                "{name}.bin 长度不是 4 的倍数"
            );
            bytes
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

fn build_meta(directory: &Path) -> CollisionGpuMeta {
    let meta = read_meta(directory);
    CollisionGpuMeta {
        nworld: meta_value(&meta, "nworld"),
        ngeom: meta_value(&meta, "ngeom"),
        g1: meta_value(&meta, "g1"),
        g2: meta_value(&meta, "g2"),
        type1: meta_value(&meta, "type1"),
        type2: meta_value(&meta, "type2"),
        geomgeomid: 0,
        naccdmax: meta_value(&meta, "naccdmax"),
        hfield_id: meta_value(&meta, "hfield_id"),
        oct_root: meta_value(&meta, "oct_root"),
        noct: meta_value(&meta, "noct"),
        tolerance: meta_value(&meta, "tolerance"),
        margin: meta_value(&meta, "margin"),
        gap: meta_value(&meta, "gap"),
        gjk_iterations: meta_value(&meta, "gjk_iterations"),
        epa_iterations: meta_value(&meta, "epa_iterations"),
        condim: meta_value(&meta, "condim"),
        pairid0: meta_value(&meta, "pairid0"),
        pairid1: meta_value(&meta, "pairid1"),
        friction: meta_f32s(&meta, "friction"),
        solref: meta_f32s(&meta, "solref"),
        solreffriction: meta_f32s(&meta, "solreffriction"),
        solimp: meta_f32s(&meta, "solimp"),
        adhesion: meta_value(&meta, "adhesion"),
        sdf_initpoints: meta_value(&meta, "sdf_initpoints"),
        sdf_iterations: meta_value(&meta, "sdf_iterations"),
        naconmax: meta_value(&meta, "naconmax"),
        efc_width: meta_value(&meta, "efc_width"),
    }
}

/// 一个 case 目录装配出的 32 项输入（拥有所有权，`inputs()` 借出 slice）。
pub(crate) struct MeshCaseInputs {
    pub(crate) geom_xpos: Vec<f32>,
    pub(crate) geom_xmat: Vec<f32>,
    pub(crate) geom_size: Vec<f32>,
    pub(crate) geom_margin: Vec<f32>,
    pub(crate) geom_gap: Vec<f32>,
    pub(crate) geom_rbound: Vec<f32>,
    pub(crate) geom_aabb: Vec<f32>,
    pub(crate) geom_type: Vec<i32>,
    pub(crate) geom_dataid: Vec<i32>,
    pub(crate) mesh_vertadr: Vec<i32>,
    pub(crate) mesh_vertnum: Vec<i32>,
    pub(crate) mesh_graphadr: Vec<i32>,
    pub(crate) mesh_octadr: Vec<i32>,
    pub(crate) mesh_vert: Vec<f32>,
    pub(crate) mesh_graph: Vec<i32>,
    pub(crate) mesh_polynum: Vec<i32>,
    pub(crate) mesh_polyadr: Vec<i32>,
    pub(crate) mesh_polynormal: Vec<f32>,
    pub(crate) mesh_polyvertadr: Vec<i32>,
    pub(crate) mesh_polyvertnum: Vec<i32>,
    pub(crate) mesh_polyvert: Vec<i32>,
    pub(crate) mesh_polymapadr: Vec<i32>,
    pub(crate) mesh_polymapnum: Vec<i32>,
    pub(crate) mesh_polymap: Vec<i32>,
    pub(crate) hfield_size: Vec<f32>,
    pub(crate) hfield_nrow: Vec<i32>,
    pub(crate) hfield_ncol: Vec<i32>,
    pub(crate) hfield_adr: Vec<i32>,
    pub(crate) hfield_data: Vec<f32>,
    pub(crate) oct_aabb: Vec<f32>,
    pub(crate) oct_child: Vec<i32>,
    pub(crate) oct_coeff: Vec<f32>,
    pub(crate) meta: CollisionGpuMeta,
}

impl MeshCaseInputs {
    /// 装配 `CollisionGpuInputs`（32 项；9 个 mesh_poly 缺失时为空 slice）。
    pub(crate) fn inputs(&self) -> CollisionGpuInputs<'_> {
        CollisionGpuInputs {
            geom_xpos: &self.geom_xpos,
            geom_xmat: &self.geom_xmat,
            geom_size: &self.geom_size,
            geom_margin: &self.geom_margin,
            geom_gap: &self.geom_gap,
            geom_rbound: &self.geom_rbound,
            geom_aabb: &self.geom_aabb,
            geom_type: &self.geom_type,
            geom_dataid: &self.geom_dataid,
            mesh_vertadr: &self.mesh_vertadr,
            mesh_vertnum: &self.mesh_vertnum,
            mesh_graphadr: &self.mesh_graphadr,
            mesh_octadr: &self.mesh_octadr,
            mesh_vert: &self.mesh_vert,
            mesh_graph: &self.mesh_graph,
            mesh_polynum: &self.mesh_polynum,
            mesh_polyadr: &self.mesh_polyadr,
            mesh_polynormal: &self.mesh_polynormal,
            mesh_polyvertadr: &self.mesh_polyvertadr,
            mesh_polyvertnum: &self.mesh_polyvertnum,
            mesh_polyvert: &self.mesh_polyvert,
            mesh_polymapadr: &self.mesh_polymapadr,
            mesh_polymapnum: &self.mesh_polymapnum,
            mesh_polymap: &self.mesh_polymap,
            hfield_size: &self.hfield_size,
            hfield_nrow: &self.hfield_nrow,
            hfield_ncol: &self.hfield_ncol,
            hfield_adr: &self.hfield_adr,
            hfield_data: &self.hfield_data,
            oct_aabb: &self.oct_aabb,
            oct_child: &self.oct_child,
            oct_coeff: &self.oct_coeff,
            meta: self.meta,
        }
    }

    /// M0a sidecar 构造条件（`collision_gpu.rs` 用 `!mesh_polynum.is_empty()` 判定）。
    pub(crate) fn mesh_poly_present(&self) -> bool {
        !self.mesh_polynum.is_empty()
    }

    /// 9 个 mesh_poly slice 的长度（顺序 = `MjwarpMeshPolyArgs` 指针顺序）。
    pub(crate) fn mesh_poly_lengths(&self) -> [usize; 9] {
        [
            self.mesh_polynum.len(),
            self.mesh_polyadr.len(),
            self.mesh_polynormal.len(),
            self.mesh_polyvertadr.len(),
            self.mesh_polyvertnum.len(),
            self.mesh_polyvert.len(),
            self.mesh_polymapadr.len(),
            self.mesh_polymapnum.len(),
            self.mesh_polymap.len(),
        ]
    }
}

/// 读一个 case 目录的 32 项输入：23 项必需、9 个 mesh_poly 可选（缺则空 slice）。
pub(crate) fn load_case(directory: &Path) -> MeshCaseInputs {
    MeshCaseInputs {
        geom_xpos: read_f32(directory, "geom_xpos"),
        geom_xmat: read_f32(directory, "geom_xmat"),
        geom_size: read_f32(directory, "geom_size"),
        geom_margin: read_f32(directory, "geom_margin"),
        geom_gap: read_f32(directory, "geom_gap"),
        geom_rbound: read_f32(directory, "geom_rbound"),
        geom_aabb: read_f32(directory, "geom_aabb"),
        geom_type: read_i32(directory, "geom_type"),
        geom_dataid: read_i32(directory, "geom_dataid"),
        mesh_vertadr: read_i32(directory, "mesh_vertadr"),
        mesh_vertnum: read_i32(directory, "mesh_vertnum"),
        mesh_graphadr: read_i32(directory, "mesh_graphadr"),
        mesh_octadr: read_i32(directory, "mesh_octadr"),
        mesh_vert: read_f32(directory, "mesh_vert"),
        mesh_graph: read_i32(directory, "mesh_graph"),
        mesh_polynum: read_optional_i32(directory, "mesh_polynum"),
        mesh_polyadr: read_optional_i32(directory, "mesh_polyadr"),
        mesh_polynormal: read_optional_f32(directory, "mesh_polynormal"),
        mesh_polyvertadr: read_optional_i32(directory, "mesh_polyvertadr"),
        mesh_polyvertnum: read_optional_i32(directory, "mesh_polyvertnum"),
        mesh_polyvert: read_optional_i32(directory, "mesh_polyvert"),
        mesh_polymapadr: read_optional_i32(directory, "mesh_polymapadr"),
        mesh_polymapnum: read_optional_i32(directory, "mesh_polymapnum"),
        mesh_polymap: read_optional_i32(directory, "mesh_polymap"),
        hfield_size: read_f32(directory, "hfield_size"),
        hfield_nrow: read_i32(directory, "hfield_nrow"),
        hfield_ncol: read_i32(directory, "hfield_ncol"),
        hfield_adr: read_i32(directory, "hfield_adr"),
        hfield_data: read_f32(directory, "hfield_data"),
        oct_aabb: read_f32(directory, "oct_aabb"),
        oct_child: read_i32(directory, "oct_child"),
        oct_coeff: read_f32(directory, "oct_coeff"),
        meta: build_meta(directory),
    }
}

/// 小写十六进制 sha256（M0b/M3 装配测试共用；自 `g10_p5_mesh_poly_case_assembly.rs` 挪入）。
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// f32 切片的位型列表（测试侧共用）。
pub(crate) fn f32_bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

/// JSON 数字/数组的 f32 位型展开（测试侧共用）。
pub(crate) fn json_f32_bits(value: &Value) -> Vec<u32> {
    match value {
        Value::Array(items) => items.iter().flat_map(json_f32_bits).collect(),
        Value::Number(number) => {
            vec![(number.as_f64().expect("数值") as f32).to_bits()]
        }
        other => panic!("参考字段不是数字/数组：{other}"),
    }
}
