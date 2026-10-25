//! G10@P5：产品 NVRTC 入口的 GPU 窗口验收（默认为 ignored，本机不跑）。
//!
//! 用产品 API（`CollisionGpuProgram::new(0)` + `run`）对 3 个冻结 case 重跑 G5 探针输入，
//! 并与探针 `out1/` 逐字段比对：先位级比较并打印 bit_equal；非位级按 G5 既有判据断言
//! （`dist` 自身量级 ≤4 ULP、`ref == 0` 用 4 ULP@1.0；`pos` 按 `max_i |ref_i|` 的
//! 4·ULP(scale)；`frame` 按 scale=1.0 的 4 ULP=4.768e-7；参数/整数/`nacon`/`nccd`/
//! `overflow` 位级）。数组按 probe 口径取前 `nacon` 行。
//! 运行：`cargo test --locked --features cuda-probe --test g10_p5_nvrtc_launcher -- --ignored --test-threads=1 --nocapture`

#![cfg(feature = "cuda-probe")]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mjwarp_rs::runtime::collision_gpu::{
    CollisionGpuInputs, CollisionGpuMeta, CollisionGpuOutput, CollisionGpuProgram, CollisionKernel,
};

const CASES: [&str; 3] = ["hfield-sphere", "sdf-box", "ellipsoid-box"];

const FOUR_ULP_AT_ONE: f32 = 4.0 * (1.0 / 8_388_608.0);

fn cases_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs/development-evidence/g10-p5-gpu-kernels/gpu-window/cases")
}

fn read_bytes(directory: &Path, name: &str) -> Vec<u8> {
    let path = directory.join(format!("{name}.bin"));
    std::fs::read(&path).unwrap_or_else(|error| panic!("读取 {} 失败：{error}", path.display()))
}

fn read_f32(directory: &Path, name: &str) -> Vec<f32> {
    let bytes = read_bytes(directory, name);
    assert!(
        bytes.len().is_multiple_of(4),
        "{name}.bin 长度不是 4 的倍数"
    );
    let (chunks, _) = bytes.as_chunks::<4>();
    chunks
        .iter()
        .map(|chunk| f32::from_le_bytes(*chunk))
        .collect()
}

fn read_i32(directory: &Path, name: &str) -> Vec<i32> {
    let bytes = read_bytes(directory, name);
    assert!(
        bytes.len().is_multiple_of(4),
        "{name}.bin 长度不是 4 的倍数"
    );
    let (chunks, _) = bytes.as_chunks::<4>();
    chunks
        .iter()
        .map(|chunk| i32::from_le_bytes(*chunk))
        .collect()
}

fn read_efc_width(directory: &Path) -> usize {
    let path = directory.join("efc_width.txt");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("读取 {} 失败：{error}", path.display()));
    text.trim()
        .parse()
        .unwrap_or_else(|error| panic!("{} 解析失败：{error}", path.display()))
}

fn read_meta(directory: &Path) -> BTreeMap<String, String> {
    let path = directory.join("meta.txt");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("读取 {} 失败：{error}", path.display()));
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
        .unwrap_or_else(|error| panic!("meta {key} 不是 i32：{error}"))
}

