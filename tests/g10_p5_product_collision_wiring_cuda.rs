//! S3：产品碰撞入口端到端 target（真实 pinned MJB → S1 快照（qpos）→ S2 adapter → 设备
//! 输出 → M1（mesh-box / mesh-cylinder）/ M3（mesh-mesh）冻结参考）。
//!
//! 判据沿用 M2/M3 口径、零放宽：位级优先 + dist 4 ULP@量级（ref=0→4 ULP@1.0）+
//! pos 4·ULP(scale) + frame 4 ULP@1.0 + 结构/参数零容差 + `nacon` 与参考 count 零容差 +
//! `contact.geom` 按 pinned 类型序 + overflow 全 0 + efc 参考未记录→设备全 -1。
//! 参考只读（sha 守卫写死在 support），不重采、不改判据。
//!
//! 两个 ignored 用例（窗口执行；测试本身不启动 Python）：
//! - `s1_snapshot_of_pinned_mjb_matches_frozen_case_bins`（需 pinned DLL，不需 GPU）：
//!   S1 快照 32 项与 M0b/M3 冻结 bins 逐字节相等 —— 端到端输入宿主侧与冻结 bins 同口径。
//! - `product_entry_device_output_matches_m1_and_m3_frozen_references`（需 GPU + NVRTC + DLL）：
//!   产品入口 `BackendChoice::Device` 的 32 项输入由 S1 快照 + S2 adapter 组装，设备回读对照
//!   冻结参考；三态回退中 `Host(..)` 视为设备路径未执行并带原因失败。
//!
//! 运行（pinned MuJoCo 3.12.0 DLL + NVIDIA GPU + NVRTC）：
//! ```text
//! cargo +1.99.0-x86_64-pc-windows-msvc test --locked --features native-model-probe \
//!   --test g10_p5_product_collision_wiring_cuda -- --ignored --test-threads=1 --nocapture
//! ```

#![cfg(all(feature = "cuda", feature = "native-model-probe"))]

#[path = "support/collision_inputs_adapter_support.rs"]
mod adapter_support;
#[path = "support/mesh_multicontact_compare_support.rs"]
mod compare;
#[path = "support/product_collision_wiring_support.rs"]
mod wiring_support;

use adapter_support::{AdapterCase, input_bytes, parse_meta};
use mjwarp_rs::io::{CollisionInputsFrame, CollisionInputsModelInput, NativeModelProbe};
use mjwarp_rs::runtime::backend::BackendChoice;
use mjwarp_rs::runtime::collision_gpu::{
    CollisionInputsSnapshot, CollisionKernel, ProductCollisionOutcome, ProductCollisionRequest,
    inputs_from_snapshot, run_product_collision,
};
use serde_json::Value;

use wiring_support::{
    CASES, CaseFixture, dll_path, mjb_bytes, qpos_f64, reference_value, repo_path,
};

/// 读取 pinned MJB → S1 模型快照 + qpos 帧快照（全部为拥有式数据，probe 可在本函数结束时释放）。
fn snapshot_for(
    case: &CaseFixture,
    case_reference: &Value,
) -> (CollisionInputsModelInput, CollisionInputsFrame) {
    let mjb = mjb_bytes(case);
    // SAFETY: 调用方保证 `MJWARP_MUJOCO_DLL` 指向 pinned/可信 MuJoCo 3.12.0 DLL；MJB 字节
    // 已按冻结 sha256 校验；probe 在本函数内保持存活，快照析构早于 probe 释放。
    let probe = unsafe { NativeModelProbe::load_trusted(&dll_path(), &mjb) }
        .unwrap_or_else(|error| panic!("{}: 加载 pinned MJB 失败：{error}", case.name));
    let model = probe
        .collision_inputs_snapshot()
        .unwrap_or_else(|error| panic!("{}: S1 模型快照失败：{error}", case.name))
        .into_inputs()
        .unwrap_or_else(|error| panic!("{}: S1 模型字段收窄失败：{error}", case.name));
    let qpos = qpos_f64(case_reference);
    let mut data = probe
        .collision_inputs_data()
        .unwrap_or_else(|error| panic!("{}: 创建 mjData 失败：{error}", case.name));
    assert_eq!(
        usize::try_from(data.nq()).expect("nq"),
        qpos.len(),
        "{}: 参考 qpos 长度须等于模型 nq",
        case.name
    );
    data.set_qpos(&qpos)
        .unwrap_or_else(|error| panic!("{}: set_qpos/mj_kinematics 失败：{error}", case.name));
    let frame = data
        .capture_frame()
        .unwrap_or_else(|error| panic!("{}: 复制 geom_xpos/geom_xmat 失败：{error}", case.name));
    (model, frame)
}

