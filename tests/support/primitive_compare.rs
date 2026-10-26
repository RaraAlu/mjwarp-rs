//! B3 分支覆盖测试的参考/设备逐字段对照支持（被 `g10_p5_primitive_branches_cuda.rs` 以
//! `#[path]` 引入；不作为独立测试目标）。判据与 B1/B2 同口径，未放宽。

use mjwarp_rs::runtime::collision_gpu::CollisionGpuOutput;

pub(crate) const FOUR_ULP_AT_ONE: f32 = 4.0 * (1.0 / 8_388_608.0);

pub(crate) fn ulp_distance(device: f32, reference: f32) -> u64 {
    let key = |value: f32| -> i64 {
        let bits = value.to_bits();
        if bits & 0x8000_0000 != 0 {
            -i64::from(bits & 0x7fff_ffff)
        } else {
            i64::from(bits)
        }
    };
    (key(device) - key(reference)).unsigned_abs()
}

pub(crate) fn vector_scale(values: &[f32]) -> f32 {
    let scale = values.iter().fold(0.0_f32, |scale, v| scale.max(v.abs()));
    if scale == 0.0 { 1.0 } else { scale }
}

pub(crate) struct Row {
    pub id: i32,
    pub worldid: i32,
    pub contact_type: i32,
    pub dim: i32,
    pub geom: [i32; 2],
    pub dist: f32,
    pub pos: [f32; 3],
    pub frame: [f32; 9],
    pub includemargin: f32,
    pub adhesion: f32,
    pub friction: [f32; 5],
    pub solref: [f32; 2],
    pub solreffriction: [f32; 2],
    pub solimp: [f32; 5],
}

pub(crate) fn flatten_f32(value: &serde_json::Value) -> Vec<f32> {
    value
        .as_array()
        .expect("fields")
        .iter()
        .flat_map(|row| row.as_array().expect("row"))
        .map(|cell| cell.as_f64().expect("f32") as f32)
        .collect()
}

pub(crate) fn flatten_i32(value: &serde_json::Value) -> Vec<i32> {
    value
        .as_array()
        .expect("fields")
        .iter()
        .flat_map(|row| row.as_array().expect("row"))
        .map(|cell| cell.as_i64().expect("int") as i32)
        .collect()
}

fn reference_rows(expected: &serde_json::Value, count: usize) -> Vec<Row> {
    let f = |name: &str| flatten_f32(&expected["fields"][name]);
    let i = |name: &str| flatten_i32(&expected["fields"][name]);
    let (dist, pos, frame) = (f("dist"), f("pos"), f("frame"));
    let (margin, adhesion) = (f("includemargin"), f("adhesion"));
    let (friction, solref) = (f("friction"), f("solref"));
    let (solreffr, solimp) = (f("solreffriction"), f("solimp"));
    let (geom, dim, worldid, ctype, ids) = (
        i("geom"),
        i("dim"),
        i("worldid"),
        i("type"),
        i("geomcollisionid"),
    );
    (0..count)
        .map(|index| {
            let mut frame_row = [0.0_f32; 9];
            frame_row.copy_from_slice(&frame[index * 9..index * 9 + 9]);
            let five = |values: &[f32]| {
                [
                    values[index * 5],
                    values[index * 5 + 1],
                    values[index * 5 + 2],
                    values[index * 5 + 3],
                    values[index * 5 + 4],
                ]
            };
            Row {
                id: ids[index],
                worldid: worldid[index],
                contact_type: ctype[index],
                dim: dim[index],
                geom: [geom[index * 2], geom[index * 2 + 1]],
                dist: dist[index],
                pos: [pos[index * 3], pos[index * 3 + 1], pos[index * 3 + 2]],
                frame: frame_row,
                includemargin: margin[index],
                adhesion: adhesion[index],
                friction: five(&friction),
                solref: [solref[index * 2], solref[index * 2 + 1]],
                solreffriction: [solreffr[index * 2], solreffr[index * 2 + 1]],
                solimp: five(&solimp),
            }
        })
        .collect()
}

/// 参考侧解析结果（`count` + 逐行 `Row`）；CPU 可独立调用以做窗口前支持层自检。
pub(crate) struct Expectation {
    pub count: usize,
    pub rows: Vec<Row>,
}

/// 解析 `reference["cases"][name]["output"]`：`count` 与 `fields[name]` 的 `[count][width]`。
pub(crate) fn reference_expectation(name: &str, reference: &serde_json::Value) -> Expectation {
    let expected = &reference["cases"][name]["output"];
    let count = expected["count"].as_u64().expect("count") as usize;
    Expectation {
        count,
        rows: reference_rows(expected, count),
    }
}

