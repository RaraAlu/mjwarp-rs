//! S2：S1 快照 → `CollisionGpuInputs` adapter 的纯 CPU 对账（无 GPU、无产品接线）。
//!
//! 写死口径：M0b/M3 的 32 数组 bins 逐数组 sha/位型相等；旧 23 项输入（无 9 项 mesh_poly）
//! → 9 项空 slice、`validate_collision_inputs` 通过、sidecar 不构造；部分为空 → `InvalidArgument`。

#[path = "support/collision_inputs_adapter_support.rs"]
mod adapter_support;

use adapter_support::{AdapterCase, CASES, MESH_POLY_IDS, input_bytes};
#[cfg(feature = "native-model-probe")]
use adapter_support::{f32_bytes, i32_bytes};
use mjwarp_rs::{
    model::collision_inputs::COLLISION_INPUTS_FIELDS,
    runtime::collision_gpu::{
        CollisionGpuError, inputs_from_snapshot, mesh_poly_sidecar_present,
        validate_collision_inputs,
    },
};
use sha2::{Digest, Sha256};

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn adapter_matches_m0b_bins_field_by_field() {
    assert_eq!(COLLISION_INPUTS_FIELDS.len(), 32);
    for &(name, dir) in &CASES {
        let case = AdapterCase::read(name, dir);
        let counts = case.counts();
        assert_eq!(case.meta.nworld, 1, "{name}");
        assert_eq!(i64::from(case.meta.ngeom), counts.ngeom, "{name}");
        // Every field length must equal the S1 spec expectation derived from bins.
        for id in 0..32_u32 {
            let spec = &COLLISION_INPUTS_FIELDS[id as usize];
            let expected = counts.expected(spec).unwrap();
            // Host-side inputs are always 4-byte scalars (f32/i32), not native widths.
            assert_eq!(
                case.field(id).raw.len(),
                expected * 4,
                "{name} {}",
                spec.name
            );
        }
        let view = case.snapshot(true);
        let inputs = inputs_from_snapshot(&view, case.meta).unwrap();
        validate_collision_inputs(&inputs).unwrap();
        assert!(mesh_poly_sidecar_present(&inputs), "{name}");
        for id in 0..32_u32 {
            let spec = &COLLISION_INPUTS_FIELDS[id as usize];
            let actual = input_bytes(&inputs, id);
            let expected = case.field(id).raw.as_slice();
            assert_eq!(actual.as_slice(), expected, "{name} {}", spec.name);
            assert_eq!(
                sha256(&actual),
                sha256(expected),
                "{name} {} sha",
                spec.name
            );
        }
    }
}

#[test]
fn legacy_23_inputs_leave_mesh_poly_empty_and_sidecar_null() {
    let case = AdapterCase::read(CASES[0].0, CASES[0].1);
    let view = case.snapshot(false);
    let inputs = inputs_from_snapshot(&view, case.meta).unwrap();
    validate_collision_inputs(&inputs).unwrap();
    assert!(!mesh_poly_sidecar_present(&inputs), "sidecar 必须为 null");
    for id in MESH_POLY_IDS {
        assert!(input_bytes(&inputs, id).is_empty(), "字段 {id} 应为空");
    }
    for id in 0..32_u32 {
        if MESH_POLY_IDS.contains(&id) {
            continue;
        }
        assert_eq!(
            input_bytes(&inputs, id).as_slice(),
            case.field(id).raw.as_slice(),
            "23 项基础字段 {id} 必须原样透传"
        );
    }
}

#[test]
fn partial_mesh_poly_and_inconsistent_inputs_are_rejected() {
    let case = AdapterCase::read(CASES[0].0, CASES[0].1);
    // 部分为空：只给 mesh_polynum，其余 8 项空 → InvalidArgument（不静默禁用）。
    let mut partial = case.snapshot(false);
    partial.mesh_polynum = case.i32s(15);
    let error = inputs_from_snapshot(&partial, case.meta).unwrap_err();
    match error {
        CollisionGpuError::InvalidArgument(message) => {
            assert!(message.contains("不完整"), "message={message}");
        }
        other => panic!("期望 InvalidArgument，得到 {other:?}"),
    }
    // 基础数组长度与 meta.ngeom 不一致。
    let mut bad_xpos = case.snapshot(true);
    bad_xpos.geom_xpos = &[];
    let error = inputs_from_snapshot(&bad_xpos, case.meta).unwrap_err();
    match error {
        CollisionGpuError::InvalidArgument(message) => {
            assert!(message.contains("geom_xpos"), "message={message}");
        }
        other => panic!("期望 InvalidArgument，得到 {other:?}"),
    }
    let mut bad_type = case.snapshot(true);
    bad_type.geom_type = &case.i32s(7)[..case.counts().ngeom as usize - 1];
    let error = inputs_from_snapshot(&bad_type, case.meta).unwrap_err();
    match error {
        CollisionGpuError::InvalidArgument(message) => {
            assert!(message.contains("geom_type"), "message={message}");
        }
        other => panic!("期望 InvalidArgument，得到 {other:?}"),
    }
    // meta 非法：nworld/naconmax/efc_width 非正。
    for (mut meta, needle) in [
        (case.meta, "nworld"),
        (case.meta, "naconmax"),
        (case.meta, "efc_width"),
    ] {
        match needle {
            "nworld" => meta.nworld = 0,
            "naconmax" => meta.naconmax = 0,
            _ => meta.efc_width = 0,
        }
        let error = inputs_from_snapshot(&case.snapshot(true), meta).unwrap_err();
        match error {
            CollisionGpuError::InvalidArgument(message) => {
                assert!(message.contains(needle), "message={message}");
            }
            other => panic!("期望 InvalidArgument，得到 {other:?}"),
        }
    }
    // 空 model 帧（ngeom>0 但 xpos/xmat 空）在 from_model_input 之外仍由 validate 拒绝。
    let mut empty_frame = case.snapshot(true);
    empty_frame.geom_xpos = &[];
    assert!(inputs_from_snapshot(&empty_frame, case.meta).is_err());
}

