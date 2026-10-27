#![cfg(feature = "native-model-probe")]
//! S1：产品碰撞输入 data 快照探针的主机验收。
//! 只读取冻结MJB与pinned oracle；不执行物理、不使用GPU、不做产品接线。
//!
//! oracle 由 pinned MuJoCo 3.12.0 Python 生成
//! （`docs/development-evidence/product-collision-inputs-wiring/scripts/generate_collision_inputs_oracle.py`）：
//! 32项字段的计数与逐元素原生值、以及 qpos→`mj_kinematics` 后 `geom_xpos`/`geom_xmat`
//! 的 f32 位型（numpy astype 同舍入）。oct_* 体量过大，sdf 模型仅用体积桥做同模型
//! 逐元素交叉核对。

#[path = "support/collision_inputs_oracle_support.rs"]
mod oracle_support;
#[path = "support/collision_inputs_preflight.rs"]
mod preflight;

use mjwarp_rs::{
    io::{CollisionInputsFieldData, CollisionInputsFieldValue},
    model::{
        VolumeFieldData,
        collision_inputs::{COLLISION_INPUTS_FIELDS, CollisionInputsFieldSource},
    },
};
use oracle_support::{MODELS, bits, counts_of, f64s, i32s, load, model, oracle, sha256, u32s};

const ORACLE: &str = include_str!("support/collision_inputs_oracle.json");

#[test]
#[ignore = "需要可信MuJoCo DLL与冻结MJB"]
fn snapshot_matches_pinned_mjmodel_oracle_for_all_32_fields() {
    let oracle = oracle(ORACLE);
    assert_eq!(COLLISION_INPUTS_FIELDS.len(), 32);
    assert_eq!(
        COLLISION_INPUTS_FIELDS
            .iter()
            .filter(|spec| spec.source == CollisionInputsFieldSource::Model)
            .count(),
        30
    );
    for &(name, bytes) in &MODELS {
        let entry = model(&oracle, name);
        assert_eq!(
            sha256(bytes),
            entry["mjb_sha256"].as_str().unwrap(),
            "{name}"
        );
        let probe = load(bytes);
        let snapshot = probe.collision_inputs_snapshot().unwrap();
        assert_eq!(snapshot.info(), probe.info(), "{name}");
        assert_eq!(snapshot.counts(), counts_of(entry), "{name}");
        // Data fields are served by the mjData snapshot, not the model copy.
        assert!(snapshot.field(0).is_none() && snapshot.field(1).is_none());
        for id in 2..32_u32 {
            let spec = &COLLISION_INPUTS_FIELDS[id as usize];
            assert_eq!(
                spec.source,
                CollisionInputsFieldSource::Model,
                "{name} {id}"
            );
            let field = snapshot.field(id).unwrap();
            let expected = &entry["fields"][id.to_string()];
            assert_eq!(expected["kind"].as_u64(), Some(u64::from(spec.kind.code())));
            assert_eq!(
                expected["count"].as_i64(),
                Some(i64::try_from(field.len()).unwrap()),
                "{name} {}",
                spec.name
            );
            // oct_* arrays are cross-checked against the volume bridge below.
            if expected.get("volume_oracle").is_some() {
                continue;
            }
            match field {
                CollisionInputsFieldData::Index(values) => {
                    assert_eq!(values, &i32s(&expected["i32"]), "{name} {}", spec.name)
                }
                CollisionInputsFieldData::Num(values) => {
                    assert_eq!(values, &f64s(&expected["f64_bits"]), "{name} {}", spec.name)
                }
                CollisionInputsFieldData::Float(values) => {
                    assert_eq!(
                        bits(values),
                        u32s(&expected["f32_bits"]),
                        "{name} {}",
                        spec.name
                    )
                }
            }
        }
        // Host narrowing: Num goes through the existing f64->f32 entry, Float is
        // reused as-is; both must match the numpy astype bit patterns.
        let inputs = snapshot.into_inputs().unwrap();
        assert_eq!(inputs.counts(), counts_of(entry), "{name}");
        assert!(inputs.field(0).is_none() && inputs.field(1).is_none());
        for id in 2..32_u32 {
            let spec = &COLLISION_INPUTS_FIELDS[id as usize];
            let expected = &entry["fields"][id.to_string()];
            if expected.get("volume_oracle").is_some() {
                continue;
            }
            match inputs.field(id).unwrap() {
                CollisionInputsFieldValue::Index(values) => {
                    assert_eq!(values, &i32s(&expected["i32"]), "{name} {}", spec.name)
                }
                CollisionInputsFieldValue::Float(values) => {
                    assert_eq!(
                        bits(values),
                        u32s(&expected["f32_bits"]),
                        "{name} {}",
                        spec.name
                    )
                }
            }
        }
    }
}

