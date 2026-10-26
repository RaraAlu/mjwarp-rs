//! launcher 回归比对的共享实现（行序归一化 + 逐字段检查器）。
//!
//! - 行序不作判据：先按 `worldid` 分组、组内按 G5 `sort_contacts` 键 `dist → pos[0..2] →
//!   geomcollisionid` 稳定排序，再逐字段走原检查器（判据/容差/字段集未变）。
//! - 本模块由 `tests/g10_p5_nvrtc_launcher.rs` 以 `#[path = "support/launcher_compare_support.rs"]`
//!   引入；拆分原因是单文件 500 行上限（normify size gate）。
use std::path::Path;

use mjwarp_rs::runtime::collision_gpu::{CollisionGpuOutput, CollisionKernel};

use super::{read_efc_width, read_f32, read_i32};

const FOUR_ULP_AT_ONE: f32 = 4.0 * (1.0 / 8_388_608.0);

pub(crate) struct CaseReport {
    pub(crate) case: String,
    pub(crate) kind: CollisionKernel,
    pub(crate) nacon: i32,
    pub(crate) nccd: i32,
    pub(crate) bit_equal: usize,
    pub(crate) total: usize,
    pub(crate) max_dist_ulp: u64,
    pub(crate) max_pos_ulp: u64,
    pub(crate) max_frame_ulp: u64,
    pub(crate) failures: Vec<String>,
}

impl CaseReport {
    pub(crate) fn new(case: &str, kind: CollisionKernel, output: &CollisionGpuOutput) -> Self {
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

    pub(crate) fn print(&self) {
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
pub(crate) fn check_exact<T: PartialEq + Copy + std::fmt::Display>(
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
pub(crate) fn check_exact_f32(
    report: &mut CaseReport,
    field: &str,
    device: &[f32],
    reference: &[f32],
) {
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

pub(crate) fn check_dist(report: &mut CaseReport, field: &str, device: &[f32], reference: &[f32]) {
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

pub(crate) fn check_vectors(
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

// （原 `compare_exact!` 宏已删除：行序归一化后所有字段改为显式传入 permuted 切片。）

/// 行序归一化（G5 口径）：先按 `worldid` 分组，组内按 G5 `sort_contacts` 排序键
/// （`dist → pos[0..2] → geomcollisionid`）稳定排序，再逐字段比较。行序不作判据（多 world 的接触
/// 写入依赖共享 `atomicAdd`，跨 world 行序无保证）；判据本身不变（位级 / 4 ULP / 结构零容差）。
pub(crate) fn g5_contact_order(
    worldid: &[i32],
    dist: &[f32],
    pos: &[f32],
    geomcollisionid: &[i32],
) -> Vec<usize> {
    let mut order: Vec<usize> = (0..worldid.len()).collect();
    order.sort_by(|&a, &b| {
        worldid[a]
            .cmp(&worldid[b])
            .then(dist[a].total_cmp(&dist[b]))
            .then(pos[a * 3].total_cmp(&pos[b * 3]))
            .then(pos[a * 3 + 1].total_cmp(&pos[b * 3 + 1]))
            .then(pos[a * 3 + 2].total_cmp(&pos[b * 3 + 2]))
            .then(geomcollisionid[a].cmp(&geomcollisionid[b]))
    });
    order
}

pub(crate) fn permute_rows_f32(values: &[f32], width: usize, order: &[usize]) -> Vec<f32> {
    let mut out = Vec::with_capacity(values.len());
    for &row in order {
        out.extend_from_slice(&values[row * width..(row + 1) * width]);
    }
    out
}

pub(crate) fn permute_rows_i32(values: &[i32], width: usize, order: &[usize]) -> Vec<i32> {
    let mut out = Vec::with_capacity(values.len());
    for &row in order {
        out.extend_from_slice(&values[row * width..(row + 1) * width]);
    }
    out
}

pub(crate) fn compare_case(
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
    // 行序归一化（G5 口径：worldid 分组 + 组内 sort_contacts 键）；判据与字段集不变。
    let expected_worldid = read_i32(&expected, "worldid");
    let expected_dist_raw = read_f32(&expected, "dist");
    let expected_pos_raw = read_f32(&expected, "pos");
    let expected_id_raw = read_i32(&expected, "geomcollisionid");
    if output.worldid.len() != rows
        || expected_worldid.len() != rows
        || output.dist.len() != rows
        || expected_dist_raw.len() != rows
        || output.pos.len() != rows * 3
        || expected_pos_raw.len() != rows * 3
        || output.geomcollisionid.len() != rows
        || expected_id_raw.len() != rows
    {
        report.length("rows(normalize)", output.worldid.len(), rows);
        return report;
    }
    let device_order = g5_contact_order(
        &output.worldid,
        &output.dist,
        &output.pos,
        &output.geomcollisionid,
    );
    let reference_order = g5_contact_order(
        &expected_worldid,
        &expected_dist_raw,
        &expected_pos_raw,
        &expected_id_raw,
    );
    let dev_f = |values: &[f32], width: usize| permute_rows_f32(values, width, &device_order);
    let ref_f = |field: &str, width: usize| {
        permute_rows_f32(&read_f32(&expected, field), width, &reference_order)
    };
    let dev_i = |values: &[i32], width: usize| permute_rows_i32(values, width, &device_order);
    let ref_i = |field: &str, width: usize| {
        permute_rows_i32(&read_i32(&expected, field), width, &reference_order)
    };

    check_dist(
        &mut report,
        "dist",
        &dev_f(&output.dist, 1),
        &ref_f("dist", 1),
    );
    check_vectors(
        &mut report,
        "pos",
        &dev_f(&output.pos, 3),
        &ref_f("pos", 3),
        3,
        false,
    );
    check_vectors(
        &mut report,
        "frame",
        &dev_f(&output.frame, 9),
        &ref_f("frame", 9),
        9,
        true,
    );
    check_exact_f32(
        &mut report,
        "includemargin",
        &dev_f(&output.includemargin, 1),
        &ref_f("includemargin", 1),
    );
    check_exact_f32(
        &mut report,
        "friction",
        &dev_f(&output.friction, 5),
        &ref_f("friction", 5),
    );
    check_exact_f32(
        &mut report,
        "solref",
        &dev_f(&output.solref, 2),
        &ref_f("solref", 2),
    );
    check_exact_f32(
        &mut report,
        "solreffriction",
        &dev_f(&output.solreffriction, 2),
        &ref_f("solreffriction", 2),
    );
    check_exact_f32(
        &mut report,
        "solimp",
        &dev_f(&output.solimp, 5),
        &ref_f("solimp", 5),
    );
    check_exact_f32(
        &mut report,
        "adhesion",
        &dev_f(&output.adhesion, 1),
        &ref_f("adhesion", 1),
    );
    check_exact(&mut report, "dim", &dev_i(&output.dim, 1), &ref_i("dim", 1));
    check_exact(
        &mut report,
        "geom",
        &dev_i(&output.geom, 2),
        &ref_i("geom", 2),
    );
    check_exact(
        &mut report,
        "efc_address",
        &dev_i(&output.efc_address, efc_width),
        &ref_i("efc_address", efc_width),
    );
    check_exact(
        &mut report,
        "worldid",
        &dev_i(&output.worldid, 1),
        &ref_i("worldid", 1),
    );
    check_exact(
        &mut report,
        "geomcollisionid",
        &dev_i(&output.geomcollisionid, 1),
        &ref_i("geomcollisionid", 1),
    );
    check_exact(
        &mut report,
        "contact_type",
        &dev_i(&output.contact_type, 1),
        &ref_i("type", 1),
    );
    report
}
