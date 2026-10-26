//! B3 分支覆盖测试的 case 定义、mesh 数据与分支命中谓词复算（被
//! `g10_p5_primitive_branches_cuda.rs` 以 `#[path]` 引入；不作为独立测试目标）。

use crate::primitive_compare::flatten_f32;

pub(crate) const MJ_MINVAL: f32 = 1.0e-15;

pub(crate) struct BranchCase {
    pub(crate) name: &'static str,
    /// 两个 geom 的 `geom_size`（f32；来自 `fixtures/g10-p5-primitive-branches/<name>.xml`）
    pub(crate) geom_size: [[f32; 3]; 2],
    /// 两个 geom 的 `mjtGeom` 类型（调用方顺序 = B3 捕获的类型序）
    pub(crate) geom_type: [i32; 2],
    /// 是否 (PLANE, MESH)：mesh 顶点/图按 `pentagon_prism.obj`（10 顶点 → graph 分支）
    pub(crate) mesh: bool,
}

pub(crate) const CASES: [BranchCase; 6] = [
    BranchCase {
        name: "plane-mesh-graph",
        geom_size: [[1.0, 1.0, 0.1], [0.1, 0.095_105_66, 0.1]],
        geom_type: [0, 7],
        mesh: true,
    },
    BranchCase {
        name: "sphere-sphere-concentric",
        geom_size: [[0.1, 0.0, 0.0], [0.1, 0.0, 0.0]],
        geom_type: [2, 2],
        mesh: false,
    },
    BranchCase {
        name: "sphere-capsule-clamp",
        geom_size: [[0.1, 0.0, 0.0], [0.05, 0.12, 0.0]],
        geom_type: [2, 3],
        mesh: false,
    },
    BranchCase {
        name: "sphere-cylinder-cap",
        geom_size: [[0.1, 0.0, 0.0], [0.06, 0.1, 0.0]],
        geom_type: [2, 5],
        mesh: false,
    },
    BranchCase {
        name: "sphere-cylinder-corner",
        geom_size: [[0.1, 0.0, 0.0], [0.06, 0.1, 0.0]],
        geom_type: [2, 5],
        mesh: false,
    },
    BranchCase {
        name: "capsule-capsule-nonparallel",
        geom_size: [[0.05, 0.12, 0.0], [0.05, 0.12, 0.0]],
        geom_type: [3, 3],
        mesh: false,
    },
];

/// `fixtures/g10-p5-primitive-branches/pentagon_prism.obj` 经 pinned mujoco 3.15 解析后的
/// `model.mesh_vert`（f32，10 顶点 × 3，meshadr=0）；设备侧 mesh 顶点取的就是这份数据。
pub(crate) const PRISM_VERT: [f32; 30] = [
    0.1,
    -3.562_492_4e-19,
    -0.1,
    0.030_901_697,
    -0.095_105_66,
    -0.1,
    -0.080_901_7,
    -0.058_778_524,
    -0.1,
    -0.080_901_7,
    0.058_778_524,
    -0.1,
    0.030_901_697,
    0.095_105_66,
    -0.1,
    0.1,
    -3.562_492_4e-19,
    0.1,
    0.030_901_697,
    -0.095_105_66,
    0.1,
    -0.080_901_7,
    -0.058_778_524,
    0.1,
    -0.080_901_7,
    0.058_778_524,
    0.1,
    0.030_901_697,
    0.095_105_66,
    0.1,
];

/// `model.mesh_graph`（int32，graphadr=0，128 项；布局由 pinned warp 内核 `plane_convex`
/// graph 分支使用，设备侧按同一索引访问）。
pub(crate) const PRISM_GRAPH: [i32; 128] = [
    10, 16, 0, 8, 12, 19, 25, 29, 36, 41, 45, 50, 3, 4, 0, 1, 2, 9, 5, 8, 6, 7, 1, 2, 3, 4, 5, 7,
    9, -1, 0, 2, 5, -1, 0, 1, 3, 5, 6, 8, -1, 0, 4, 2, 8, 9, -1, 0, 3, 9, -1, 1, 2, 6, 0, 7, 9, -1,
    5, 2, 8, 9, -1, 5, 0, 9, -1, 3, 2, 6, 9, -1, 5, 6, 7, 8, 0, 4, 3, -1, 3, 4, 0, 3, 1, 2, 1, 3,
    0, 4, 9, 0, 9, 5, 0, 3, 9, 4, 9, 3, 8, 6, 1, 0, 5, 6, 0, 9, 7, 5, 7, 9, 8, 7, 6, 5, 7, 3, 2, 3,
    7, 8, 1, 7, 2, 6, 7, 1,
];

/// 三层数组摊平（`[nworld][ngeom][3/9]`）：`geom_xpos` / `geom_xmat` 专用。
/// 注意不能用两层的 [`crate::primitive_compare::flatten_f32`]（那只适用于 `fields` 的
/// `[count][width]`）——attempt1 的 panic 即由此误用导致。
pub(crate) fn flatten_f32_nested(value: &serde_json::Value) -> Vec<f32> {
    let mut out = Vec::new();
    for world in value.as_array().expect("worlds") {
        for geom in world.as_array().expect("geoms") {
            for cell in geom.as_array().expect("components") {
                out.push(cell.as_f64().expect("f32 3d") as f32);
            }
        }
    }
    out
}

