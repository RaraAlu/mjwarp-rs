//! G10@P5 M3：mesh-mesh multicontact 设备入口的 GPU 窗口验收（默认 ignored）。
//!
//! 用产品 API 重跑 M3 产出的 32 数组 bins（同一 pinned 模型 + 同一 fixture
//! `fixtures/g10-p5-mesh-multicontact/mesh-mesh.xml`），对照 M3 上游 GPU 参考
//! `docs/development-evidence/mesh-multicontact-m3/upstream-mesh-multicontact-mesh-mesh-gpu.json`
//! （sha 守卫 `96e7895d…`，authoring-only、非产品精度结论）。一个 case：
//! - `mesh-mesh`：`(MESH, MESH)`，调用方按 geom 序传 `g1=0, g2=1`、`type1=type2=7`
//!   → `contact.geom=[0,1]`（两侧同类型、pinned 类型序不交换）。
//!
//! 内核入口：`CollisionKernel::ContactMulti`（上游 `use_multiccd=true` 特化）。
//!
//! 判据同 M2（`support/mesh_multicontact_compare_support.rs`，零放宽）：位级优先、dist 4 ULP
//! @量级（ref=0→4 ULP@1.0）、pos 4·ULP(scale)、frame 4 ULP@1.0、结构/参数零容差、nacon 与
//! 参考 count 零容差、overflow 全 0、efc 参考未记录→设备全 -1。
//!
//! 运行：`cargo test --locked --features cuda-probe --test g10_p5_mesh_multicontact_m3_cuda -- --ignored --test-threads=1 --nocapture`

#![cfg(feature = "cuda-probe")]

#[path = "support/mesh_poly_case_support.rs"]
mod case_support;
#[path = "support/mesh_multicontact_compare_support.rs"]
mod compare;

use mjwarp_rs::runtime::collision_gpu::{CollisionGpuProgram, CollisionKernel};
use sha2::{Digest, Sha256};

use case_support::{load_case, m3_cases_root, m3_reference};

const CASES: [&str; 1] = ["mesh-mesh"];
/// M3 参考（authoring-only 参考）的冻结 sha256。
const M3_REFERENCE_SHA256: &str =
    "96e7895d13c3ba23475a013fcaa586485e93f95cced5351e2874973917e149a1";
/// `nmesh=2 / nmeshvert=16 / nmeshpoly=12 / nmeshpolyvert=48 / nmeshpolymap=48` 推导出的
/// 9 个 sidecar slice 长度（polyadr 逐 mesh、polyvert* 逐 polygon、polymap* 逐 vertex）。
const M3_MESH_POLY_LENGTHS: [usize; 9] = [2, 2, 36, 12, 12, 48, 16, 16, 48];

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
#[ignore = "requires NVIDIA GPU + NVRTC"]
fn mesh_mesh_multicontact_launcher_matches_m3_upstream_gpu() {
    let reference_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "docs/development-evidence/mesh-multicontact-m3/upstream-mesh-multicontact-mesh-mesh-gpu.json",
    );
    let reference_bytes = std::fs::read(&reference_path).expect("读取 M3 参考");
    assert_eq!(
        sha256_hex(&reference_bytes),
        M3_REFERENCE_SHA256,
        "M3 参考 sha256 与冻结值不符（参考不可被改动）"
    );
    let reference = m3_reference();

    let program = CollisionGpuProgram::new(0).expect("NVRTC 编译设备源并加载 6 个入口");
    let mut failures = 0usize;
    for case in CASES {
        let case_reference = &reference["cases"][case];
        assert_eq!(
            case_reference["two_captures_bitwise_identical"].as_bool(),
            Some(true),
            "{case}: M3 参考应记录每 case 两次捕获逐位一致"
        );

        let inputs = load_case(&m3_cases_root().join(case));
        assert!(
            inputs.mesh_poly_present(),
            "{case}: M3 bins 应带 9 项 sidecar"
        );
        assert_eq!(
            inputs.mesh_poly_lengths(),
            M3_MESH_POLY_LENGTHS,
            "{case}: 9 个 sidecar slice 长度须与 nmesh/nmeshvert/nmeshpoly 推导一致"
        );
        // (MESH, MESH)：两侧同类型，调用方按 geom 序传 g1=0 / g2=1（pinned 类型序不交换）。
        assert_eq!(inputs.meta.g1, 0, "{case}: g1 应为 mesh A（geom 0）");
        assert_eq!(inputs.meta.g2, 1, "{case}: g2 应为 mesh B（geom 1）");
        assert_eq!(inputs.meta.type1, 7, "{case}: type1 应为 G5_MESH(7)");
        assert_eq!(inputs.meta.type2, 7, "{case}: type2 应为 G5_MESH(7)");
        assert_eq!(
            inputs.geom_dataid,
            vec![0, 1],
            "{case}: geom_dataid 应为两个 mesh（0/1）"
        );

        let output = program
            .run(CollisionKernel::ContactMulti, &inputs.inputs())
            .unwrap_or_else(|error| panic!("{case} 运行失败：{error}"));
        let report = compare::compare_case_with_geom_order(case, case_reference, &output, "[0,1]");
        report.print();
        failures += report.failures.len();
    }
    assert_eq!(failures, 0, "mesh-mesh multicontact 对照失败 {failures} 条");
}
