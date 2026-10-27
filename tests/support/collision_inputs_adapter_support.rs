//! S2 adapter CPU 对账支持：读取 M0b/M3 的 32 数组 case bins 与 `meta.txt`，
//! 按字段 id 建宿主视图，并提供位型/摘要工具。
//!
//! 路径口径（写死）：
//! - `mesh-box` / `mesh-cylinder`：`docs/development-evidence/mesh-multicontact-m0b/cases/<case>/`
//! - `mesh-mesh`（对应 `fixtures/g10-p5-mesh-multicontact/mesh-mesh-two-assets.xml`）：
//!   `docs/development-evidence/mesh-multicontact-m3/cases/mesh-mesh/`（M0b 只落了 mesh-box /
//!   mesh-cylinder 两个 case；32 数组同口径由 M3 捕获窗产出，见证据 README）。
//!
//! 只读这些文件，不写、不改；测试不启动 Python、不使用 GPU。

use mjwarp_rs::{
    model::collision_inputs::{COLLISION_INPUTS_FIELDS, CollisionInputsCounts},
    runtime::collision_gpu::{CollisionGpuInputs, CollisionGpuMeta, CollisionInputsSnapshot},
};
use std::path::Path;

pub const ROOT: &str = env!("CARGO_MANIFEST_DIR");

/// 三份 case 的目录（相对仓库根）；`mesh-mesh` 的 fixture 名是 `mesh-mesh-two-assets`。
pub const CASES: [(&str, &str); 3] = [
    (
        "mesh-box",
        "docs/development-evidence/mesh-multicontact-m0b/cases/mesh-box",
    ),
    (
        "mesh-cylinder",
        "docs/development-evidence/mesh-multicontact-m0b/cases/mesh-cylinder",
    ),
    (
        "mesh-mesh",
        "docs/development-evidence/mesh-multicontact-m3/cases/mesh-mesh",
    ),
];

/// 9 项可选 mesh_poly 字段号（其余 23 项是基础输入）。
pub const MESH_POLY_IDS: [u32; 9] = [15, 16, 17, 18, 19, 20, 21, 22, 23];

#[derive(Clone, Debug)]
pub enum FieldValue {
    F32(Vec<f32>),
    I32(Vec<i32>),
}

#[derive(Clone, Debug)]
pub struct Field {
    pub raw: Vec<u8>,
    pub value: FieldValue,
}

/// 一个 case 的 32 项 bin（缺失项为 None，对应旧 23 项输入）与 meta。
#[derive(Clone, Debug)]
pub struct AdapterCase {
    pub name: &'static str,
    pub meta: CollisionGpuMeta,
    pub fields: Vec<Option<Field>>,
}

impl AdapterCase {
    pub fn read(name: &'static str, dir: &'static str) -> Self {
        let root = Path::new(ROOT).join(dir);
        let mut fields: Vec<Option<Field>> = Vec::new();
        for spec in COLLISION_INPUTS_FIELDS.iter() {
            let path = root.join(format!("{}.bin", spec.name));
            let raw = std::fs::read(&path)
                .unwrap_or_else(|error| panic!("缺少 case bin {}: {error}", path.display()));
            let (words, tail) = raw.as_chunks::<4>();
            assert!(tail.is_empty(), "{} 长度须为 4 的倍数", spec.name);
            let value = if spec.kind.device_is_float() {
                FieldValue::F32(
                    words
                        .iter()
                        .map(|bytes| f32::from_le_bytes(*bytes))
                        .collect(),
                )
            } else {
                FieldValue::I32(
                    words
                        .iter()
                        .map(|bytes| i32::from_le_bytes(*bytes))
                        .collect(),
                )
            };
            fields.push(Some(Field { raw, value }));
        }
        Self {
            name,
            meta: parse_meta(&root.join("meta.txt")),
            fields,
        }
    }

    pub fn field(&self, id: u32) -> &Field {
        self.fields[id as usize]
            .as_ref()
            .unwrap_or_else(|| panic!("{} 缺少字段 {id}", self.name))
    }

    pub fn f32s(&self, id: u32) -> &[f32] {
        match &self.field(id).value {
            FieldValue::F32(values) => values,
            FieldValue::I32(_) => panic!("字段 {id} 应为 f32"),
        }
    }

