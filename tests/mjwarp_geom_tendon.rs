#![cfg(feature = "cuda-probe")]
//! 冻结MJWarp的并列诊断。
//! 数值统计不宣称容限验收。
#[path = "support/geom_tendon_input.rs"]
mod input;
use input::{fixture, floats, ids, motion_fixture, plan, write};
use mjwarp_rs::runtime::TransferSession;
use serde_json::{Value, json};
#[path = "support/mjwarp_geom_reference.rs"]
mod upstream_reference;
use upstream_reference::{float_bytes, provenance, sha, upstream};

// 折叠正负零，保持数值顺序。
fn ordered_bits(value: f32) -> u32 {
    let bits = value.to_bits();
    let magnitude = bits & 0x7fff_ffff;
    if value.is_sign_negative() {
        0x8000_0000 - magnitude
    } else {
        0x8000_0000 + magnitude
    }
}

fn statistics(actual: &[f32], expected: &[f32]) -> Value {
    assert!(!actual.is_empty());
    assert_eq!(actual.len(), expected.len());
    let mut max_abs = 0.0_f64;
    let mut max_relative = 0.0_f64;
    let mut max_ulp = 0_u32;
    let mut bit_mismatches = 0;
    let mut worst_index = 0;
    for (index, (&a, &e)) in actual.iter().zip(expected).enumerate() {
        assert!(a.is_finite() && e.is_finite(), "nonfinite index={index}");
        let absolute = (f64::from(a) - f64::from(e)).abs();
        let scale = f64::from(a.abs().max(e.abs()));
        let relative = if scale == 0.0 { 0.0 } else { absolute / scale };
        if absolute > max_abs {
            max_abs = absolute;
            worst_index = index;
        }
        max_relative = max_relative.max(relative);
        max_ulp = max_ulp.max(ordered_bits(a).abs_diff(ordered_bits(e)));
        bit_mismatches += usize::from(a.to_bits() != e.to_bits());
    }
    json!({
        "count": actual.len(), "bit_mismatches": bit_mismatches,
        "max_abs": max_abs, "max_symmetric_relative": max_relative, "max_ulp": max_ulp,
        "worst_abs_index": worst_index,
        "worst_actual": actual[worst_index], "worst_upstream": expected[worst_index]
    })
}

#[test]
fn metrics_distinguish_ulp_signed_zero_and_large_differences() {
    let next = f32::from_bits(1.0_f32.to_bits() + 1);
    let result = statistics(&[next, -next, -0.0], &[1.0, -1.0, 0.0]);
    assert_eq!(result["max_ulp"], 1);
    assert_eq!(result["bit_mismatches"], 3);
    assert_eq!(result["max_abs"], f64::from(f32::EPSILON));
    assert_eq!(ordered_bits(-0.0), ordered_bits(0.0));
    assert_eq!(
        ordered_bits(-f32::from_bits(1)).abs_diff(ordered_bits(f32::from_bits(1))),
        2
    );
    assert_eq!(statistics(&[1.0], &[-1.0])["max_symmetric_relative"], 2.0);
}

#[test]
#[should_panic(expected = "nonfinite")]
fn metrics_reject_nonfinite_results() {
    statistics(&[f32::NAN], &[0.0]);
}

#[test]
fn mjwarp_reference_hashes_and_quantization_are_frozen() {
    upstream_reference::validate();
}

