//! G10@P5 B2：sphere/capsule 族 primitive 设备入口对照 B0 上游 GPU 参考（默认 ignored，需 GPU 窗口）。
//!
//! 单 test 函数：一次 NVRTC 编译 + 一次建 context，循环 4 case（避免每 case 各编译一次把窗口拉长）。
//! 参考：`docs/development-evidence/g10-p5-primitive-capture/upstream-primitive-gpu.json`
//! （B0 GPU 捕获，sha `3a6475ac…`）；判据：位级优先；非位级 dist ≤4 ULP@自身量级（ref=0 用
//! 4 ULP@1.0）、pos 4·ULP(scale)、frame 4 ULP@1.0；count/结构/overflow/efc 零容差。
//! **nacon 按 GPU 参考**（4 例均为 1；`capsule-capsule` 的 CPU 预检 nacon=2 属后端差异，
//! 只证 fixture 有效，不按 CPU 预检凑 count）。
//!
//! 输入几何取自 `fixtures/g10-p5-primitive/<case>.xml`（sha 清单见 B0
//! `inputs-sha256.json`）；参考 JSON 只存 `geom_xpos`/`geom_xmat`，两份 geom 的 `size` 与
//! `type` 按 fixture 硬编码在此（与 B1 plane 测试同口径）。
//!
//! 回归（B1 plane 6 case / 既有 primitive 3 case / launcher 3 case）用各自现成 test 二进制，
//! 不在本文件重写。
//!
//! 运行：`cargo test --locked --features cuda-probe --test g10_p5_primitive_sphere_capsule_cuda -- --ignored --test-threads=1 --nocapture`

#![cfg(feature = "cuda-probe")]
// 对照测试：索引/排序/条件写法刻意直白，按需豁免对应 clippy 提示（与既有 G5 对照测试一致）。
#![allow(
    clippy::manual_checked_ops,
    clippy::identity_op,
    clippy::unnecessary_sort_by,
    clippy::collapsible_if
)]

use mjwarp_rs::runtime::collision_gpu::{
    CollisionGpuInputs, CollisionGpuMeta, CollisionGpuOutput, CollisionGpuProgram, CollisionKernel,
};

const FOUR_ULP_AT_ONE: f32 = 4.0 * (1.0 / 8_388_608.0);

struct SphereCapsuleCase {
    /// 对照 GPU 参考的 case 名（B0 JSON 的 key）
    name: &'static str,
    /// 两个 geom 的 `geom_size`（f32；来自 `fixtures/g10-p5-primitive/<name>.xml`）
    geom_size: [[f32; 3]; 2],
    /// 两个 geom 的 `mjtGeom` 类型（配对顺序与 B0 捕获一致：geom0 为类型序在前者）
    geom_type: [i32; 2],
}

const CASES: [SphereCapsuleCase; 4] = [
    // sphere size=0.1；sphere size=0.1
    SphereCapsuleCase {
        name: "sphere-sphere",
        geom_size: [[0.1, 0.0, 0.0], [0.1, 0.0, 0.0]],
        geom_type: [2, 2],
    },
    // sphere size=0.1；capsule size="0.05 0.12"
    SphereCapsuleCase {
        name: "sphere-capsule",
        geom_size: [[0.1, 0.0, 0.0], [0.05, 0.12, 0.0]],
        geom_type: [2, 3],
    },
    // sphere size=0.1；cylinder size="0.06 0.1"
    SphereCapsuleCase {
        name: "sphere-cylinder",
        geom_size: [[0.1, 0.0, 0.0], [0.06, 0.1, 0.0]],
        geom_type: [2, 5],
    },
    // capsule size="0.05 0.12"；capsule size="0.05 0.1"
    SphereCapsuleCase {
        name: "capsule-capsule",
        geom_size: [[0.05, 0.12, 0.0], [0.05, 0.1, 0.0]],
        geom_type: [3, 3],
    },
];

fn root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_reference() -> serde_json::Value {
    let path = root()
        .join("docs/development-evidence/g10-p5-primitive-capture/upstream-primitive-gpu.json");
    serde_json::from_str(&std::fs::read_to_string(&path).expect("读 B0 GPU 参考")).expect("B0 JSON")
}

