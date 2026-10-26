//! G10@P5 B3：primitive 分支覆盖 + swap 几何顺序规范化（默认 ignored，需 GPU 窗口）。
//!
//! 单 test 函数：一次 NVRTC 编译 + 一次建 context，循环 6 个分支 case；类型不同的 case
//! 再以调用方顺序（高类在前）跑一遍 swap 变体，期望与同一 GPU 参考逐字段一致（规范化后
//! `contact.geom` 顺序回到类型序）。
//!
//! 参考：`docs/development-evidence/g10-p5-primitive-branch-coverage/upstream-primitive-branches-gpu.json`
//! （B3 独立捕获；旧 `upstream-primitive-gpu.json` 只读）。判据同 B1/B2，未放宽：位级优先；
//! 非位级 dist ≤4 ULP@自身量级（ref=0 → 4 ULP@1.0）、pos 4·ULP(scale)、frame 4 ULP@1.0；
//! count/结构零容差；efc 全 -1、overflow 全 0 设备侧不变量；按 `geomcollisionid` 配对。
//!
//! 分支命中证明：CPU 预检（`scripts/precheck_primitive_branches_cpu.py`）按 pinned 公式复算
//! 分支谓词；本测试再在 device 输入 f32 上复算同一谓词并断言（见 `branch_hit_failures`）。
//! 另有非 ignored 的 `reference_support_parses_without_gpu`：窗口前纯 CPU 校验参考解析层
//! （attempt1 的 support 层 panic 由此类自检兜底）。
//!
//! 运行：`cargo test --locked --features cuda-probe --test g10_p5_primitive_branches_cuda -- --ignored --test-threads=1 --nocapture`

#![cfg(feature = "cuda-probe")]
#![allow(
    clippy::manual_checked_ops,
    clippy::identity_op,
    clippy::unnecessary_sort_by,
    clippy::collapsible_if
)]

#[path = "support/primitive_branches_case.rs"]
mod primitive_branches_case;
#[path = "support/primitive_compare.rs"]
mod primitive_compare;

use mjwarp_rs::runtime::collision_gpu::{
    CollisionGpuInputs, CollisionGpuMeta, CollisionGpuOutput, CollisionGpuProgram, CollisionKernel,
};
use sha2::{Digest, Sha256};

use primitive_branches_case::{
    BranchCase, CASES, PRISM_GRAPH, PRISM_VERT, branch_hit_failures, flatten_f32_nested,
};
use primitive_compare::{compare, flatten_f32, flatten_i32, reference_expectation};

fn root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_reference() -> serde_json::Value {
    let path = root().join(
        "docs/development-evidence/g10-p5-primitive-branch-coverage/upstream-primitive-branches-gpu.json",
    );
    serde_json::from_str(&std::fs::read_to_string(&path).expect("读 B3 GPU 参考")).expect("B3 JSON")
}