    pub fn i32s(&self, id: u32) -> &[i32] {
        match &self.field(id).value {
            FieldValue::I32(values) => values,
            FieldValue::F32(_) => panic!("字段 {id} 应为 i32"),
        }
    }

    /// 从 bins 反推 32 项计数（与 S1 `CollisionInputsCounts` 同口径）。
    pub fn counts(&self) -> CollisionInputsCounts {
        let ngeom = self.i32s(7).len();
        let nmesh_vertadr = self.i32s(9).len();
        CollisionInputsCounts {
            nq: 0,
            ngeom: ngeom as i64,
            nmesh: nmesh_vertadr as i64,
            nmeshvert: self.f32s(13).len() as i64 / 3,
            nmeshgraph: self.i32s(14).len() as i64,
            nmeshpoly: self.f32s(17).len() as i64 / 3,
            nmeshpolyvert: self.i32s(20).len() as i64,
            nmeshpolymap: self.i32s(23).len() as i64,
            nhfield: self.i32s(25).len() as i64,
            nhfielddata: self.f32s(28).len() as i64,
            noct: self.f32s(29).len() as i64 / 6,
        }
    }

    /// 按字段 id 建视图；`mesh_poly=false` 时 9 项可选字段用空 slice（旧 23 项输入）。
    pub fn snapshot(&self, mesh_poly: bool) -> CollisionInputsSnapshot<'_> {
        let keep = |id: u32| !MESH_POLY_IDS.contains(&id) || mesh_poly;
        let f = |id: u32| -> &[f32] { if keep(id) { self.f32s(id) } else { &[] } };
        let i = |id: u32| -> &[i32] { if keep(id) { self.i32s(id) } else { &[] } };
        CollisionInputsSnapshot {
            geom_xpos: f(0),
            geom_xmat: f(1),
            geom_size: f(2),
            geom_margin: f(3),
            geom_gap: f(4),
            geom_rbound: f(5),
            geom_aabb: f(6),
            geom_type: i(7),
            geom_dataid: i(8),
            mesh_vertadr: i(9),
            mesh_vertnum: i(10),
            mesh_graphadr: i(11),
            mesh_octadr: i(12),
            mesh_vert: f(13),
            mesh_graph: i(14),
            mesh_polynum: i(15),
            mesh_polyadr: i(16),
            mesh_polynormal: f(17),
            mesh_polyvertadr: i(18),
            mesh_polyvertnum: i(19),
            mesh_polyvert: i(20),
            mesh_polymapadr: i(21),
            mesh_polymapnum: i(22),
            mesh_polymap: i(23),
            hfield_size: f(24),
            hfield_nrow: i(25),
            hfield_ncol: i(26),
            hfield_adr: i(27),
            hfield_data: f(28),
            oct_aabb: f(29),
            oct_child: i(30),
            oct_coeff: f(31),
        }
    }
}

