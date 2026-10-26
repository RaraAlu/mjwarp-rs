//! stage-2 等价性窗口的**输入侧**（被 `tests/stage2_equivalence_window.rs` 以 `#[path]` 引入）。
//!
//! 两类输入，全部来自**已入库**材料，不新捕获：
//! 1. G5 冻结 case 目录 `docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases/<case>/`
//!    的 23 个输入 bin（`prepare_cases.py --device cuda` 产物，原样上传）；
//! 2. `convex-mesh`/`degenerate-mesh`（G2 五 case 中无 G5 设备输入的两例）：复用
//!    `src/physics/collision/gjk/case_data.rs` 里嵌入的 G2 静态输入（commit 内数据，不新捕获），
//!    运行时只**读取并解析**该文件的 `CASE_DATA` 字符串，不改动它。
//!
//! 这里的每一条 `OwnedInputs` 同时喂给设备入口与宿主复算，保证两侧输入逐位相同。

#![allow(dead_code)]
#![allow(clippy::chunks_exact_to_as_chunks)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mjwarp_rs::runtime::collision_gpu::{CollisionGpuInputs, CollisionGpuMeta, CollisionKernel};

/// G2 五 case（含设备输入缺失的 `convex-mesh`/`degenerate-mesh`）。
pub const G2_CASES: [&str; 5] = [
    "cylinder-box",
    "ellipsoid-box",
    "mesh-mesh",
    "convex-mesh",
    "degenerate-mesh",
];
/// 3 个 primitive 输入 case（设备侧既有 primive 内核证据在 G5 后续片）。
pub const PRIMITIVE_CASES: [&str; 3] = ["sphere-box-shallow", "sphere-box-deep", "capsule-box"];
/// 9 个冻结 case（G2 五 + 3 primitive + sdf-box）。
pub const ALL_CASES: [&str; 9] = [
    "cylinder-box",
    "ellipsoid-box",
    "mesh-mesh",
    "convex-mesh",
    "degenerate-mesh",
    "sphere-box-shallow",
    "sphere-box-deep",
    "capsule-box",
    "sdf-box",
];

/// 无 G5 设备输入 bin 的两例（设备输入由 case_data 的 G2 静态输入合成）。
pub fn is_synth(case: &str) -> bool {
    matches!(case, "convex-mesh" | "degenerate-mesh")
}

/// 每个 case 在窗口内要跑的设备内核：`Contact`（单接触 ccd）与 `ContactMulti`
/// （`use_multiccd=true` 静态特化）都跑；SDF 走 `Sdf`。
pub fn kernels(case: &str) -> &'static [CollisionKernel] {
    if case == "sdf-box" {
        &[CollisionKernel::Sdf]
    } else {
        &[CollisionKernel::Contact, CollisionKernel::ContactMulti]
    }
}

/// 设备内核 → 宿主 `ccd_pair` 的 `use_multiccd` 口径（同算法对照，不自造判据）。
pub fn host_multiccd(kernel: CollisionKernel) -> bool {
    kernel == CollisionKernel::ContactMulti
}

pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn bins_dir(case: &str) -> PathBuf {
    root()
        .join("docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases")
        .join(case)
}

/// 窗口产物目录（不提交；`target/` 已忽略）。
pub fn window_dir() -> PathBuf {
    root().join("target/stage2-impl/window")
}

fn read_bytes(directory: &Path, name: &str) -> Vec<u8> {
    let path = directory.join(format!("{name}.bin"));
    std::fs::read(&path).unwrap_or_else(|error| panic!("读取 {} 失败：{error}", path.display()))
}

pub(crate) fn read_f32(directory: &Path, name: &str) -> Vec<f32> {
    let bytes = read_bytes(directory, name);
    let (chunks, tail) = bytes.as_chunks::<4>();
    assert!(tail.is_empty(), "{name}.bin 长度不是 4 的倍数");
    chunks.iter().map(|c| f32::from_le_bytes(*c)).collect()
}

pub(crate) fn read_i32(directory: &Path, name: &str) -> Vec<i32> {
    let bytes = read_bytes(directory, name);
    let (chunks, tail) = bytes.as_chunks::<4>();
    assert!(tail.is_empty(), "{name}.bin 长度不是 4 的倍数");
    chunks.iter().map(|c| i32::from_le_bytes(*c)).collect()
}

pub(crate) fn read_meta(directory: &Path) -> BTreeMap<String, String> {
    let text = std::fs::read_to_string(directory.join("meta.txt")).expect("读 meta.txt");
    text.lines()
        .filter_map(|line| line.split_once(' '))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}

fn meta_i32(meta: &BTreeMap<String, String>, key: &str) -> i32 {
    meta.get(key)
        .unwrap_or_else(|| panic!("meta 缺少 {key}"))
        .trim()
        .parse()
        .unwrap_or_else(|error| panic!("meta {key} 非 i32：{error}"))
}

fn meta_f32(meta: &BTreeMap<String, String>, key: &str) -> f32 {
    meta.get(key)
        .unwrap_or_else(|| panic!("meta 缺少 {key}"))
        .trim()
        .parse()
        .unwrap_or_else(|error| panic!("meta {key} 非 f32：{error}"))
}