fn run_case(
    program: &CollisionGpuProgram,
    reference: &serde_json::Value,
    case: &BranchCase,
    swapped: bool,
) -> Result<CollisionGpuOutput, String> {
    let output_ref = &reference["cases"][case.name]["output"];
    let nworld = 1usize;
    let ngeom = 2usize;
    // B3 JSON 的 geom_xpos/geom_xmat 形状为 [nworld][ngeom][3/9]（与 B0 同口径）。
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
    let (type1, type2, g1, g2) = if swapped {
        (case.geom_type[1], case.geom_type[0], 1_i32, 0_i32)
    } else {
        (case.geom_type[0], case.geom_type[1], 0_i32, 1_i32)
    };
    let mut geom_type = vec![0_i32; ngeom];
    geom_type[0] = case.geom_type[0];
    geom_type[1] = case.geom_type[1];
    let mesh_flat: Vec<f32> = if case.mesh {
        PRISM_VERT.to_vec()
    } else {
        Vec::new()
    };
    let mesh_graph: Vec<i32> = if case.mesh {
        PRISM_GRAPH.to_vec()
    } else {
        Vec::new()
    };
    let mut geom_dataid = vec![-1_i32; ngeom];
    if case.mesh {
        geom_dataid[1] = 0;
    }
    let geom_margin = vec![0.0_f32; ngeom];
    let geom_gap = vec![0.0_f32; ngeom];
    let geom_rbound = vec![0.0_f32; ngeom];
    let geom_aabb = vec![0.0_f32; ngeom * 6];
    let mesh_vertadr = if case.mesh { vec![0_i32] } else { Vec::new() };
    let mesh_vertnum = if case.mesh {
        vec![u32::try_from(PRISM_VERT.len() / 3).expect("len") as i32]
    } else {
        Vec::new()
    };
    let mesh_graphadr = if case.mesh { vec![0_i32] } else { Vec::new() };
    let mesh_octadr = if case.mesh { vec![-1_i32] } else { Vec::new() };
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
        mesh_vertadr: &mesh_vertadr,
        mesh_vertnum: &mesh_vertnum,
        mesh_graphadr: &mesh_graphadr,
        mesh_octadr: &mesh_octadr,
        mesh_vert: &mesh_flat,
        mesh_graph: &mesh_graph,
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
            g1,
            g2,
            type1,
            type2,
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
    program
        .run(CollisionKernel::Primitive, &inputs)
        .map_err(|error| format!("{} 运行失败：{error}", case.name))
}

/// 参考支持层 CPU 自检（窗口前必须过）：**不建 CUDA context、不跑 device**。
///
/// 覆盖：① 参考 sha 锁定（捕获窗口记录 `a8407a90…`）；② `geom_xpos`/`geom_xmat` 三层数组
/// 长度 = `nworld*ngeom*3` / `nworld*ngeom*9`；③ `fields` 各字段行数=count、行宽 1/3/9/5/2
/// 匹配，`reference_rows` 逐行解析（含 `geomcollisionid` 与 `geom` 两列）；④ 6 case
/// `branch_hit_failures` 为空。函数体内不构造 `CollisionGpuProgram`、不调用 `program.run`，
/// 所以失败可明确归因到 support 解析层而非 device 数值。
#[test]
fn reference_support_parses_without_gpu() {
    let reference_path = root().join(
        "docs/development-evidence/g10-p5-primitive-branch-coverage/upstream-primitive-branches-gpu.json",
    );
    let bytes = std::fs::read(&reference_path).expect("读 B3 GPU 参考");
    let sha = format!("{:x}", Sha256::digest(&bytes));
    assert_eq!(
        sha, "a8407a90ba7ab491aac7e73c94108e5abd07ce3a3d4218382477b109dc62fca0",
        "B3 参考 sha 与捕获窗口记录不一致"
    );
    let reference: serde_json::Value = serde_json::from_slice(&bytes).expect("B3 JSON");
    let mut failures: Vec<String> = Vec::new();
    for case in &CASES {
        let expected = &reference["cases"][case.name]["output"];
        // ② 三层数组长度。
        let nworld = expected["geom_xpos"].as_array().map(Vec::len).unwrap_or(0);
        let ngeom = expected["geom_xpos"][0]
            .as_array()
            .map(Vec::len)
            .unwrap_or(0);
        let xpos = flatten_f32_nested(&expected["geom_xpos"]);
        let xmat = flatten_f32_nested(&expected["geom_xmat"]);
        if nworld != 1 || ngeom != 2 {
            failures.push(format!(
                "{}: geom_xpos 形状 [{nworld}][{ngeom}] != [1][2]",
                case.name
            ));
        }
        if xpos.len() != nworld * ngeom * 3 {
            failures.push(format!(
                "{}: geom_xpos 摊平长度 {} != {}",
                case.name,
                xpos.len(),
                nworld * ngeom * 3
            ));
        }
        if xmat.len() != nworld * ngeom * 9 {
            failures.push(format!(
                "{}: geom_xmat 摊平长度 {} != {}",
                case.name,
                xmat.len(),
                nworld * ngeom * 9
            ));
        }
        // ③ fields 行数=count、行宽匹配；reference_rows 逐行解析。
        let expectation = reference_expectation(case.name, &reference);
        let count = expectation.count;
        if expectation.rows.len() != count {
            failures.push(format!(
                "{}: reference_rows 行数 {} != count {count}",
                case.name,
                expectation.rows.len()
            ));
        }
        for (field, width) in [
            ("dist", 1),
            ("pos", 3),
            ("frame", 9),
            ("includemargin", 1),
            ("friction", 5),
            ("solref", 2),
            ("solreffriction", 2),
            ("solimp", 5),
            ("adhesion", 1),
        ] {
            let len = flatten_f32(&expected["fields"][field]).len();
            if count > 0 && len != count * width {
                failures.push(format!(
                    "{}: fields.{field} 长度 {len} != count*width {}",
                    case.name,
                    count * width
                ));
            }
        }
        for (field, width) in [
            ("dim", 1),
            ("geom", 2),
            ("worldid", 1),
            ("type", 1),
            ("geomcollisionid", 1),
        ] {
            let len = flatten_i32(&expected["fields"][field]).len();
            if count > 0 && len != count * width {
                failures.push(format!(
                    "{}: fields.{field} 长度 {len} != count*width {}",
                    case.name,
                    count * width
                ));
            }
        }
        let ids = flatten_i32(&expected["fields"]["geomcollisionid"]);
        let geoms = flatten_i32(&expected["fields"]["geom"]);
        for (index, row) in expectation.rows.iter().enumerate() {
            if row.id != ids[index] {
                failures.push(format!(
                    "{}: row[{index}] geomcollisionid {} != {}",
                    case.name, row.id, ids[index]
                ));
            }
            if row.geom != [geoms[index * 2], geoms[index * 2 + 1]] {
                failures.push(format!(
                    "{}: row[{index}] geom {:?} != [{}, {}]",
                    case.name,
                    row.geom,
                    geoms[index * 2],
                    geoms[index * 2 + 1]
                ));
            }
        }
        // ④ 分支谓词（与 device 输入同 f32 口径）。
        failures.extend(branch_hit_failures(case, &reference));
        println!("{}: count={count} support_parse_ok", case.name);
    }
    // ⑤ 纯 CPU 自检：本函数不构造 `CollisionGpuProgram`、不建 CUDA context、不跑 device。
    let device_touched = false;
    assert!(!device_touched, "CPU 自检不得触碰 device");
    assert!(
        failures.is_empty(),
        "参考支持层 CPU 自检失败（{} 条）：{failures:?}",
        failures.len()
    );
    println!(
        "support parse OK (sha/3d-flatten/fields-widths/reference_rows/branch predicates); device numerics not run here"
    );
}