/// 参考中该 case 的节点（`cases.<name>`）。
fn case_reference<'a>(reference: &'a Value, case: &CaseFixture) -> &'a Value {
    &reference["cases"][case.name]
}

/// 冻结 case `meta.txt`（M0b/M3 只读；与本 target 的运行时配对参数同口径）。
fn case_meta(case: &CaseFixture) -> mjwarp_rs::runtime::collision_gpu::CollisionGpuMeta {
    parse_meta(&repo_path(case.bins).join("meta.txt"))
}

#[test]
#[ignore = "需要 pinned MuJoCo 3.12.0 DLL（无 GPU）"]
fn s1_snapshot_of_pinned_mjb_matches_frozen_case_bins() {
    let mut failures = 0usize;
    for case in &CASES {
        let reference = reference_value(case);
        let (model, frame) = snapshot_for(case, case_reference(&reference, case));
        let view = CollisionInputsSnapshot::from_model_input(&model, &frame)
            .unwrap_or_else(|error| panic!("{}: S1 快照装 S2 视图失败：{error}", case.name));
        let inputs = inputs_from_snapshot(&view, case_meta(case))
            .unwrap_or_else(|error| panic!("{}: S2 adapter 校验失败：{error}", case.name));
        let bins = AdapterCase::read(case.name, case.bins);
        let mut mismatch = 0usize;
        for id in 0..32_u32 {
            if input_bytes(&inputs, id) != bins.field(id).raw {
                mismatch += 1;
                println!("FAIL {}: 字段 {id} 与冻结 bins 不等", case.name);
            }
        }
        println!(
            "case={} s1_fields=32/32 bins_match=true mismatches={mismatch}",
            case.name
        );
        failures += mismatch;
    }
    assert_eq!(failures, 0, "S1 快照与冻结 bins 对账失败 {failures} 项");
}

#[test]
#[ignore = "需要 NVIDIA GPU + NVRTC + pinned MuJoCo 3.12.0 DLL"]
fn product_entry_device_output_matches_m1_and_m3_frozen_references() {
    let mut failures = 0usize;
    let mut bit_rows = 0usize;
    let mut total_rows = 0usize;
    for case in &CASES {
        let reference = reference_value(case);
        let reference_case = case_reference(&reference, case);
        assert_eq!(
            reference_case["two_captures_bitwise_identical"].as_bool(),
            Some(true),
            "{}: 冻结参考应记录两次捕获逐位一致",
            case.name
        );
        let (model, frame) = snapshot_for(case, reference_case);
        let view = CollisionInputsSnapshot::from_model_input(&model, &frame)
            .unwrap_or_else(|error| panic!("{}: S1 快照装 S2 视图失败：{error}", case.name));
        let request = ProductCollisionRequest {
            snapshot: view,
            meta: case_meta(case),
            kernel: CollisionKernel::ContactMulti,
        };
        let output = match run_product_collision(BackendChoice::Device, request)
            .unwrap_or_else(|error| panic!("{}: 产品入口返回 CPU 错误：{error}", case.name))
        {
            ProductCollisionOutcome::Device(output) => output,
            ProductCollisionOutcome::Host(reason) => {
                panic!("{}: 设备路径未执行，回退 Host：{reason:?}", case.name)
            }
        };
        let report = compare::compare_case_with_geom_order(
            case.name,
            reference_case,
            &output,
            case.geom_order,
        );
        report.print();
        bit_rows += report.bit_equal;
        total_rows += report.total;
        failures += report.failures.len();
    }
    println!(
        "e2e_summary cases={} bit_equal={bit_rows}/{total_rows} failures={failures}",
        CASES.len()
    );
    assert_eq!(failures, 0, "端到端对照失败 {failures} 条");
}
