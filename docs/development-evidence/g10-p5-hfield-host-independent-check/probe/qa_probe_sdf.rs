//! QA 产品侧探针（SDF 相对旋转约定判定）：只在临时 `git worktree` 的 HEAD 副本内编译。
//!
//! 目的：用**非单位相对旋转**回答「产品 SDF host 路径是否真的错」：
//! 1. `product`：产品 `sdf_pair_contacts`（内部 `descent::relative_matrix`，研发实现）；
//! 2. `corrected`：QA 复制上游结构、只把 `rel_mat` 换成上游语义 `rot1ᵀ·rot2`（行优先），
//!    其余全部调用产品公开件（`gradient_step`/`sdf_grad`/`make_frame`/`halton`），
//!    用于证明「唯一差异项就是 rel_mat」；
//! 3. `rel_mat_product` / `rel_mat_upstream`：两份矩阵逐元素输出。
//!
//! 输入：QA_SDF_CASES=qa-sdf-rotation-cases.json；QA_SDF_OCT=fixtures/g10-p5/sdf-f32；
//! 输出：QA_SDF_OUT=qa-sdf-rotation-product.json。不改产品代码。

use super::super::gjk::math::{
    add3, make_frame, mul_mat3_vec, normalize3, scale3, sub3, transpose_mul_mat3_vec,
};
use super::descent::{OptimizationParams, gradient_step, relative_matrix, sdf_grad};
use super::{SdfContact, SdfPair, SdfShape, SdfType, VolumeData, halton, sdf_pair_contacts};
use serde_json::{Value, json};

fn f32_of(value: &Value) -> f32 {
    value.as_f64().expect("number") as f32
}
fn i32_of(value: &Value) -> i32 {
    value.as_i64().expect("int") as i32
}
fn vec_f32(value: &Value) -> Vec<f32> {
    value.as_array().expect("array").iter().map(f32_of).collect()
}
fn vec_f32_3(value: &Value) -> [f32; 3] {
    let v = vec_f32(value);
    [v[0], v[1], v[2]]
}
fn at_index(value: &Value, index: usize) -> i64 {
    match value {
        Value::Array(items) => items[index].as_i64().expect("int"),
        other => other.as_i64().expect("int"),
    }
}
fn mat9(value: &Value) -> [f32; 9] {
    let v = vec_f32(value);
    let mut out = [0.0_f32; 9];
    out.copy_from_slice(&v);
    out
}

/// 上游语义 `transpose(a) * b`（行优先 `out[row][col] = Σ_k a[k][row]*b[k][col]`）。
fn upstream_relative_matrix(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
    let mut out = [0.0_f32; 9];
    for row in 0..3 {
        for column in 0..3 {
            out[row * 3 + column] = a[row] * b[column]
                + a[3 + row] * b[3 + column]
                + a[6 + row] * b[6 + column];
        }
    }
    out
}

/// 上游语义 `transpose(a) * b`（行优先），**逐项 fma 语义**：warp `native/mat.h` 的
/// `mul(mat,mat)` 用 `muladd<float>`（`fmaf`）按 k=0..2 累加；因本机测试档（opt-level 0）
/// `f32::mul_add` 未得到正确融合结果，这里用 f64 精确乘加后一次舍入到 f32 模拟 fma。
fn upstream_relative_matrix_fma(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
    let mut out = [0.0_f32; 9];
    for row in 0..3 {
        for column in 0..3 {
            let mut sum = a[row] * b[column];
            sum = ((a[3 + row] as f64) * (b[3 + column] as f64) + (sum as f64)) as f32;
            sum = ((a[6 + row] as f64) * (b[6 + column] as f64) + (sum as f64)) as f32;
            out[row * 3 + column] = sum;
        }
    }
    out
}

fn read_bytes(path: &std::path::Path) -> Vec<u8> {
    std::fs::read(path).expect("read oct binary")
}
fn f32s(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}
fn i32s(bytes: &[u8]) -> Vec<i32> {
    bytes
        .chunks_exact(4)
        .map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}