fn device_rows(output: &CollisionGpuOutput) -> Vec<Row> {
    (0..output.nacon.max(0) as usize)
        .map(|index| {
            let mut frame = [0.0_f32; 9];
            frame.copy_from_slice(&output.frame[index * 9..index * 9 + 9]);
            let five = |values: &[f32]| {
                [
                    values[index * 5],
                    values[index * 5 + 1],
                    values[index * 5 + 2],
                    values[index * 5 + 3],
                    values[index * 5 + 4],
                ]
            };
            Row {
                id: output.geomcollisionid[index],
                worldid: output.worldid[index],
                contact_type: output.contact_type[index],
                dim: output.dim[index],
                geom: [output.geom[index * 2], output.geom[index * 2 + 1]],
                dist: output.dist[index],
                pos: [
                    output.pos[index * 3],
                    output.pos[index * 3 + 1],
                    output.pos[index * 3 + 2],
                ],
                frame,
                includemargin: output.includemargin[index],
                adhesion: output.adhesion[index],
                friction: five(&output.friction),
                solref: [output.solref[index * 2], output.solref[index * 2 + 1]],
                solreffriction: [
                    output.solreffriction[index * 2],
                    output.solreffriction[index * 2 + 1],
                ],
                solimp: five(&output.solimp),
            }
        })
        .collect()
}

/// 逐 case 结构零容差 + 浮点判据（位级优先；非位级 dist 4 ULP@自身量级、pos 4·ULP(scale)、
/// frame 4 ULP@1.0；efc/overflow 设备侧不变量；行序按 `geomcollisionid` 配对）。
pub(crate) fn compare(
    name: &str,
    reference: &serde_json::Value,
    output: &CollisionGpuOutput,
) -> Vec<String> {
    let mut failures = Vec::new();
    let expectation = reference_expectation(name, reference);
    let count = expectation.count;
    let rows = output.nacon.max(0) as usize;
    if rows != count {
        failures.push(format!("contact 数 dev={rows} ref={count}（零容差）"));
    }
    if output.efc_address.iter().any(|value| *value != -1) {
        failures.push("efc_address 存在非 -1（设备侧不变量）".to_string());
    }
    if output.overflow.iter().any(|value| *value != 0) {
        failures.push(format!(
            "overflow 非 0（设备侧不变量）：{:?}",
            output.overflow
        ));
    }
    let mut device_rows = device_rows(output);
    let mut reference_rows = expectation.rows;
    device_rows.sort_by_key(|row| row.id);
    reference_rows.sort_by_key(|row| row.id);
    let device_ids: Vec<i32> = device_rows.iter().map(|row| row.id).collect();
    let reference_ids: Vec<i32> = reference_rows.iter().map(|row| row.id).collect();
    if device_ids != reference_ids {
        failures.push(format!(
            "geomcollisionid 集合不一致 dev={device_ids:?} ref={reference_ids:?}（零容差）"
        ));
    }
    for (device, reference) in device_rows.iter().zip(&reference_rows) {
        let id = reference.id;
        if device.worldid != reference.worldid
            || device.contact_type != reference.contact_type
            || device.dim != reference.dim
            || device.geom != reference.geom
            || device.includemargin != reference.includemargin
            || device.adhesion != reference.adhesion
            || device.friction != reference.friction
            || device.solref != reference.solref
            || device.solreffriction != reference.solreffriction
            || device.solimp != reference.solimp
        {
            failures.push(format!("id={id} 结构/接触参数非逐位相同（零容差）"));
        }
        let dist_ok = if reference.dist == 0.0 {
            (device.dist - reference.dist).abs() <= FOUR_ULP_AT_ONE
        } else {
            ulp_distance(device.dist, reference.dist) <= 4
        };
        if !dist_ok {
            failures.push(format!(
                "id={id} dist dev={:e} ref={:e} 超 4 ULP@自身量级",
                device.dist, reference.dist
            ));
        }
        let scale = vector_scale(&reference.pos);
        let bound = 4.0 * (f32::from_bits(scale.to_bits() + 1) - scale);
        for axis in 0..3 {
            if (device.pos[axis] - reference.pos[axis]).abs() > bound {
                failures.push(format!(
                    "id={id} pos[{axis}] |dev-ref|={:e} >{bound:e}（4·ULP(scale)）",
                    (device.pos[axis] - reference.pos[axis]).abs()
                ));
            }
        }
        for cell in 0..9 {
            if (device.frame[cell] - reference.frame[cell]).abs() > FOUR_ULP_AT_ONE {
                failures.push(format!(
                    "id={id} frame[{cell}] 超 4 ULP@1.0（dev={:e} ref={:e}）",
                    device.frame[cell], reference.frame[cell]
                ));
            }
        }
    }
    failures
}
