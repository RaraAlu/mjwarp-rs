#![allow(clippy::chunks_exact_to_as_chunks, clippy::unnecessary_sort_by)]
//! G3 数值判据：ours vs 同配方上游 f32（warp CPU 后端）sdf-box case。
//! 期望来自 `fixtures/g10-p5/sdf-f32/`（authoring 只用，产品测试不跑 Python）。

use super::*;
use serde_json::Value;

const JSON: &str = include_str!("../../../../fixtures/g10-p5/sdf-f32/sdf-box-cpu.json");

fn read_bytes(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/fixtures/g10-p5/sdf-f32/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read(&path).expect("读取 SDF 参考二进制")
}

fn f32s(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

fn i32s(bytes: &[u8]) -> Vec<i32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

fn triple(values: &[f32], index: usize) -> [f32; 3] {
    let base = index * 3;
    [values[base], values[base + 1], values[base + 2]]
}

fn mat9(values: &[f32], index: usize) -> [f32; 9] {
    let mut out = [0.0_f32; 9];
    for (offset, slot) in out.iter_mut().enumerate() {
        *slot = values[index * 9 + offset];
    }
    out
}

fn build_volume(value: &Value) -> VolumeData {
    let noct = value["model"]["files"]["oct_aabb"]["count"]
        .as_u64()
        .expect("noct") as usize
        / 6;
    let aabb_values = f32s(&read_bytes("oct_aabb.f32"));
    let child_values = i32s(&read_bytes("oct_child.i32"));
    let coeff_values = f32s(&read_bytes("oct_coeff.f32"));
    let oct_aabb: Vec<([f32; 3], [f32; 3])> = (0..noct)
        .map(|node| {
            (
                triple(&aabb_values, node * 2),
                triple(&aabb_values, node * 2 + 1),
            )
        })
        .collect();
    let oct_child = (0..noct)
        .map(|node| {
            let mut row = [0_i32; 8];
            row.copy_from_slice(&child_values[node * 8..node * 8 + 8]);
            row
        })
        .collect();
    let oct_coeff = (0..noct)
        .map(|node| {
            let mut row = [0.0_f32; 8];
            row.copy_from_slice(&coeff_values[node * 8..node * 8 + 8]);
            row
        })
        .collect();
    let root = value["model"]["arrays"]["mesh_octadr"][0]
        .as_i64()
        .expect("mesh_octadr") as usize;
    let (center, half_size) = oct_aabb[root];
    VolumeData::new(center, half_size, oct_aabb, oct_child, oct_coeff, root).expect("VolumeData")
}

fn scalar_at(field: &[Value], index: usize) -> f32 {
    match &field[index] {
        Value::Array(values) => values[0].as_f64().unwrap() as f32,
        other => other.as_f64().unwrap() as f32,
    }
}

fn int_at(field: &[Value], index: usize) -> i32 {
    match &field[index] {
        Value::Array(values) => values[0].as_i64().unwrap() as i32,
        other => other.as_i64().unwrap() as i32,
    }
}

fn bits(value: f32) -> u32 {
    value.to_bits()
}

fn ulp_diff(a: f32, b: f32) -> u32 {
    let (a, b) = (bits(a) as i64, bits(b) as i64);
    // 同号、有限值假设下按位型差计算 ULP。
    (a - b).unsigned_abs() as u32
}

fn scaled_ulp(ours: [f32; 3], upstream: [f32; 3]) -> f32 {
    let scale = upstream[0]
        .abs()
        .max(upstream[1].abs())
        .max(upstream[2].abs());
    if scale == 0.0 {
        return 0.0;
    }
    let mut max_ulp = 0.0_f32;
    for axis in 0..3 {
        let diff = (ours[axis] - upstream[axis]).abs();
        let ulp = scale.to_bits().wrapping_add(1);
        let next = f32::from_bits(ulp);
        max_ulp = max_ulp.max(diff / (next - scale).abs().max(f32::MIN_POSITIVE));
    }
    max_ulp
}

#[test]
fn sdf_box_host_matches_upstream_f32_within_4ulp() {
    let value: Value = serde_json::from_str(JSON).expect("解析 SDF 参考");
    assert_eq!(value["schema"], 1);
    assert_eq!(
        value["upstream_revision"],
        "71da24d956378a87a703b6e1442b13aec0c4ac29"
    );
    assert_eq!(value["case"]["two_captures_bitwise_identical"], true);
    let world_count = value["case"]["worlds"].as_u64().expect("worlds") as usize;
    let initpoints = value["case"]["sdf_initpoints"]
        .as_u64()
        .expect("initpoints") as usize;
    let iterations = value["case"]["sdf_iterations"]
        .as_u64()
        .expect("iterations") as usize;
    let output = &value["case"]["output"];
    let reference_count = output["count"].as_u64().expect("count") as usize;
    assert_eq!(reference_count, world_count * initpoints);

    let geom_pos: Vec<[f32; 3]> = output["geom_xpos"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|world| world.as_array().unwrap().iter())
        .map(|pos| {
            [
                pos[0].as_f64().unwrap() as f32,
                pos[1].as_f64().unwrap() as f32,
                pos[2].as_f64().unwrap() as f32,
            ]
        })
        .collect();
    let geom_mat: Vec<f32> = output["geom_xmat"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|world| world.as_array().unwrap().iter())
        .map(|value| value.as_f64().unwrap() as f32)
        .collect();
    let geom_type = &value["model"]["arrays"]["geom_type"];
    let geom_size = &value["model"]["arrays"]["geom_size"];
    let geom_aabb = &value["model"]["arrays"]["geom_aabb"];
    assert_eq!(geom_type[1].as_i64(), Some(GEOM_SDF as i64));
    let size1 = [
        geom_size[0][0].as_f64().unwrap() as f32,
        geom_size[0][1].as_f64().unwrap() as f32,
        geom_size[0][2].as_f64().unwrap() as f32,
    ];
    let aabb1 = (
        [
            geom_aabb[0][0][0].as_f64().unwrap() as f32,
            geom_aabb[0][0][1].as_f64().unwrap() as f32,
            geom_aabb[0][0][2].as_f64().unwrap() as f32,
        ],
        [
            geom_aabb[0][1][0].as_f64().unwrap() as f32,
            geom_aabb[0][1][1].as_f64().unwrap() as f32,
            geom_aabb[0][1][2].as_f64().unwrap() as f32,
        ],
    );
    let aabb2 = (
        [
            geom_aabb[1][0][0].as_f64().unwrap() as f32,
            geom_aabb[1][0][1].as_f64().unwrap() as f32,
            geom_aabb[1][0][2].as_f64().unwrap() as f32,
        ],
        [
            geom_aabb[1][1][0].as_f64().unwrap() as f32,
            geom_aabb[1][1][1].as_f64().unwrap() as f32,
            geom_aabb[1][1][2].as_f64().unwrap() as f32,
        ],
    );
    let volume = build_volume(&value);
    let shape1 = SdfShape::new(SdfType::Box, size1);
    let shape2 = SdfShape::new(SdfType::Sdf, [1.0, 1.0, 1.0]).with_volume(&volume);

    let fields = &output["fields"];
    let distance = fields["dist"].as_array().unwrap();
    let position = fields["pos"].as_array().unwrap();
    let frame = fields["frame"].as_array().unwrap();
    let worldid = fields["worldid"].as_array().unwrap();
    let mut upstream: Vec<(i32, f32, [f32; 3], [f32; 9])> = (0..reference_count)
        .map(|index| {
            (
                int_at(worldid, index),
                scalar_at(distance, index),
                [
                    position[index][0].as_f64().unwrap() as f32,
                    position[index][1].as_f64().unwrap() as f32,
                    position[index][2].as_f64().unwrap() as f32,
                ],
                mat9(
                    &frame
                        .iter()
                        .flat_map(|row| row.as_array().unwrap().iter())
                        .map(|cell| cell.as_f64().unwrap() as f32)
                        .collect::<Vec<f32>>(),
                    index,
                ),
            )
        })
        .collect();

    let mut ours: Vec<(i32, f32, [f32; 3], [f32; 9])> = Vec::new();
    for world in 0..world_count {
        let pair = SdfPair {
            shape1,
            shape2,
            pos1: geom_pos[world * 2],
            rot1: mat9(&geom_mat, world * 2),
            pos2: geom_pos[world * 2 + 1],
            rot2: mat9(&geom_mat, world * 2 + 1),
            aabb1,
            aabb2,
            sdf_initpoints: initpoints,
            sdf_iterations: iterations,
        };
        let contacts = sdf_pair_contacts(&pair);
        assert_eq!(contacts.len(), initpoints, "world {world} 接触数");
        for contact in contacts {
            ours.push((world as i32, contact.dist, contact.pos, contact.frame));
        }
    }

    let sort_key = |row: &(i32, f32, [f32; 3], [f32; 9])| {
        (
            row.0,
            row.1.total_cmp(&row.1),
            row.2[0].total_cmp(&row.2[0]),
            row.2[1].total_cmp(&row.2[1]),
            row.2[2].total_cmp(&row.2[2]),
        )
    };
    ours.sort_by(|left, right| sort_key(left).cmp(&sort_key(right)));
    upstream.sort_by(|left, right| sort_key(left).cmp(&sort_key(right)));

    let mut dist_max_ulp = 0_u32;
    let mut pos_max_ulp = 0.0_f32;
    let mut frame_max_abs = 0.0_f32;
    let mut dist_zero = 0_usize;
    let mut frame_zero = 0_usize;
    for (ours, upstream) in ours.iter().zip(&upstream) {
        assert_eq!(ours.0, upstream.0, "worldid");
        let ulp = ulp_diff(ours.1, upstream.1);
        if ulp == 0 {
            dist_zero += 1;
        }
        dist_max_ulp = dist_max_ulp.max(ulp);
        pos_max_ulp = pos_max_ulp.max(scaled_ulp(ours.2, upstream.2));
        for cell in 0..9 {
            let diff = (ours.3[cell] - upstream.3[cell]).abs();
            if diff == 0.0 {
                frame_zero += 1;
            }
            frame_max_abs = frame_max_abs.max(diff);
        }
    }
    println!(
        "[SDF判据] sdf-box count {}/{} | dist 位等 {}/{} max {} ULP | pos max {:.2} ULP@尺度 | frame 位等分量 {}/{} max abs {:e}",
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
    assert!(dist_max_ulp <= 4, "dist {dist_max_ulp} ULP");
    assert!(pos_max_ulp <= 4.0, "pos {pos_max_ulp} ULP");
    assert!(frame_max_abs <= 4.8e-7, "frame {frame_max_abs:e}");
}

#[test]
fn halton_matches_reference_sequence() {
    assert_eq!(halton(0, 2), 0.0);
    assert_eq!(halton(1, 2), 0.5);
    assert_eq!(halton(2, 2), 0.25);
    assert_eq!(halton(1, 3), 1.0 / 3.0);
    assert_eq!(halton(2, 3), 2.0 / 3.0);
    assert_eq!(halton(1, 5), 0.2);
    for index in 0..40 {
        let value = halton(index, 2);
        assert!((0.0..1.0).contains(&value), "halton({index},2)");
    }
}

#[test]
fn box_project_keeps_inside_points_and_expands_outside() {
    let center = [0.0, 0.0, 0.0];
    let half = [0.1, 0.08, 0.06];
    assert_eq!(
        box_project(center, half, [0.0, 0.0, 0.0]),
        (0.0, [0.0, 0.0, 0.0])
    );
    let (dist, point) = box_project(center, half, [0.3, 0.0, 0.0]);
    assert!(dist > 0.0);
    // 上游 eps=1e-4：投影点贴到盒面外 1e-4。
    assert!((point[0] - 0.0999).abs() < 1.0e-6, "{point:?}");
}