/// device 输入侧分支谓词复算（与 CPU 预检同式）：证明本 case 的 f32 输入确实落在目标分支。
pub(crate) fn branch_hit_failures(case: &BranchCase, reference: &serde_json::Value) -> Vec<String> {
    let output = &reference["cases"][case.name]["output"];
    let xpos: Vec<f32> = flatten_f32_nested(&output["geom_xpos"]);
    let xmat: Vec<f32> = flatten_f32_nested(&output["geom_xmat"]);
    let mut failures = Vec::new();
    let p = |geom: usize| [xpos[geom * 3], xpos[geom * 3 + 1], xpos[geom * 3 + 2]];
    let third = |geom: usize| [xmat[geom * 9 + 2], xmat[geom * 9 + 5], xmat[geom * 9 + 8]];
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let rot = |geom: usize| {
        let mut m = [0.0_f32; 9];
        m.copy_from_slice(&xmat[geom * 9..geom * 9 + 9]);
        m
    };
    let transpose_mul = |m: [f32; 9], v: [f32; 3]| {
        [
            m[0] * v[0] + m[1] * v[1] + m[2] * v[2],
            m[3] * v[0] + m[4] * v[1] + m[5] * v[2],
            m[6] * v[0] + m[7] * v[1] + m[8] * v[2],
        ]
    };
    match case.name {
        "plane-mesh-graph" => {
            // graph 分支条件：graphadr != -1 且 vertnum >= 10；且 max_support >= 0（有顶点在平面下）。
            let vertnum = PRISM_GRAPH[0];
            let plane_normal = transpose_mul(rot(0), [0.0, 0.0, 1.0]);
            let convex_pos = p(1);
            let delta = [
                p(0)[0] - convex_pos[0],
                p(0)[1] - convex_pos[1],
                p(0)[2] - convex_pos[2],
            ];
            let plane_pos_local = transpose_mul(rot(1), delta);
            let n = transpose_mul(rot(1), plane_normal);
            let mut max_support = f32::NEG_INFINITY;
            for index in 0..10 {
                let vert = [
                    PRISM_VERT[index * 3],
                    PRISM_VERT[index * 3 + 1],
                    PRISM_VERT[index * 3 + 2],
                ];
                let d = [
                    plane_pos_local[0] - vert[0],
                    plane_pos_local[1] - vert[1],
                    plane_pos_local[2] - vert[2],
                ];
                max_support = max_support.max(dot(d, n));
            }
            if vertnum < 10 {
                failures.push(format!("B3 分支命中失败：prism vertnum={vertnum} < 10"));
            }
            if max_support < 0.0 {
                failures.push(format!(
                    "B3 分支命中失败：plane-mesh max_support={max_support:e} < 0"
                ));
            }
        }
        "sphere-sphere-concentric" => {
            let delta = [p(1)[0] - p(0)[0], p(1)[1] - p(0)[1], p(1)[2] - p(0)[2]];
            let dist = dot(delta, delta).sqrt();
            if dist != 0.0 {
                failures.push(format!(
                    "B3 分支命中失败：同心输入 center_dist={dist:e} != 0"
                ));
            }
            let frame = flatten_f32(&output["fields"]["frame"]);
            if frame[0] != 1.0 || frame[1] != 0.0 || frame[2] != 0.0 {
                failures.push(format!(
                    "B3 分支命中失败：dist==0 fallback 法线应为 (1,0,0)，参考 frame 首行={:?}",
                    &frame[0..3]
                ));
            }
        }
        "sphere-capsule-clamp" => {
            let axis = third(1);
            let delta = [p(0)[0] - p(1)[0], p(0)[1] - p(1)[1], p(0)[2] - p(1)[2]];
            let projection = dot(axis, delta);
            if projection.abs() <= 0.12 {
                failures.push(format!(
                    "B3 分支命中失败：sphere-capsule 投影={projection:e} 未超过 half_length=0.12（未 clamp）"
                ));
            }
        }
        "sphere-cylinder-cap" | "sphere-cylinder-corner" => {
            let axis = third(1);
            let d = [p(0)[0] - p(1)[0], p(0)[1] - p(1)[1], p(0)[2] - p(1)[2]];
            let x = dot(d, axis);
            let p_proj = [d[0] - axis[0] * x, d[1] - axis[1] * x, d[2] - axis[2] * x];
            let p_proj_sqr = dot(p_proj, p_proj);
            let mut side = x.abs() < 0.1;
            let mut cap = p_proj_sqr < 0.06 * 0.06;
            if side && cap {
                if 0.1 - x.abs() < 0.06 - p_proj_sqr.sqrt() {
                    side = false;
                } else {
                    cap = false;
                }
            }
            let taken = if side {
                "side"
            } else if cap {
                "cap"
            } else {
                "corner"
            };
            let wanted = if case.name == "sphere-cylinder-cap" {
                "cap"
            } else {
                "corner"
            };
            if taken != wanted {
                failures.push(format!(
                    "B3 分支命中失败：sphere-cylinder 应走 {wanted}，f32 复算走 {taken}"
                ));
            }
        }
        "capsule-capsule-nonparallel" => {
            let axis1 = third(0);
            let axis2 = third(1);
            let a1 = [axis1[0] * 0.12, axis1[1] * 0.12, axis1[2] * 0.12];
            let a2 = [axis2[0] * 0.12, axis2[1] * 0.12, axis2[2] * 0.12];
            let ma = dot(a1, a1);
            let mb = -dot(a1, a2);
            let mc = dot(a2, a2);
            let det = ma * mc - mb * mb;
            if det.abs() < MJ_MINVAL || det.abs() < 1.0e-6 {
                failures.push(format!(
                    "B3 分支命中失败：capsule-capsule det={det:e} 不够显著（需 |det| >= 1e-6）"
                ));
            }
        }
        _ => {}
    }
    failures
}