#[test]
#[ignore = "需要可信MuJoCo DLL与冻结MJB"]
fn data_snapshot_matches_pinned_mjdata_oracle_and_reuses_mjdata() {
    let oracle = oracle(ORACLE);
    for &(name, bytes) in &MODELS {
        let entry = model(&oracle, name);
        let counts = counts_of(entry);
        let probe = load(bytes);
        let mut data = probe.collision_inputs_data().unwrap();
        assert_eq!(data.nq(), counts.nq, "{name}");
        assert_eq!(data.ngeom(), counts.ngeom, "{name}");
        let cases = entry["qpos_cases"].as_array().unwrap();
        assert!(cases.len() >= 2, "{name}");
        let mut frames = Vec::new();
        for case in cases {
            data.set_qpos(&f64s(&case["qpos_bits"])).unwrap();
            let frame = data.capture_frame().unwrap();
            assert_eq!(
                i64::try_from(frame.geom_xpos.len()).unwrap(),
                counts.ngeom * 3,
                "{name}"
            );
            assert_eq!(
                i64::try_from(frame.geom_xmat.len()).unwrap(),
                counts.ngeom * 9,
                "{name}"
            );
            assert_eq!(
                bits(&frame.geom_xpos),
                u32s(&case["geom_xpos_bits"]),
                "{name} geom_xpos"
            );
            assert_eq!(
                bits(&frame.geom_xmat),
                u32s(&case["geom_xmat_bits"]),
                "{name} geom_xmat"
            );
            frames.push(frame);
        }
        // mjData is created once: a second pass with the first qpos reproduces it.
        data.set_qpos(&f64s(&cases[0]["qpos_bits"])).unwrap();
        assert_eq!(data.capture_frame().unwrap(), frames[0], "{name} reuse");
        // Wrong qpos lengths are rejected without panicking.
        let expected = usize::try_from(counts.nq).unwrap();
        assert!(data.set_qpos(&vec![0.0; expected]).is_ok(), "{name}");
        assert!(data.set_qpos(&vec![0.0; expected + 1]).is_err(), "{name}");
        if expected > 0 {
            assert!(data.set_qpos(&vec![0.0; expected - 1]).is_err(), "{name}");
        }
    }
}

#[test]
#[ignore = "需要可信MuJoCo DLL与冻结MJB"]
fn snapshot_outlives_owner_and_octree_matches_volume_bridge() {
    let oracle = oracle(ORACLE);
    for &(name, bytes) in &MODELS {
        let entry = model(&oracle, name);
        let probe = load(bytes);
        let snapshot = probe.collision_inputs_snapshot().unwrap();
        let inputs = snapshot.clone().into_inputs().unwrap();
        drop(probe);
        // Owned vectors stay valid after the native owner is closed.
        assert_eq!(snapshot.counts(), counts_of(entry), "{name}");
        assert_eq!(inputs.counts(), counts_of(entry), "{name}");
        assert_eq!(snapshot, snapshot.clone());
    }
    // Non-zero octree arrays: cross-check every element against the independent
    // G01 volume bridge reader over the same pinned model, plus the oracle counts.
    let entry = model(&oracle, "sdf-cylinder");
    let bytes = MODELS
        .iter()
        .find(|(name, _)| *name == "sdf-cylinder")
        .map(|(_, bytes)| *bytes)
        .unwrap();
    let probe = load(bytes);
    let snapshot = probe.collision_inputs_snapshot().unwrap();
    assert_eq!(snapshot.counts(), counts_of(entry));
    assert!(snapshot.counts().noct > 0);
    let volume = probe.volume_snapshot().unwrap();
    let pairs: [(u32, &str); 3] = [(29, "oct_aabb"), (30, "oct_child"), (31, "oct_coeff")];
    for (id, volume_name) in pairs {
        let spec = &COLLISION_INPUTS_FIELDS[id as usize];
        let expected = &entry["fields"][id.to_string()];
        assert_eq!(
            expected["count"].as_i64(),
            Some(volume.counts().noct as i64 * spec.width)
        );
        match (
            snapshot.field(id).unwrap(),
            volume.field(volume_name).unwrap(),
        ) {
            (CollisionInputsFieldData::Num(mine), VolumeFieldData::Num(theirs)) => {
                assert_eq!(mine, theirs, "{volume_name}")
            }
            (CollisionInputsFieldData::Index(mine), VolumeFieldData::Int(theirs)) => {
                assert_eq!(mine, theirs, "{volume_name}")
            }
            other => panic!("unexpected {volume_name} data {other:?}"),
        }
    }
    let inputs = snapshot.into_inputs().unwrap();
    for (id, _) in pairs {
        match inputs.field(id).unwrap() {
            CollisionInputsFieldValue::Float(values) => {
                assert!(!values.is_empty());
                assert!(values.iter().all(|value| value.is_finite()));
            }
            CollisionInputsFieldValue::Index(values) => assert!(!values.is_empty()),
        }
    }
}

#[test]
#[ignore = "需要可信MuJoCo DLL与冻结MJB"]
fn rejects_invalid_native_arguments_without_partial_writes() {
    preflight::run(MODELS[0].1);
}