fn meta_f32(meta: &BTreeMap<String, String>, key: &str) -> f32 {
    meta.get(key)
        .unwrap_or_else(|| panic!("meta 缺少 {key}"))
        .trim()
        .parse()
        .unwrap_or_else(|error| panic!("meta {key} 不是 f32：{error}"))
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

struct CaseReport {
    case: String,
    kind: CollisionKernel,
    nacon: i32,
    nccd: i32,
    bit_equal: usize,
    total: usize,
    max_dist_ulp: u64,
    max_pos_ulp: u64,
    max_frame_ulp: u64,
    failures: Vec<String>,
}

impl CaseReport {
    fn new(case: &str, kind: CollisionKernel, output: &CollisionGpuOutput) -> Self {
        Self {
            case: case.to_owned(),
            kind,
            nacon: output.nacon,
            nccd: output.nccd,
            bit_equal: 0,
            total: 0,
            max_dist_ulp: 0,
            max_pos_ulp: 0,
            max_frame_ulp: 0,
            failures: Vec::new(),
        }
    }

    fn length(&mut self, field: &str, device: usize, reference: usize) {
        self.failures
            .push(format!("{field} 长度不符：dev={device} ref={reference}"));
    }

    fn print(&self) {
        println!(
            "case={} kind={:?} nacon={} nccd={} bit_equal={}/{} max_dist_ulp={} max_pos_ulp={} max_frame_ulp={} failures={}",
            self.case,
            self.kind,
            self.nacon,
            self.nccd,
            self.bit_equal,
            self.total,
            self.max_dist_ulp,
            self.max_pos_ulp,
            self.max_frame_ulp,
            self.failures.len()
        );
    }
}

fn ulp_at(value: f32) -> f32 {
    f32::from_bits(value.to_bits() + 1) - value
}

fn vector_scale(reference: &[f32]) -> f32 {
    let scale = reference
        .iter()
        .fold(0.0_f32, |scale, value| scale.max(value.abs()));
    if scale == 0.0 { 1.0 } else { scale }
}

fn vector_bound(scale: f32) -> f32 {
    4.0 * ulp_at(scale)
}

/// 有符号 f32 的单调位序（`-0.0` 与 `+0.0` 视为同一点）。
fn ordered(value: f32) -> i64 {
    let bits = value.to_bits();
    if bits & 0x8000_0000 != 0 {
        -i64::from(bits & 0x7fff_ffff)
    } else {
        i64::from(bits)
    }
}

fn ulp_distance(device: f32, reference: f32) -> u64 {
    (ordered(device) - ordered(reference)).unsigned_abs()
}

/// `dist` 判据：非零参考按 ≤4 ULP；零参考按 4 ULP@1.0 绝对差。
fn dist_ok(device: f32, reference: f32) -> bool {
    if reference == 0.0 {
        (device - reference).abs() <= FOUR_ULP_AT_ONE
    } else {
        ulp_distance(device, reference) <= 4
    }
}

/// 整数/结构字段：位级零容差。
fn check_exact<T: PartialEq + Copy + std::fmt::Display>(
    report: &mut CaseReport,
    field: &str,
    device: &[T],
    reference: &[T],
) {
    if device.len() != reference.len() {
        report.length(field, device.len(), reference.len());
        return;
    }
    for (index, (&dev, &refv)) in device.iter().zip(reference).enumerate() {
        report.total += 1;
        if dev == refv {
            report.bit_equal += 1;
            continue;
        }
        report
            .failures
            .push(format!("{field}[{index}] 位级不等 dev={dev} ref={refv}"));
    }
}

/// 接触参数（f32）：位级零容差。
fn check_exact_f32(report: &mut CaseReport, field: &str, device: &[f32], reference: &[f32]) {
    if device.len() != reference.len() {
        report.length(field, device.len(), reference.len());
        return;
    }
    for (index, (&dev, &refv)) in device.iter().zip(reference).enumerate() {
        report.total += 1;
        if dev.to_bits() == refv.to_bits() {
            report.bit_equal += 1;
            continue;
        }
        report.failures.push(format!(
            "{field}[{index}] 位级不等 dev={dev:e} ref={refv:e}"
        ));
    }
}

fn check_dist(report: &mut CaseReport, field: &str, device: &[f32], reference: &[f32]) {
    if device.len() != reference.len() {
        report.length(field, device.len(), reference.len());
        return;
    }
    for (index, (&dev, &refv)) in device.iter().zip(reference).enumerate() {
        report.total += 1;
        if dev.to_bits() == refv.to_bits() {
            report.bit_equal += 1;
            continue;
        }
        let ulp = ulp_distance(dev, refv);
        report.max_dist_ulp = report.max_dist_ulp.max(ulp);
        if !dist_ok(dev, refv) {
            report.failures.push(format!(
                "{field}[{index}] 超过 4 ULP dev={dev:e} ref={refv:e} ulp={ulp}"
            ));
        }
    }
}

fn check_vectors(
    report: &mut CaseReport,
    field: &str,
    device: &[f32],
    reference: &[f32],
    width: usize,
    unit_scale: bool,
) {
    if device.len() != reference.len() {
        report.length(field, device.len(), reference.len());
        return;
    }
    if !reference.len().is_multiple_of(width) {
        report.failures.push(format!(
            "{field} 长度 {} 不是 {width} 的倍数",
            reference.len()
        ));
        return;
    }
    for (row, reference_row) in reference.chunks_exact(width).enumerate() {
        let device_row = &device[row * width..(row + 1) * width];
        let scale = if unit_scale {
            1.0
        } else {
            vector_scale(reference_row)
        };
        let bound = vector_bound(scale);
        for (component, (&dev, &refv)) in device_row.iter().zip(reference_row).enumerate() {
            report.total += 1;
            if dev.to_bits() == refv.to_bits() {
                report.bit_equal += 1;
                continue;
            }
            let ulp = ulp_distance(dev, refv);
            if unit_scale {
                report.max_frame_ulp = report.max_frame_ulp.max(ulp);
            } else {
                report.max_pos_ulp = report.max_pos_ulp.max(ulp);
            }
            let difference = (dev - refv).abs();
            if difference > bound {
                report.failures.push(format!(
                    "{field}[{row}][{component}] |dev-ref|={difference:e} > bound={bound:e}（scale={scale:e}, ulp={ulp}）"
                ));
            }
        }
    }
}

macro_rules! compare_exact {
    ($checker:ident, $reader:ident, $report:ident, $directory:ident, $output:ident, $($field:ident),* $(,)?) => {
        $(
            $checker(
                &mut $report,
                stringify!($field),
                &$output.$field,
                &$reader(&$directory, stringify!($field)),
            );
        )*
    };
}

fn compare_case(
    case: &str,
    kind: CollisionKernel,
    directory: &Path,
    output: &CollisionGpuOutput,
) -> CaseReport {
    let expected = directory.join("out1");
    let mut report = CaseReport::new(case, kind, output);
    let expected_nacon = read_i32(&expected, "nacon");
    let expected_nccd = read_i32(&expected, "nccd");
    let efc_width = read_efc_width(&expected);
    if expected_nacon.len() != 1 || expected_nccd.len() != 1 {
        report.length("nacon/nccd", expected_nacon.len(), 1);
        return report;
    }
    if output.nacon != expected_nacon[0] {
        report.failures.push(format!(
            "nacon dev={} ref={}",
            output.nacon, expected_nacon[0]
        ));
    }
    if output.nccd != expected_nccd[0] {
        report
            .failures
            .push(format!("nccd dev={} ref={}", output.nccd, expected_nccd[0]));
    }
    let expected_overflow: Vec<u32> = read_i32(&expected, "overflow")
        .iter()
        .map(|value| *value as u32)
        .collect();
    check_exact(
        &mut report,
        "overflow",
        &output.overflow,
        &expected_overflow,
    );

    let rows = output.nacon.max(0) as usize;
    if output.efc_address.len() != rows * efc_width {
        report.length("efc_address", output.efc_address.len(), rows * efc_width);
    }
    check_dist(
        &mut report,
        "dist",
        &output.dist,
        &read_f32(&expected, "dist"),
    );
    check_vectors(
        &mut report,
        "pos",
        &output.pos,
        &read_f32(&expected, "pos"),
        3,
        false,
    );
    check_vectors(
        &mut report,
        "frame",
        &output.frame,
        &read_f32(&expected, "frame"),
        9,
        true,
    );
    #[rustfmt::skip]
    compare_exact!(
        check_exact_f32, read_f32, report, expected, output, includemargin, friction, solref,
        solreffriction, solimp, adhesion,
    );
    #[rustfmt::skip]
    compare_exact!(
        check_exact, read_i32, report, expected, output, dim, geom, efc_address, worldid,
        geomcollisionid,
    );
    check_exact(
        &mut report,
        "contact_type",
        &output.contact_type,
        &read_i32(&expected, "type"),
    );
    report
}

#[test]
#[ignore = "requires NVIDIA GPU + NVRTC"]
fn product_nvrtc_launcher_matches_g5_probe_out1() {
    let program = CollisionGpuProgram::new(0).expect("NVRTC 编译设备源并加载模块/入口");
    let mut failures: Vec<String> = Vec::new();
    for case in CASES {
        let directory = cases_root().join(case);
        let meta_map = read_meta(&directory);
        let kind = match meta_map.get("kind").map(String::as_str) {
            Some("contact") => CollisionKernel::Contact,
            Some("hfield") => CollisionKernel::Hfield,
            Some("sdf") => CollisionKernel::Sdf,
            other => panic!("未知 kind：{other:?}"),
        };
        let meta = build_meta(&meta_map);

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
        macro_rules! inputs {
            ($meta:expr, $($field:ident),* $(,)?) => {
                CollisionGpuInputs { $($field: &$field,)* meta: $meta }
            };
        }
        #[rustfmt::skip]
        let inputs = inputs!(
            meta,
            geom_xpos, geom_xmat, geom_size, geom_margin, geom_gap, geom_rbound, geom_aabb,
            geom_type, geom_dataid, mesh_vertadr, mesh_vertnum, mesh_graphadr, mesh_octadr,
            mesh_vert, mesh_graph, hfield_size, hfield_nrow, hfield_ncol, hfield_adr,
            hfield_data, oct_aabb, oct_child, oct_coeff,
        );

        let output = program.run(kind, &inputs).unwrap_or_else(|error| {
            panic!("{case} 运行失败：{error}");
        });
        let report = compare_case(case, kind, &directory, &output);
        report.print();
        for failure in &report.failures {
            println!("FAIL {case}: {failure}");
        }
        let tagged = |failure: String| format!("{case}: {failure}");
        failures.extend(report.failures.into_iter().map(tagged));
    }
    assert!(
        failures.is_empty(),
        "产品 NVRTC 入口与 G5 探针 out1 不一致（{} 条）",
        failures.len()
    );
}
