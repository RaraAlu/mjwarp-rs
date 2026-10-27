//! G10@P5 S4 · UNC-3 分支收口：新 target（branch 验收组 9 case + limits 记录组 2 case）。
//!
//! - 参考：S3 捕获窗落盘的 `docs/development-evidence/g10-p5-branch-closure-capture/`
//!   `upstream-branch-closure-gpu.json`（branch 验收组：U1/U3×5/U4/U5/U6）与
//!   `upstream-mesh-limits-gpu.json`（limits 组：U7/U8，**record-only**）。
//! - 输入：逐 case 的 `qpos` 不参与设备输入（设备吃 `geom_xpos`/`geom_xmat` f32 + size/type +
//!   mesh 拓扑）；`geom_xpos`/`geom_xmat` 逐位取自 S3 参考 JSON（上游 GPU 采集的同一帧），
//!   mesh 拓扑来自 `support/branch_closure_mesh_data.rs`（pinned mujoco 3.15 对同一批 S3
//!   fixture 的解析；见该文件头）。fixture sha256 由 CPU 测试按 S3 签名清单复核。
//! - 判据：沿用既有口径、**零放宽** —— 位级优先；非位级 `dist` ≤4 ULP@自身量级（ref==0 用
//!   4 ULP@1.0）、`pos` 4·ULP(参考三轴最大绝对值)、`frame` 4 ULP@1.0；count/结构字段/
//!   接触参数零容差；`efc_address` 全 -1、`overflow` 全 0 设备侧不变量（复用
//!   `support/primitive_compare.rs`）。
//! - **limits 组 record-only**：只打印设备与上游的 count/dist/pos，不进 pass/fail 断言。
//!
//! 运行：`cargo test --locked --features cuda-probe --test g10_p5_branch_closure_cuda -- --ignored --test-threads=1 --nocapture`

#![cfg(feature = "cuda-probe")]
#![allow(
    clippy::manual_checked_ops,
    clippy::identity_op,
    clippy::unnecessary_sort_by,
    clippy::collapsible_if
)]

#[path = "support/branch_closure_case.rs"]
mod branch_closure_case;
#[path = "support/branch_closure_mesh_data.rs"]
mod branch_closure_mesh_data;
#[path = "support/primitive_compare.rs"]
mod primitive_compare;

use std::path::PathBuf;

use mjwarp_rs::runtime::collision_gpu::{CollisionGpuOutput, CollisionGpuProgram, CollisionKernel};
use sha2::{Digest, Sha256};

use branch_closure_case::{BRANCH_CASES, Case, CaseInputs, FIXTURE_SHA256, Kernel, LIMIT_CASES};
use primitive_compare::compare;

const BRANCH_REFERENCE: &str =
    "docs/development-evidence/g10-p5-branch-closure-capture/upstream-branch-closure-gpu.json";
const LIMITS_REFERENCE: &str =
    "docs/development-evidence/g10-p5-branch-closure-capture/upstream-mesh-limits-gpu.json";
/// S3 证据 §5 的冻结 sha256（参考不可被改写；target 先锁 sha 再解析）。
const BRANCH_REFERENCE_SHA256: &str =
    "24d89e9248cccece96baff2089954e438ca7956fe8a71ccc61df318208709db1";
const LIMITS_REFERENCE_SHA256: &str =
    "b6abb8fa5820c103f6d880936d6ae95fa60035e5b9ce49a0364b15de1def19ad";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// 读参考 JSON 并先校验冻结 sha256（锁死参考、防止被改写）。
fn read_reference(relative: &str, expected_sha256: &str) -> serde_json::Value {
    let path = root().join(relative);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("读取 {} 失败：{error}", path.display()));
    assert_eq!(
        sha256_hex(&bytes),
        expected_sha256,
        "参考 sha256 与 S3 冻结值不符（参考不可被改动）"
    );
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("解析 {} 失败：{error}", path.display()))
}

fn kernel_of(case: &Case) -> CollisionKernel {
    match case.kernel {
        Kernel::Primitive => CollisionKernel::Primitive,
        Kernel::ContactMulti => CollisionKernel::ContactMulti,
    }
}

fn run(
    program: &CollisionGpuProgram,
    case: &Case,
    reference: &serde_json::Value,
) -> CollisionGpuOutput {
    let inputs = CaseInputs::build(case, reference);
    program
        .run(kernel_of(case), &inputs.inputs())
        .unwrap_or_else(|error| panic!("{} 运行失败：{error}", case.name))
}