#[test]
fn fallback_reference_rejects_wrong_batch_state_and_world() {
    let v = fixture();
    let motion = motion_fixture();
    let inputs = [&v["cases"][0], &v["cases"][1], &motion["cases"][0]];
    let hashes = upstream_reference::input_hashes(&inputs);
    assert!(
        upstream_reference::motion_row(3, 2, 310, &hashes)
            .unwrap()
            .is_some()
    );
    assert!(
        upstream_reference::motion_row(3, 1, 310, &hashes)
            .unwrap()
            .is_none()
    );
    assert!(
        upstream_reference::motion_row(4, 2, 310, &hashes)
            .unwrap()
            .is_none()
    );
    assert!(
        upstream_reference::motion_row(3, 2, 311, &hashes)
            .unwrap()
            .is_none()
    );
    let mut wrong = hashes;
    wrong["qpos"] = json!("wrong");
    assert_eq!(
        upstream_reference::motion_row(3, 2, 310, &wrong).unwrap_err(),
        "batch_input_hash_mismatch"
    );
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC；仅报告差异"]
fn reports_mjwarp_parallel_differences_without_native_tolerance() {
    mjwarp_reference_hashes_and_quantization_are_frozen();
    let v = fixture();
    let motion = motion_fixture();
    let u = upstream();
    let session = TransferSession::new(0).unwrap();
    let plan = plan(&session, &v);
    let mut observations = Vec::new();
    let mut integer_mismatches = 0;
    for worlds in [3, 513] {
        let mut data = plan.create_data(worlds).unwrap();
        for (kind, case) in (0..4)
            .map(|i| ("base", i))
            .chain([310, 344, 393].map(|i| ("motion", i)))
        {
            let inputs: Vec<_> = (0..worlds)
                .map(|w| {
                    if kind == "motion" && matches!(w, 2 | 512) {
                        motion["cases"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|c| c["source_frame"] == case)
                            .unwrap()
                    } else {
                        &v["cases"][if kind == "base" {
                            case * 3 + w % 3
                        } else {
                            w % 3
                        }]
                    }
                })
                .collect();
            let hashes: Value = ["qpos", "mocap_pos", "mocap_quat"]
                .into_iter()
                .map(|key| {
                    let values: Vec<_> = inputs.iter().flat_map(|c| floats(c, key)).collect();
                    (key.to_owned(), json!(sha(&float_bytes(&values))))
                })
                .collect();
            for (world, input) in inputs.iter().enumerate() {
                write(&mut data, world, input);
            }
            plan.update(&mut data).unwrap();
            let output = data.readback().unwrap();
            let rows: Vec<_> = u["rows"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|row| row["worlds"] == worlds && row["kind"] == kind && row["case"] == case)
                .collect();
            assert_eq!(
                rows.len(),
                match (worlds, kind) {
                    (3, "base") => 3,
                    (_, "base") => 4,
                    (3, _) => 1,
                    _ => 2,
                }
            );
            for row in rows {
                assert_eq!(row["input_hashes"], hashes);
                let world = row["world"].as_u64().unwrap() as usize;
                let rigid = output.rigid().world(world).unwrap();
                let com = output.com().world(world).unwrap();
                let attached = output.attached().world(world).unwrap();
                let tendon = output.spatial_tendon().world(world).unwrap();
                let mut fields = serde_json::Map::new();
                for (key, actual) in [
                    ("xpos", rigid.xpos),
                    ("xquat", rigid.xquat),
                    ("xmat", rigid.xmat),
                    ("xipos", rigid.xipos),
                    ("ximat", rigid.ximat),
                    ("xanchor", rigid.xanchor),
                    ("xaxis", rigid.xaxis),
                    ("subtree_com", com.subtree_com),
                    ("cdof", com.cdof),
                    ("geom_xpos", attached.geom_xpos),
                    ("geom_xmat", attached.geom_xmat),
                    ("site_xpos", attached.site_xpos),
                    ("site_xmat", attached.site_xmat),
                    ("ten_length", tendon.ten_length),
                    ("ten_J", tendon.ten_jacobian),
                    ("wrap_xpos", tendon.wrap_xpos),
                ] {
                    fields.insert(key.to_owned(), statistics(actual, &floats(row, key)));
                }
                for (key, actual) in [
                    ("ten_wrapadr", tendon.ten_wrapadr),
                    ("ten_wrapnum", tendon.ten_wrapnum),
                    ("wrap_obj", tendon.wrap_obj),
                ] {
                    let expected = ids(row, key);
                    assert_eq!(actual.len(), expected.len());
                    let mismatches = actual.iter().zip(&expected).filter(|(a, e)| a != e).count();
                    integer_mismatches += mismatches;
                    fields.insert(
                        key.to_owned(),
                        json!({"count": actual.len(), "mismatches": mismatches}),
                    );
                }
                let observation = json!({"worlds": worlds, "world": world, "kind": kind, "case": case,
                    "fields": fields, "ten_J_actual": tendon.ten_jacobian, "ten_J_upstream": row["ten_J"]});
                eprintln!("MJWARP_PARALLEL {observation}");
                observations.push(observation);
            }
        }
    }
    assert_eq!(observations.len(), 37);
    let report = json!({"schema": 1, "scope": "G01 geom tendon diagnostic subset", "parity": "not_evaluated",
        "report_kind": "diagnostic", "np03_admitted": false, "admission_label": "诊断完成，不作放行",
        "native_tolerance_used": false, "integer_mismatches": integer_mismatches,
        "observations": observations, "upstream": provenance()});
    if let Some(path) = std::env::var_os("MJWARP_PARALLEL_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    assert_eq!(integer_mismatches, 0, "MJWarp discrete topology differs");
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC；验证局部舍入修复"]
fn inside_norm_order_preserves_frozen_short_segment_contacts() {
    // 仅锁定已解释的接触点回归。
    // 原生优先门禁仍独立判定全部字段。
    mjwarp_reference_hashes_and_quantization_are_frozen();
    let v = fixture();
    let motion = motion_fixture();
    let case = motion["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["source_frame"] == 393)
        .unwrap();
    let session = TransferSession::new(0).unwrap();
    let plan = plan(&session, &v);
    let mut checked = 0;
    for worlds in [3, 513] {
        let mut data = plan.create_data(worlds).unwrap();
        let inputs: Vec<_> = (0..worlds)
            .map(|w| {
                if matches!(w, 2 | 512) {
                    case
                } else {
                    &v["cases"][w % 3]
                }
            })
            .collect();
        let hashes = upstream_reference::input_hashes(&inputs);
        for (w, input) in inputs.iter().enumerate() {
            write(&mut data, w, input);
        }
        plan.update(&mut data).unwrap();
        let out = data.readback().unwrap();
        for w in [2, 512].into_iter().filter(|&w| w < worlds) {
            let row = upstream_reference::motion_row(worlds, w, 393, &hashes)
                .unwrap()
                .unwrap();
            let tendon = out.spatial_tendon().world(w).unwrap();
            assert_eq!(tendon.ten_wrapadr, ids(&row, "ten_wrapadr"));
            assert_eq!(tendon.ten_wrapnum, ids(&row, "ten_wrapnum"));
            assert_eq!(tendon.wrap_obj, ids(&row, "wrap_obj"));
            assert_eq!(tendon.ten_wrapnum[4], 4);
            let start = 3 * (tendon.ten_wrapadr[4] as usize + 1);
            let expected = floats(&row, "wrap_xpos");
            let actual = &tendon.wrap_xpos[start..start + 6];
            assert!(actual.iter().all(|v| v.is_finite()));
            assert_eq!(
                actual.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
                expected[start..start + 6]
                    .iter()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>(),
                "frame393 contacts: worlds={worlds} world={w}"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 3);
}