/// 解析 `meta.txt`（`key value...` 每行）；`geomgeomid` 不在文件里，按 0 处理
/// （`validate_inputs` 不使用该字段，产品接线时由 S3 显式给出）。
pub fn parse_meta(path: &Path) -> CollisionGpuMeta {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("缺少 meta {}: {error}", path.display()));
    let mut meta = CollisionGpuMeta {
        nworld: 0,
        ngeom: 0,
        g1: 0,
        g2: 0,
        type1: 0,
        type2: 0,
        geomgeomid: 0,
        naccdmax: 0,
        hfield_id: 0,
        oct_root: 0,
        noct: 0,
        tolerance: 0.0,
        margin: 0.0,
        gap: 0.0,
        gjk_iterations: 0,
        epa_iterations: 0,
        condim: 0,
        pairid0: 0,
        pairid1: 0,
        friction: [0.0; 5],
        solref: [0.0; 2],
        solreffriction: [0.0; 2],
        solimp: [0.0; 5],
        adhesion: 0.0,
        sdf_initpoints: 0,
        sdf_iterations: 0,
        naconmax: 0,
        efc_width: 0,
    };
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let Some(key) = parts.next() else { continue };
        let rest: Vec<&str> = parts.collect();
        let int = |index: usize| -> i32 { rest[index].parse().unwrap() };
        let float = |index: usize| -> f32 { rest[index].parse::<f64>().unwrap() as f32 };
        match key {
            "nworld" => meta.nworld = int(0),
            "ngeom" => meta.ngeom = int(0),
            "naconmax" => meta.naconmax = int(0),
            "naccdmax" => meta.naccdmax = int(0),
            "efc_width" => meta.efc_width = int(0),
            "g1" => meta.g1 = int(0),
            "g2" => meta.g2 = int(0),
            "type1" => meta.type1 = int(0),
            "type2" => meta.type2 = int(0),
            "tolerance" => meta.tolerance = float(0),
            "gjk_iterations" => meta.gjk_iterations = int(0),
            "epa_iterations" => meta.epa_iterations = int(0),
            "margin" => meta.margin = float(0),
            "gap" => meta.gap = float(0),
            "condim" => meta.condim = int(0),
            "friction" => {
                for (slot, value) in meta.friction.iter_mut().enumerate() {
                    *value = float(slot);
                }
            }
            "solref" => {
                for (slot, value) in meta.solref.iter_mut().enumerate() {
                    *value = float(slot);
                }
            }
            "solreffriction" => {
                for (slot, value) in meta.solreffriction.iter_mut().enumerate() {
                    *value = float(slot);
                }
            }
            "solimp" => {
                for (slot, value) in meta.solimp.iter_mut().enumerate() {
                    *value = float(slot);
                }
            }
            "adhesion" => meta.adhesion = float(0),
            "pairid0" => meta.pairid0 = int(0),
            "pairid1" => meta.pairid1 = int(0),
            "sdf_initpoints" => meta.sdf_initpoints = int(0),
            "sdf_iterations" => meta.sdf_iterations = int(0),
            "hfield_id" => meta.hfield_id = int(0),
            "oct_root" => meta.oct_root = int(0),
            "noct" => meta.noct = int(0),
            "case" => {}
            other => panic!("meta.txt 未知键 {other}"),
        }
    }
    meta
}

/// f32 slice 的 little-endian 字节（与 bin 落盘口径一致）。
pub fn f32_bytes(values: &[f32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

/// i32 slice 的 little-endian 字节。
pub fn i32_bytes(values: &[i32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

/// 32 项输入中某个字段的字节（f32/i32 按字段 id 的 kind 取）。
pub fn input_bytes(inputs: &CollisionGpuInputs<'_>, id: u32) -> Vec<u8> {
    match id {
        0 => f32_bytes(inputs.geom_xpos),
        1 => f32_bytes(inputs.geom_xmat),
        2 => f32_bytes(inputs.geom_size),
        3 => f32_bytes(inputs.geom_margin),
        4 => f32_bytes(inputs.geom_gap),
        5 => f32_bytes(inputs.geom_rbound),
        6 => f32_bytes(inputs.geom_aabb),
        7 => i32_bytes(inputs.geom_type),
        8 => i32_bytes(inputs.geom_dataid),
        9 => i32_bytes(inputs.mesh_vertadr),
        10 => i32_bytes(inputs.mesh_vertnum),
        11 => i32_bytes(inputs.mesh_graphadr),
        12 => i32_bytes(inputs.mesh_octadr),
        13 => f32_bytes(inputs.mesh_vert),
        14 => i32_bytes(inputs.mesh_graph),
        15 => i32_bytes(inputs.mesh_polynum),
        16 => i32_bytes(inputs.mesh_polyadr),
        17 => f32_bytes(inputs.mesh_polynormal),
        18 => i32_bytes(inputs.mesh_polyvertadr),
        19 => i32_bytes(inputs.mesh_polyvertnum),
        20 => i32_bytes(inputs.mesh_polyvert),
        21 => i32_bytes(inputs.mesh_polymapadr),
        22 => i32_bytes(inputs.mesh_polymapnum),
        23 => i32_bytes(inputs.mesh_polymap),
        24 => f32_bytes(inputs.hfield_size),
        25 => i32_bytes(inputs.hfield_nrow),
        26 => i32_bytes(inputs.hfield_ncol),
        27 => i32_bytes(inputs.hfield_adr),
        28 => f32_bytes(inputs.hfield_data),
        29 => f32_bytes(inputs.oct_aabb),
        30 => i32_bytes(inputs.oct_child),
        31 => f32_bytes(inputs.oct_coeff),
        other => panic!("未知字段 id {other}"),
    }
}
