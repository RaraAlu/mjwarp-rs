//! QA 独立 oracle：按 pinned 上游 `mujoco_warp/_src/collision_convex.py` 的
//! `_hfield_filter` + `ccd_hfield_kernel` 语义**另写一遍** hfield 宿主驱动：
//! 粗筛（box-sphere / 支撑紧 AABB / box-box）→ 子网格 cell 范围 → prism 六顶点扫描
//! → 四接触选择 + `MIN_DIST_TO_NEXT_CONTACT=1e-3` + `MJ_MAXCONPAIR=50` 溢出。
//!
//! 独立性：
//! - filter / 子网格 / prism / 接触装配 / 选择 / 去重 / 溢出全部由本文件独立实现
//!   （不 include 产品 `src/physics/collision/hfield/**`）；
//! - 唯一的共享件是窄相 `ccd()`（产品公开 API，G2 已单独独立复核、另有证据），
//!   这样差异只会落在 G4 自己的逻辑上。
//!
//! 输入：`qa-upstream-cases.json`（QA 自采上游场景，含模型数组与 f32 运动学）。
//! 输出：`qa-oracle-output.json`（逐场景逐 world 的 filter 判定/bounds、候选 cell 数、
//! prism 计数、溢出、四接触、以及全部缓存接触的重数统计）。

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use mjwarp_rs::physics::{CcdOptions, Geom, MeshView, ccd, geom_type};
use serde_json::{Value, json};

