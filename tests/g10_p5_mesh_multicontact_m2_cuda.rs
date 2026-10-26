//! G10@P5 M2：mesh multicontact 设备入口的 GPU 窗口验收（默认 ignored）。
//!
//! 用产品 API 重跑 M0b 产出的 32 数组 bins（同一 pinned 模型 + 同一 fixture），对照 M1 上游
//! GPU 参考 `docs/development-evidence/mesh-multicontact-m1/upstream-mesh-multicontact-gpu.json`
//! （sha 守卫 `c83baffe…`，authoring-only、非产品精度结论）。两个 case：
//! - `mesh-box`：`(BOX, MESH)`，调用方按 pinned 类型序传 `g1=1(箱), g2=0(mesh)`、
//!   `type1=6, type2=7` → `contact.geom=[1,0]`；
//! - `mesh-cylinder`：`(CYLINDER, MESH)`，同上。
//!
//! 内核入口：`CollisionKernel::ContactMulti`（上游 `use_multiccd=true` 特化）。
//!
//! 判据：见 `support/mesh_multicontact_compare_support.rs`（位级优先、dist 4 ULP@量级
//! （ref=0→4 ULP@1.0）、pos 4·ULP(scale)、frame 4 ULP@1.0、结构/参数零容差、
//! `contact.geom` 顺序 `[1,0]`、overflow 全 0、efc 参考未记录→设备全 -1）。
//!
//! 运行：`cargo test --locked --features cuda-probe --test g10_p5_mesh_multicontact_m2_cuda -- --ignored --test-threads=1 --nocapture`

#![cfg(feature = "cuda-probe")]

#[path = "support/mesh_poly_case_support.rs"]
mod case_support;
#[path = "support/mesh_multicontact_compare_support.rs"]
mod compare;

use mjwarp_rs::runtime::collision_gpu::{CollisionGpuProgram, CollisionKernel};
use sha2::{Digest, Sha256};

use case_support::{load_case, m0b_cases_root, m1_reference};

const CASES: [&str; 2] = ["mesh-box", "mesh-cylinder"];
/// M1 参考（authoring-only 参考）的冻结 sha256。
const M1_REFERENCE_SHA256: &str =
    "c83baffe6de695fdefda6852c574d5d0d7312ebb591c1b487dfc59ec126c206a";

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
#[ignore = "requires NVIDIA GPU + NVRTC"]
fn mesh_multicontact_launcher_matches_m1_upstream_gpu() {
    let reference_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs/development-evidence/mesh-multicontact-m1/upstream-mesh-multicontact-gpu.json");
    let reference_bytes = std::fs::read(&reference_path).expect("读取 M1 参考");
    assert_eq!(
        sha256_hex(&reference_bytes),
        M1_REFERENCE_SHA256,
        "M1 参考 sha256 与冻结值不符（参考不可被改动）"
    );
    let reference = m1_reference();

    let program = CollisionGpuProgram::new(0).expect("NVRTC 编译设备源并加载 6 个入口");
    let mut failures = 0usize;
    for case in CASES {
        let case_reference = &reference["cases"][case];
        assert_eq!(
            case_reference["two_captures_bitwise_identical"].as_bool(),
            Some(true),
            "{case}: M1 参考应记录每 case 两次捕获逐位一致"
        );

        let inputs = load_case(&m0b_cases_root().join(case));
        assert!(
            inputs.mesh_poly_present(),
            "{case}: M0b bins 应带 9 项 sidecar"
        );
        // 调用方按 pinned `_add_geom_pair` 类型序传入：box(6) < mesh(7) → (g1=箱, g2=mesh)。
        assert_eq!(inputs.meta.g1, 1, "{case}: g1 应为箱/柱（低类在前）");
        assert_eq!(inputs.meta.g2, 0, "{case}: g2 应为 mesh");
        assert!(
            inputs.meta.type2 == 7,
            "{case}: type2 应为 G5_MESH(7)，实际 {}",
            inputs.meta.type2
        );
        assert!(
            inputs.meta.type1 == 6 || inputs.meta.type1 == 5,
            "{case}: type1 应为 BOX(6) 或 CYLINDER(5)，实际 {}",
            inputs.meta.type1
        );

        let output = program
            .run(CollisionKernel::ContactMulti, &inputs.inputs())
            .unwrap_or_else(|error| panic!("{case} 运行失败：{error}"));
        let report = compare::compare_case(case, case_reference, &output);
        report.print();
        failures += report.failures.len();
    }
    assert_eq!(failures, 0, "mesh multicontact 对照失败 {failures} 条");
}