fn meta_f32s<const N: usize>(meta: &BTreeMap<String, String>, key: &str) -> [f32; N] {
    let values: Vec<f32> = meta
        .get(key)
        .unwrap_or_else(|| panic!("meta 缺少 {key}"))
        .split_whitespace()
        .map(|value| {
            value
                .parse()
                .unwrap_or_else(|error| panic!("meta {key} 分量：{error}"))
        })
        .collect();
    assert_eq!(values.len(), N, "meta {key} 分量数");
    let mut out = [0.0; N];
    out.copy_from_slice(&values);
    out
}

pub(crate) fn build_meta(meta: &BTreeMap<String, String>) -> CollisionGpuMeta {
    CollisionGpuMeta {
        nworld: meta_i32(meta, "nworld"),
        ngeom: meta_i32(meta, "ngeom"),
        g1: meta_i32(meta, "g1"),
        g2: meta_i32(meta, "g2"),
        type1: meta_i32(meta, "type1"),
        type2: meta_i32(meta, "type2"),
        geomgeomid: 0,
        naccdmax: meta_i32(meta, "naccdmax"),
        hfield_id: meta_i32(meta, "hfield_id"),
        oct_root: meta_i32(meta, "oct_root"),
        noct: meta_i32(meta, "noct"),
        tolerance: meta_f32(meta, "tolerance"),
        margin: meta_f32(meta, "margin"),
        gap: meta_f32(meta, "gap"),
        gjk_iterations: meta_i32(meta, "gjk_iterations"),
        epa_iterations: meta_i32(meta, "epa_iterations"),
        condim: meta_i32(meta, "condim"),
        pairid0: meta_i32(meta, "pairid0"),
        pairid1: meta_i32(meta, "pairid1"),
        friction: meta_f32s::<5>(meta, "friction"),
        solref: meta_f32s::<2>(meta, "solref"),
        solreffriction: meta_f32s::<2>(meta, "solreffriction"),
        solimp: meta_f32s::<5>(meta, "solimp"),
        adhesion: meta_f32(meta, "adhesion"),
        sdf_initpoints: meta_i32(meta, "sdf_initpoints"),
        sdf_iterations: meta_i32(meta, "sdf_iterations"),
        naconmax: meta_i32(meta, "naconmax"),
        efc_width: meta_i32(meta, "efc_width"),
    }
}

/// 23 个上传数组的 owned 形态（`view()` 再借出成 `CollisionGpuInputs`）。
pub struct OwnedInputs {
    pub meta: CollisionGpuMeta,
    pub geom_xpos: Vec<f32>,
    pub geom_xmat: Vec<f32>,
    pub geom_size: Vec<f32>,
    pub geom_margin: Vec<f32>,
    pub geom_gap: Vec<f32>,
    pub geom_rbound: Vec<f32>,
    pub geom_aabb: Vec<f32>,
    pub geom_type: Vec<i32>,
    pub geom_dataid: Vec<i32>,
    pub mesh_vertadr: Vec<i32>,
    pub mesh_vertnum: Vec<i32>,
    pub mesh_graphadr: Vec<i32>,
    pub mesh_octadr: Vec<i32>,
    pub mesh_vert: Vec<f32>,
    pub mesh_graph: Vec<i32>,
    pub hfield_size: Vec<f32>,
    pub hfield_nrow: Vec<i32>,
    pub hfield_ncol: Vec<i32>,
    pub hfield_adr: Vec<i32>,
    pub hfield_data: Vec<f32>,
    pub oct_aabb: Vec<f32>,
    pub oct_child: Vec<i32>,
    pub oct_coeff: Vec<f32>,
}

impl OwnedInputs {
    pub fn view(&self) -> CollisionGpuInputs<'_> {
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
}

/// 设备输入：G5 冻结 bin 原样上传。
pub fn device_inputs_from_bins(case: &str) -> OwnedInputs {
    let directory = bins_dir(case);
    let meta = build_meta(&read_meta(&directory));
    let read = |name: &str| read_f32(&directory, name);
    let read_i = |name: &str| read_i32(&directory, name);
    OwnedInputs {
        geom_xpos: read("geom_xpos"),
        geom_xmat: read("geom_xmat"),
        geom_size: read("geom_size"),
        geom_margin: read("geom_margin"),
        geom_gap: read("geom_gap"),
        geom_rbound: read("geom_rbound"),
        geom_aabb: read("geom_aabb"),
        geom_type: read_i("geom_type"),
        geom_dataid: read_i("geom_dataid"),
        mesh_vertadr: read_i("mesh_vertadr"),
        mesh_vertnum: read_i("mesh_vertnum"),
        mesh_graphadr: read_i("mesh_graphadr"),
        mesh_octadr: read_i("mesh_octadr"),
        mesh_vert: read("mesh_vert"),
        mesh_graph: read_i("mesh_graph"),
        hfield_size: read("hfield_size"),
        hfield_nrow: read_i("hfield_nrow"),
        hfield_ncol: read_i("hfield_ncol"),
        hfield_adr: read_i("hfield_adr"),
        hfield_data: read("hfield_data"),
        oct_aabb: read("oct_aabb"),
        oct_child: read_i("oct_child"),
        oct_coeff: read("oct_coeff"),
        meta,
    }
}

// `convex-mesh`/`degenerate-mesh` 的 G2 静态输入合成拆到独立 support 文件（500 行棘轮）；
// 这里 re-export，行为与数据逐位不变。
pub use crate::cases_synth::{g2_pair_inputs, host_mesh_topology, host_pair_inputs_from_bins};

pub fn device_inputs(case: &str) -> OwnedInputs {
    if is_synth(case) {
        crate::cases_synth::device_inputs_from_g2(case)
    } else {
        device_inputs_from_bins(case)
    }
}
