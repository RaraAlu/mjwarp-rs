//! G10@P5 修正片测试共享 helper（`g10_p5_ccd_overflow.rs` 以 `#[path]` 引入）。
//!
//! 只读证据目录（`docs/development-evidence/g10-p5-ccd-overflow-bit-fix/`）与 G5 冻结 case bins；
//! 只做字节/JSON 读取与接触行规范化，不含任何判据阈值。
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use serde_json::Value;

pub(crate) const EPA_HORIZON: u32 = 1 << 8;
pub(crate) const CCD: u32 = 1 << 4;
pub(crate) const FORCED_HORIZON_CASES: [&str; 3] = ["cylinder-box", "box-box-margin", "mesh-mesh"];
pub(crate) const REGRESSION_CONTACT_CASES: [&str; 4] = [
    "hfield-sphere",
    "hfield-capsule",
    "sdf-box",
    "ellipsoid-box",
];
pub(crate) const REGRESSION_WITNESS_CASES: [&str; 3] =
    ["cylinder-box", "box-box-margin", "mesh-mesh"];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(crate) fn evidence() -> PathBuf {
    root().join("docs/development-evidence/g10-p5-ccd-overflow-bit-fix")
}

pub(crate) fn frozen_cases() -> PathBuf {
    root().join("docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases")
}

pub(crate) fn read_bytes(path: &Path) -> Vec<u8> {
    let bytes =
        std::fs::read(path).unwrap_or_else(|error| panic!("读 {}: {error}", path.display()));
    assert!(!bytes.is_empty(), "{} 为空", path.display());
    bytes
}

/// 读 f32/u32 小端 bin 的 32 位位型（`dist`/`x1`/`x2` 等 f32 字段也用它取 bits）。
pub(crate) fn u32_values(path: &Path) -> Vec<u32> {
    read_bytes(path)
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| u32::from_le_bytes(*chunk))
        .collect()
}

pub(crate) fn i32_values(path: &Path) -> Vec<i32> {
    read_bytes(path)
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| i32::from_le_bytes(*chunk))
        .collect()
}

pub(crate) fn json_i32_array(value: &Value, pointer: &str) -> Vec<i32> {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("JSON 缺少 {pointer}"))
        .iter()
        .map(|item| i32::try_from(item.as_i64().expect("整数")).expect("i32"))
        .collect()
}

pub(crate) fn json_u32_array(value: &Value, pointer: &str) -> Vec<u32> {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("JSON 缺少 {pointer}"))
        .iter()
        .map(|item| u32::try_from(item.as_i64().expect("整数")).expect("u32"))
        .collect()
}

pub(crate) fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).expect("json")).expect("写证据");
    println!("wrote {}", path.display());
}

pub(crate) fn bin_names(directory: &Path) -> Vec<String> {
    let mut names = Vec::new();
    for entry in std::fs::read_dir(directory).expect("读目录") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|extension| extension == "bin") {
            names.push(
                path.file_name()
                    .expect("name")
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
    names.sort();
    names
}

/// contact 输出目录的字段行宽（与 G5 `probe.cu` 的 `write_bin` 一一对应）。
pub(crate) const CONTACT_FLOAT_FIELDS: [(&str, usize); 9] = [
    ("dist", 1),
    ("pos", 3),
    ("frame", 9),
    ("includemargin", 1),
    ("friction", 5),
    ("solref", 2),
    ("solreffriction", 2),
    ("solimp", 5),
    ("adhesion", 1),
];

pub(crate) fn efc_width(directory: &Path) -> usize {
    std::fs::read_to_string(directory.join("efc_width.txt"))
        .expect("efc_width.txt")
        .trim()
        .parse()
        .expect("efc_width")
}

/// 一行接触：浮点字段位型 + 整数字段值（排序后可整体位级比较）。
pub(crate) type ContactRow = (Vec<u32>, Vec<i32>);
/// 摘要 bin 与 `efc_width.txt`（文件名 + 原始字节）。
pub(crate) type ContactSummary = Vec<(String, Vec<u8>)>;

/// 把 contact 输出目录按行内全字段稳定排序成行集合。G5 判据同口径：原始行顺序由
/// `nacon` 原子计数决定、不稳定（G5 冻结 out1/out2 之间行序也不同），排序后位级比较。
pub(crate) fn canonical_contact_rows(directory: &Path) -> (Vec<ContactRow>, ContactSummary) {
    let efc = efc_width(directory);
    let int_fields: [(&str, usize); 6] = [
        ("dim", 1),
        ("geom", 2),
        ("efc_address", efc),
        ("worldid", 1),
        ("type", 1),
        ("geomcollisionid", 1),
    ];
    let rows = u32_values(&directory.join("dist.bin")).len();
    let float_columns: Vec<Vec<u32>> = CONTACT_FLOAT_FIELDS
        .iter()
        .map(|(name, _)| u32_values(&directory.join(format!("{name}.bin"))))
        .collect();
    let int_columns: Vec<Vec<i32>> = int_fields
        .iter()
        .map(|(name, _)| i32_values(&directory.join(format!("{name}.bin"))))
        .collect();
    let mut sorted_rows = Vec::with_capacity(rows);
    for row in 0..rows {
        let mut float_row = Vec::new();
        for (column, (_, width)) in CONTACT_FLOAT_FIELDS.iter().enumerate() {
            for offset in 0..*width {
                float_row.push(float_columns[column][row * width + offset]);
            }
        }
        let mut int_row = Vec::new();
        for (column, (_, width)) in int_fields.iter().enumerate() {
            for offset in 0..*width {
                int_row.push(int_columns[column][row * width + offset]);
            }
        }
        sorted_rows.push((float_row, int_row));
    }
    sorted_rows.sort();
    let mut summary = Vec::new();
    for name in ["overflow.bin", "nccd.bin", "nacon.bin"] {
        let path = directory.join(name);
        if path.is_file() {
            summary.push((name.to_owned(), read_bytes(&path)));
        }
    }
    summary.push((
        "efc_width.txt".to_owned(),
        read_bytes(&directory.join("efc_width.txt")),
    ));
    (sorted_rows, summary)
}
