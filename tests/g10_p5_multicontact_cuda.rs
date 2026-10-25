//! G10@P5：multicontact 设备入口的 GPU 窗口验收（默认 ignored，本机不跑）。
//!
//! 用产品 API 重跑 3 个冻结 case（与上游 dispatch/fixture flag 对应）：
//! - `cylinder-box`（multiccd=enable）→ `CollisionKernel::ContactMulti`（<=4 接触）；
//! - `box-box-margin`（nativeccd=disable → `(BOX,BOX)` 走 primitive）→ `CollisionKernel::BoxBox`；
//! - `mesh-mesh`（multiccd 未启用）→ 既有 `CollisionKernel::Contact`（单接触）。
//!
//! 参考是 QA 已提交的上游 f32 GPU 捕获 `qa-upstream-gpu.json`（输入 23/23 sha256 与 G5 case bins
//! 相同，故直接复用；运行时重算 sha256 守卫）。
//!
//! 判据沿用 G5：按 `(worldid, geomcollisionid)` 稳定排序后逐字段比较；位级优先，非位级
//! `dist` 按自身量级 ≤4 ULP、`pos` 按 `scale=max_i|ref_i|` 的 4·ULP、`frame` 按 scale=1.0 的
//! 4 ULP；参数/整数/`nacon`/`overflow` 位级。按 case 早停：首个失败 case 打印明细后 panic。
//!
//! 运行：`cargo test --locked --features cuda-probe --test g10_p5_multicontact_cuda -- --ignored --test-threads=1 --nocapture`

#![cfg(feature = "cuda-probe")]

#[path = "support/multicontact_compare_support.rs"]
mod compare;
#[path = "support/multicontact_qa_support.rs"]
mod support;

use mjwarp_rs::runtime::collision_gpu::CollisionGpuProgram;

const CASES: [&str; 3] = ["cylinder-box", "box-box-margin", "mesh-mesh"];

#[test]
#[ignore = "requires NVIDIA GPU + NVRTC"]
fn multicontact_launcher_matches_qa_upstream_gpu() {
    let program = CollisionGpuProgram::new(0).expect("NVRTC 编译设备源并加载 6 个入口");
    let reference = support::load_reference();
    for case in CASES {
        let (kernel, output) = support::run_case(&program, case, &reference);
        let report = compare::compare_case(
            case,
            &format!("{kernel:?}"),
            &reference["cases"][case],
            &output,
        );
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