fn ulp_distance(device: f32, reference: f32) -> u64 {
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

fn vector_scale(values: &[f32]) -> f32 {
    let scale = values
        .iter()
        .fold(0.0_f32, |scale, value| scale.max(value.abs()));
    if scale == 0.0 { 1.0 } else { scale }
}

struct ContactRow {
    id: i32,
    worldid: i32,
    contact_type: i32,
    dim: i32,
    geom: [i32; 2],
    dist: f32,
    pos: [f32; 3],
    frame: [f32; 9],
    includemargin: f32,
    adhesion: f32,
    friction: [f32; 5],
    solref: [f32; 2],
    solreffriction: [f32; 2],
    solimp: [f32; 5],
}

/// 逐 case 对照 B0 上游 GPU 参考：
/// - count / 结构（worldid/type/dim/geom/geomcollisionid/includemargin/adhesion/friction/
///   solref/solreffriction/solimp）**零容差**；行序不作判据，按 `geomcollisionid` 结构键配对；
/// - dist：位级优先；非位级 ≤4 ULP@自身量级（参考值为 0 时用 4 ULP@1.0）；
/// - pos：4·ULP(参考三轴最大绝对值)；frame：4 ULP@1.0（矩阵元素量级为 1）。
/// - `efc_address` / `overflow` 参考未采集 → 只做设备侧不变量断言（efc 全 -1、overflow 全 0）。
fn compare(
    case: &SphereCapsuleCase,
    reference: &serde_json::Value,
    output: &CollisionGpuOutput,
) -> Vec<String> {
    let mut failures = Vec::new();
    let expected = &reference["cases"][case.name]["output"];
    let count = expected["count"].as_u64().expect("count") as usize;
    let rows = output.nacon.max(0) as usize;
    if rows != count {
        failures.push(format!("contact 数 dev={rows} ref={count}（零容差）"));
    }
    let floats = |name: &str| -> Vec<f32> {
        let mut out = Vec::new();
        for row in expected["fields"][name].as_array().expect("fields") {
            for cell in row.as_array().expect("row") {
                out.push(cell.as_f64().expect("f32") as f32);
            }
        }
        out
    };
    let ints = |name: &str| -> Vec<i32> {
        let mut out = Vec::new();
        for row in expected["fields"][name].as_array().expect("fields") {
            for cell in row.as_array().expect("row") {
                out.push(cell.as_i64().expect("int") as i32);
            }
        }
        out
    };
    let width = |cells: usize| if count == 0 { 0 } else { cells / count };

    let ref_dist = floats("dist");
    let ref_pos = floats("pos");
    let ref_frame = floats("frame");
    let ref_includemargin = floats("includemargin");
    let ref_adhesion = floats("adhesion");
    let ref_friction = floats("friction");
    let ref_solref = floats("solref");
    let ref_solreffriction = floats("solreffriction");
    let ref_solimp = floats("solimp");
    let ref_geom = ints("geom");
    let ref_dim = ints("dim");
    let ref_worldid = ints("worldid");
    let ref_type = ints("type");
    let ref_id = ints("geomcollisionid");
    let dist_w = width(ref_dist.len());
    let pos_w = width(ref_pos.len());
    let frame_w = width(ref_frame.len());

    let mut reference_rows: Vec<ContactRow> = (0..count)
        .map(|index| {
            let mut frame = [0.0_f32; 9];
            frame.copy_from_slice(&ref_frame[index * frame_w..index * frame_w + 9]);
            ContactRow {
                id: ref_id[index],
                worldid: ref_worldid[index],
                contact_type: ref_type[index],
                dim: ref_dim[index],
                geom: [ref_geom[index * 2], ref_geom[index * 2 + 1]],
                dist: ref_dist[index * dist_w],
                pos: [
                    ref_pos[index * pos_w],
                    ref_pos[index * pos_w + 1],
                    ref_pos[index * pos_w + 2],
                ],
                frame,
                includemargin: ref_includemargin[index * width(ref_includemargin.len())],
                adhesion: ref_adhesion[index * width(ref_adhesion.len())],
                friction: [
                    ref_friction[index * 5],
                    ref_friction[index * 5 + 1],
                    ref_friction[index * 5 + 2],
                    ref_friction[index * 5 + 3],
                    ref_friction[index * 5 + 4],
                ],
                solref: [ref_solref[index * 2], ref_solref[index * 2 + 1]],
                solreffriction: [
                    ref_solreffriction[index * width(ref_solreffriction.len())],
                    ref_solreffriction[index * width(ref_solreffriction.len()) + 1],
                ],
                solimp: [
                    ref_solimp[index * 5],
                    ref_solimp[index * 5 + 1],
                    ref_solimp[index * 5 + 2],
                    ref_solimp[index * 5 + 3],
                    ref_solimp[index * 5 + 4],
                ],
            }
        })
        .collect();

    let mut device_rows: Vec<ContactRow> = (0..rows)
        .map(|index| {
            let mut frame = [0.0_f32; 9];
            frame.copy_from_slice(&output.frame[index * 9..index * 9 + 9]);
            ContactRow {
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
                friction: [
                    output.friction[index * 5],
                    output.friction[index * 5 + 1],
                    output.friction[index * 5 + 2],
                    output.friction[index * 5 + 3],
                    output.friction[index * 5 + 4],
                ],
                solref: [output.solref[index * 2], output.solref[index * 2 + 1]],
                solreffriction: [
                    output.solreffriction[index * 2],
                    output.solreffriction[index * 2 + 1],
                ],
                solimp: [
                    output.solimp[index * 5],
                    output.solimp[index * 5 + 1],
                    output.solimp[index * 5 + 2],
                    output.solimp[index * 5 + 3],
                    output.solimp[index * 5 + 4],
                ],
            }
        })
        .collect();

    // 参考未采集 efc_address / overflow：只做设备侧不变量。
    if output.efc_address.iter().any(|value| *value != -1) {
        failures.push("efc_address 存在非 -1（设备侧不变量）".to_string());
    }
    if output.overflow.iter().any(|value| *value != 0) {
        failures.push(format!(
            "overflow 非 0（设备侧不变量）：{:?}",
            output.overflow
        ));
    }

    device_rows.sort_by_key(|row| row.id);
    reference_rows.sort_by_key(|row| row.id);
    let device_ids: Vec<i32> = device_rows.iter().map(|row| row.id).collect();
    let reference_ids: Vec<i32> = reference_rows.iter().map(|row| row.id).collect();
    if device_ids != reference_ids {
        failures.push(format!(
            "geomcollisionid 集合不一致 dev={device_ids:?} ref={reference_ids:?}（零容差，行序不作判据）"
        ));
    }

    for (device, reference) in device_rows.iter().zip(&reference_rows) {
        let id = reference.id;
        if device.worldid != reference.worldid {
            failures.push(format!(
                "id={id} worldid dev={} ref={}（零容差）",
                device.worldid, reference.worldid
            ));
        }
        if device.contact_type != reference.contact_type {
            failures.push(format!(
                "id={id} type dev={} ref={}（零容差）",
                device.contact_type, reference.contact_type
            ));
        }
        if device.dim != reference.dim {
            failures.push(format!(
                "id={id} dim dev={} ref={}（零容差）",
                device.dim, reference.dim
            ));
        }
        if device.geom != reference.geom {
            failures.push(format!(
                "id={id} geom dev={:?} ref={:?}（零容差）",
                device.geom, reference.geom
            ));
        }
        if device.includemargin != reference.includemargin
            || device.adhesion != reference.adhesion
            || device.friction != reference.friction
            || device.solref != reference.solref
            || device.solreffriction != reference.solreffriction
            || device.solimp != reference.solimp
        {
            failures.push(format!("id={id} 接触参数（margin/adhesion/friction/solref/solreffriction/solimp）非逐位相同（零容差）"));
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

#[test]
#[ignore = "requires NVIDIA GPU + NVRTC"]
fn sphere_capsule_family_matches_b0_upstream_gpu_reference() {
    let program = CollisionGpuProgram::new(0).expect("NVRTC 编译设备源并加载模块/入口");
    let reference = read_reference();
    let mut failures: Vec<String> = Vec::new();
    for case in &CASES {
        let output_ref = &reference["cases"][case.name]["output"];
        let nworld = 1usize;
        let ngeom = 2usize;
        // B0 JSON 的 geom_xpos/geom_xmat 形状为 [nworld][ngeom][3/9]。
        let mut geom_xpos = Vec::with_capacity(nworld * ngeom * 3);
        let mut geom_xmat = Vec::with_capacity(nworld * ngeom * 9);
        for world in 0..nworld {
            for geom in 0..ngeom {
                for axis in 0..3 {
                    geom_xpos.push(
                        output_ref["geom_xpos"][world][geom][axis]
                            .as_f64()
                            .expect("xpos") as f32,
                    );
                }
                for cell in 0..9 {
                    geom_xmat.push(
                        output_ref["geom_xmat"][world][geom][cell]
                            .as_f64()
                            .expect("xmat") as f32,
                    );
                }
            }
        }
        let mut geom_size = vec![0.0_f32; ngeom * 3];
        for geom in 0..ngeom {
            geom_size[geom * 3..geom * 3 + 3].copy_from_slice(&case.geom_size[geom]);
        }
        let geom_type = vec![case.geom_type[0], case.geom_type[1]];
        let geom_dataid = vec![-1_i32; ngeom];
        let geom_margin = vec![0.0_f32; ngeom];
        let geom_gap = vec![0.0_f32; ngeom];
        let geom_rbound = vec![0.0_f32; ngeom];
        let geom_aabb = vec![0.0_f32; ngeom * 6];
        let inputs = CollisionGpuInputs {
            geom_xpos: &geom_xpos,
            geom_xmat: &geom_xmat,
            geom_size: &geom_size,
            geom_margin: &geom_margin,
            geom_gap: &geom_gap,
            geom_rbound: &geom_rbound,
            geom_aabb: &geom_aabb,
            geom_type: &geom_type,
            geom_dataid: &geom_dataid,
            mesh_vertadr: &[],
            mesh_vertnum: &[],
            mesh_graphadr: &[],
            mesh_octadr: &[],
            mesh_vert: &[],
            mesh_graph: &[],
            hfield_size: &[],
            hfield_nrow: &[],
            hfield_ncol: &[],
            hfield_adr: &[],
            hfield_data: &[],
            oct_aabb: &[],
            oct_child: &[],
            oct_coeff: &[],
            meta: CollisionGpuMeta {
                nworld: nworld as i32,
                ngeom: ngeom as i32,
                g1: 0,
                g2: 1,
                type1: case.geom_type[0],
                type2: case.geom_type[1],
                geomgeomid: 0,
                naccdmax: 8192,
                hfield_id: -1,
                oct_root: -1,
                noct: 0,
                tolerance: 1.0e-6,
                margin: 0.0,
                gap: 0.0,
                gjk_iterations: 35,
                epa_iterations: 35,
                condim: 3,
                pairid0: -1,
                pairid1: -1,
                friction: [1.0, 1.0, 0.005, 1.0e-4, 1.0e-4],
                solref: [0.02, 1.0],
                solreffriction: [0.0, 0.0],
                solimp: [0.9, 0.95, 0.001, 0.5, 2.0],
                adhesion: 0.0,
                sdf_initpoints: 40,
                sdf_iterations: 10,
                naconmax: 8192,
                efc_width: 4,
            },
        };
        let output = program
            .run(CollisionKernel::Primitive, &inputs)
            .unwrap_or_else(|error| panic!("{} 运行失败：{error}", case.name));
        let case_failures = compare(case, &reference, &output);
        println!(
            "{}: nacon={} failures={}",
            case.name,
            output.nacon,
            case_failures.len()
        );
        for failure in &case_failures {
            println!("FAIL {}: {failure}", case.name);
        }
        failures.extend(
            case_failures
                .into_iter()
                .map(|failure| format!("{}: {failure}", case.name)),
        );
    }
    assert!(
        failures.is_empty(),
        "sphere/capsule 族对照未全部通过（{} 条）",
        failures.len()
    );
}
