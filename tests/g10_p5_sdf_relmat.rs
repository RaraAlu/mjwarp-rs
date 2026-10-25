//! G10@P5 G3 修正片：非单位相对旋转 SDF 验收（产品 host vs 同配方上游 f32 CPU 参考）。
//!
//! 上游 `collision_sdf.py:724` 的 `params.rel_mat = wp.transpose(rot1) * rot2`（行优先
//! `aᵀb`，累加按 `warp/native/mat.h mul(mat,mat)` 的 fma 链）。产品旧
//! `sdf/descent.rs::relative_matrix` 构造 `bᵀa` 且普通乘加；本片已改为复用
//! `relmat::upstream_relative_matrix`。本测试用 authoring 捕获的
//! `fixtures/g10-p5/sdf-rotation/sdf-rotation-cpu.json`（warp CPU，两次捕获位等）做
//! **40/40 位级 0 ULP** 判据，并重跑 G3 `sdf-box` 80/80 位级回归。
//!
//! 参考数据只由 `scripts/authoring/g10-p5/capture_sdf_rotation_references.py` 生成；
//! 产品测试不启动 Python。

// 证据比对：字节切片解析刻意直白；按需豁免对应 clippy 提示。
#![allow(clippy::chunks_exact_to_as_chunks)]

use mjwarp_rs::physics::{SdfPair, SdfShape, SdfType, VolumeData, sdf_pair_contacts};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_json(relative: &str) -> Value {
    let text = std::fs::read_to_string(root().join(relative)).expect("读取 JSON 参考");
    serde_json::from_str(&text).expect("解析 JSON 参考")
}

