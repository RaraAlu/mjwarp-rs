//! stage-2 等价性窗口的**判据与比对**（被 `tests/stage2_equivalence_window.rs` 以 `#[path]` 引入）。
//!
//! 判据逐字沿用 G5 已冻结口径（Arch 2026-10-25 裁决，不放宽）：
//! - `dist`：按自身量级 ≤4 ULP；`host == 0` 用 4 ULP@1.0 绝对差；
//! - `pos`：`scale = max_i |host_i|`（整向量全零回落 1.0），逐分量 |dev-host| ≤ 4·ULP(scale)；
//! - `frame`：`scale = 1.0`，绝对差 ≤ 4 ULP@1.0（≈4.768e-7），不设近零带；
//! - 结构字段（世界归属、每 world 接触数、overflow）位级/计数相等；位级优先。
//!
//! 行匹配：先按 `worldid` 分组，组内按 `(dist, pos, frame, geomcollisionid)` 稳定排序后
//! 逐行（重复点按多重集）比较 —— 与 G5 探针对照的排序口径一致；行序本身不作判据。

#![allow(dead_code)]

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::host::HostRow;

pub const FOUR_ULP_AT_ONE: f32 = 4.0 * (1.0 / 8_388_608.0);

pub fn ulp_at(value: f32) -> f32 {
    f32::from_bits(value.to_bits() + 1) - value
}

fn sorted_bits(value: f32) -> i64 {
    let bits = value.to_bits();
    if bits & 0x8000_0000 != 0 {
        -i64::from(bits & 0x7fff_ffff)
    } else {
        i64::from(bits)
    }
}

pub fn ulp_distance(device: f32, host: f32) -> u64 {
    (sorted_bits(device) - sorted_bits(host)).unsigned_abs()
}

pub fn vector_scale(values: &[f32]) -> f32 {
    let scale = values
        .iter()
        .fold(0.0_f32, |scale, value| scale.max(value.abs()));
    if scale == 0.0 { 1.0 } else { scale }
}

/// 设备侧接触行（product dump 或 G5 冻结探针 `out1/`）。
#[derive(Clone, Debug)]
pub struct DeviceRows {
    pub nacon: i32,
    pub nccd: i32,
    pub overflow: Vec<u32>,
    pub rows: Vec<HostRow>,
}

impl DeviceRows {
    fn _reserved(&self) {}
}

/// 从 product dump 的 run 对象取设备行（`nacon`/`nccd`/`overflow`/`dist`/`pos`/`frame`/`worldid`）。
pub fn device_rows_from_dump(run: &Value) -> DeviceRows {
    let rows = run["nacon"].as_i64().expect("nacon") as usize;
    let dist = run["dist"].as_array().expect("dist");
    let pos = run["pos"].as_array().expect("pos");
    let frame = run["frame"].as_array().expect("frame");
    let worldid = run["worldid"].as_array().expect("worldid");
    assert_eq!(dist.len(), rows, "dump dist 行数");
    assert_eq!(pos.len(), rows * 3, "dump pos 行数");
    assert_eq!(frame.len(), rows * 9, "dump frame 行数");
    let mut out = Vec::with_capacity(rows);
    for index in 0..rows {
        let mut position = [0.0_f32; 3];
        for axis in 0..3 {
            position[axis] = pos[index * 3 + axis].as_f64().expect("pos") as f32;
        }
        let mut matrix = [0.0_f32; 9];
        for cell in 0..9 {
            matrix[cell] = frame[index * 9 + cell].as_f64().expect("frame") as f32;
        }
        out.push(HostRow {
            worldid: worldid[index].as_i64().expect("worldid") as i32,
            geomcollisionid: 0,
            dist: dist[index].as_f64().expect("dist") as f32,
            pos: position,
            frame: matrix,
        });
    }
    DeviceRows {
        nacon: rows as i32,
        nccd: run["nccd"].as_i64().unwrap_or(0) as i32,
        overflow: run["overflow"]
            .as_array()
            .map(|values| {
                values
                    .iter()
                    .map(|value| value.as_u64().expect("overflow") as u32)
                    .collect()
            })
            .unwrap_or_default(),
        rows: out,
    }
}

