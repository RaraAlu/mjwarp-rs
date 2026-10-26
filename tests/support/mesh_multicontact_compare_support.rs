//! G10@P5 M2：convex-vs-mesh multicontact 的 GPU 对照（M1 上游 GPU 参考口径）。
//!
//! 参考：`docs/development-evidence/mesh-multicontact-m1/upstream-mesh-multicontact-gpu.json`
//! （`cases.<case>.output.fields.<name>` 形状为 `[count][width]`；`worldid/type/dim/geom/
//! geomcollisionid/includemargin/adhesion/friction/solref/solreffriction/solimp` 齐全，
//! **未记录 `efc_address`**）。
//!
//! 判据（沿用 G5/B1 口径，不放宽）：
//! - `nacon` 与参考 `count` 零容差；行配对按结构键 `(worldid, geomcollisionid)` 排序后逐行比；
//! - 位级优先；非位级时 `dist` 4 ULP@自身量级（`ref==0` → 4 ULP@1.0）、
//!   `pos` 4·ULP(参考三轴最大绝对值)、`frame` 4 ULP@1.0；
//! - `friction/solref/solreffriction/solimp/includemargin/adhesion/worldid/type/dim/geom/
//!   geomcollisionid` 零容差；
//! - `overflow` 全 0；`efc_address` 参考未记录 → 设备侧全 `-1` 不变量。
//! - `contact.geom` 顺序按 pinned 类型序（box 6 < mesh 7 → `[1,0]`，由调用方按规范化顺序
//!   传入；device 不做归一化）——本模块单独断言该顺序。
//!
//! 本模块只读参考 JSON 与设备回读结果，不写任何文件。

use mjwarp_rs::runtime::collision_gpu::CollisionGpuOutput;
use serde_json::Value;

/// `4 ULP @ 1.0`（f32）：2^-23 * 4。
const FOUR_ULP_AT_ONE: f32 = 4.0 * (1.0 / 8_388_608.0);

/// 一行参考接触（字段宽度与 pinned 一致）。
#[derive(Clone, Debug)]
pub(crate) struct ReferenceRow {
    pub dist: f32,
    pub pos: [f32; 3],
    pub frame: [f32; 9],
    pub includemargin: f32,
    pub friction: [f32; 5],
    pub solref: [f32; 2],
    pub solreffriction: [f32; 2],
    pub solimp: [f32; 5],
    pub adhesion: f32,
    pub dim: i32,
    pub geom: [i32; 2],
    pub worldid: i32,
    pub contact_type: i32,
    pub geomcollisionid: i32,
}

fn field_rows(fields: &Value, name: &str) -> Vec<Vec<f64>> {
    fields[name]
        .as_array()
        .unwrap_or_else(|| panic!("参考缺少 fields.{name}"))
        .iter()
        .map(|row| {
            row.as_array()
                .unwrap_or_else(|| panic!("fields.{name} 行不是数组"))
                .iter()
                .map(|value| value.as_f64().expect("字段分量非数值"))
                .collect()
        })
        .collect()
}

