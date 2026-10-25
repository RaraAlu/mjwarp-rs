//! G5 GPU 窗口对照辅助（被 `tests/g10_p5_gpu_kernels.rs` 以 `#[path]` 引入，不是独立测试目标）。
//!
//! 判据（Arch 2026-10-25 裁决）：
//! - 接触数组全等仅限上游走 CONVEX/SDF/HFIELD 的 case；primitive/multicontact 不在本片对照面。
//! - `frame`/`normal` 是单位向量 → ULP 统一按 **scale=1.0** 计，分量绝对差 ≤ 4 ULP@1.0
//!   （`4 × 2^-23 ≈ 4.77e-7`），零/非零分量一视同仁（不设近零特殊带）。
//! - `dist`/`pos` 按各自量级 ULP；参考分量为 0 时 ULP 无定义，用 ≤ 4 ULP@1.0 绝对差兜底并计数。
//! - 其余浮点（接触参数）与整数字段位级相等；count 相等；`efc_address` 全 -1；`OverflowType` 0。
//! - 原始行顺序由 atomic nacon 决定（不稳定），统一按 worldid 分组 + 稳定排序后比较。

// 证据比对辅助：字节切片解析与排序键刻意直白；按需豁免对应 clippy 提示。
#![allow(clippy::chunks_exact_to_as_chunks)]
#![allow(clippy::type_complexity)]

use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const CONTACT_CASES: [&str; 4] = [
    "ellipsoid-box",
    "hfield-sphere",
    "hfield-capsule",
    "sdf-box",
];
pub const WITNESS_CASES: [&str; 3] = ["cylinder-box", "box-box-margin", "mesh-mesh"];
/// 上游 dispatch 走 `primitive_narrowphase`（本片未实装）的 case，不列入设备 ccd 对照。
pub const PRIMITIVE_CASES: [&str; 3] = ["sphere-box-shallow", "sphere-box-deep", "capsule-box"];

const FLOAT_FIELDS: [&str; 9] = [
    "dist",
    "pos",
    "frame",
    "includemargin",
    "friction",
    "solref",
    "solreffriction",
    "solimp",
    "adhesion",
];
const INT_FIELDS: [&str; 5] = ["dim", "geom", "worldid", "type", "geomcollisionid"];
/// 上游接触参数（写接触时原样复制，必须位级相等）。
const PARAM_FLOAT_FIELDS: [&str; 6] = [
    "includemargin",
    "friction",
    "solref",
    "solreffriction",
    "solimp",
    "adhesion",
];
/// 4 ULP @1.0（f32）。
const FOUR_ULP_AT_ONE: f32 = 4.0 * (1.0 / 8_388_608.0);

pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn cases_dir() -> PathBuf {
    root().join("docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases")
}

fn read_f32(directory: &Path, name: &str) -> Vec<f32> {
    let path = directory.join(format!("{name}.bin"));
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("读取 {}: {error}", path.display()));
    assert_eq!(bytes.len() % 4, 0, "{} 非 f32 对齐", path.display());
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