fn read_f32(directory: &std::path::Path, name: &str) -> Vec<f32> {
    let bytes = std::fs::read(directory.join(format!("{name}.bin")))
        .unwrap_or_else(|error| panic!("读取 {} 失败：{error}", directory.join(name).display()));
    let (chunks, tail) = bytes.as_chunks::<4>();
    assert!(tail.is_empty(), "{name}.bin 长度");
    chunks.iter().map(|c| f32::from_le_bytes(*c)).collect()
}

fn read_i32(directory: &std::path::Path, name: &str) -> Vec<i32> {
    let bytes = std::fs::read(directory.join(format!("{name}.bin")))
        .unwrap_or_else(|error| panic!("读取 {} 失败：{error}", directory.join(name).display()));
    let (chunks, tail) = bytes.as_chunks::<4>();
    assert!(tail.is_empty(), "{name}.bin 长度");
    chunks.iter().map(|c| i32::from_le_bytes(*c)).collect()
}

/// 从 G5 冻结探针 `out1/` 取设备行（CPU 侧 dry-run，不新捕获）。
pub fn device_rows_from_out1(directory: &std::path::Path) -> DeviceRows {
    let nacon = read_i32(directory, "nacon")[0];
    let nccd = read_i32(directory, "nccd")[0];
    let rows = nacon.max(0) as usize;
    let dist = read_f32(directory, "dist");
    let pos = read_f32(directory, "pos");
    let frame = read_f32(directory, "frame");
    let worldid = read_i32(directory, "worldid");
    let overflow: Vec<u32> = read_i32(directory, "overflow")
        .into_iter()
        .map(|value| value as u32)
        .collect();
    assert_eq!(dist.len(), rows, "out1 dist 行数");
    assert_eq!(pos.len(), rows * 3, "out1 pos 行数");
    assert_eq!(frame.len(), rows * 9, "out1 frame 行数");
    let mut out = Vec::with_capacity(rows);
    for index in 0..rows {
        out.push(HostRow {
            worldid: worldid[index],
            geomcollisionid: 0,
            dist: dist[index],
            pos: [pos[index * 3], pos[index * 3 + 1], pos[index * 3 + 2]],
            frame: {
                let mut matrix = [0.0_f32; 9];
                matrix.copy_from_slice(&frame[index * 9..index * 9 + 9]);
                matrix
            },
        });
    }
    DeviceRows {
        nacon,
        nccd,
        overflow,
        rows: out,
    }
}

fn sort_rows(rows: &mut [HostRow]) {
    rows.sort_by(|left, right| {
        left.worldid
            .cmp(&right.worldid)
            .then(left.dist.total_cmp(&right.dist))
            .then(left.pos[0].total_cmp(&right.pos[0]))
            .then(left.pos[1].total_cmp(&right.pos[1]))
            .then(left.pos[2].total_cmp(&right.pos[2]))
            .then(left.frame[0].total_cmp(&right.frame[0]))
            .then(left.geomcollisionid.cmp(&right.geomcollisionid))
    });
}

fn group_by_world(rows: &[HostRow]) -> BTreeMap<i32, Vec<HostRow>> {
    let mut grouped: BTreeMap<i32, Vec<HostRow>> = BTreeMap::new();
    for row in rows {
        grouped.entry(row.worldid).or_default().push(*row);
    }
    for rows in grouped.values_mut() {
        sort_rows(rows);
    }
    grouped
}