fn triple(values: &[f32], index: usize) -> [f32; 3] {
    let base = index * 3;
    [values[base], values[base + 1], values[base + 2]]
}

fn build_volume(cases: &Value, oct_dir: &std::path::Path) -> VolumeData {
    let files = &cases["oct_files"];
    let noct = files["oct_aabb"]["fresh_count"].as_u64().expect("count") as usize / 6;
    let aabb_values = f32s(&read_bytes(&oct_dir.join("oct_aabb.f32")));
    let child_values = i32s(&read_bytes(&oct_dir.join("oct_child.i32")));
    let coeff_values = f32s(&read_bytes(&oct_dir.join("oct_coeff.f32")));
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
    let root = 0_usize; // 由调用方按 scenes[].mesh_octadr 覆盖
    let (center, half_size) = oct_aabb[root];
    VolumeData::new(center, half_size, oct_aabb, oct_child, oct_coeff, root).expect("VolumeData")
}

/// 上游 `gradient_descent` 的两步结构；`rel_mat` 由调用方给出（上游语义）。
#[allow(clippy::too_many_arguments)]
fn corrected_gradient_descent(
    shape1: &SdfShape,
    shape2: &SdfShape,
    x0: [f32; 3],
    pos1: [f32; 3],
    rot1: [f32; 9],
    pos2: [f32; 3],
    rot2: [f32; 9],
    rel_mat: [f32; 9],
    iterations: usize,
) -> (f32, [f32; 3], [f32; 3]) {
    let params = OptimizationParams {
        rel_mat,
        rel_pos: transpose_mul_mat3_vec(&rot1, sub3(pos2, pos1)),
        attr1: shape1.size,
        attr2: shape2.size,
    };
    let (_, x) = gradient_step(shape1, shape2, x0, &params, iterations, false);
    let (dist, x) = gradient_step(shape1, shape2, x, &params, 1, true);
    let x1 = add3(mul_mat3_vec(&params.rel_mat, x), params.rel_pos);
    let grad1 = normalize3(transpose_mul_mat3_vec(
        &params.rel_mat,
        sdf_grad(shape1, x1),
    ));
    let grad2 = normalize3(sdf_grad(shape2, x));
    let n = normalize3(sub3(grad1, grad2));
    let pos = add3(mul_mat3_vec(&rot2, x), pos2);
    let n = mul_mat3_vec(&rot2, n);
    (dist, sub3(pos, scale3(n, dist / 2.0)), n)
}

fn transform_aabb(
    center: [f32; 3],
    size: [f32; 3],
    pos: [f32; 3],
    ori: &[f32; 9],
) -> ([f32; 3], [f32; 3]) {
    let mut min = [1.0e9_f32; 3];
    let mut max = [-1.0e9_f32; 3];
    for index in 0..8 {
        let corner = [
            size[0] * if index & 1 != 0 { 1.0 } else { -1.0 },
            size[1] * if index & 2 != 0 { 1.0 } else { -1.0 },
            size[2] * if index & 4 != 0 { 1.0 } else { -1.0 },
        ];
        let frame = add3(mul_mat3_vec(ori, add3(corner, center)), pos);
        for axis in 0..3 {
            min[axis] = min[axis].min(frame[axis]);
            max[axis] = max[axis].max(frame[axis]);
        }
    }
    (min, max)
}

