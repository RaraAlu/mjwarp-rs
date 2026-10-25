//! QA 产品侧探针（只在临时 `git worktree` 的 1f882361 副本内编译，不进产品树）。
//!
//! 目的：绕过 `physics::collision` 私有可见性，**直接调用产品** G4 代码
//! （`driver::hfield_pair_contacts` 与 `filter::hfield_filter`），把产品输出落 JSON，
//! 供 QA 与「自写 oracle / 上游 f32」三方比对。
//!
//! 输入：QA_CASES=qa-upstream-cases.json；输出：QA_OUT=qa-product-output.json。
//! 现场构造与交付 `hfield/tests.rs` 相同（geom2 → hfield 帧、margin/gap 取两侧之和），
//! 但本探针不复用交付断言，只导出产品行为。

use super::super::gjk::math::{sub3, transpose_mul_mat3_vec};
use super::super::gjk::{CcdOptions, Geom, MeshView};
use super::driver::relative_matrix;
use super::filter::hfield_filter;
use super::*;
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
fn first_i32(value: &Value) -> i32 {
    match value {
        Value::Array(items) => {
            if items.is_empty() {
                -1
            } else {
                i32_of(&items[0])
            }
        }
        other => i32_of(other),
    }
}

fn parse_contact(contact: &HfieldContact) -> Value {
    json!({
        "dist": contact.dist,
        "pos": [contact.pos[0], contact.pos[1], contact.pos[2]],
        "frame": contact.frame.to_vec(),
    })
}

// QA 自写的行优先矩阵原语：geom2 在 hfield 帧内的旋转 = hf_mat^T · other_rot
// （上游 `geom2.rot = hf_matT @ geom2.rot` 的逐元素展开）。
fn mul3(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
    let mut out = [0.0_f32; 9];
    for row in 0..3 {
        for column in 0..3 {
            out[row * 3 + column] = a[row * 3] * b[column]
                + a[row * 3 + 1] * b[3 + column]
                + a[row * 3 + 2] * b[6 + column];
        }
    }
    out
}

fn transpose3(m: &[f32; 9]) -> [f32; 9] {
    [
        m[0], m[3], m[6], m[1], m[4], m[7], m[2], m[5], m[8],
    ]
}

fn qa_relative_rot(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
    mul3(&transpose3(a), b)
}

fn rot_z(angle: f32) -> [f32; 9] {
    let (s, c) = (angle.sin(), angle.cos());
    [c, -s, 0.0, s, c, 0.0, 0.0, 0.0, 1.0]
}

fn rot_x(angle: f32) -> [f32; 9] {
    let (s, c) = (angle.sin(), angle.cos());
    [1.0, 0.0, 0.0, 0.0, c, -s, 0.0, s, c]
}