// ---------------- f32 原语（表达式顺序对齐上游） ----------------

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn cw_mul3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2]]
}
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn length3(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
fn fused3(a0: f32, a1: f32, a2: f32, v: [f32; 3]) -> f32 {
    a0 * v[0] + a1 * v[1] + a2 * v[2]
}
fn mul_mat3_vec(m: &[f32; 9], v: [f32; 3]) -> [f32; 3] {
    [
        fused3(m[0], m[1], m[2], v),
        fused3(m[3], m[4], m[5], v),
        fused3(m[6], m[7], m[8], v),
    ]
}
fn transpose_mul_mat3_vec(m: &[f32; 9], v: [f32; 3]) -> [f32; 3] {
    [
        fused3(m[0], m[3], m[6], v),
        fused3(m[1], m[4], m[7], v),
        fused3(m[2], m[5], m[8], v),
    ]
}
fn normalize3(a: [f32; 3]) -> [f32; 3] {
    let n = length3(a);
    if n > 0.0 {
        [a[0] / n, a[1] / n, a[2] / n]
    } else {
        [0.0; 3]
    }
}
fn make_frame(normal: [f32; 3]) -> [f32; 9] {
    let a = normalize3(normal);
    let selector = if -0.5 < a[1] && a[1] < 0.5 {
        [0.0_f32, 1.0, 0.0]
    } else {
        [0.0_f32, 0.0, 1.0]
    };
    let mut b = sub3(selector, scale3(a, dot3(a, selector)));
    b = normalize3(b);
    if length3(a) == 0.0 {
        b = [0.0; 3];
    }
    let c = cross3(a, b);
    [a[0], a[1], a[2], b[0], b[1], b[2], c[0], c[1], c[2]]
}

// 旋转约定：row-major，rot = a^T b（hfield 帧 <- world，hf_mat 为 world <- hfield）。
fn relative_rot(a: &[f32; 9], b: &[f32; 9]) -> [f32; 9] {
    let mut out = [0.0_f32; 9];
    for row in 0..3 {
        for column in 0..3 {
            out[row * 3 + column] =
                a[row] * b[column] + a[3 + row] * b[3 + column] + a[6 + row] * b[6 + column];
        }
    }
    out
}

// ---------------- 对侧 geom 的支撑函数（filter 紧 AABB 用） ----------------

struct OtherGeom {
    pos: [f32; 3],
    rot: [f32; 9],
    size: [f32; 3],
    kind: i32,
    /// MESH: 全局顶点数组与本 geom 的区间。
    verts: Vec<[f32; 3]>,
    vertadr: i32,
    vertnum: i32,
}
fn support(geom: &OtherGeom, dir: [f32; 3]) -> [f32; 3] {
    match geom.kind {
        geom_type::SPHERE => add3(geom.pos, scale3(dir, geom.size[0])),
        geom_type::BOX => {
            let local = transpose_mul_mat3_vec(&geom.rot, dir);
            let tmp = [
                if local[0] < 0.0 { -1.0 } else { 1.0 },
                if local[1] < 0.0 { -1.0 } else { 1.0 },
                if local[2] < 0.0 { -1.0 } else { 1.0 },
            ];
            add3(
                mul_mat3_vec(&geom.rot, cw_mul3(tmp, geom.size)),
                geom.pos,
            )
        }
        geom_type::CAPSULE => {
            let local = transpose_mul_mat3_vec(&geom.rot, dir);
            let mut res = scale3(local, geom.size[0]);
            res[2] += if local[2] < 0.0 {
                -geom.size[1]
            } else {
                geom.size[1]
            };
            add3(mul_mat3_vec(&geom.rot, res), geom.pos)
        }
        geom_type::ELLIPSOID => {
            let local = transpose_mul_mat3_vec(&geom.rot, dir);
            let mut res = cw_mul3(local, geom.size);
            res = normalize3(res);
            res = cw_mul3(res, geom.size);
            add3(mul_mat3_vec(&geom.rot, res), geom.pos)
        }
        geom_type::CYLINDER => {
            let local = transpose_mul_mat3_vec(&geom.rot, dir);
            let mut res = [0.0_f32; 3];
            let d = (local[0] * local[0] + local[1] * local[1]).sqrt();
            if d > 1.0e-15 {
                let scl = geom.size[0] / d;
                res[0] = local[0] * scl;
                res[1] = local[1] * scl;
            }
            res[2] = if local[2] >= 0.0 {
                geom.size[1]
            } else {
                -geom.size[1]
            };
            add3(mul_mat3_vec(&geom.rot, res), geom.pos)
        }
        geom_type::MESH => {
            // QA 场景 mesh 顶点数 < 10：走上游穷举分支。
            let local = transpose_mul_mat3_vec(&geom.rot, dir);
            let mut best = [0.0_f32; 3];
            let mut best_dist = -1.0e30_f32;
            for i in 0..geom.vertnum {
                let vert = geom.verts[(geom.vertadr + i) as usize];
                let dist = dot3(vert, local);
                if dist > best_dist {
                    best_dist = dist;
                    best = vert;
                }
            }
            add3(mul_mat3_vec(&geom.rot, best), geom.pos)
        }
        other => panic!("oracle: 未实现的支撑类型 {other}"),
    }
}

// ---------------- 上游 `_hfield_filter` ----------------

#[derive(Clone, Copy, Debug)]
struct Bounds {
    xmin: f32,
    xmax: f32,
    ymin: f32,
    ymax: f32,
    zmin: f32,
    zmax: f32,
}

struct Hfield {
    size: [f32; 4],
    nrow: i32,
    ncol: i32,
    adr: i32,
    data: Vec<f32>,
}

impl Hfield {
    fn height(&self, row: i32, col: i32) -> f32 {
        self.data[(self.adr + row * self.ncol + col) as usize]
    }
}

const MJ_MAXVAL: f32 = 1.0e10;
const MJ_MAXCONPAIR: usize = 50;
const MIN_DIST_TO_NEXT_CONTACT: f32 = 1.0e-3;

fn hfield_filter(hfield: &Hfield, geom: &OtherGeom, rbound: f32, margin: f32) -> Option<Bounds> {
    let size1 = hfield.size;
    let pos = geom.pos;
    for axis in 0..2 {
        if size1[axis] < pos[axis] - rbound - margin || -size1[axis] > pos[axis] + rbound + margin
        {
            return None;
        }
    }
    if size1[2] < pos[2] - rbound - margin {
        return None;
    }
    if -size1[3] > pos[2] + rbound + margin {
        return None;
    }
    let xmax = support(geom, [1.0, 0.0, 0.0])[0];
    let xmin = support(geom, [-1.0, 0.0, 0.0])[0];
    let ymax = support(geom, [0.0, 1.0, 0.0])[1];
    let ymin = support(geom, [0.0, -1.0, 0.0])[1];
    let zmax = support(geom, [0.0, 0.0, 1.0])[2];
    let zmin = support(geom, [0.0, 0.0, -1.0])[2];
    if xmin - margin > size1[0]
        || xmax + margin < -size1[0]
        || ymin - margin > size1[1]
        || ymax + margin < -size1[1]
        || zmin - margin > size1[2]
        || zmax + margin < -size1[3]
    {
        return None;
    }
    Some(Bounds {
        xmin,
        xmax,
        ymin,
        ymax,
        zmin,
        zmax,
    })
}

// ---------------- 上游 `ccd_hfield_kernel` 的独立重写 ----------------

struct WorldOut {
    filter_accept: bool,
    bounds: Option<Bounds>,
    candidate_cells: usize,
    height_skipped: usize,
    noncontact: usize,
    prism_count: usize,
    overflow: bool,
    contacts: Vec<([f32; 3], [f32; 9], f32)>,
    cached: Vec<(f32, [f32; 3], [f32; 3])>,
}

fn run_world(
    hfield: &Hfield,
    geom2: &mut OtherGeom,
    rbound: f32,
    margin: f32,
    gap: f32,
    options: CcdOptions,
    hf_mat: &[f32; 9],
    hf_pos: [f32; 3],
) -> WorldOut {
    let geom2_margin_zero = OtherGeom {
        pos: geom2.pos,
        rot: geom2.rot,
        size: geom2.size,
        kind: geom2.kind,
        verts: geom2.verts.clone(),
        vertadr: geom2.vertadr,
        vertnum: geom2.vertnum,
    };
    let bounds = hfield_filter(hfield, &geom2_margin_zero, rbound, margin);
    let mut out = WorldOut {
        filter_accept: bounds.is_some(),
        bounds,
        candidate_cells: 0,
        height_skipped: 0,
        noncontact: 0,
        prism_count: 0,
        overflow: false,
        contacts: Vec::new(),
        cached: Vec::new(),
    };
    let Some(bounds) = bounds else {
        return out;
    };

    let size = hfield.size;
    let nrow = hfield.nrow;
    let ncol = hfield.ncol;
    let x_scale = 0.5 * (ncol as f32 - 1.0) / size[0];
    let y_scale = 0.5 * (nrow as f32 - 1.0) / size[1];
    let cmin = (((bounds.xmin + size[0]) * x_scale).floor() as i32).max(0);
    let cmax = (((bounds.xmax + size[0]) * x_scale).ceil() as i32).min(ncol - 1);
    let rmin = (((bounds.ymin + size[1]) * y_scale).floor() as i32).max(0);
    let rmax = (((bounds.ymax + size[1]) * y_scale).ceil() as i32).min(nrow - 1);

    let dx = 2.0 * size[0] / (ncol as f32 - 1.0);
    let dy = 2.0 * size[1] / (nrow as f32 - 1.0);
    let dr = [1_i32, 0_i32];

    let mut prism = [[0.0_f32; 3]; 6];
    prism[0][2] = -size[3];
    prism[1][2] = -size[3];
    prism[2][2] = -size[3];

    // 产品 Geom（仅供窄相 ccd 使用）；独立 crate 内复用公开 API。
    let mesh_guard = geom2.verts.clone();
    let empty_i: [i32; 0] = [];
    let empty_v: [[f32; 3]; 0] = [];
    let empty_n: [[f32; 3]; 0] = [];

    let mut cached_dist: Vec<f32> = Vec::with_capacity(MJ_MAXCONPAIR);
    let mut cached_pos: Vec<[f32; 3]> = Vec::with_capacity(MJ_MAXCONPAIR);
    let mut cached_normal: Vec<[f32; 3]> = Vec::with_capacity(MJ_MAXCONPAIR);
    let mut min_dist = MJ_MAXVAL;
    let mut min_normal = [MJ_MAXVAL; 3];
    let mut min_pos = [MJ_MAXVAL; 3];
    let mut min_id: i32 = -1;
    let mut count = 0_usize;

    'scan: for row in rmin..rmax {
        if out.overflow {
            break;
        }
        for init_index in 0..2 {
            let x = dx * cmin as f32 - size[0];
            let y = dy * (row + dr[init_index]) as f32 - size[1];
            let z = hfield.height(row + dr[init_index], cmin) * size[2] + margin;
            prism[0] = prism[1];
            prism[1] = prism[2];
            prism[3] = prism[4];
            prism[4] = prism[5];
            prism[2][0] = x;
            prism[5][0] = x;
            prism[2][1] = y;
            prism[5][1] = y;
            prism[5][2] = z;
        }
        for column in (cmin + 1)..=cmax {
            for i in 0..2 {
                out.candidate_cells += 1;
                if count >= MJ_MAXCONPAIR {
                    out.overflow = true;
                    break 'scan;
                }
                let x = dx * column as f32 - size[0];
                let y = dy * (row + dr[i]) as f32 - size[1];
                let z = hfield.height(row + dr[i], column) * size[2] + margin;
                prism[0] = prism[1];
                prism[1] = prism[2];
                prism[3] = prism[4];
                prism[4] = prism[5];
                prism[2][0] = x;
                prism[5][0] = x;
                prism[2][1] = y;
                prism[5][1] = y;
                prism[5][2] = z;

                if prism[3][2] < bounds.zmin
                    && prism[4][2] < bounds.zmin
                    && prism[5][2] < bounds.zmin
                {
                    out.height_skipped += 1;
                    continue;
                }

                let mut sum = [0.0_f32; 3];
                for vertex in &prism {
                    sum = add3(sum, *vertex);
                }
                let center = scale3(sum, 1.0 / 6.0);

                let mut geom1 = Geom::simple([0.0; 3], IDENTITY, [0.0; 3], 0.0);
                geom1.polyvert = prism;

                let mut geom2_product = Geom::simple(geom2.pos, geom2.rot, geom2.size, margin);
                let mesh_view;
                if geom2.kind == geom_type::MESH {
                    mesh_view = MeshView {
                        vert: &mesh_guard,
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
                    geom2_product.vertadr = geom2.vertadr;
                    geom2_product.vertnum = geom2.vertnum;
                    geom2_product.graphadr = -1;
                    geom2_product.mesh = Some(&mesh_view);
                }
                let result = ccd(
                    &geom1,
                    &geom2_product,
                    geom_type::HFIELD,
                    geom2.kind,
                    center,
                    geom2.pos,
                    options,
                );
                if result.ncontact == 0 || result.dist >= margin + gap {
                    out.noncontact += 1;
                    continue;
                }
                let pos_local = scale3(add3(result.x1, result.x2), 0.5);
                let pos = add3(mul_mat3_vec(hf_mat, pos_local), hf_pos);
                let frame_local = make_frame(sub3(result.x1, result.x2));
                let normal_local = [frame_local[0], frame_local[1], frame_local[2]];
                let normal = mul_mat3_vec(hf_mat, normal_local);
                cached_dist.push(result.dist);
                cached_pos.push(pos);
                cached_normal.push(normal);
                if result.dist < min_dist {
                    min_dist = result.dist;
                    min_normal = normal;
                    min_pos = pos;
                    min_id = count as i32;
                }
                count += 1;
            }
        }
    }
    out.prism_count = count;
    out.cached = (0..count)
        .map(|i| (cached_dist[i], cached_pos[i], cached_normal[i]))
        .collect();
    if count == 0 {
        return out;
    }
    let min_id = min_id as usize;
    out.contacts
        .push((min_pos, make_frame(min_normal), cached_dist[min_id]));

    // contact 1：离最小距离接触点最远
    let mut id1: i32 = -1;
    let mut dist1 = -MJ_MAXVAL;
    for index in 0..count {
        if index == min_id {
            continue;
        }
        let dist = length3(sub3(cached_pos[index], min_pos));
        if dist > dist1 {
            id1 = index as i32;
            dist1 = dist;
        }
    }
    if id1 == -1 || (dist1 > 0.0 && dist1 < MIN_DIST_TO_NEXT_CONTACT) {
        return out;
    }
    let pos1 = cached_pos[id1 as usize];
    out.contacts.push((
        pos1,
        make_frame(cached_normal[id1 as usize]),
        cached_dist[id1 as usize],
    ));

    let dist_min1 = cross3(min_normal, sub3(min_pos, pos1));
    let mut id2: i32 = -1;
    let mut dist_12 = -MJ_MAXVAL;
    for index in 0..count {
        if index == min_id || index == id1 as usize {
            continue;
        }
        let dist = dot3(sub3(cached_pos[index], min_pos), dist_min1).abs();
        if dist > dist_12 {
            id2 = index as i32;
            dist_12 = dist;
        }
    }
    if id2 == -1 || (dist_12 > 0.0 && dist_12 < MIN_DIST_TO_NEXT_CONTACT) {
        return out;
    }
    let pos2 = cached_pos[id2 as usize];
    out.contacts.push((
        pos2,
        make_frame(cached_normal[id2 as usize]),
        cached_dist[id2 as usize],
    ));

    let vec_min2 = cross3(min_normal, sub3(min_pos, pos2));
    let vec_12 = cross3(min_normal, sub3(pos1, pos2));
    let mut id3: i32 = -1;
    let mut dist3 = -MJ_MAXVAL;
    for index in 0..count {
        if index == min_id || index == id1 as usize || index == id2 as usize {
            continue;
        }
        let point = cached_pos[index];
        let dist = dot3(sub3(point, min_pos), vec_min2).abs()
            + dot3(sub3(pos1, point), vec_12).abs();
        if dist > dist3 {
            id3 = index as i32;
            dist3 = dist;
        }
    }
    if id3 == -1 || (dist3 > 0.0 && dist3 < MIN_DIST_TO_NEXT_CONTACT) {
        return out;
    }
    out.contacts.push((
        cached_pos[id3 as usize],
        make_frame(cached_normal[id3 as usize]),
        cached_dist[id3 as usize],
    ));
    out
}

const IDENTITY: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

// ---------------- JSON 读写 ----------------

fn f32_of(v: &Value) -> f32 {
    v.as_f64().expect("number") as f32
}
fn i32_of(v: &Value) -> i32 {
    v.as_i64().expect("int") as i32
}
fn vec_f32(v: &Value) -> Vec<f32> {
    v.as_array().expect("array").iter().map(f32_of).collect()
}
fn vec_f32_3(v: &Value) -> [f32; 3] {
    let a = vec_f32(v);
    [a[0], a[1], a[2]]
}
fn vec_f32_9(v: &Value) -> [f32; 9] {
    let a = vec_f32(v);
    let mut out = [0.0_f32; 9];
    out.copy_from_slice(&a);
    out
}
fn first_i32(v: &Value) -> i32 {
    match v {
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

fn bits3(v: [f32; 3]) -> [u32; 3] {
    [v[0].to_bits(), v[1].to_bits(), v[2].to_bits()]
}

fn main() {
    let mut args = std::env::args().skip(1);
    let input = PathBuf::from(args.next().expect("cases json"));
    let output = PathBuf::from(args.next().expect("output json"));
    let text = fs::read_to_string(&input).expect("read cases");
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
        let geom_type_arr: Vec<i32> = arrays["geom_type"]
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

        let hfield_size4: [f32; 4] = {
            let v = vec_f32(&arrays["hfield_size"]);
            let mut out = [0.0_f32; 4];
            out.copy_from_slice(&v);
            out
        };
        let hfield = Hfield {
            size: hfield_size4,
            nrow: first_i32(&arrays["hfield_nrow"]),
            ncol: first_i32(&arrays["hfield_ncol"]),
            adr: first_i32(&arrays["hfield_adr"]),
            data: vec_f32(&arrays["hfield_data"]),
        };

        let hf_index = geom_type_arr
            .iter()
            .position(|&t| t == geom_type::HFIELD)
            .expect("hfield geom");
        let other_index = (0..geom_type_arr.len())
            .find(|&i| i != hf_index)
            .expect("other geom");
        let mesh_verts: Vec<[f32; 3]> = match &mesh_arrays["mesh_vert"] {
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
            let rot2_hf = relative_rot(&hf_mat, &other_rot);
            let mut other = OtherGeom {
                pos: pos2_hf,
                rot: rot2_hf,
                size: geom_size[other_index],
                kind: geom_type_arr[other_index],
                verts: if geom_type_arr[other_index] == geom_type::MESH {
                    mesh_verts.clone()
                } else {
                    Vec::new()
                },
                vertadr: if geom_type_arr[other_index] == geom_type::MESH {
                    mesh_vertadr
                } else {
                    -1
                },
                vertnum: if geom_type_arr[other_index] == geom_type::MESH {
                    mesh_vertnum
                } else {
                    -1
                },
            };
            let out = run_world(
                &hfield,
                &mut other,
                geom_rbound[other_index],
                geom_margin[hf_index] + geom_margin[other_index],
                geom_gap[hf_index] + geom_gap[other_index],
                options,
                &hf_mat,
                hf_pos,
            );
            // 缓存接触的重数统计（用于 1e-3 去重/重复点分析）。
            let mut distinct_pos = BTreeSet::new();
            let mut distinct_full = BTreeSet::new();
            for (dist, pos, normal) in &out.cached {
                distinct_pos.insert((dist.to_bits(), bits3(*pos)));
                distinct_full.insert((dist.to_bits(), bits3(*pos), bits3(*normal)));
            }
            let contacts: Vec<Value> = out
                .contacts
                .iter()
                .map(|(pos, frame, dist)| {
                    json!({
                        "dist": dist,
                        "pos": [pos[0], pos[1], pos[2]],
                        "frame": frame.to_vec(),
                    })
                })
                .collect();
            let bounds_json = match &out.bounds {
                None => Value::Null,
                Some(b) => json!({
                    "xmin": b.xmin, "xmax": b.xmax,
                    "ymin": b.ymin, "ymax": b.ymax,
                    "zmin": b.zmin, "zmax": b.zmax,
                }),
            };
            worlds_out.push(json!({
                "world": world,
                "filter_accept": out.filter_accept,
                "bounds": bounds_json,
                "candidate_cells": out.candidate_cells,
                "height_skipped": out.height_skipped,
                "noncontact": out.noncontact,
                "prism_count": out.prism_count,
                "overflow": out.overflow,
                "cached_count": out.cached.len(),
                "cached_distinct_pos": distinct_pos.len(),
                "cached_distinct_full": distinct_full.len(),
                "contacts": contacts,
            }));
        }
        scenes_out.push(json!({
            "name": scene["name"],
            "nworld": nworld,
            "hf_index": hf_index,
            "other_index": other_index,
            "hfield": {
                "size": hfield_size4.to_vec(),
                "nrow": hfield.nrow,
                "ncol": hfield.ncol,
                "adr": hfield.adr,
            },
            "worlds": worlds_out,
        }));
        println!(
            "oracle {} -> worlds {} ok",
            scene["name"].as_str().unwrap_or("?"),
            nworld
        );
    }

    let out = json!({
        "schema": 1,
        "task_id": "2026-10-25-g10-p5-hfield-host-independent-check",
        "kind": "qa-independent-oracle-hfield-host",
        "criterion": "独立重写上游 hfield 宿主语义；窄相复用产品公开 ccd()（G2 已复核）",
        "scenes": scenes_out,
    });
    fs::write(&output, serde_json::to_string(&out).expect("serialize")).expect("write");
    println!("wrote {}", output.display());
}