/// G5 口径的确定性检查：先按 `worldid` 分组，组内按 G5 `sort_contacts` 排序键
/// （dist → pos[0..2] → geomcollisionid）稳定排序，再逐位比较行集合。
/// **行序不作判据**（设备行序由 atomic `nacon` 计数决定，非确定性）。
pub fn deterministic(left: &DeviceRows, right: &DeviceRows) -> Result<(), String> {
    if left.nacon != right.nacon || left.nccd != right.nccd || left.overflow != right.overflow {
        return Err(format!(
            "标量不一致：nacon {} vs {}、nccd {} vs {}、overflow {:?} vs {:?}",
            left.nacon, right.nacon, left.nccd, right.nccd, left.overflow, right.overflow
        ));
    }
    let sort = |rows: &mut [HostRow]| {
        rows.sort_by(|l, r| {
            l.worldid
                .cmp(&r.worldid)
                .then(l.dist.total_cmp(&r.dist))
                .then(l.pos[0].total_cmp(&r.pos[0]))
                .then(l.pos[1].total_cmp(&r.pos[1]))
                .then(l.pos[2].total_cmp(&r.pos[2]))
                .then(l.geomcollisionid.cmp(&r.geomcollisionid))
        });
    };
    let mut a = left.rows.clone();
    let mut b = right.rows.clone();
    sort(&mut a);
    sort(&mut b);
    for (index, (x, y)) in a.iter().zip(&b).enumerate() {
        let equal = x.worldid == y.worldid
            && x.dist.to_bits() == y.dist.to_bits()
            && x.geomcollisionid == y.geomcollisionid
            && (0..3).all(|axis| x.pos[axis].to_bits() == y.pos[axis].to_bits())
            && (0..9).all(|cell| x.frame[cell].to_bits() == y.frame[cell].to_bits());
        if !equal {
            return Err(format!(
                "排序后第 {index} 行不一致（worldid={} vs {}）",
                x.worldid, y.worldid
            ));
        }
    }
    Ok(())
}
/// 单个 (case, kernel/口径) 的等价性报告。
pub struct Report {
    pub case: String,
    pub kernel: String,
    pub device_rows: usize,
    pub host_rows: usize,
    pub bit_equal: usize,
    pub total: usize,
    pub max_dist_ulp: u64,
    pub max_pos_ulp: f64,
    pub max_frame_abs: f32,
    pub overflow_equal: bool,
    pub failures: Vec<String>,
}

impl Report {
    pub fn to_json(&self) -> Value {
        json!({
            "case": self.case,
            "kernel": self.kernel,
            "device_rows": self.device_rows,
            "host_rows": self.host_rows,
            "bit_equal": self.bit_equal,
            "total": self.total,
            "max_dist_ulp": self.max_dist_ulp,
            "max_pos_ulp": self.max_pos_ulp,
            "max_frame_abs": self.max_frame_abs,
            "overflow_equal": self.overflow_equal,
            "failures": self.failures,
            "verified": self.failures.is_empty(),
        })
    }

    pub fn print(&self) {
        println!(
            "case={} kernel={} dev_rows={} host_rows={} bit_equal={}/{} max_dist_ulp={} max_pos_ulp={:.3} max_frame_abs={:.3e} overflow_equal={} failures={}",
            self.case,
            self.kernel,
            self.device_rows,
            self.host_rows,
            self.bit_equal,
            self.total,
            self.max_dist_ulp,
            self.max_pos_ulp,
            self.max_frame_abs,
            self.overflow_equal,
            self.failures.len()
        );
        for failure in &self.failures {
            println!("FAIL {} {}: {failure}", self.case, self.kernel);
        }
    }
}