fn read_i32(directory: &Path, name: &str) -> Vec<i32> {
    let path = directory.join(format!("{name}.bin"));
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("读取 {}: {error}", path.display()));
    assert_eq!(bytes.len() % 4, 0, "{} 非 i32 对齐", path.display());
    bytes
        .chunks_exact(4)
        .map(|chunk| i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

fn read_u32(directory: &Path, name: &str) -> Vec<u32> {
    let path = directory.join(format!("{name}.bin"));
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("读取 {}: {error}", path.display()));
    assert_eq!(bytes.len() % 4, 0, "{} 非 u32 对齐", path.display());
    bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

/// 逐接触行（字段 → 分量）。
struct Row {
    floats: BTreeMap<&'static str, Vec<f32>>,
    ints: BTreeMap<&'static str, Vec<i32>>,
}

impl Row {
    fn float(&self, name: &'static str) -> &[f32] {
        &self.floats[name]
    }

    fn int(&self, name: &'static str) -> &[i32] {
        &self.ints[name]
    }

    fn sort_key(&self) -> (i32, Vec<u32>, Vec<u32>, Vec<u32>, Vec<i32>, i32, i32, i32) {
        (
            self.int("worldid")[0],
            self.float("dist").iter().map(|v| v.to_bits()).collect(),
            self.float("pos").iter().map(|v| v.to_bits()).collect(),
            self.float("frame").iter().map(|v| v.to_bits()).collect(),
            self.int("geom").to_vec(),
            self.int("type")[0],
            self.int("dim")[0],
            self.int("geomcollisionid")[0],
        )
    }
}

fn split_rows(
    count: usize,
    floats: &BTreeMap<&'static str, Vec<f32>>,
    ints: &BTreeMap<&'static str, Vec<i32>>,
) -> Vec<Row> {
    let width = |length: usize| length.checked_div(count).unwrap_or(0);
    (0..count)
        .map(|index| Row {
            floats: floats
                .iter()
                .map(|(name, values)| {
                    let column = width(values.len());
                    (*name, values[index * column..(index + 1) * column].to_vec())
                })
                .collect(),
            ints: ints
                .iter()
                .map(|(name, values)| {
                    let column = width(values.len());
                    (*name, values[index * column..(index + 1) * column].to_vec())
                })
                .collect(),
        })
        .collect()
}

fn reference(
    case: &str,
) -> (
    usize,
    BTreeMap<&'static str, Vec<f32>>,
    BTreeMap<&'static str, Vec<i32>>,
) {
    let path = root().join(format!("fixtures/g10-p5/references/{case}.json"));
    let text = std::fs::read_to_string(&path).expect("读取上游参考 JSON");
    let value: Value = serde_json::from_str(&text).expect("解析上游参考 JSON");
    let count = value["output"]["count"].as_u64().expect("count") as usize;
    let fields = &value["output"]["fields"];
    let mut floats = BTreeMap::new();
    let mut ints = BTreeMap::new();
    for name in FLOAT_FIELDS {
        floats.insert(
            name,
            fields[name]["values"]
                .as_array()
                .expect("float values")
                .iter()
                .map(|item| item.as_f64().expect("float") as f32)
                .collect(),
        );
    }
    for name in INT_FIELDS {
        ints.insert(
            name,
            fields[name]["values"]
                .as_array()
                .expect("int values")
                .iter()
                .map(|item| item.as_i64().expect("int") as i32)
                .collect(),
        );
    }
    (count, floats, ints)
}

struct DeviceRun {
    count: usize,
    floats: BTreeMap<&'static str, Vec<f32>>,
    ints: BTreeMap<&'static str, Vec<i32>>,
    efc_address: Vec<i32>,
    efc_width: usize,
    overflow: Vec<u32>,
    nccd: i32,
}

fn device_run(directory: &Path) -> DeviceRun {
    let count = read_i32(directory, "nacon")[0] as usize;
    let mut floats = BTreeMap::new();
    for name in FLOAT_FIELDS {
        floats.insert(name, read_f32(directory, name));
    }
    let mut ints = BTreeMap::new();
    for name in INT_FIELDS {
        ints.insert(name, read_i32(directory, name));
    }
    let efc_width: usize = std::fs::read_to_string(directory.join("efc_width.txt"))
        .expect("efc_width.txt")
        .trim()
        .parse()
        .expect("efc width");
    DeviceRun {
        count,
        floats,
        ints,
        efc_address: read_i32(directory, "efc_address"),
        efc_width,
        overflow: read_u32(directory, "overflow"),
        nccd: read_i32(directory, "nccd")[0],
    }
}

fn ulp_diff(left: f32, right: f32) -> u32 {
    fn key(value: f32) -> i64 {
        let bits = value.to_bits() as i32;
        if bits < 0 {
            0x8000_0000_i64 - i64::from(bits)
        } else {
            i64::from(bits)
        }
    }
    (key(left) - key(right)).unsigned_abs() as u32
}

#[derive(Default)]
pub struct CaseReport {
    pub case: String,
    pub reference_count: usize,
    pub device_count: usize,
    pub dist_bit_equal: usize,
    pub pos_bit_equal: usize,
    pub frame_bit_equal: usize,
    pub dist_components: usize,
    pub pos_components: usize,
    pub frame_components: usize,
    pub dist_max_ulp: u32,
    pub pos_max_ulp: u32,
    pub frame_max_abs: f32,
    pub params_bit_equal: bool,
    pub ints_bit_equal: bool,
    pub efc_all_minus_one: bool,
    pub overflow: Vec<u32>,
    pub nccd: i32,
    pub two_runs_sorted_identical: bool,
}

fn compare_case(case: &str) -> CaseReport {
    let directory = cases_dir().join(case);
    let run1 = device_run(&directory.join("out1"));
    let run2 = device_run(&directory.join("out2"));
    let (reference_count, ref_floats, ref_ints) = reference(case);

    let mut report = CaseReport {
        case: case.to_owned(),
        reference_count,
        device_count: run1.count,
        overflow: run1.overflow.clone(),
        nccd: run1.nccd,
        params_bit_equal: true,
        ints_bit_equal: true,
        ..CaseReport::default()
    };
    assert_eq!(
        run1.count, reference_count,
        "{case}: 接触数不等（设备 {} vs 参考 {reference_count}）",
        run1.count
    );

    let mut ours = split_rows(run1.count, &run1.floats, &run1.ints);
    let mut theirs = split_rows(reference_count, &ref_floats, &ref_ints);
    ours.sort_by_key(Row::sort_key);
    theirs.sort_by_key(Row::sort_key);

    for (index, (our_row, ref_row)) in ours.iter().zip(theirs.iter()).enumerate() {
        for name in INT_FIELDS {
            if our_row.int(name) != ref_row.int(name) {
                panic!(
                    "{case}: 行 {index} 整数 {name} 不等（设备 {:?} vs 参考 {:?}）",
                    our_row.int(name),
                    ref_row.int(name)
                );
            }
        }
        for name in FLOAT_FIELDS {
            for (left, right) in our_row.float(name).iter().zip(ref_row.float(name)) {
                let equal = left.to_bits() == right.to_bits();
                match name {
                    "frame" => {
                        // 单位向量：ULP 统一按 scale=1.0，绝对差 ≤ 4 ULP@1.0。
                        let difference = (left - right).abs();
                        report.frame_max_abs = report.frame_max_abs.max(difference);
                        report.frame_components += 1;
                        report.frame_bit_equal += usize::from(equal);
                        assert!(
                            difference <= FOUR_ULP_AT_ONE,
                            "{case}: 行 {index} frame 绝对差 {difference} 超过 4 ULP@1.0（{FOUR_ULP_AT_ONE}）"
                        );
                    }
                    "dist" | "pos" => {
                        let ulp = ulp_diff(*left, *right);
                        let difference = (left - right).abs();
                        match name {
                            "dist" => {
                                report.dist_max_ulp = report.dist_max_ulp.max(ulp);
                                report.dist_components += 1;
                                report.dist_bit_equal += usize::from(equal);
                            }
                            _ => {
                                report.pos_max_ulp = report.pos_max_ulp.max(ulp);
                                report.pos_components += 1;
                                report.pos_bit_equal += usize::from(equal);
                            }
                        }
                        // 参考分量为 0 时 ULP 无定义：用 ≤4 ULP@1.0 绝对差兜底并计数。
                        if *right == 0.0 {
                            assert!(
                                difference <= FOUR_ULP_AT_ONE,
                                "{case}: 行 {index} {name} 零参考分量绝对差 {difference} 超过 4 ULP@1.0"
                            );
                        } else {
                            assert!(
                                ulp <= 4,
                                "{case}: 行 {index} {name} ULP={ulp} 超过 4（设备 {left} vs 参考 {right}）"
                            );
                        }
                    }
                    _ => {
                        if PARAM_FLOAT_FIELDS.contains(&name) && !equal {
                            panic!(
                                "{case}: 行 {index} 参数 {name} 位型不等（设备 {:#010x} vs 参考 {:#010x}）",
                                left.to_bits(),
                                right.to_bits()
                            );
                        }
                    }
                }
            }
        }
    }

    let expected_efc = run1.count * run1.efc_width;
    assert_eq!(run1.efc_address.len(), expected_efc, "{case}: efc 长度");
    report.efc_all_minus_one = run1.efc_address.iter().all(|&value| value == -1);
    assert!(report.efc_all_minus_one, "{case}: efc_address 非全 -1");
    assert!(
        run1.overflow.iter().all(|&value| value == 0),
        "{case}: overflow 非 0 {:?}",
        run1.overflow
    );
    // `nccd`：接触路径每 world 至多申请 1 次 EPA 槽；hfield/SDF 探针路径不申请（容量门归 launcher）。
    assert!(
        (0..=2).contains(&run1.nccd),
        "{case}: nccd 异常 {}",
        run1.nccd
    );

    assert_eq!(run1.count, run2.count, "{case}: 两次运行接触数不等");
    let mut first = split_rows(run1.count, &run1.floats, &run1.ints);
    let mut second = split_rows(run2.count, &run2.floats, &run2.ints);
    first.sort_by_key(Row::sort_key);
    second.sort_by_key(Row::sort_key);
    report.two_runs_sorted_identical = first.len() == second.len()
        && first
            .iter()
            .zip(second.iter())
            .all(|(left, right)| left.floats == right.floats && left.ints == right.ints);
    assert!(
        report.two_runs_sorted_identical,
        "{case}: 两次运行排序后位不等"
    );
    report
}

fn case_json(report: &CaseReport) -> Value {
    serde_json::json!({
        "case": report.case,
        "reference_count": report.reference_count,
        "device_count": report.device_count,
        "dist_bit_equal": report.dist_bit_equal,
        "dist_components": report.dist_components,
        "pos_bit_equal": report.pos_bit_equal,
        "pos_components": report.pos_components,
        "frame_bit_equal": report.frame_bit_equal,
        "frame_components": report.frame_components,
        "dist_max_ulp": report.dist_max_ulp,
        "pos_max_ulp": report.pos_max_ulp,
        "frame_max_abs": report.frame_max_abs,
        "params_bit_equal": report.params_bit_equal,
        "ints_bit_equal": report.ints_bit_equal,
        "efc_all_minus_one": report.efc_all_minus_one,
        "overflow": report.overflow,
        "nccd": report.nccd,
        "two_runs_sorted_identical": report.two_runs_sorted_identical,
    })
}

/// 跑全部可对照 case，返回 result.json 与未通过列表（不 panic 退出，便于先落证据）。
pub fn run_contact_comparison() -> (Value, Vec<String>) {
    let mut reports = Vec::new();
    let mut failures: Vec<String> = Vec::new();
    for case in CONTACT_CASES {
        let directory = cases_dir().join(case);
        assert!(
            directory.join("out1/nacon.bin").is_file(),
            "{case}: 缺少探针输出（先在 GPU 窗口运行 prepare_cases.py --device cuda 与 probe.exe）"
        );
        match std::panic::catch_unwind(|| compare_case(case)) {
            Ok(report) => reports.push(case_json(&report)),
            Err(payload) => {
                let message = payload
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| {
                        payload
                            .downcast_ref::<&str>()
                            .map(|text| (*text).to_owned())
                    })
                    .unwrap_or_default();
                failures.push(format!("{case}: {message}"));
            }
        }
    }
    let json = serde_json::json!({
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-gpu-kernels",
        "kind": "g5-device-vs-upstream-cuda-contact-arrays",
        "reference": "fixtures/g10-p5/references/<case>.json（G0b CUDA 捕获）",
        "criterion": "CONVEX/SDF/HFIELD case：count 相等 + worldid 分组/稳定排序后 dist/pos ≤4 ULP（零参考分量用 ≤4 ULP@1.0 绝对差）、frame 绝对差 ≤4 ULP@1.0（scale=1.0，不设近零带）、参数与整数字段位级相等",
        "device_probe": "target/g10-p5-g5/probe.exe（nvcc 12.8.93, fmad=true, -O3, sm_89；见 gpu-window/build-probe-report.txt）",
        "cases": reports,
        "failed_cases": failures.clone(),
        "witness_cases_not_compared": WITNESS_CASES,
        "primitive_cases_not_compared": PRIMITIVE_CASES,
        "unmet": [
            "上游 primitive narrowphase 的 case 不列入对照（参考非 ccd 产物；设备 primitive 路径本片未实装）",
            "multicontact 设备版未实装：cylinder-box/box-box-margin/mesh-mesh 接触数组不做全等；witness 对照待补",
            "NVRTC 产品 launcher 未接线：本测试比对的是 nvcc 独立探针（同一批 device 源、fmad=true）",
            "--fmad=false 仅诊断；sdf-box 在其下大幅偏移（梯度下降分岔），官方判据用 fmad=true",
        ],
    });
    (json, failures)
}
