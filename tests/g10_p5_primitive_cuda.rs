//! G10@P5：primitive narrowphase 设备入口的 GPU 窗口验收（默认 ignored，本机不跑）。
//!
//! 用产品 API（`CollisionGpuProgram::new(0)` + `run(CollisionKernel::Primitive, ..)`）重跑
//! 3 个冻结 case（`sphere-box-shallow` / `sphere-box-deep` / `capsule-box`），参考是独立
//! QA 已提交的上游 f32 GPU 捕获 `qa-upstream-gpu.json`（`cases.<case>`；输入 23/23
//! sha256 与 G5 case bins 相同，故可直接复用）。`box-box-margin` 本片未实现，不在覆盖内。
//!
//! 判据沿用 G5：先按 `(worldid, dist, pos, frame)` 稳定排序（重复点按多重集），逐字段比较；
//! 位级优先，非位级 `dist` 按自身量级 ≤4 ULP（`ref == 0` 用 4 ULP@1.0）、`pos` 按
//! `scale = max_i |ref_i|` 的 `4·ULP(scale)`、`frame` 按 scale=1.0 的 4 ULP；参数/整数/
//! `nacon`/`nccd`/`overflow` 位级。按 case 早停：首个失败 case 打印明细后立即 panic，
//! 后续 case 不跑（避免用其余 case 通过掩盖单个失败）。
//!
//! 共享 helper 见 `tests/support/primitive_{qa,compare}_support.rs`（`#[path]` 引入）。
//! 运行：`cargo test --locked --features cuda-probe --test g10_p5_primitive_cuda -- --ignored --test-threads=1 --nocapture`

#![cfg(feature = "cuda-probe")]

#[path = "support/primitive_compare_support.rs"]
mod compare;
#[path = "support/primitive_qa_support.rs"]
mod support;

use mjwarp_rs::runtime::collision_gpu::CollisionGpuProgram;

const CASES: [&str; 3] = ["sphere-box-shallow", "sphere-box-deep", "capsule-box"];

#[test]
#[ignore = "requires NVIDIA GPU + NVRTC"]
fn primitive_launcher_matches_qa_upstream_gpu() {
    let program = CollisionGpuProgram::new(0).expect("NVRTC 编译设备源并加载模块/入口");
    let reference = support::load_reference();
    for case in CASES {
        let output = support::run_case(&program, case, &reference);
        let report = compare::compare_case(case, &reference["cases"][case], &output);
        report.print();
        if !report.failures.is_empty() {
            for failure in &report.failures {
                println!("FAIL {case}: {failure}");
            }
            panic!(
                "case {case} 未通过判据（{} 条）；按窗口纪律早停，后续 case 未跑",
                report.failures.len()
            );
        }
    }
}
