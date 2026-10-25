//! G10@P5 multicontact 片：接触数组比较（G5 既有口径）。
//!
//! 排序口径：先按 `(worldid, geomcollisionid)` 稳定排序（`geomcollisionid` 就是上游
//! `write_contact` 的 `id_` 写入序号；同一 world 内与上游循环顺序一致），再逐字段比较；
//! `dist`/`pos`/`frame` 允许 4 ULP 级误差，参数/整数/计数零容差。

use mjwarp_rs::runtime::collision_gpu::CollisionGpuOutput;
use serde_json::Value;

/// `4 ULP @ 1.0`（f32）。
const FOUR_ULP_AT_ONE: f32 = 4.0 * (1.0 / 8_388_608.0);

fn json_f32(case_reference: &Value, name: &str) -> Vec<f32> {
    case_reference["fields"][name]
        .as_array()
        .unwrap_or_else(|| panic!("QA 参考缺少 fields.{name}"))
        .iter()
        .map(|value| value.as_f64().expect("fields 分量非数值") as f32)
        .collect()
}

fn json_i32(case_reference: &Value, name: &str) -> Vec<i32> {
    case_reference["fields"][name]
        .as_array()
        .unwrap_or_else(|| panic!("QA 参考缺少 fields.{name}"))
        .iter()
        .map(|value| value.as_i64().expect("fields 分量非整数") as i32)
        .collect()
}

pub(crate) struct CaseReport {
    pub(crate) case: String,
    pub(crate) kind: String,
    pub(crate) nacon: i32,
    pub(crate) nccd: i32,
    pub(crate) total: usize,
    pub(crate) bit_equal: usize,
    pub(crate) max_dist_ulp: u64,
    pub(crate) max_pos_ulp: u64,
    pub(crate) max_frame_ulp: u64,
    pub(crate) failures: Vec<String>,
}

impl CaseReport {
    fn new(case: &str, kind: &str, output: &CollisionGpuOutput) -> Self {
        Self {
            case: case.to_owned(),
            kind: kind.to_owned(),
            nacon: output.nacon,
            nccd: output.nccd,
            total: 0,
            bit_equal: 0,
            max_dist_ulp: 0,
            max_pos_ulp: 0,
            max_frame_ulp: 0,
            failures: Vec::new(),
        }
    }