fn read_bytes(relative: &str) -> Vec<u8> {
    std::fs::read(root().join(relative)).expect("读取二进制参考")
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

fn scalar(value: &Value) -> f32 {
    value.as_f64().expect("数值") as f32
}

fn vector3(value: &Value) -> [f32; 3] {
    let rows = value.as_array().expect("vec3");
    assert_eq!(rows.len(), 3);
    [scalar(&rows[0]), scalar(&rows[1]), scalar(&rows[2])]
}

fn mat9(values: &[f32], index: usize) -> [f32; 9] {
    let mut out = [0.0_f32; 9];
    for (offset, slot) in out.iter_mut().enumerate() {
        *slot = values[index * 9 + offset];
    }
    out
}

/// 从 `fixtures/g10-p5/sdf-f32/oct_*` 读体积（两个参考文件共用同一 volme）。
fn load_volume(root_octadr: usize) -> VolumeData {
    let aabb_bytes = read_bytes("fixtures/g10-p5/sdf-f32/oct_aabb.f32");
    let child_bytes = read_bytes("fixtures/g10-p5/sdf-f32/oct_child.i32");
    let coeff_bytes = read_bytes("fixtures/g10-p5/sdf-f32/oct_coeff.f32");
    let aabb_values = f32s(&aabb_bytes);
    let child_values = i32s(&child_bytes);
    let coeff_values = f32s(&coeff_bytes);
    let noct = aabb_values.len() / 6;
    assert_eq!(child_values.len(), noct * 8, "oct_child 长度");
    assert_eq!(coeff_values.len(), noct * 8, "oct_coeff 长度");
    let oct_aabb: Vec<([f32; 3], [f32; 3])> = (0..noct)
        .map(|node| {
            let base = node * 6;
            (
                [
                    aabb_values[base],
                    aabb_values[base + 1],
                    aabb_values[base + 2],
                ],
                [
                    aabb_values[base + 3],
                    aabb_values[base + 4],
                    aabb_values[base + 5],
                ],
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
    assert!(noct > root_octadr, "oct root 超界：{root_octadr} / {noct}");
    let (center, half_size) = oct_aabb[root_octadr];
    VolumeData::new(
        center,
        half_size,
        oct_aabb,
        oct_child,
        oct_coeff,
        root_octadr,
    )
    .expect("VolumeData")
}

#[derive(Clone, Debug)]
struct Row {
    world: i32,
    dist: f32,
    pos: [f32; 3],
    frame: [f32; 9],
}

fn row_key(row: &Row) -> (i32, Vec<u32>, Vec<u32>, Vec<u32>) {
    (
        row.world,
        vec![row.dist.to_bits()],
        row.pos.iter().map(|v| v.to_bits()).collect(),
        row.frame.iter().map(|v| v.to_bits()).collect(),
    )
}

fn rows_from_reference(output: &Value) -> Vec<Row> {
    let count = output["count"].as_u64().expect("count") as usize;
    let fields = &output["fields"];
    let dist = fields["dist"].as_array().expect("dist");
    let pos = fields["pos"].as_array().expect("pos");
    let frame = fields["frame"].as_array().expect("frame");
    let world = fields["worldid"].as_array().expect("worldid");
    (0..count)
        .map(|index| Row {
            world: world[index][0].as_i64().expect("worldid") as i32,
            dist: scalar(&dist[index][0]),
            pos: vector3(&pos[index]),
            frame: {
                let cells = frame[index].as_array().expect("frame row");
                let mut out = [0.0_f32; 9];
                for (slot, cell) in out.iter_mut().zip(cells) {
                    *slot = scalar(cell);
                }
                out
            },
        })
        .collect()
}

/// 产品侧：按 fixture 记录的 geom 位姿/类型/尺寸构造 `SdfPair` 并跑宿主窄相。
fn product_rows(
    scene: &Value,
    volume: &VolumeData,
    g1: usize,
    g2: usize,
    worlds: usize,
    initpoints: usize,
    iterations: usize,
) -> Vec<Row> {
    let arrays = &scene["model"]["arrays"];
    let geom_type = arrays["geom_type"].as_array().expect("geom_type");
    let geom_size = arrays["geom_size"].as_array().expect("geom_size");
    let geom_aabb = arrays["geom_aabb"].as_array().expect("geom_aabb");
    let size1 = vector3(&geom_size[g1]);
    let size2 = vector3(&geom_size[g2]);
    let type1 = geom_type[g1].as_i64().expect("type1") as i32;
    let type2 = geom_type[g2].as_i64().expect("type2") as i32;
    assert_eq!(
        SdfType::from_geom(type2),
        Some(SdfType::Sdf),
        "geom2 必须是体积 SDF"
    );
    let shape1 = SdfShape::new(SdfType::from_geom(type1).expect("geom1 SdfType"), size1);
    let shape2 = SdfShape::new(SdfType::Sdf, size2).with_volume(volume);
    let aabb_of = |index: usize| -> ([f32; 3], [f32; 3]) {
        (vector3(&geom_aabb[index][0]), vector3(&geom_aabb[index][1]))
    };
    let aabb1 = aabb_of(g1);
    let aabb2 = aabb_of(g2);
    let output = &scene["output"];
    let geom_xpos = output["geom_xpos"].as_array().expect("geom_xpos");
    let geom_xmat = output["geom_xmat"].as_array().expect("geom_xmat");
    let mut rows = Vec::new();
    for world in 0..worlds {
        let ngeom = geom_xpos[world].as_array().expect("xpos world").len();
        let mut xpos = Vec::with_capacity(ngeom);
        for value in geom_xpos[world].as_array().expect("xpos rows") {
            xpos.push(vector3(value));
        }
        let mut xmat = Vec::with_capacity(ngeom * 9);
        for value in geom_xmat[world].as_array().expect("xmat rows") {
            xmat.push(scalar(value));
        }
        let pair = SdfPair {
            shape1,
            shape2,
            pos1: xpos[g1],
            rot1: mat9(&xmat, g1),
            pos2: xpos[g2],
            rot2: mat9(&xmat, g2),
            aabb1,
            aabb2,
            sdf_initpoints: initpoints,
            sdf_iterations: iterations,
        };
        for contact in sdf_pair_contacts(&pair) {
            rows.push(Row {
                world: world as i32,
                dist: contact.dist,
                pos: contact.pos,
                frame: contact.frame,
            });
        }
    }
    rows
}

/// 逐 world 排序后严格位级比较；返回 (bit 相等接触数, 总数, 最大 dist 绝对差,
/// 最大 pos 绝对差, 最大 frame 绝对差)。
fn compare_bitwise(mut ours: Vec<Row>, mut reference: Vec<Row>) -> (usize, usize, f32, f32, f32) {
    let key = |row: &Row| {
        let (world, dist, pos, frame) = row_key(row);
        (world, dist, pos, frame)
    };
    ours.sort_by_key(key);
    reference.sort_by_key(key);
    assert_eq!(ours.len(), reference.len(), "接触数不等");
    let total = ours.len();
    let mut bit_equal = 0_usize;
    let mut dist_max = 0.0_f32;
    let mut pos_max = 0.0_f32;
    let mut frame_max = 0.0_f32;
    for (left, right) in ours.iter().zip(&reference) {
        let dist_equal = left.dist.to_bits() == right.dist.to_bits();
        let pos_equal = left
            .pos
            .iter()
            .zip(&right.pos)
            .all(|(a, b)| a.to_bits() == b.to_bits());
        let frame_equal = left
            .frame
            .iter()
            .zip(&right.frame)
            .all(|(a, b)| a.to_bits() == b.to_bits());
        if dist_equal && pos_equal && frame_equal {
            bit_equal += 1;
        }
        dist_max = dist_max.max((left.dist - right.dist).abs());
        for axis in 0..3 {
            pos_max = pos_max.max((left.pos[axis] - right.pos[axis]).abs());
        }
        for cell in 0..9 {
            frame_max = frame_max.max((left.frame[cell] - right.frame[cell]).abs());
        }
    }
    (bit_equal, total, dist_max, pos_max, frame_max)
}

fn acceptance_scenes(fixture: &Value) -> Vec<String> {
    fixture["acceptance_scenes"]
        .as_array()
        .expect("acceptance_scenes")
        .iter()
        .map(|value| value.as_str().expect("scene name").to_owned())
        .collect()
}

/// fixture 结构与 provenance：sha256、两次捕获位等、oct 数组与既有 fixture 逐字节一致。
#[test]
fn sdf_rotation_fixture_is_well_formed() {
    let fixture = read_json("fixtures/g10-p5/sdf-rotation/sdf-rotation-cpu.json");
    assert_eq!(fixture["schema"], 1);
    assert_eq!(fixture["task_id"], "2026-10-25-g10-p5-sdf-relmat-fix");
    assert_eq!(
        fixture["upstream_revision"],
        "71da24d956378a87a703b6e1442b13aec0c4ac29"
    );
    let provenance = &fixture["provenance"];
    assert_eq!(provenance["device"], "cpu");
    assert_eq!(provenance["authoring_only"], true);
    assert_eq!(provenance["reads_product_tree"], false);
    assert_eq!(
        provenance["two_captures_bitwise_identical_all_scenes"],
        true
    );
    let script = provenance["capture_script"].as_str().expect("script");
    let digest = format!("{:x}", Sha256::digest(read_bytes(script)));
    assert_eq!(
        digest,
        provenance["capture_script_sha256"]
            .as_str()
            .expect("script sha")
    );

    // oct 数组：路径存在、长度与 sha256 与声明一致。
    for (name, record) in fixture["model"]["oct_files"]
        .as_object()
        .expect("oct_files")
    {
        let path = record["path"].as_str().expect("oct path");
        let bytes = read_bytes(path);
        let count = record["count"].as_u64().expect("oct count") as usize;
        assert_eq!(bytes.len(), count * 4, "{name} 字节数");
        let sha = format!("{:x}", Sha256::digest(&bytes));
        assert_eq!(sha, record["sha256"].as_str().expect("oct sha"), "{name}");
        assert_eq!(record["bitwise_equal_to_fixture"], true, "{name}");
    }

    let scenes = fixture["scenes"].as_array().expect("scenes");
    let names: Vec<&str> = scenes
        .iter()
        .map(|scene| scene["name"].as_str().expect("name"))
        .collect();
    for required in acceptance_scenes(&fixture) {
        assert!(
            names.contains(&required.as_str()),
            "缺少验收场景 {required}"
        );
    }
    for scene in scenes {
        assert_eq!(scene["worlds"], 2, "worlds");
        assert_eq!(scene["sdf_initpoints"], 40, "initpoints");
        assert_eq!(scene["sdf_iterations"], 10, "iterations");
        assert_eq!(scene["two_captures_bitwise_identical"], true);
        let count = scene["output"]["count"].as_u64().expect("count") as usize;
        assert_eq!(count, 80, "scene {} 接触数", scene["name"]);
        let worldid = scene["output"]["fields"]["worldid"]
            .as_array()
            .expect("worldid");
        for world in 0..2 {
            let rows = worldid
                .iter()
                .filter(|row| row[0].as_i64() == Some(world as i64))
                .count();
            assert_eq!(rows, 40, "scene {} world{world} 接触数", scene["name"]);
        }
    }
}

/// 4 个非单位旋转场景：产品 vs 上游 f32 CPU **40/40 位级 0 ULP**（每 world 40）。
#[test]
fn sdf_rotation_product_matches_upstream_f32_bitwise() {
    let fixture = read_json("fixtures/g10-p5/sdf-rotation/sdf-rotation-cpu.json");
    let root_octadr = fixture["scenes"][0]["model"]["arrays"]["mesh_octadr"][0]
        .as_i64()
        .expect("mesh_octadr") as usize;
    let volume = load_volume(root_octadr);
    let scenes = fixture["scenes"].as_array().expect("scenes");
    for scene in scenes {
        let name = scene["name"].as_str().expect("name");
        let geom = scene["output"]["fields"]["geom"][0]
            .as_array()
            .expect("geom pair");
        let g1 = geom[0].as_i64().expect("g1") as usize;
        let g2 = geom[1].as_i64().expect("g2") as usize;
        let worlds = scene["worlds"].as_u64().expect("worlds") as usize;
        let initpoints = scene["sdf_initpoints"].as_u64().expect("initpoints") as usize;
        let iterations = scene["sdf_iterations"].as_u64().expect("iterations") as usize;
        let reference = rows_from_reference(&scene["output"]);
        let ours = product_rows(scene, &volume, g1, g2, worlds, initpoints, iterations);
        let (bit_equal, total, dist_max, pos_max, frame_max) = compare_bitwise(ours, reference);
        println!(
            "[SDF-rot] {name}: {bit_equal}/{total} contacts bit-equal | dist_max_abs {dist_max:e} | pos_max_abs {pos_max:e} | frame_max_abs {frame_max:e}"
        );
        assert_eq!(total, 80, "{name} 接触数");
        assert_eq!(
            bit_equal, total,
            "{name}: {bit_equal}/{total} 位级不等于上游 f32 CPU"
        );
        assert_eq!(dist_max, 0.0, "{name} dist 位级");
        assert_eq!(pos_max, 0.0, "{name} pos 位级");
        assert_eq!(frame_max, 0.0, "{name} frame 位级");
    }
}

/// G3 回归：`sdf-box` 80/80 接触全字段位级不变（复用 `product_rows` 的公开产品路径）。
#[test]
fn sdf_box_g3_regression_remains_bitwise() {
    let fixture = read_json("fixtures/g10-p5/sdf-f32/sdf-box-cpu.json");
    assert_eq!(fixture["case"]["two_captures_bitwise_identical"], true);
    let case = &fixture["case"];
    let root_octadr = fixture["model"]["arrays"]["mesh_octadr"][0]
        .as_i64()
        .expect("mesh_octadr") as usize;
    let volume = load_volume(root_octadr);
    let scene = serde_json::json!({"model": fixture["model"], "output": case["output"]});
    let worlds = case["worlds"].as_u64().expect("worlds") as usize;
    let initpoints = case["sdf_initpoints"].as_u64().expect("initpoints") as usize;
    let iterations = case["sdf_iterations"].as_u64().expect("iterations") as usize;
    let reference = rows_from_reference(&case["output"]);
    let ours = product_rows(&scene, &volume, 0, 1, worlds, initpoints, iterations);
    let (bit_equal, total, dist_max, pos_max, frame_max) = compare_bitwise(ours, reference);
    println!(
        "[SDF-rot] sdf-box(G3 回归): {bit_equal}/{total} contacts bit-equal | dist_max_abs {dist_max:e} | pos_max_abs {pos_max:e} | frame_max_abs {frame_max:e}"
    );
    assert_eq!(total, 80, "sdf-box 接触数");
    assert_eq!(bit_equal, total, "sdf-box {bit_equal}/{total} 位级不等");
}

/// fixture 的 `rel_mat_warp` 必须等于上游 `transpose(rot1)*rot2`（aᵀb，fma 链位型），
/// 且对非对称相对旋转与 `bᵀa` 不同——证明参考确实编码了上游约定、而非旧 `bᵀa`。
#[test]
fn sdf_rotation_reference_encodes_upstream_rel_mat_convention() {
    fn fma_f32(a: f32, b: f32, c: f32) -> f32 {
        (f64::from(a) * f64::from(b) + f64::from(c)) as f32
    }
    fn upstream_ab(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
        let mut out = [0.0_f32; 9];
        for row in 0..3 {
            for column in 0..3 {
                let sum = fma_f32(a[row], b[column], 0.0);
                let sum = fma_f32(a[3 + row], b[3 + column], sum);
                out[row * 3 + column] = fma_f32(a[6 + row], b[6 + column], sum);
            }
        }
        out
    }
    fn transpose(matrix: &[f32; 9]) -> [f32; 9] {
        let mut out = [0.0_f32; 9];
        for row in 0..3 {
            for column in 0..3 {
                out[row * 3 + column] = matrix[column * 3 + row];
            }
        }
        out
    }
    let fixture = read_json("fixtures/g10-p5/sdf-rotation/sdf-rotation-cpu.json");
    let mut compared = 0_usize;
    let mut discriminating = 0_usize;
    for scene in fixture["scenes"].as_array().expect("scenes") {
        let worlds = scene["worlds"].as_u64().expect("worlds") as usize;
        let geom_xmat = scene["output"]["geom_xmat"].as_array().expect("xmat");
        let rel_all = scene["rel_mat_warp"].as_array().expect("rel_mat_warp");
        for world in 0..worlds {
            let values: Vec<f32> = geom_xmat[world]
                .as_array()
                .expect("xmat world")
                .iter()
                .map(scalar)
                .collect();
            let rot1 = mat9(&values, 0);
            let rot2 = mat9(&values, 1);
            let expected = upstream_ab(&rot1, &rot2);
            let recorded: Vec<f32> = rel_all[world]
                .as_array()
                .expect("rel row")
                .iter()
                .map(scalar)
                .collect();
            for cell in 0..9 {
                assert_eq!(
                    expected[cell].to_bits(),
                    recorded[cell].to_bits(),
                    "scene {} world{world} rel_mat cell {cell}",
                    scene["name"]
                );
            }
            compared += 1;
            let transposed = transpose(&expected);
            if transposed != expected {
                discriminating += 1;
            }
        }
    }
    println!(
        "[SDF-rot] rel_mat_warp aᵀb 位级 {compared}/{compared} world；其中 {discriminating} world 与 bᵀa 可区分"
    );
    assert!(compared >= 10, "至少 5 场景 × 2 world");
    assert!(
        discriminating >= 8,
        "非单位旋转场景应暴露 aᵀb ≠ bᵀa（实际 {discriminating}）"
    );
}
