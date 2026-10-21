use super::reference::reference_hashes_cover_dynamic_sphere_cylinder_and_inside_paths;
use super::*;

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn wrapped_and_direct_paths_match_native_across_cuda_blocks() {
    reference_hashes_cover_dynamic_sphere_cylinder_and_inside_paths();
    let v = fixture();
    let s = TransferSession::new(0).unwrap();
    let p = plan(&s, &v);
    for worlds in [1, 2, 5, 513] {
        let mut data = p.create_data(worlds).unwrap();
        for round in 0..4 {
            if round > 0 {
                for w in 0..worlds {
                    write(&mut data, w, &v["cases"][3 * round + w % 3]);
                }
            }
            p.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            for w in 0..worlds {
                eprintln!(
                    "G01-geom-tendon kind=native world={w} max_abs_error={:e}",
                    native(&out, w, &v["cases"][3 * round + w % 3])
                );
            }
        }
    }
}
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn geometry_dofs_match_length_derivatives_in_wrapped_and_direct_states() {
    let v = fixture();
    let s = TransferSession::new(0).unwrap();
    let p = plan(&s, &v);
    let mut data = p.create_data(513).unwrap();
    for id in [2, 5, 8, 11] {
        write(&mut data, 512, &v["cases"][id]);
        p.update(&mut data).unwrap();
        let base = data.readback().unwrap();
        native(&base, 512, &v["cases"][id]);
        for dof in 0..2 {
            let mut plus = floats(&v["cases"][id], "qpos");
            let mut minus = plus.clone();
            plus[dof] += 0.001;
            minus[dof] -= 0.001;
            data.write_world_qpos(512, &plus).unwrap();
            p.update(&mut data).unwrap();
            let a = data.readback().unwrap();
            data.write_world_qpos(512, &minus).unwrap();
            p.update(&mut data).unwrap();
            let b = data.readback().unwrap();
            let rows = p.spatial_tendon_rows();
            let original = base.spatial_tendon().world(512).unwrap();
            for t in 0..10 {
                let derivative = (f64::from(a.spatial_tendon().world(512).unwrap().ten_length[t])
                    - f64::from(b.spatial_tendon().world(512).unwrap().ten_length[t]))
                    / f64::from(plus[dof] - minus[dof]);
                let moment = rows
                    .row(t)
                    .unwrap()
                    .binary_search(&(dof as i32))
                    .map(|i| original.ten_jacobian[rows.rowadr()[t] as usize + i])
                    .unwrap_or(0.0);
                assert!(
                    (derivative - f64::from(moment)).abs() < 0.003,
                    "case={id} t={t} dof={dof} derivative={derivative} moment={moment}"
                );
            }
        }
    }
}
#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn inside_cylinder_motion_frames_follow_native_first_error_redline() {
    reference_hashes_cover_dynamic_sphere_cylinder_and_inside_paths();
    let v = fixture();
    let motion = motion_fixture();
    let s = TransferSession::new(0).unwrap();
    let p = plan(&s, &v);
    let mut audit = Audit::new("G01-geom-motion");
    for worlds in [3, 513] {
        let mut data = p.create_data(worlds).unwrap();
        for c in motion["cases"].as_array().unwrap() {
            // 完整批量状态匹配冻结上游。
            let inputs: Vec<_> = (0..worlds)
                .map(|w| {
                    if matches!(w, 2 | 512) {
                        c
                    } else {
                        &v["cases"][w % 3]
                    }
                })
                .collect();
            let hashes = upstream_reference::input_hashes(&inputs);
            for (w, input) in inputs.iter().enumerate() {
                write(&mut data, w, input);
            }
            p.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            for w in [2, 512].into_iter().filter(|&w| w < worlds) {
                let a = out.spatial_tendon().world(w).unwrap();
                for (key, actual) in [
                    ("ten_wrapadr", a.ten_wrapadr),
                    ("ten_wrapnum", a.ten_wrapnum),
                    ("wrap_obj", a.wrap_obj),
                ] {
                    assert_eq!(actual, ids(c, key));
                }
                let frame = c["source_frame"].as_u64().unwrap();
                for (key, actual) in [
                    ("ten_length", a.ten_length),
                    ("ten_J", a.ten_jacobian),
                    ("wrap_xpos", a.wrap_xpos),
                ] {
                    let expected = c[key]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_f64().unwrap())
                        .collect();
                    let identity =
                        format!("MuJoCo3.12.0:inside-cylinder-motion:frame={frame}:field={key}");
                    let decision = evaluate(
                        actual,
                        Some(native_reference(identity, expected, 2e-5, 2e-5)),
                        || {
                            let Some(row) =
                                upstream_reference::motion_row(worlds, w, frame, &hashes)?
                            else {
                                return Ok(None);
                            };
                            for (name, actual) in [
                                ("ten_wrapadr", a.ten_wrapadr),
                                ("ten_wrapnum", a.ten_wrapnum),
                                ("wrap_obj", a.wrap_obj),
                            ] {
                                if actual != ids(&row, name) {
                                    return Err(format!("topology_mismatch:{name}"));
                                }
                            }
                            Ok(Some(SecondaryReference {
                                identity: format!(
                                    "MJWarp:{}:sha256={}:worlds={worlds}:world={w}:frame={frame}:field={key}",
                                    upstream_reference::REVISION,
                                    upstream_reference::sha(include_bytes!(
                                        "../../fixtures/mjwarp-geom-tendon/reference.json"
                                    ))
                                ),
                                conditions: format!(
                                    "validated_model_and_batch:{}",
                                    upstream_reference::sha(&serde_json::to_vec(&hashes).unwrap())
                                ),
                                values: row[key]
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .map(|v| v.as_f64().unwrap() as f32)
                                    .collect(),
                                relative_slack: Some(MJWARP_RELATIVE_SLACK),
                            }))
                        },
                    );
                    audit.record(serde_json::json!({"worlds": worlds, "world": w, "frame": frame, "field": key}), decision);
                }
            }
        }
    }
    audit.finish();
}