fn flat_f32(fields: &Value, name: &str, width: usize) -> Vec<f32> {
    field_rows(fields, name)
        .into_iter()
        .flat_map(|row| {
            assert_eq!(row.len(), width, "fields.{name} 行宽应为 {width}");
            row.into_iter()
                .map(|value| value as f32)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn flat_i32(fields: &Value, name: &str, width: usize) -> Vec<i32> {
    field_rows(fields, name)
        .into_iter()
        .flat_map(|row| {
            assert_eq!(row.len(), width, "fields.{name} 行宽应为 {width}");
            row.into_iter()
                .map(|value| value as i32)
                .collect::<Vec<_>>()
        })
        .collect()
}

/// 参考某个 case 的接触行数（`output.count`）。
pub(crate) fn reference_count(case_reference: &Value) -> usize {
    case_reference["output"]["count"]
        .as_u64()
        .unwrap_or_else(|| panic!("参考缺少 output.count")) as usize
}

/// 参考行按 `(worldid, geomcollisionid)` 排序后的列表。
pub(crate) fn reference_rows(case_reference: &Value) -> Vec<ReferenceRow> {
    let fields = &case_reference["output"]["fields"];
    let count = reference_count(case_reference);
    let dist = flat_f32(fields, "dist", 1);
    let pos = flat_f32(fields, "pos", 3);
    let frame = flat_f32(fields, "frame", 9);
    let includemargin = flat_f32(fields, "includemargin", 1);
    let friction = flat_f32(fields, "friction", 5);
    let solref = flat_f32(fields, "solref", 2);
    let solreffriction = flat_f32(fields, "solreffriction", 2);
    let solimp = flat_f32(fields, "solimp", 5);
    let adhesion = flat_f32(fields, "adhesion", 1);
    let dim = flat_i32(fields, "dim", 1);
    let geom = flat_i32(fields, "geom", 2);
    let worldid = flat_i32(fields, "worldid", 1);
    let contact_type = flat_i32(fields, "type", 1);
    let geomcollisionid = flat_i32(fields, "geomcollisionid", 1);

    let mut rows = Vec::with_capacity(count);
    for index in 0..count {
        rows.push(ReferenceRow {
            dist: dist[index],
            pos: [pos[index * 3], pos[index * 3 + 1], pos[index * 3 + 2]],
            frame: std::array::from_fn(|k| frame[index * 9 + k]),
            includemargin: includemargin[index],
            friction: std::array::from_fn(|k| friction[index * 5 + k]),
            solref: [solref[index * 2], solref[index * 2 + 1]],
            solreffriction: [solreffriction[index * 2], solreffriction[index * 2 + 1]],
            solimp: std::array::from_fn(|k| solimp[index * 5 + k]),
            adhesion: adhesion[index],
            dim: dim[index],
            geom: [geom[index * 2], geom[index * 2 + 1]],
            worldid: worldid[index],
            contact_type: contact_type[index],
            geomcollisionid: geomcollisionid[index],
        });
    }
    rows.sort_by_key(|row| (row.worldid, row.geomcollisionid));
    rows
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

fn four_ulp_at(value: f32) -> f32 {
    4.0 * ulp_at(value.abs().max(f32::MIN_POSITIVE))
}

fn check_dist(device: f32, reference: f32) -> Result<(), String> {
    let ok = if reference == 0.0 {
        (device - reference).abs() <= FOUR_ULP_AT_ONE
    } else {
        ulp_distance(device, reference) <= 4
    };
    if ok {
        Ok(())
    } else {
        Err(format!(
            "dist dev={device:e} ref={reference:e} ulp={}",
            ulp_distance(device, reference)
        ))
    }
}

fn check_pos(device: &[f32], reference: &[f32]) -> Result<(), String> {
    let scale = reference
        .iter()
        .fold(0.0_f32, |acc, value| acc.max(value.abs()));
    for index in 0..3 {
        let tolerance = four_ulp_at(scale);
        if (device[index] - reference[index]).abs() > tolerance {
            return Err(format!(
                "pos[{index}] dev={:e} ref={:e} (scale={scale:e})",
                device[index], reference[index]
            ));
        }
    }
    Ok(())
}

fn check_frame(device: &[f32], reference: &[f32]) -> Result<(), String> {
    for index in 0..9 {
        let ok = if reference[index] == 0.0 {
            device[index].abs() <= FOUR_ULP_AT_ONE
        } else {
            ulp_distance(device[index], reference[index]) <= 4
        };
        if !ok {
            return Err(format!(
                "frame[{index}] dev={:e} ref={:e} ulp={}",
                device[index],
                reference[index],
                ulp_distance(device[index], reference[index])
            ));
        }
    }
    Ok(())
}

fn check_exact_f32(field: &str, device: &[f32], reference: &[f32]) -> Result<(), String> {
    for (index, (&dev, &refv)) in device.iter().zip(reference).enumerate() {
        if dev.to_bits() != refv.to_bits() {
            return Err(format!(
                "{field}[{index}] 位级不等 dev={dev:e} ref={refv:e}"
            ));
        }
    }
    Ok(())
}

fn check_exact_i32(field: &str, device: &[i32], reference: &[i32]) -> Result<(), String> {
    for (index, (&dev, &refv)) in device.iter().zip(reference).enumerate() {
        if dev != refv {
            return Err(format!("{field}[{index}] 不等 dev={dev} ref={refv}"));
        }
    }
    Ok(())
}

/// 设备回读结果（按 `(worldid, geomcollisionid)` 排序后的逐行视图）与参考的比较报告。
pub(crate) struct MeshCaseReport {
    pub case: String,
    pub nacon: i32,
    pub reference_count: usize,
    pub bit_equal: usize,
    pub total: usize,
    pub max_dist_ulp: u64,
    pub max_pos_ulp: u64,
    pub max_frame_ulp: u64,
    pub geom_order: String,
    pub failures: Vec<String>,
}

impl MeshCaseReport {
    pub fn print(&self) {
        println!(
            "case={} nacon={} ref_count={} bit_equal={}/{} max_dist_ulp={} max_pos_ulp={} max_frame_ulp={} contact.geom={} failures={}",
            self.case,
            self.nacon,
            self.reference_count,
            self.bit_equal,
            self.total,
            self.max_dist_ulp,
            self.max_pos_ulp,
            self.max_frame_ulp,
            self.geom_order,
            self.failures.len()
        );
        for failure in &self.failures {
            println!("FAIL {}: {failure}", self.case);
        }
    }
}

/// 用 M1 参考对照一次 mesh multicontact 设备输出（M2 口径：期望 `contact.geom=[1,0]`）。
/// M2 目标使用本入口；M3 目标只用 [`compare_case_with_geom_order`]（共享 support 模块，
/// 单个测试目标内允许未使用）。
#[allow(dead_code)]
pub(crate) fn compare_case(
    case: &str,
    case_reference: &Value,
    output: &CollisionGpuOutput,
) -> MeshCaseReport {
    compare_case_with_geom_order(case, case_reference, output, "[1,0]")
}

/// 同 [`compare_case`]，但 `contact.geom` 的期望顺序由调用方给出：M2 的 (BOX/CYLINDER, MESH)
/// 按 pinned 类型序为 `[1,0]`；M3 的 (MESH, MESH) 两侧同类型，调用方按 geom 序传 `[0,1]`。
pub(crate) fn compare_case_with_geom_order(
    case: &str,
    case_reference: &Value,
    output: &CollisionGpuOutput,
    expected_geom_order: &str,
) -> MeshCaseReport {
    let reference = reference_rows(case_reference);
    let mut report = MeshCaseReport {
        case: case.to_owned(),
        nacon: output.nacon,
        reference_count: reference.len(),
        bit_equal: 0,
        total: 0,
        max_dist_ulp: 0,
        max_pos_ulp: 0,
        max_frame_ulp: 0,
        geom_order: String::new(),
        failures: Vec::new(),
    };

    // 计数零容差：nacon 必须等于参考 count（每对最多 4 个接触）。
    if output.nacon as usize != reference.len() {
        report.failures.push(format!(
            "nacon dev={} ref={}",
            output.nacon,
            reference.len()
        ));
        return report;
    }
    let count = reference.len();
    let row_field = |name: &str| -> Vec<f32> {
        let field: &Vec<f32> = match name {
            "dist" => &output.dist,
            "pos" => &output.pos,
            "frame" => &output.frame,
            "includemargin" => &output.includemargin,
            "friction" => &output.friction,
            "solref" => &output.solref,
            "solreffriction" => &output.solreffriction,
            "solimp" => &output.solimp,
            "adhesion" => &output.adhesion,
            other => panic!("未知输出字段 {other}"),
        };
        field.clone()
    };

    // 设备行按结构键排序（参考已排序；两者都用 (worldid, geomcollisionid)）。
    let mut order: Vec<usize> = (0..count).collect();
    order.sort_by_key(|&index| (output.worldid[index], output.geomcollisionid[index]));

    for (row_index, &device_index) in order.iter().enumerate() {
        let reference_row = &reference[row_index];
        report.total += 1;

        // 位级优先：统计三个浮点接触字段的全位级行数（全字段位级才算一行）。
        let bit_row = {
            let dist = row_field("dist")[device_index].to_bits() == reference_row.dist.to_bits();
            let pos = (0..3).all(|k| {
                row_field("pos")[device_index * 3 + k].to_bits() == reference_row.pos[k].to_bits()
            });
            let frame = (0..9).all(|k| {
                row_field("frame")[device_index * 9 + k].to_bits()
                    == reference_row.frame[k].to_bits()
            });
            dist && pos && frame
        };
        if bit_row {
            report.bit_equal += 1;
        }

        // dist / pos / frame（位级优先，否则 4 ULP 口径）。
        let dist = row_field("dist")[device_index];
        let dist_ulp = ulp_distance(dist, reference_row.dist);
        report.max_dist_ulp = report.max_dist_ulp.max(dist_ulp);
        if let Err(error) = check_dist(dist, reference_row.dist) {
            report.failures.push(error);
        }
        let pos: Vec<f32> = (0..3)
            .map(|k| row_field("pos")[device_index * 3 + k])
            .collect();
        let pos_ulp = pos
            .iter()
            .zip(reference_row.pos.iter())
            .map(|(&dev, &refv)| ulp_distance(dev, refv))
            .max()
            .unwrap_or(0);
        report.max_pos_ulp = report.max_pos_ulp.max(pos_ulp);
        if let Err(error) = check_pos(&pos, &reference_row.pos) {
            report.failures.push(error);
        }
        let frame: Vec<f32> = (0..9)
            .map(|k| row_field("frame")[device_index * 9 + k])
            .collect();
        let frame_ulp = frame
            .iter()
            .zip(reference_row.frame.iter())
            .map(|(&dev, &refv)| ulp_distance(dev, refv))
            .max()
            .unwrap_or(0);
        report.max_frame_ulp = report.max_frame_ulp.max(frame_ulp);
        if let Err(error) = check_frame(&frame, &reference_row.frame) {
            report.failures.push(error);
        }

        // 结构/参数零容差。
        for (name, reference_values, device_values) in [
            (
                "includemargin",
                reference_row.includemargin.to_bits(),
                row_field("includemargin")[device_index].to_bits(),
            ),
            (
                "adhesion",
                reference_row.adhesion.to_bits(),
                row_field("adhesion")[device_index].to_bits(),
            ),
        ] {
            if reference_values != device_values {
                report.failures.push(format!(
                    "{name} 位级不等 dev={:e}",
                    f32::from_bits(device_values)
                ));
            }
        }
        let friction: Vec<f32> = (0..5)
            .map(|k| row_field("friction")[device_index * 5 + k])
            .collect();
        if let Err(error) = check_exact_f32("friction", &friction, &reference_row.friction) {
            report.failures.push(error);
        }
        let solref: Vec<f32> = (0..2)
            .map(|k| row_field("solref")[device_index * 2 + k])
            .collect();
        if let Err(error) = check_exact_f32("solref", &solref, &reference_row.solref) {
            report.failures.push(error);
        }
        let solref_fric: Vec<f32> = (0..2)
            .map(|k| row_field("solreffriction")[device_index * 2 + k])
            .collect();
        if let Err(error) = check_exact_f32(
            "solreffriction",
            &solref_fric,
            &reference_row.solreffriction,
        ) {
            report.failures.push(error);
        }
        let solimp: Vec<f32> = (0..5)
            .map(|k| row_field("solimp")[device_index * 5 + k])
            .collect();
        if let Err(error) = check_exact_f32("solimp", &solimp, &reference_row.solimp) {
            report.failures.push(error);
        }
        let device_ints = [
            (
                "worldid",
                output.worldid[device_index],
                reference_row.worldid,
            ),
            (
                "type",
                output.contact_type[device_index],
                reference_row.contact_type,
            ),
            ("dim", output.dim[device_index], reference_row.dim),
            (
                "geom[0]",
                output.geom[device_index * 2],
                reference_row.geom[0],
            ),
            (
                "geom[1]",
                output.geom[device_index * 2 + 1],
                reference_row.geom[1],
            ),
            (
                "geomcollisionid",
                output.geomcollisionid[device_index],
                reference_row.geomcollisionid,
            ),
        ];
        for (name, device, reference) in device_ints {
            if let Err(error) = check_exact_i32(name, &[device], &[reference]) {
                report.failures.push(error);
            }
        }

        if row_index == 0 {
            report.geom_order = format!(
                "[{},{}]",
                output.geom[device_index * 2],
                output.geom[device_index * 2 + 1]
            );
            // 该顺序由调用方按 pinned 类型序传入（M2: box 6 < mesh 7 → [1,0]；M3: 两侧 mesh
            // 同类型 → [0,1]；device 不做归一化）——本模块单独断言该顺序。
            if report.geom_order != expected_geom_order {
                report.failures.push(format!(
                    "contact.geom 顺序应为 pinned 类型序 {expected_geom_order}（调用方传入），实际 {}",
                    report.geom_order
                ));
            }
        }
    }

    // 设备侧不变量：overflow 全 0；efc_address 参考未记录 → 全 -1。
    if output.overflow.iter().any(|&bits| bits != 0) {
        report
            .failures
            .push(format!("overflow 非 0：{:?}", output.overflow));
    }
    if !output.efc_address.is_empty() {
        for index in 0..output.efc_address.len() {
            if output.efc_address[index] != -1 {
                report
                    .failures
                    .push(format!("efc_address[{index}] 应为 -1（参考未记录 efc）"));
                break;
            }
        }
    }
    report
}
