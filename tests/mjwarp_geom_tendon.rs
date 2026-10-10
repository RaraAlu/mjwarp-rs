#![cfg(feature = "cuda-probe")]
//! 冻结MJWarp的并列诊断。
//! 数值统计不宣称容限验收。
#[path = "support/geom_tendon_input.rs"]
mod input;
use input::{fixture, floats, ids, motion_fixture, plan, write};
use mjwarp_rs::runtime::TransferSession;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn upstream() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/mjwarp-geom-tendon/reference.json"
    ))
    .unwrap()
}

fn provenance() -> Value {
    serde_json::from_str(include_str!(
        "../fixtures/mjwarp-geom-tendon/provenance.json"
    ))
    .unwrap()
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn float_bytes(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

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
    let manifest: Value =
        serde_json::from_str(include_str!("../fixtures/mjwarp-geom-tendon/manifest.json")).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in manifest["files"].as_array().unwrap() {
        let path = file["path"].as_str().unwrap();
        assert_eq!(
            sha(&std::fs::read(root.join(path)).unwrap()),
            file["sha256"],
            "{path}"
        );
    }
    let u = upstream();
    let p = provenance();
    for value in [&manifest, &u, &p] {
        assert_eq!(
            value["upstream_revision"],
            "71da24d956378a87a703b6e1442b13aec0c4ac29"
        );
    }
    assert_eq!(u["rows"].as_array().unwrap().len(), 37);
    assert_eq!(u["float_fields"].as_array().unwrap().len(), 16);
    assert_eq!(u["int_fields"].as_array().unwrap().len(), 3);
    assert_eq!(p["packages"]["warp-lang"], "1.15.0");
    assert_eq!(p["packages"]["mujoco"], "3.15.0");
    assert_eq!(p["module_options"]["fuse_fp"], true);
    assert_eq!(p["module_options"]["fast_math"], false);
    let v = fixture();
    for (key, mapping) in p["device_model_fields"].as_object().unwrap() {
        let bytes = if key == "geom_size" {
            float_bytes(
                &(0..3)
                    .flat_map(|i| floats(&v["cases"][i], key))
                    .collect::<Vec<_>>(),
            )
        } else if mapping["dtype"] == "float32" {
            float_bytes(&floats(&v["model"], key))
        } else {
            assert_eq!(mapping["dtype"], "int32");
            ids(&v["model"], key)
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect()
        };
        assert_eq!(sha(&bytes), mapping["sha256"], "model field={key}");
    }
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
        "native_tolerance_used": false, "integer_mismatches": integer_mismatches,
        "observations": observations, "upstream": provenance()});
    if let Some(path) = std::env::var_os("MJWARP_PARALLEL_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    assert_eq!(integer_mismatches, 0, "MJWarp discrete topology differs");
}
