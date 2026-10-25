//! G4 数值判据：ours vs 同配方上游 f32（warp CPU 后端）hfield 三个 case。
//! 期望来自 `fixtures/g10-p5/hfield-f32/`（authoring 只用；产品测试不跑 Python）。
//! 参考含重复接触点，按全字段（dist/pos/frame + worldid）多重集比较。

use super::super::gjk::math::sub3;
use super::super::gjk::{CcdOptions, Geom};
use super::*;
use serde_json::Value;

const CASES: [&str; 3] = ["hfield-box", "hfield-sphere", "hfield-capsule"];

fn read_json(name: &str) -> Value {
    let path = format!(
        "{}/fixtures/g10-p5/hfield-f32/{name}-cpu.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).expect("读取 hfield 参考");
    serde_json::from_str(&text).expect("解析 hfield 参考")
}

fn scalar(value: &Value) -> f32 {
    value.as_f64().unwrap_or_else(|| panic!("数值: {value}")) as f32
}

fn vector<const N: usize>(value: &Value) -> [f32; N] {
    let rows = value.as_array().expect("数组");
    assert_eq!(rows.len(), N);
    let mut out = [0.0_f32; N];
    for (slot, row) in out.iter_mut().zip(rows) {
        *slot = scalar(row);
    }
    out
}

fn floats(value: &Value) -> Vec<f32> {
    value.as_array().expect("数组").iter().map(scalar).collect()
}

fn rows_floats(value: &Value) -> Vec<Vec<f32>> {
    value
        .as_array()
        .expect("数组")
        .iter()
        .map(|row| match row {
            Value::Array(items) => items.iter().map(scalar).collect(),
            other => vec![scalar(other)],
        })
        .collect()
}

fn ints(value: &Value) -> Vec<i32> {
    value
        .as_array()
        .expect("数组")
        .iter()
        .map(|row| scalar(row) as i32)
        .collect()
}

fn bits(value: f32) -> u32 {
    value.to_bits()
}

fn ulp_diff(a: f32, b: f32) -> u32 {
    (bits(a) as i64 - bits(b) as i64).unsigned_abs() as u32
}

fn scaled_ulp(ours: [f32; 3], upstream: [f32; 3]) -> f32 {
    let scale = upstream[0]
        .abs()
        .max(upstream[1].abs())
        .max(upstream[2].abs());
    if scale == 0.0 {
        return 0.0;
    }
    let next = f32::from_bits(scale.to_bits().wrapping_add(1));
    let ulp = (next - scale).abs().max(f32::MIN_POSITIVE);
    let mut max_ulp = 0.0_f32;
    for axis in 0..3 {
        max_ulp = max_ulp.max((ours[axis] - upstream[axis]).abs() / ulp);
    }
    max_ulp
}

#[derive(Clone, Copy, PartialEq)]
struct Row {
    world: i32,
    dist: f32,
    pos: [f32; 3],
    frame: [f32; 9],
}

fn sort_rows(rows: &mut [Row]) {
    rows.sort_by(|left, right| {
        left.world
            .cmp(&right.world)
            .then(left.dist.total_cmp(&right.dist))
            .then(left.pos[0].total_cmp(&right.pos[0]))
            .then(left.pos[1].total_cmp(&right.pos[1]))
            .then(left.pos[2].total_cmp(&right.pos[2]))
            .then_with(|| {
                left.frame
                    .iter()
                    .zip(&right.frame)
                    .fold(std::cmp::Ordering::Equal, |order, (a, b)| {
                        order.then(a.total_cmp(b))
                    })
            })
    });
}

fn check_case(name: &str) {
    let value = read_json(name);
    assert_eq!(value["schema"], 1);
    assert_eq!(
        value["upstream_revision"],
        "71da24d956378a87a703b6e1442b13aec0c4ac29"
    );
    assert_eq!(value["case"]["two_captures_bitwise_identical"], true);
    let world_count = scalar(&value["case"]["worlds"]) as usize;
    let arrays = &value["model"]["arrays"];
    let opt = &value["model"]["opt"];
    let options = CcdOptions {
        tolerance: scalar(&opt["ccd_tolerance"]),
        cutoff: 0.0,
        gjk_iterations: scalar(&opt["ccd_iterations"]) as usize,
        epa_iterations: scalar(&opt["ccd_iterations"]) as usize,
    };
    let geom_type = ints(&arrays["geom_type"]);
    let geom_size: Vec<[f32; 3]> = arrays["geom_size"]
        .as_array()
        .unwrap()
        .iter()
        .map(vector::<3>)
        .collect();
    let geom_margin = floats(&arrays["geom_margin"]);
    let geom_gap = floats(&arrays["geom_gap"]);
    let geom_rbound = floats(&arrays["geom_rbound"]);
    let hf_index = geom_type
        .iter()
        .position(|&kind| kind == super::super::gjk::geom_type::HFIELD)
        .expect("hfield geom");
    let other_index = (0..geom_type.len())
        .find(|&index| index != hf_index)
        .expect("对侧 geom");
    let hfield = HfieldData::new(
        vector::<4>(&arrays["hfield_size"]),
        ints(&arrays["hfield_nrow"])[0],
        ints(&arrays["hfield_ncol"])[0],
        ints(&arrays["hfield_adr"])[0],
        floats(&arrays["hfield_data"]),
    )
    .expect("HfieldData");
    let output = &value["case"]["output"];
    let reference_count = scalar(&output["count"]) as usize;
    let pos_rows: Vec<Vec<[f32; 3]>> = output["geom_xpos"]
        .as_array()
        .unwrap()
        .iter()
        .map(|world| world.as_array().unwrap().iter().map(vector::<3>).collect())
        .collect();
    let mat_rows: Vec<Vec<[f32; 9]>> = output["geom_xmat"]
        .as_array()
        .unwrap()
        .iter()
        .map(|world| {
            world
                .as_array()
                .unwrap()
                .chunks(9)
                .map(|chunk| {
                    let mut matrix = [0.0_f32; 9];
                    for (slot, cell) in matrix.iter_mut().zip(chunk) {
                        *slot = scalar(cell);
                    }
                    matrix
                })
                .collect()
        })
        .collect();
    let fields = &output["fields"];
    let dist = rows_floats(&fields["dist"]);
    let reference_pos = rows_floats(&fields["pos"]);
    let reference_frame = rows_floats(&fields["frame"]);
    let worldid = rows_floats(&fields["worldid"]);
    let geom = rows_floats(&fields["geom"]);

    let mut ours = Vec::new();
    for world in 0..world_count {
        let hf_pos = pos_rows[world][hf_index];
        let hf_mat = mat_rows[world][hf_index];
        let other_pos = pos_rows[world][other_index];
        let other_rot = mat_rows[world][other_index];
        let pos2_hf =
            super::super::gjk::math::transpose_mul_mat3_vec(&hf_mat, sub3(other_pos, hf_pos));
        let rot2_hf = super::driver::relative_matrix(&hf_mat, &other_rot);
        let geom2 = Geom::simple(pos2_hf, rot2_hf, geom_size[other_index], 0.0);
        let input = HfieldInput {
            hfield: &hfield,
            geom2,
            geomtype2: geom_type[other_index],
            other_rbound: geom_rbound[other_index],
            margin: geom_margin[hf_index] + geom_margin[other_index],
            gap: geom_gap[hf_index] + geom_gap[other_index],
            options,
            hf_mat,
            hf_pos,
        };
        let result = hfield_pair_contacts(&input);
        assert!(!result.overflow, "{name} world {world} 不应溢出");
        for contact in result.contacts {
            ours.push(Row {
                world: world as i32,
                dist: contact.dist,
                pos: contact.pos,
                frame: contact.frame,
            });
        }
    }

    let mut expected: Vec<Row> = (0..reference_count)
        .map(|index| Row {
            world: worldid[index][0] as i32,
            dist: dist[index][0],
            pos: [
                reference_pos[index][0],
                reference_pos[index][1],
                reference_pos[index][2],
            ],
            frame: {
                let mut out = [0.0_f32; 9];
                out.copy_from_slice(&reference_frame[index]);
                out
            },
        })
        .collect();
    assert_eq!(geom.len(), reference_count, "{name} 参考 geom 结构");
    for pair in &geom {
        assert_eq!(pair[0] as i32, hf_index as i32, "{name} hfield 为 g1");
        assert_eq!(pair[1] as i32, other_index as i32, "{name} 对侧为 g2");
    }
    for world in 0..world_count {
        let count = ours.iter().filter(|row| row.world == world as i32).count();
        let reference = expected
            .iter()
            .filter(|row| row.world == world as i32)
            .count();
        assert_eq!(count, reference, "{name} world {world} 接触数");
        assert!(count > 0, "{name} world {world} 无接触");
        assert!(count <= 4, "{name} world {world} 超过四接触");
    }
    sort_rows(&mut ours);
    sort_rows(&mut expected);
    let mut dist_max_ulp = 0_u32;
    let mut pos_max_ulp = 0.0_f32;
    let mut frame_max_abs = 0.0_f32;
    let mut dist_zero = 0_usize;
    let mut frame_zero = 0_usize;
    for (ours, expected) in ours.iter().zip(&expected) {
        assert_eq!(ours.world, expected.world, "{name} worldid");
        let ulp = ulp_diff(ours.dist, expected.dist);
        dist_zero += usize::from(ulp == 0);
        dist_max_ulp = dist_max_ulp.max(ulp);
        pos_max_ulp = pos_max_ulp.max(scaled_ulp(ours.pos, expected.pos));
        for cell in 0..9 {
            let diff = (ours.frame[cell] - expected.frame[cell]).abs();
            frame_zero += usize::from(diff == 0.0);
            frame_max_abs = frame_max_abs.max(diff);
        }
    }
    println!(
        "[hfield判据] {name:<15} count {}/{} | dist 位等 {}/{} max {} ULP | pos max {:.2} ULP@尺度 | frame 位等分量 {}/{} max abs {:e}",
        ours.len(),
        reference_count,
        dist_zero,
        reference_count,
        dist_max_ulp,
        pos_max_ulp,
        frame_zero,
        reference_count * 9,
        frame_max_abs
    );
    assert!(dist_max_ulp <= 4, "{name} dist {dist_max_ulp} ULP");
    assert!(pos_max_ulp <= 4.0, "{name} pos {pos_max_ulp} ULP");
    assert!(frame_max_abs <= 4.8e-7, "{name} frame {frame_max_abs:e}");
}

#[test]
fn hfield_host_matches_upstream_f32_within_4ulp() {
    for name in CASES {
        check_case(name);
    }
}

fn flat_hfield(nrow: i32, ncol: i32) -> HfieldData {
    let data = vec![0.0_f32; (nrow * ncol) as usize];
    HfieldData::new([1.0, 1.0, 0.1, 0.1], nrow, ncol, 0, data).expect("hfield")
}

fn default_options() -> CcdOptions {
    CcdOptions::default()
}

#[test]
fn filter_rejects_far_geoms_and_accepts_overlap() {
    let hfield = flat_hfield(3, 3);
    let options = default_options();
    let far = Geom::simple(
        [5.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        [0.5, 0.0, 0.0],
        0.0,
    );
    let far_input = HfieldInput {
        hfield: &hfield,
        geom2: far,
        geomtype2: super::super::gjk::geom_type::SPHERE,
        other_rbound: 0.5,
        margin: 0.0,
        gap: 0.0,
        options,
        hf_mat: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        hf_pos: [0.0; 3],
    };
    assert!(hfield_pair_contacts(&far_input).contacts.is_empty());
    let overlapping = Geom::simple(
        [0.0, 0.0, 0.05],
        [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        [0.5, 0.0, 0.0],
        0.0,
    );
    let overlap_input = HfieldInput {
        hfield: &hfield,
        geom2: overlapping,
        geomtype2: super::super::gjk::geom_type::SPHERE,
        other_rbound: 0.5,
        margin: 0.0,
        gap: 0.0,
        options,
        hf_mat: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        hf_pos: [0.0; 3],
    };
    let output = hfield_pair_contacts(&overlap_input);
    assert!(!output.contacts.is_empty());
    assert!(output.contacts.len() <= 4);
}

#[test]
fn maxconpair_overflow_sets_flag_and_caps_scan() {
    let hfield = flat_hfield(11, 11);
    let sphere = Geom::simple(
        [0.0, 0.0, 0.05],
        [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        [2.0, 0.0, 0.0],
        0.0,
    );
    let input = HfieldInput {
        hfield: &hfield,
        geom2: sphere,
        geomtype2: super::super::gjk::geom_type::SPHERE,
        other_rbound: 2.0,
        margin: 0.0,
        gap: 0.0,
        options: default_options(),
        hf_mat: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        hf_pos: [0.0; 3],
    };
    let output = hfield_pair_contacts(&input);
    assert!(output.overflow, "大子网格应触发 MJ_MAXCONPAIR 溢出");
    assert_eq!(output.prism_count, super::driver::MJ_MAXCONPAIR);
    assert!(!output.contacts.is_empty());
    assert!(output.contacts.len() <= 4);
}