/// 单一接触行逐字段判据（G5 冻结口径）。
fn compare_row(report: &mut Report, world: i32, device: &HostRow, host: &HostRow) {
    // dist：位级优先，否则自身量级 ≤4 ULP（host==0 用 4 ULP@1.0）。
    report.total += 1;
    if device.dist.to_bits() == host.dist.to_bits() {
        report.bit_equal += 1;
    } else {
        let ulp = ulp_distance(device.dist, host.dist);
        report.max_dist_ulp = report.max_dist_ulp.max(ulp);
        let ok = if host.dist == 0.0 {
            (device.dist - host.dist).abs() <= FOUR_ULP_AT_ONE
        } else {
            ulp <= 4
        };
        if !ok {
            report.failures.push(format!(
                "world {world} dist 超 4 ULP：dev={:e} host={:e} ulp={ulp}",
                device.dist, host.dist
            ));
        }
    }
    // pos：向量尺度 4·ULP(scale)。
    let scale = vector_scale(&host.pos);
    let bound = 4.0 * ulp_at(scale);
    for axis in 0..3 {
        report.total += 1;
        if device.pos[axis].to_bits() == host.pos[axis].to_bits() {
            report.bit_equal += 1;
        } else {
            let difference = (device.pos[axis] - host.pos[axis]).abs();
            report.max_pos_ulp = report
                .max_pos_ulp
                .max(f64::from(difference) / f64::from(bound));
            if difference > bound {
                report.failures.push(format!(
                    "world {world} pos[{axis}] |dev-host|={difference:e} > 4·ULP(scale={scale:e})={bound:e}"
                ));
            }
        }
    }
    // frame：scale=1.0 的 4 ULP 绝对差。
    let frame_bound = FOUR_ULP_AT_ONE;
    for cell in 0..9 {
        report.total += 1;
        if device.frame[cell].to_bits() == host.frame[cell].to_bits() {
            report.bit_equal += 1;
        } else {
            let difference = (device.frame[cell] - host.frame[cell]).abs();
            report.max_frame_abs = report.max_frame_abs.max(difference);
            if difference > frame_bound {
                report.failures.push(format!(
                    "world {world} frame[{cell}] |dev-host|={difference:e} > 4 ULP@1.0={frame_bound:e}"
                ));
            }
        }
    }
}

/// host vs device 逐字段等价比较。`device` 的 `overflow` 空表示不比对（out1 由调用方处理）。
pub fn compare(
    case: &str,
    kernel: &str,
    device: &DeviceRows,
    host_rows: &[HostRow],
    host_overflow: &[u32],
) -> Report {
    let mut report = Report {
        case: case.to_owned(),
        kernel: kernel.to_owned(),
        device_rows: device.rows.len(),
        host_rows: host_rows.len(),
        bit_equal: 0,
        total: 0,
        max_dist_ulp: 0,
        max_pos_ulp: 0.0,
        max_frame_abs: 0.0,
        overflow_equal: true,
        failures: Vec::new(),
    };
    if device.nacon as usize != device.rows.len() {
        report.failures.push(format!(
            "设备 nacon={} 与行数 {} 不一致",
            device.nacon,
            device.rows.len()
        ));
    }
    if !device.overflow.is_empty() {
        if device.overflow.len() != host_overflow.len() {
            report.overflow_equal = false;
            report.failures.push(format!(
                "overflow 长度 dev={} host={}",
                device.overflow.len(),
                host_overflow.len()
            ));
        } else {
            for (world, (dev, host)) in device.overflow.iter().zip(host_overflow).enumerate() {
                if dev != host {
                    report.overflow_equal = false;
                    report
                        .failures
                        .push(format!("world {world} overflow dev={dev} host={host}"));
                }
            }
        }
    }
    let device_grouped = group_by_world(&device.rows);
    let host_grouped = group_by_world(host_rows);
    let worlds: Vec<i32> = device_grouped
        .keys()
        .chain(host_grouped.keys())
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    if device_grouped.len() != host_grouped.len() {
        report.failures.push(format!(
            "世界集合不一致 dev={:?} host={:?}",
            device_grouped.keys().collect::<Vec<_>>(),
            host_grouped.keys().collect::<Vec<_>>()
        ));
    }
    for world in worlds {
        let empty = Vec::new();
        let device_world = device_grouped.get(&world).unwrap_or(&empty);
        let host_world = host_grouped.get(&world).unwrap_or(&empty);
        if device_world.len() != host_world.len() {
            report.failures.push(format!(
                "world {world} 接触数 dev={} host={}",
                device_world.len(),
                host_world.len()
            ));
        }
        for (device, host) in device_world.iter().zip(host_world) {
            compare_row(&mut report, world, device, host);
        }
    }
    // 行数守恒：配对行数必须等于两侧行数（否则说明上面的计数失败未覆盖）。
    let paired: usize = device_grouped
        .values()
        .zip(host_grouped.values())
        .map(|(device, host)| device.len().min(host.len()))
        .sum();
    if paired != device.rows.len().min(host_rows.len()) {
        report
            .failures
            .push("世界分组后配对行数少于两侧最小值".to_owned());
    }
    report
}