/// QA 复制的上游窄相结构（rel_mat 由调用方给出），其余为产品件。
fn corrected_contacts(pair: &SdfPair, rel_mat: [f32; 9]) -> Vec<SdfContact> {
    let identity = [1.0_f32, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    let (aabb1_min, aabb1_max) =
        transform_aabb(pair.aabb1.0, pair.aabb1.1, [0.0; 3], &identity);
    let rel_pos = transpose_mul_mat3_vec(&pair.rot1, sub3(pair.pos2, pair.pos1));
    let (aabb2_min, aabb2_max) =
        transform_aabb(pair.aabb2.0, pair.aabb2.1, rel_pos, &rel_mat);
    let mut min = [0.0_f32; 3];
    let mut max = [0.0_f32; 3];
    for axis in 0..3 {
        min[axis] = aabb1_min[axis].max(aabb2_min[axis]);
        max[axis] = aabb1_max[axis].min(aabb2_max[axis]);
        if min[axis] > max[axis] {
            return Vec::new();
        }
    }
    let mut contacts = Vec::with_capacity(pair.sdf_initpoints);
    for index in 0..pair.sdf_initpoints {
        let x_g2 = [
            min[0] + (max[0] - min[0]) * halton(index as u32, 2),
            min[1] + (max[1] - min[1]) * halton(index as u32, 3),
            min[2] + (max[2] - min[2]) * halton(index as u32, 5),
        ];
        let x = add3(mul_mat3_vec(&pair.rot1, x_g2), pair.pos1);
        let x0_initial = transpose_mul_mat3_vec(&pair.rot2, sub3(x, pair.pos2));
        let (dist, pos, normal) = corrected_gradient_descent(
            &pair.shape1,
            &pair.shape2,
            x0_initial,
            pair.pos1,
            pair.rot1,
            pair.pos2,
            pair.rot2,
            rel_mat,
            pair.sdf_iterations,
        );
        contacts.push(SdfContact {
            dist,
            pos,
            frame: make_frame(normal),
        });
    }
    contacts
}

fn contact_json(contact: &SdfContact) -> Value {
    json!({
        "dist": contact.dist,
        "pos": [contact.pos[0], contact.pos[1], contact.pos[2]],
        "frame": contact.frame.to_vec(),
    })
}

#[test]
fn qa_probe_sdf_rotation() {
    let cases_path = std::env::var("QA_SDF_CASES").expect("QA_SDF_CASES");
    let oct_dir = std::env::var("QA_SDF_OCT").expect("QA_SDF_OCT");
    let out_path = std::env::var("QA_SDF_OUT").expect("QA_SDF_OUT");
    let cases: Value =
        serde_json::from_str(&std::fs::read_to_string(&cases_path).expect("read cases"))
            .expect("parse cases");
    let volume = build_volume(&cases, std::path::Path::new(&oct_dir));
    let mut scenes_out = Vec::new();
    for scene in cases["scenes"].as_array().expect("scenes") {
        let arrays = &scene["model"]["arrays"];
        let geom_type: Vec<i32> = arrays["geom_type"]
            .as_array()
            .expect("geom_type")
            .iter()
            .map(i32_of)
            .collect();
        let geom_size: Vec<[f32; 3]> = arrays["geom_size"]
            .as_array()
            .expect("geom_size")
            .iter()
            .map(vec_f32_3)
            .collect();
        let geom_aabb: Vec<([f32; 3], [f32; 3])> = arrays["geom_aabb"]
            .as_array()
            .expect("geom_aabb")
            .iter()
            .map(|pair| {
                (
                    vec_f32_3(&pair[0]),
                    vec_f32_3(&pair[1]),
                )
            })
            .collect();
        // 本实验固定 geom0=box、geom1=sdf；root = mesh_octadr[sdf 的 dataid]。
        let sdf_index = geom_type
            .iter()
            .position(|&kind| kind == super::GEOM_SDF)
            .expect("sdf geom");
        let box_index = (0..geom_type.len())
            .find(|&index| geom_type[index] == super::GEOM_BOX)
            .expect("box geom");
        let dataid = arrays["geom_dataid"][sdf_index]
            .as_i64()
            .expect("geom_dataid") as usize;
        let root = at_index(&arrays["mesh_octadr"], dataid) as usize;
        // 覆盖 root：VolumeData 字段公开，重设根节点与根 AABB。
        let (center, half_size) = volume.oct_aabb[root];
        let volume = VolumeData {
            center,
            half_size,
            oct_aabb: volume.oct_aabb.clone(),
            oct_child: volume.oct_child.clone(),
            oct_coeff: volume.oct_coeff.clone(),
            root,
        };
        let initpoints = scene["sdf_initpoints"].as_u64().expect("initpoints") as usize;
        let iterations = scene["sdf_iterations"].as_u64().expect("iterations") as usize;
        let nworld = scene["worlds"].as_u64().expect("worlds") as usize;
        let rel_mat_warp = scene["rel_mat_warp"].as_array().expect("rel_mat_warp");
        let xpos = scene["output"]["geom_xpos"].as_array().expect("geom_xpos");
        let xmat = scene["output"]["geom_xmat"].as_array().expect("geom_xmat");
        let shape1 = SdfShape::new(SdfType::Box, geom_size[box_index]);
        let shape2 = SdfShape::new(SdfType::Sdf, geom_size[sdf_index]).with_volume(&volume);
        let mut worlds_out = Vec::new();
        for world in 0..nworld {
            let pos_rows: Vec<[f32; 3]> = xpos[world]
                .as_array()
                .expect("pos row")
                .iter()
                .map(vec_f32_3)
                .collect();
            let mat_flat = vec_f32(&xmat[world]);
            let mat_at = |index: usize| -> [f32; 9] {
                let mut out = [0.0_f32; 9];
                out.copy_from_slice(&mat_flat[index * 9..index * 9 + 9]);
                out
            };
            let pair = SdfPair {
                shape1,
                shape2,
                pos1: pos_rows[box_index],
                rot1: mat_at(box_index),
                pos2: pos_rows[sdf_index],
                rot2: mat_at(sdf_index),
                aabb1: geom_aabb[box_index],
                aabb2: geom_aabb[sdf_index],
                sdf_initpoints: initpoints,
                sdf_iterations: iterations,
            };
            let product = sdf_pair_contacts(&pair);
            let rel_computed = upstream_relative_matrix(&pair.rot1, &pair.rot2);
            let corrected_computed = corrected_contacts(&pair, rel_computed);
            let rel_warp = mat9(&rel_mat_warp[world]);
            let corrected_warp = corrected_contacts(&pair, rel_warp);
            let rel_fma = upstream_relative_matrix_fma(&pair.rot1, &pair.rot2);
            let corrected_fma = corrected_contacts(&pair, rel_fma);
            let rel_product = relative_matrix(&pair.rot1, &pair.rot2);
            let product_rows: Vec<Value> = product.iter().map(contact_json).collect();
            let computed_rows: Vec<Value> = corrected_computed.iter().map(contact_json).collect();
            let warp_rows: Vec<Value> = corrected_warp.iter().map(contact_json).collect();
            let fma_rows: Vec<Value> = corrected_fma.iter().map(contact_json).collect();
            worlds_out.push(json!({
                "world": world,
                "rot1": pair.rot1.to_vec(),
                "rot2": pair.rot2.to_vec(),
                "rel_mat_product": rel_product.to_vec(),
                "rel_mat_upstream_computed": rel_computed.to_vec(),
                "rel_mat_upstream_fma": rel_fma.to_vec(),
                "rel_mat_warp": rel_warp.to_vec(),
                "rel_mat_fma_matches_warp": rel_fma
                    .iter()
                    .zip(rel_warp.iter())
                    .all(|(x, y)| x.to_bits() == y.to_bits()),
                "product_contacts": product_rows,
                "corrected_computed_relmat_contacts": computed_rows,
                "corrected_warp_relmat_contacts": warp_rows,
                "corrected_fma_relmat_contacts": fma_rows,
            }));
        }
        scenes_out.push(json!({
            "name": scene["name"],
            "nworld": nworld,
            "initpoints": initpoints,
            "iterations": iterations,
            "box_index": box_index,
            "sdf_index": sdf_index,
            "worlds": worlds_out,
        }));
        println!("probe_sdf {} ok", scene["name"].as_str().unwrap_or("?"));
    }
    let out = json!({
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-hfield-host-independent-check",
        "kind": "qa-product-probe-sdf-relative-rotation",
        "note": "product=sdf_pair_contacts 原样；corrected=QA 复制上游窄相结构、rel_mat 用上游语义、其余调产品公开件",
        "scenes": scenes_out,
    });
    std::fs::write(&out_path, serde_json::to_string(&out).expect("serialize")).expect("write");
}