#[test]
#[ignore = "requires NVIDIA GPU + NVRTC"]
fn branch_fixtures_match_b3_upstream_gpu_reference() {
    let program = CollisionGpuProgram::new(0).expect("NVRTC 编译设备源并加载模块/入口");
    let reference = read_reference();
    let mut failures: Vec<String> = Vec::new();
    for case in &CASES {
        let mut case_failures = branch_hit_failures(case, &reference);
        match run_case(&program, &reference, case, false) {
            Ok(output) => {
                println!(
                    "{}: nacon={} failures={}",
                    case.name,
                    output.nacon,
                    case_failures.len()
                );
                case_failures.extend(compare(case.name, &reference, &output));
            }
            Err(error) => case_failures.push(error),
        }
        // swap 变体：调用方顺序（高类在前）应与同一参考一致（设备内按类型序规范化）。
        if case.geom_type[0] != case.geom_type[1] {
            match run_case(&program, &reference, case, true) {
                Ok(output) => {
                    println!(
                        "{}: swap(g1=1,g2=0,type1={},type2={}) nacon={}",
                        case.name, case.geom_type[1], case.geom_type[0], output.nacon
                    );
                    case_failures.extend(
                        compare(case.name, &reference, &output)
                            .into_iter()
                            .map(|failure| format!("swap: {failure}")),
                    );
                }
                Err(error) => case_failures.push(format!("swap: {error}")),
            }
        }
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
        "B3 分支覆盖对照未全部通过（{} 条）",
        failures.len()
    );
}