#[cfg(feature = "native-model-probe")]
#[test]
#[ignore = "需要可信MuJoCo DLL"]
fn from_model_input_maps_s1_snapshot_into_adapter_view() {
    use mjwarp_rs::io::NativeModelProbe;
    use std::path::PathBuf;

    const MJB: &[u8] = include_bytes!("../fixtures/render-bvh/scene-a/scene-312.mjb");
    fn model_bytes(model: &mjwarp_rs::io::CollisionInputsModelInput, id: u32) -> Vec<u8> {
        match model.field(id).unwrap() {
            mjwarp_rs::io::CollisionInputsFieldValue::Float(values) => f32_bytes(values),
            mjwarp_rs::io::CollisionInputsFieldValue::Index(values) => i32_bytes(values),
        }
    }
    let dll: PathBuf = std::env::var_os("MJWARP_MUJOCO_DLL").unwrap().into();
    // SAFETY: verified 3.12 DLL and frozen MJB, as in the S1 host tests.
    let probe = unsafe { NativeModelProbe::load_trusted(&dll, MJB) }.unwrap();
    let model = probe
        .collision_inputs_snapshot()
        .unwrap()
        .into_inputs()
        .unwrap();
    let counts = model.counts();
    let mut data = probe.collision_inputs_data().unwrap();
    assert_eq!(
        usize::try_from(data.nq()).unwrap(),
        usize::try_from(counts.nq).unwrap()
    );
    data.set_qpos(&vec![0.0; usize::try_from(data.nq()).unwrap()])
        .unwrap();
    let frame = data.capture_frame().unwrap();
    assert_eq!(
        frame.geom_xpos.len(),
        usize::try_from(counts.ngeom).unwrap() * 3
    );
    assert_eq!(
        frame.geom_xmat.len(),
        usize::try_from(counts.ngeom).unwrap() * 9
    );
    let view = mjwarp_rs::runtime::collision_gpu::CollisionInputsSnapshot::from_model_input(
        &model, &frame,
    )
    .unwrap();
    let meta = mjwarp_rs::runtime::collision_gpu::CollisionGpuMeta {
        nworld: 1,
        ngeom: i32::try_from(counts.ngeom).unwrap(),
        g1: 0,
        g2: 1,
        type1: 7,
        type2: 7,
        geomgeomid: 0,
        naccdmax: 64,
        hfield_id: -1,
        oct_root: -1,
        noct: i32::try_from(counts.noct).unwrap(),
        tolerance: 1e-6,
        margin: 0.0,
        gap: 0.0,
        gjk_iterations: 35,
        epa_iterations: 35,
        condim: 3,
        pairid0: -1,
        pairid1: -1,
        friction: [1.0, 1.0, 0.005, 1e-4, 1e-4],
        solref: [0.02, 1.0],
        solreffriction: [0.0, 0.0],
        solimp: [0.9, 0.95, 0.001, 0.5, 2.0],
        adhesion: 0.0,
        sdf_initpoints: 40,
        sdf_iterations: 10,
        naconmax: 64,
        efc_width: 4,
    };
    let inputs = inputs_from_snapshot(&view, meta).unwrap();
    validate_collision_inputs(&inputs).unwrap();
    assert_eq!(input_bytes(&inputs, 0), f32_bytes(&frame.geom_xpos));
    assert_eq!(input_bytes(&inputs, 1), f32_bytes(&frame.geom_xmat));
    for id in 2..32_u32 {
        assert_eq!(
            input_bytes(&inputs, id).as_slice(),
            model_bytes(&model, id).as_slice(),
            "字段 {id}"
        );
    }
}