#[test]
fn qa_probe_cases() {
    let cases_path = std::env::var("QA_CASES").expect("QA_CASES");
    let out_path = std::env::var("QA_OUT").expect("QA_OUT");
    let text = std::fs::read_to_string(&cases_path).expect("read cases");
    let root: Value = serde_json::from_str(&text).expect("parse cases");
    let mut scenes_out = Vec::new();
    for scene in root["scenes"].as_array().expect("scenes") {
        let arrays = &scene["model"]["arrays"];
        let mesh_arrays = &scene["model"]["mesh_arrays"];
        let opt = &scene["model"]["opt"];
        let options = CcdOptions {
            tolerance: f32_of(&opt["ccd_tolerance"]),
            cutoff: 0.0,
            gjk_iterations: f32_of(&opt["ccd_iterations"]) as usize,
            epa_iterations: f32_of(&opt["ccd_iterations"]) as usize,
        };
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
        let geom_margin = vec_f32(&arrays["geom_margin"]);
        let geom_gap = vec_f32(&arrays["geom_gap"]);
        let geom_rbound = vec_f32(&arrays["geom_rbound"]);
        let hfield_size4 = {
            let v = vec_f32(&arrays["hfield_size"]);
            let mut out = [0.0_f32; 4];
            out.copy_from_slice(&v);
            out
        };
        let hfield = HfieldData::new(
            hfield_size4,
            first_i32(&arrays["hfield_nrow"]),
            first_i32(&arrays["hfield_ncol"]),
            first_i32(&arrays["hfield_adr"]),
            vec_f32(&arrays["hfield_data"]),
        )
        .expect("HfieldData");
        let hf_index = geom_type
            .iter()
            .position(|&kind| kind == super::super::gjk::geom_type::HFIELD)
            .expect("hfield geom");
        let other_index = (0..geom_type.len())
            .find(|&index| index != hf_index)
            .expect("other geom");
        // mesh 拓扑（只有 QA 的 mesh 场景用；顶点数 < 10 走穷举分支）。
        let mesh_vert: Vec<[f32; 3]> = match &mesh_arrays["mesh_vert"] {
            Value::Array(items) => items.iter().map(vec_f32_3).collect(),
            _ => Vec::new(),
        };
        let mesh_vertadr = mesh_arrays
            .get("mesh_vertadr")
            .map(first_i32)
            .unwrap_or(-1);
        let mesh_vertnum = mesh_arrays
            .get("mesh_vertnum")
            .map(first_i32)
            .unwrap_or(-1);
        let empty_i: [i32; 0] = [];
        let empty_n: [[f32; 3]; 0] = [];
        let mesh_view = MeshView {
            vert: &mesh_vert,
            graph: &empty_i,
            polyadr: -1,
            polynormal: &empty_n,
            polyvertadr: &empty_i,
            polyvertnum: &empty_i,
            polyvert: &empty_i,
            polymapadr: &empty_i,
            polymapnum: &empty_i,
            polymap: &empty_i,
        };

        let nworld = scene["nworld"].as_i64().expect("nworld") as usize;
        let xpos = scene["output"]["geom_xpos"].as_array().expect("xpos");
        let xmat = scene["output"]["geom_xmat"].as_array().expect("xmat");
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
            let hf_pos = pos_rows[hf_index];
            let hf_mat = mat_at(hf_index);
            let other_pos = pos_rows[other_index];
            let other_rot = mat_at(other_index);
            let pos2_hf = transpose_mul_mat3_vec(&hf_mat, sub3(other_pos, hf_pos));
            let rot2_hf = qa_relative_rot(&hf_mat, &other_rot);
            let margin = geom_margin[hf_index] + geom_margin[other_index];
            let gap = geom_gap[hf_index] + geom_gap[other_index];

            let mut geom2 = Geom::simple(pos2_hf, rot2_hf, geom_size[other_index], 0.0);
            if geom_type[other_index] == super::super::gjk::geom_type::MESH {
                geom2.vertadr = mesh_vertadr;
                geom2.vertnum = mesh_vertnum;
                geom2.graphadr = -1;
                geom2.mesh = Some(&mesh_view);
            }

            // filter 判定与 bounds：直接调产品 filter（driver 内部同样调用）。
            let bounds = hfield_filter(
                &hfield,
                &geom2,
                geom_type[other_index],
                geom_rbound[other_index],
                margin,
            );
            let bounds_json = match bounds {
                None => Value::Null,
                Some(b) => json!({
                    "xmin": b.xmin, "xmax": b.xmax,
                    "ymin": b.ymin, "ymax": b.ymax,
                    "zmin": b.zmin, "zmax": b.zmax,
                }),
            };

            let input = HfieldInput {
                hfield: &hfield,
                geom2,
                geomtype2: geom_type[other_index],
                other_rbound: geom_rbound[other_index],
                margin,
                gap,
                options,
                hf_mat,
                hf_pos,
            };
            let output = hfield_pair_contacts(&input);
            let contacts: Vec<Value> = output.contacts.iter().map(parse_contact).collect();
            worlds_out.push(json!({
                "world": world,
                "filter_accept": bounds_json != Value::Null,
                "bounds": bounds_json,
                "prism_count": output.prism_count,
                "overflow": output.overflow,
                "contacts": contacts,
            }));
        }
        scenes_out.push(json!({
            "name": scene["name"],
            "nworld": nworld,
            "hf_index": hf_index,
            "other_index": other_index,
            "worlds": worlds_out,
        }));
        println!("probe {} ok", scene["name"].as_str().unwrap_or("?"));
    }
    // 交付测试 helper `driver::relative_matrix` 旋转约定核对：
    // 上游 geom2.rot = hf_mat^T · other_rot（行优先）。用两个不对易的旋转暴露转置问题。
    let a = rot_z(0.7);
    let b = rot_x(0.4);
    let expected = qa_relative_rot(&a, &b);
    let delivered = relative_matrix(&a, &b);
    let expected_transposed = transpose3(&expected);
    let mut max_abs_diff = 0.0_f32;
    let mut bit_equal = true;
    let mut equals_transpose = true;
    for index in 0..9 {
        max_abs_diff = max_abs_diff.max((delivered[index] - expected[index]).abs());
        if delivered[index].to_bits() != expected[index].to_bits() {
            bit_equal = false;
        }
        if delivered[index].to_bits() != expected_transposed[index].to_bits() {
            equals_transpose = false;
        }
    }
    let helper_check = json!({
        "a": a.to_vec(),
        "b": b.to_vec(),
        "expected_upstream_aTb": expected.to_vec(),
        "delivered_relative_matrix": delivered.to_vec(),
        "bit_equal": bit_equal,
        "delivered_equals_transpose_of_expected": equals_transpose,
        "max_abs_diff": max_abs_diff,
        "identity_case_bit_equal": {
            "delivered(I,I)": relative_matrix(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
                                              &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]).to_vec(),
        },
    });
    let out = json!({
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-hfield-host-independent-check",
        "kind": "qa-product-probe-hfield-host",
        "product_commit": "1f882361",
        "helper_rotation_check": helper_check,
        "scenes": scenes_out,
    });
    std::fs::write(&out_path, serde_json::to_string(&out).expect("serialize")).expect("write");
}