/// 参考 JSON 的某一个 `[count][width]` 字段 → f32 平铺。
fn reference_floats(reference: &serde_json::Value, field: &str) -> Vec<f32> {
    let mut out = Vec::new();
    for row in reference["output"]["fields"][field]
        .as_array()
        .unwrap_or_else(|| panic!("参考缺少 fields.{field}"))
    {
        for cell in row.as_array().expect("row") {
            out.push(cell.as_f64().expect("f32") as f32);
        }
    }
    out
}

fn format_floats(values: &[f32]) -> String {
    values
        .iter()
        .map(|value| format!("{value:e}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn format_points(values: &[f32]) -> String {
    values
        .chunks(3)
        .map(|point| format!("({:e},{:e},{:e})", point[0], point[1], point[2]))
        .collect::<Vec<_>>()
        .join(";")
}

#[test]
#[ignore = "requires NVIDIA GPU + NVRTC"]
fn branch_closure_acceptance_group_matches_s3_upstream_gpu() {
    let program = CollisionGpuProgram::new(0).expect("NVRTC 编译设备源并加载模块/入口");
    let reference = read_reference(BRANCH_REFERENCE, BRANCH_REFERENCE_SHA256);
    let mut failures: Vec<String> = Vec::new();
    for case in &BRANCH_CASES {
        let case_reference = &reference["cases"][case.name];
        assert_eq!(
            case_reference["two_captures_bitwise_identical"].as_bool(),
            Some(true),
            "{}: S3 参考应记录两次捕获逐位一致",
            case.name
        );
        let output = run(&program, case, case_reference);
        let case_failures = compare(case.name, &reference, &output);
        println!(
            "case={} kernel={:?} nacon={} ref_count={} failures={}",
            case.name,
            kernel_of(case),
            output.nacon,
            case_reference["output"]["count"]
                .as_u64()
                .unwrap_or(u64::MAX),
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
        "branch 验收组与 S3 上游 GPU 参考不一致（{} 条）",
        failures.len()
    );
}

#[test]
#[ignore = "requires NVIDIA GPU + NVRTC"]
fn mesh_limits_group_records_device_vs_upstream_values() {
    // U7/U8 = record-only：只打印/落 facts（count/dist/pos），不参与 pass/fail 分母。
    // U8 口径：上游支持 20 顶点 polygon（动态容量），设备 FACE_MAX=16 逐 polygon 回退 →
    // 行为差异 / 限制存在；设备实测 != 4 如实记录，不动 fixture/判据/代码。
    let program = CollisionGpuProgram::new(0).expect("NVRTC 编译设备源并加载模块/入口");
    let reference = read_reference(LIMITS_REFERENCE, LIMITS_REFERENCE_SHA256);
    for case in &LIMIT_CASES {
        let case_reference = &reference["cases"][case.name];
        assert_eq!(
            case_reference["two_captures_bitwise_identical"].as_bool(),
            Some(true),
            "{}: S3 参考应记录两次捕获逐位一致",
            case.name
        );
        let output = run(&program, case, case_reference);
        let ref_count = case_reference["output"]["count"]
            .as_u64()
            .unwrap_or(u64::MAX);
        let device_dist: Vec<f32> = output.dist.clone();
        let reference_dist = reference_floats(case_reference, "dist");
        let device_pos: Vec<f32> = output.pos.clone();
        let reference_pos = reference_floats(case_reference, "pos");
        println!(
            "RECORD-ONLY limits case={} device_count={} ref_count={} device_dist=[{}] ref_dist=[{}] device_pos=[{}] ref_pos=[{}]",
            case.name,
            output.nacon,
            ref_count,
            format_floats(&device_dist),
            format_floats(&reference_dist),
            format_points(&device_pos),
            format_points(&reference_pos)
        );
    }
}

#[test]
fn reference_support_parses_without_gpu() {
    let branch = read_reference(BRANCH_REFERENCE, BRANCH_REFERENCE_SHA256);
    let limits = read_reference(LIMITS_REFERENCE, LIMITS_REFERENCE_SHA256);
    assert_eq!(branch["cases"].as_object().expect("branch cases").len(), 9);
    assert_eq!(limits["cases"].as_object().expect("limits cases").len(), 2);
    let expected_branch_counts = [3_u64, 4, 4, 2, 2, 1, 2, 4, 3];
    for (case, expected) in BRANCH_CASES.iter().zip(expected_branch_counts) {
        let reference = &branch["cases"][case.name];
        assert_eq!(
            reference["two_captures_bitwise_identical"].as_bool(),
            Some(true),
            "{}: 两次捕获逐位一致",
            case.name
        );
        assert_eq!(
            reference["output"]["count"].as_u64(),
            Some(expected),
            "{}: 参考 count",
            case.name
        );
        for row in reference["output"]["fields"]["geom"]
            .as_array()
            .expect("geom")
        {
            assert_eq!(
                row.as_array().expect("geom row").len(),
                2,
                "{}: geom 宽度",
                case.name
            );
        }
    }
    let expected_limit_counts = [1_u64, 4];
    for (case, expected) in LIMIT_CASES.iter().zip(expected_limit_counts) {
        let reference = &limits["cases"][case.name];
        assert_eq!(
            reference["output"]["count"].as_u64(),
            Some(expected),
            "{}: 参考 count",
            case.name
        );
    }
}

#[test]
fn fixtures_match_s3_signature() {
    for (relative, expected) in FIXTURE_SHA256 {
        let path = root().join(relative);
        let bytes =
            std::fs::read(&path).unwrap_or_else(|error| panic!("读取 {relative} 失败：{error}"));
        assert_eq!(
            sha256_hex(&bytes),
            expected,
            "{relative}: sha256 与 S3 签名集合不一致"
        );
    }
}

#[test]
fn mesh_topology_matches_s3_precheck_and_validates() {
    let branch = read_reference(BRANCH_REFERENCE, BRANCH_REFERENCE_SHA256);
    let limits = read_reference(LIMITS_REFERENCE, LIMITS_REFERENCE_SHA256);
    for case in BRANCH_CASES.iter().chain(LIMIT_CASES.iter()) {
        let Some(mesh) = case.mesh else {
            continue;
        };
        let nmesh = mesh.polynum.len();
        assert_eq!(
            nmesh,
            mesh.polyadr.len(),
            "{}: polynum/polyadr 长度",
            case.name
        );
        assert_eq!(nmesh, mesh.vertadr.len(), "{}: vertadr 长度", case.name);
        assert_eq!(
            mesh.vertadr.len(),
            mesh.vertnum.len(),
            "{}: vertadr/vertnum",
            case.name
        );
        // 与 S3 precheck 的拓扑统计逐项对账。
        let meshvert = mesh.polymapadr.len();
        let meshpoly = mesh.polyvertadr.len();
        match case.name {
            "mesh-mesh-two-assets" => {
                assert_eq!((nmesh, meshvert, meshpoly), (2, 12, 10), "U6 拓扑");
            }
            "mesh-limits-cone-box" => {
                assert_eq!((nmesh, meshvert, meshpoly), (1, 25, 25), "U7 拓扑");
                assert!(mesh.polynum[0] == 25, "U7 polynum");
            }
            "mesh-limits-prism-box" => {
                assert_eq!((nmesh, meshvert, meshpoly), (1, 40, 22), "U8 拓扑");
                assert!(mesh.polynum[0] == 22, "U8 polynum");
            }
            other => panic!("未知 mesh case {other}"),
        }
        // 逐 case 规模/值域：polyvert 元素为逐 mesh 局部顶点号、polymap 为逐 mesh 局部 polygon 号。
        for mesh_index in 0..nmesh {
            let vertnum = mesh.vertnum[mesh_index] as usize;
            let polynum = mesh.polynum[mesh_index] as usize;
            let polyadr = mesh.polyadr[mesh_index] as usize;
            for polygon in polyadr..polyadr + polynum {
                let adr = mesh.polyvertadr[polygon] as usize;
                let num = mesh.polyvertnum[polygon] as usize;
                assert!(
                    mesh.polyvert[adr..adr + num]
                        .iter()
                        .all(|value| *value >= 0 && (*value as usize) < vertnum),
                    "{}: polyvert 值域",
                    case.name
                );
            }
            let vertadr = mesh.vertadr[mesh_index] as usize;
            for vertex in vertadr..vertadr + vertnum {
                let adr = mesh.polymapadr[vertex] as usize;
                let num = mesh.polymapnum[vertex] as usize;
                assert!(
                    mesh.polymap[adr..adr + num]
                        .iter()
                        .all(|value| *value >= 0 && (*value as usize) < polynum),
                    "{}: polymap 值域",
                    case.name
                );
            }
        }
        // 真实校验入口：sidecar 存在时必须 Ok（含 O7 值域校验；不建 CUDA context）。
        let source = if case.name.starts_with("mesh-limits") {
            &limits
        } else {
            &branch
        };
        let inputs = CaseInputs::build(case, &source["cases"][case.name]);
        assert_eq!(
            mjwarp_rs::runtime::collision_gpu::validate_collision_inputs(&inputs.inputs()),
            Ok(()),
            "{}: 真实 validate_collision_inputs 应通过（32 项 + mesh sidecar）",
            case.name
        );
    }
}