    pub(crate) fn print(&self) {
        println!(
            "case={} kind={} nacon={} nccd={} bit_equal={}/{} max_dist_ulp={} max_pos_ulp={} max_frame_ulp={} failures={}",
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

fn check_exact<T: PartialEq + Copy + std::fmt::Display>(
    report: &mut CaseReport,
    field: &str,
    device: &[T],
    reference: &[T],
) {
    if device.len() != reference.len() {
        report.failures.push(format!(
            "{field} 长度不符：dev={} ref={}",
            device.len(),
            reference.len()
        ));
        return;
    }
    for (index, (&dev, &refv)) in device.iter().zip(reference).enumerate() {
        report.total += 1;
        if dev == refv {
            report.bit_equal += 1;
        } else {
            report
                .failures
                .push(format!("{field}[{index}] 位级不等 dev={dev} ref={refv}"));
        }
    }
}

fn check_dist(report: &mut CaseReport, device: &[f32], reference: &[f32]) {
    if device.len() != reference.len() {
        report.failures.push(format!(
            "dist 长度不符：dev={} ref={}",
            device.len(),
            reference.len()
        ));
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
        let ok = if refv == 0.0 {
            (dev - refv).abs() <= FOUR_ULP_AT_ONE
        } else {
            ulp <= 4
        };
        if !ok {
            report.failures.push(format!(
                "dist[{index}] 超过 4 ULP dev={dev:e} ref={refv:e} ulp={ulp}"
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
        report.failures.push(format!(
            "{field} 长度不符：dev={} ref={}",
            device.len(),
            reference.len()
        ));
        return;
    }
    for (row, expected) in reference.chunks_exact(width).enumerate() {
        let actual = &device[row * width..(row + 1) * width];
        let max = expected
            .iter()
            .fold(0.0_f32, |scale, value| scale.max(value.abs()));
        let scale = if unit_scale || max == 0.0 { 1.0 } else { max };
        let bound = 4.0 * ulp_at(scale);
        for (component, (&dev, &refv)) in actual.iter().zip(expected).enumerate() {
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
            if (dev - refv).abs() > bound {
                report.failures.push(format!(
                    "{field}[{row}][{component}] |dev-ref|={:e} > bound={bound:e}（scale={scale:e}, ulp={ulp}）",
                    (dev - refv).abs()
                ));
            }
        }
    }
}

fn permute<T: Copy>(values: &[T], order: &[usize], width: usize) -> Vec<T> {
    let mut output = Vec::with_capacity(order.len() * width);
    for &row in order {
        output.extend_from_slice(&values[row * width..(row + 1) * width]);
    }
    output
}

/// 按 `(worldid, geomcollisionid)` 稳定排序的行序。
fn row_order(worldid: &[i32], geomcollisionid: &[i32]) -> Vec<usize> {
    let rows = worldid.len().min(geomcollisionid.len());
    let mut order: Vec<usize> = (0..rows).collect();
    order.sort_by_key(|&index| (worldid[index], geomcollisionid[index]));
    order
}

pub(crate) fn compare_case(
    case: &str,
    kind: &str,
    case_reference: &Value,
    output: &CollisionGpuOutput,
) -> CaseReport {
    let mut report = CaseReport::new(case, kind, output);
    let count = case_reference["count"].as_u64().expect("QA count") as i32;
    let written = case_reference["written"].as_u64().expect("QA written") as usize;
    if output.nacon != count {
        report
            .failures
            .push(format!("nacon dev={} ref={count}", output.nacon));
    }
    // 上游参考对本片 3 个 case 的 `nccd` 为 null（QA 参考未捕获）；设备 `ccd` 路径会申请
    // naccd 槽，故 `nccd` 只记录、不入判据（与 G5 口径一致）。
    let reference_overflow: Vec<u32> = case_reference["overflow"]
        .as_array()
        .expect("QA overflow")
        .iter()
        .map(|value| value.as_u64().expect("overflow 分量") as u32)
        .collect();
    check_exact(
        &mut report,
        "overflow",
        &output.overflow,
        &reference_overflow,
    );

    let reference_dist = json_f32(case_reference, "dist");
    let reference_pos = json_f32(case_reference, "pos");
    let reference_frame = json_f32(case_reference, "frame");
    if reference_dist.len() != written
        || reference_pos.len() != written * 3
        || reference_frame.len() != written * 9
    {
        report.failures.push(format!(
            "参考行宽不符：written={written} dist={} pos={} frame={}",
            reference_dist.len(),
            reference_pos.len(),
            reference_frame.len()
        ));
        return report;
    }
    if output.nacon as usize != written {
        report.failures.push(format!(
            "设备接触行数与参考 written 不同：dev={} written={written}",
            output.nacon
        ));
        return report;
    }

    let reference_order = row_order(
        &json_i32(case_reference, "worldid"),
        &json_i32(case_reference, "geomcollisionid"),
    );
    let device_order = row_order(&output.worldid, &output.geomcollisionid);

    check_dist(
        &mut report,
        &permute(&output.dist, &device_order, 1),
        &permute(&reference_dist, &reference_order, 1),
    );
    check_vectors(
        &mut report,
        "pos",
        &permute(&output.pos, &device_order, 3),
        &permute(&reference_pos, &reference_order, 3),
        3,
        false,
    );
    check_vectors(
        &mut report,
        "frame",
        &permute(&output.frame, &device_order, 9),
        &permute(&reference_frame, &reference_order, 9),
        9,
        true,
    );
    macro_rules! exact_f32 {
        ($($field:ident : $width:expr),* $(,)?) => {
            $(
                let device = permute(&output.$field, &device_order, $width);
                let reference = permute(&json_f32(case_reference, stringify!($field)), &reference_order, $width);
                check_exact(&mut report, stringify!($field), &bits(&device), &bits(&reference));
            )*
        };
    }
    exact_f32!(
        includemargin: 1,
        friction: 5,
        solref: 2,
        solreffriction: 2,
        solimp: 5,
        adhesion: 1,
    );
    macro_rules! exact_i32 {
        ($($field:ident : $width:expr),* $(,)?) => {
            $(
                let device = permute(&output.$field, &device_order, $width);
                let reference = permute(&json_i32(case_reference, stringify!($field)), &reference_order, $width);
                check_exact(&mut report, stringify!($field), &device, &reference);
            )*
        };
    }
    exact_i32!(dim: 1, geom: 2, worldid: 1, geomcollisionid: 1);
    check_exact(
        &mut report,
        "type",
        &permute(&output.contact_type, &device_order, 1),
        &permute(&json_i32(case_reference, "type"), &reference_order, 1),
    );

    // `efc_address`：上游 `write_contact` 只写 -1（真实编号由约束管线填），参考未捕获；
    // 这里核对产品输出初值。
    report.total += output.efc_address.len();
    report.bit_equal += output
        .efc_address
        .iter()
        .filter(|value| **value == -1)
        .count();
    for (index, &value) in output.efc_address.iter().enumerate() {
        if value != -1 {
            report.failures.push(format!(
                "efc_address[{index}] 应为 -1（上游写接触时只置 -1）dev={value}"
            ));
        }
    }
    report
}

fn bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}
