#![cfg(feature = "cuda-probe")]
//! G01相机与光源辅助验收。
//! 产品测试只读取静态参考。

use mjwarp_rs::{
    diagnostics::{InputError, TransferError},
    model::{
        AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput,
        CamLightParameter as F, CamLightParameters, InertialFields, InertialModelInput,
        KinematicFields, KinematicModelInput, KinematicsParameters, MocapModelInput,
        ParameterBatch,
    },
    physics::{KinematicsPlan, KinematicsSnapshot},
    runtime::TransferSession,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
#[path = "support/rigid_reference.rs"]
mod reference;
use reference::floats;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/camlight/mixed-tree.json")).unwrap()
}
fn ids(v: &Value, key: &str) -> Vec<i32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| i32::try_from(x.as_i64().unwrap()).unwrap())
        .collect()
}
fn model(v: &Value) -> CamLightModelInput {
    let m = &v["model"];
    let attached = AttachedModelInput::new(
        reference::model(v),
        AttachedFields {
            geom_bodyid: ids(m, "geom_bodyid"),
            geom_pos: floats(m, "geom_pos"),
            geom_quat: floats(m, "geom_quat"),
            site_bodyid: ids(m, "site_bodyid"),
            site_pos: floats(m, "site_pos"),
            site_quat: floats(m, "site_quat"),
        },
    )
    .unwrap();
    let base = MocapModelInput::new(attached, ids(m, "body_mocapid")).unwrap();
    CamLightModelInput::new(
        base,
        CamLightFields {
            cam_mode: ids(m, "cam_mode"),
            cam_bodyid: ids(m, "cam_bodyid"),
            cam_targetbodyid: ids(m, "cam_targetbodyid"),
            cam_pos: floats(m, "cam_pos"),
            cam_quat: floats(m, "cam_quat"),
            cam_poscom0: floats(m, "cam_poscom0"),
            cam_pos0: floats(m, "cam_pos0"),
            cam_mat0: floats(m, "cam_mat0"),
            light_mode: ids(m, "light_mode"),
            light_bodyid: ids(m, "light_bodyid"),
            light_targetbodyid: ids(m, "light_targetbodyid"),
            light_pos: floats(m, "light_pos"),
            light_dir: floats(m, "light_dir"),
            light_poscom0: floats(m, "light_poscom0"),
            light_pos0: floats(m, "light_pos0"),
            light_dir0: floats(m, "light_dir0"),
        },
    )
    .unwrap()
}
fn flattened(out: &KinematicsSnapshot, w: usize) -> Vec<f32> {
    let c = out.camlight().world(w).unwrap();
    [c.cam_xpos, c.cam_xmat, c.light_xpos, c.light_xdir].concat()
}
fn compare(actual: &[f32], expected: &[f64]) -> f64 {
    assert_eq!(actual.len(), expected.len());
    let mut max: f64 = 0.0;
    for (index, (&a, &e)) in actual.iter().zip(expected).enumerate() {
        let error = (f64::from(a) - e).abs();
        max = max.max(error);
        assert!(
            a.is_finite() && e.is_finite() && error <= 2e-5 + 2e-5 * e.abs(),
            "G01 camlight index={index} actual={a} expected={e} error={error}"
        );
    }
    max
}
fn native_expected(case: &Value) -> Vec<f64> {
    ["cam_xpos", "cam_xmat", "light_xpos", "light_xdir"]
        .into_iter()
        .flat_map(|f| {
            case[f]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().unwrap())
        })
        .collect()
}
fn write_cases(data: &mut mjwarp_rs::physics::KinematicsData, v: &Value, round: usize) {
    let selected: Vec<_> = (0..data.worlds())
        .map(|w| &v["cases"][(w + round) % 8])
        .collect();
    data.write_qpos(
        &selected
            .iter()
            .flat_map(|c| floats(c, "qpos"))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    data.write_mocap(
        &selected
            .iter()
            .flat_map(|c| floats(c, "mocap_pos"))
            .collect::<Vec<_>>(),
        &selected
            .iter()
            .flat_map(|c| floats(c, "mocap_quat"))
            .collect::<Vec<_>>(),
    )
    .unwrap();
}

#[test]
fn reference_hashes_and_all_five_modes_match() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/camlight");
    let manifest: Value =
        serde_json::from_str(include_str!("../fixtures/camlight/manifest.json")).unwrap();
    for file in manifest["files"].as_array().unwrap() {
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(std::fs::read(root.join(file["path"].as_str().unwrap())).unwrap())
            ),
            file["sha256"].as_str().unwrap()
        );
    }
    let v = fixture();
    let model = model(&v);
    assert_eq!(v["native_version"], 3012000);
    assert_eq!(v["seed"], 1789);
    assert_eq!(v["absolute_tolerance"].as_f64(), Some(2e-5));
    assert_eq!(v["relative_tolerance"].as_f64(), Some(2e-5));
    assert_eq!(
        (model.ncam(), model.nlight(), model.mocap().nmocap()),
        (8, 8, 1)
    );
    for modes in [&model.fields().cam_mode, &model.fields().light_mode] {
        assert_eq!(modes, &[0, 0, 1, 2, 3, 4, 3, 4]);
    }
    assert_eq!(model.fields().cam_targetbodyid[6..], [-2, -1]);
    assert_eq!(model.fields().light_targetbodyid[6..], [-2, -1]);
    assert_eq!(v["cases"].as_array().unwrap().len(), 8);
    for case in v["cases"].as_array().unwrap() {
        for f in ["qpos", "mocap_pos", "mocap_quat"] {
            for e in case[f].as_array().unwrap() {
                let e = e.as_f64().unwrap();
                assert!((e - f64::from(e as f32)).abs() <= f64::EPSILON * e.abs());
            }
        }
    }
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn resident_camlight_matches_native_across_modes_and_cuda_blocks() {
    reference_hashes_and_all_five_modes_match();
    let v = fixture();
    let session = TransferSession::new(0).unwrap();
    let plan = KinematicsPlan::with_camlight(
        &session,
        model(&v),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    assert_eq!((plan.ncam(), plan.nlight()), (8, 8));
    for worlds in [1, 2, 5, 513] {
        let mut data = plan.create_data(worlds).unwrap();
        plan.update(&mut data).unwrap();
        let out = data.readback().unwrap();
        for world in 0..worlds {
            eprintln!(
                "G01-camlight kind=native world={world} max_abs_error={:e}",
                compare(&flattened(&out, world), &native_expected(&v["cases"][0]))
            );
        }
        for round in 1..=3 {
            write_cases(&mut data, &v, round);
            plan.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            for world in 0..worlds {
                eprintln!(
                    "G01-camlight kind=native world={world} max_abs_error={:e}",
                    compare(
                        &flattened(&out, world),
                        &native_expected(&v["cases"][(world + round) % 8])
                    )
                );
            }
        }
    }
}

// Independent f64 oracle, only used by tests. No CPU engine fallback.
fn vector(v: &[f32]) -> [f64; 3] {
    [f64::from(v[0]), f64::from(v[1]), f64::from(v[2])]
}
fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn unit(v: [f64; 3]) -> [f64; 3] {
    let n = v.iter().map(|v| v * v).sum::<f64>().sqrt();
    if n > 0.0 { v.map(|v| v / n) } else { [0.0; 3] }
}
fn rotate(q: &[f32], v: [f64; 3]) -> [f64; 3] {
    let q: Vec<f64> = q.iter().map(|&q| f64::from(q)).collect();
    let u = [q[1], q[2], q[3]];
    let dot: f64 = u.iter().zip(v).map(|(u, v)| u * v).sum();
    let square = q[0] * q[0] - u.iter().map(|u| u * u).sum::<f64>();
    let c = cross(u, v);
    std::array::from_fn(|i| square * v[i] + 2.0 * dot * u[i] + 2.0 * q[0] * c[i])
}
fn shared(fields: &CamLightFields, f: F) -> &[f32] {
    match f {
        F::CamPos => &fields.cam_pos,
        F::CamQuat => &fields.cam_quat,
        F::CamPoscom0 => &fields.cam_poscom0,
        F::CamPos0 => &fields.cam_pos0,
        F::CamMat0 => &fields.cam_mat0,
        F::LightPos => &fields.light_pos,
        F::LightDir => &fields.light_dir,
        F::LightPoscom0 => &fields.light_poscom0,
        F::LightPos0 => &fields.light_pos0,
        F::LightDir0 => &fields.light_dir0,
    }
}
fn oracle(
    out: &KinematicsSnapshot,
    w: usize,
    fields: &CamLightFields,
    p: &CamLightParameters,
) -> Vec<f64> {
    let r = out.rigid().world(w).unwrap();
    let com = out.com().world(w).unwrap();
    let row = |f| p.get(f).map_or_else(|| shared(fields, f), |b| b.world(w));
    let mut cx = vec![];
    let mut cm = vec![];
    let mut lx = vec![];
    let mut ld = vec![];
    for (c, &b) in fields.cam_bodyid.iter().enumerate() {
        let b = b as usize;
        let mode = fields.cam_mode[c];
        let t = fields.cam_targetbodyid[c];
        let mut pos = add(
            vector(&r.xpos[3 * b..]),
            rotate(&r.xquat[4 * b..], vector(&row(F::CamPos)[3 * c..])),
        );
        if mode == 1 {
            pos = add(vector(&r.xpos[3 * b..]), vector(&row(F::CamPos0)[3 * c..]));
        }
        if mode == 2 {
            pos = add(
                vector(&com.subtree_com[3 * b..]),
                vector(&row(F::CamPoscom0)[3 * c..]),
            );
        }
        cx.extend(pos);
        if mode == 1 || mode == 2 {
            cm.extend(
                row(F::CamMat0)[9 * c..9 * c + 9]
                    .iter()
                    .map(|&v| f64::from(v)),
            );
        } else if (mode == 3 || mode == 4) && t >= 0 {
            let target = vector(if mode == 3 {
                &r.xpos[3 * t as usize..]
            } else {
                &com.subtree_com[3 * t as usize..]
            });
            let z = unit(sub(pos, target));
            let x = unit(cross([0.0, 0.0, 1.0], z));
            let y = unit(cross(z, x));
            for i in 0..3 {
                cm.extend([x[i], y[i], z[i]]);
            }
        } else {
            // Rotate each local camera basis through local then body frames.
            let cols: [[f64; 3]; 3] = std::array::from_fn(|i| {
                let mut axis = [0.0; 3];
                axis[i] = 1.0;
                rotate(&r.xquat[4 * b..], rotate(&row(F::CamQuat)[4 * c..], axis))
            });
            for ((x, y), z) in cols[0].into_iter().zip(cols[1]).zip(cols[2]) {
                cm.extend([x, y, z]);
            }
        }
    }
    for (l, &b) in fields.light_bodyid.iter().enumerate() {
        let b = b as usize;
        let mode = fields.light_mode[l];
        let t = fields.light_targetbodyid[l];
        let mut pos = add(
            vector(&r.xpos[3 * b..]),
            rotate(&r.xquat[4 * b..], vector(&row(F::LightPos)[3 * l..])),
        );
        if mode == 1 {
            pos = add(
                vector(&r.xpos[3 * b..]),
                vector(&row(F::LightPos0)[3 * l..]),
            );
        }
        if mode == 2 {
            pos = add(
                vector(&com.subtree_com[3 * b..]),
                vector(&row(F::LightPoscom0)[3 * l..]),
            );
        }
        lx.extend(pos);
        let dir = if mode == 1 || mode == 2 {
            vector(&row(F::LightDir0)[3 * l..])
        } else if (mode == 3 || mode == 4) && t >= 0 {
            sub(
                vector(if mode == 3 {
                    &r.xpos[3 * t as usize..]
                } else {
                    &com.subtree_com[3 * t as usize..]
                }),
                pos,
            )
        } else {
            rotate(&r.xquat[4 * b..], vector(&row(F::LightDir)[3 * l..]))
        };
        ld.extend(if (mode == 3 || mode == 4) && t < 0 {
            dir
        } else {
            unit(dir)
        });
    }
    [cx, cm, lx, ld].concat()
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn ten_independent_parameter_periods_match_frozen_formulas() {
    let v = fixture();
    let model = model(&v);
    let fields = model.fields().clone();
    let periods = [2, 3, 5, 7, 11, 3, 5, 7, 2, 11];
    let mut p = CamLightParameters::default();
    for (i, f) in F::ALL.into_iter().enumerate() {
        let base = shared(&fields, f);
        let mut values = vec![];
        for row in 0..periods[i] {
            let mut v = base.to_vec();
            if f == F::CamQuat {
                let a = row as f32 * 0.13;
                for q in v.as_chunks_mut::<4>().0 {
                    q.copy_from_slice(&[a.cos(), 0.0, 0.0, a.sin()]);
                }
            } else {
                for (j, v) in v.iter_mut().enumerate() {
                    *v += row as f32 * 0.17 + j as f32 * 0.003;
                }
            }
            values.extend(v);
        }
        p.set(
            f,
            ParameterBatch::new(periods[i], base.len(), values).unwrap(),
        );
    }
    let session = TransferSession::new(0).unwrap();
    let plan =
        KinematicsPlan::with_camlight(&session, model, KinematicsParameters::default(), p.clone())
            .unwrap();
    assert_eq!(plan.camlight_parameters(), &p);
    for worlds in [1, 2, 5, 513] {
        let mut data = plan.create_data(worlds).unwrap();
        for round in 0..2 {
            write_cases(&mut data, &v, round);
            plan.update(&mut data).unwrap();
            let out = data.readback().unwrap();
            for w in 0..worlds {
                eprintln!(
                    "G01-camlight kind=parameters world={w} max_abs_error={:e}",
                    compare(&flattened(&out, w), &oracle(&out, w, &fields, &p))
                );
            }
        }
    }
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn camlight_readiness_identity_lifetime_and_world_writes_are_strict() {
    let v = fixture();
    let session = TransferSession::new(0).unwrap();
    let make = || {
        KinematicsPlan::with_camlight(
            &session,
            model(&v),
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap()
    };
    let plan = make();
    let other = make();
    let mut data = plan.create_data(513).unwrap();
    let mut second = plan.create_data(2).unwrap();
    assert_eq!(
        plan.update_camlight(&mut data),
        Err(TransferError::StageNotReady { stage: "rigid" })
    );
    plan.update_rigid(&mut data).unwrap();
    assert_eq!(
        plan.update_camlight(&mut data),
        Err(TransferError::StageNotReady { stage: "com" })
    );
    plan.update_attached(&mut data).unwrap();
    plan.update_com(&mut data).unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady { stage: "camlight" })
    ));
    plan.update_camlight(&mut data).unwrap();
    let saved = data.readback().unwrap();
    plan.update(&mut second).unwrap();
    assert_eq!(
        other.update_camlight(&mut data),
        Err(TransferError::ModelMismatch)
    );
    assert_eq!(
        flattened(&saved, 512),
        flattened(&data.readback().unwrap(), 512)
    );
    assert!(matches!(
        data.write_world_qpos(513, &[]),
        Err(TransferError::Input(InputError::InvalidIndex { .. }))
    ));
    assert!(data.write_world_qpos(512, &[f32::NAN, 0.0]).is_err());
    assert_eq!(
        flattened(&saved, 512),
        flattened(&data.readback().unwrap(), 512)
    );
    for (world, id) in [(0, 1), (256, 4), (512, 7)] {
        let c = &v["cases"][id];
        data.write_world_qpos(world, &floats(c, "qpos")).unwrap();
        data.write_world_mocap(world, &floats(c, "mocap_pos"), &floats(c, "mocap_quat"))
            .unwrap();
    }
    plan.update(&mut data).unwrap();
    let updated = data.readback().unwrap();
    for world in 0..513 {
        let id = match world {
            0 => 1,
            256 => 4,
            512 => 7,
            _ => 0,
        };
        compare(
            &flattened(&updated, world),
            &native_expected(&v["cases"][id]),
        );
    }
    compare(
        &flattened(&second.readback().unwrap(), 1),
        &native_expected(&v["cases"][0]),
    );
    plan.update_com(&mut data).unwrap();
    assert!(matches!(
        data.readback(),
        Err(TransferError::StageNotReady { stage: "camlight" })
    ));
    plan.update_camlight(&mut data).unwrap();
    plan.update_attached(&mut data).unwrap();
    assert_eq!(
        flattened(&updated, 512),
        flattened(&data.readback().unwrap(), 512)
    );
    drop(other);
    drop(plan);
    drop(session);
    assert_eq!(
        flattened(&updated, 512),
        flattened(&data.readback().unwrap(), 512)
    );
    compare(&flattened(&saved, 512), &native_expected(&v["cases"][0]));
}

fn zero_mass_base() -> MocapModelInput {
    let k = KinematicModelInput::new(
        0,
        KinematicFields {
            body_parentid: vec![0, 0],
            body_jntadr: vec![-1, -1],
            body_jntnum: vec![0, 0],
            body_pos: vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0],
            body_quat: [1.0, 0.0, 0.0, 0.0].repeat(2),
            ..Default::default()
        },
    )
    .unwrap();
    let i = InertialModelInput::new(
        k,
        InertialFields {
            body_ipos: vec![0.0; 6],
            body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(2),
            body_mass: vec![0.0; 2],
            body_inertia: vec![0.0; 6],
            ..Default::default()
        },
    )
    .unwrap();
    MocapModelInput::new(
        AttachedModelInput::new(i, AttachedFields::default()).unwrap(),
        vec![-1, 0],
    )
    .unwrap()
}
fn edge_fields() -> CamLightFields {
    CamLightFields {
        cam_mode: vec![3, 4, 3, 4, 2, 4],
        cam_bodyid: vec![0, 0, 0, 0, 1, 1],
        cam_targetbodyid: vec![0, 0, 0, 0, -1, -2],
        cam_pos: vec![
            0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1e-30, 0.0, 0.0, 1e-10, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0,
        ],
        cam_quat: [1.0, 0.0, 0.0, 0.0].repeat(6),
        cam_pos0: vec![0.0; 18],
        cam_poscom0: [0.5, -0.25, 0.75].repeat(6),
        cam_mat0: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0].repeat(6),
        light_mode: vec![0, 0, 0, 3, 4, 2, 4, 3],
        light_bodyid: vec![0, 0, 0, 1, 1, 1, 0, 0],
        light_targetbodyid: vec![-1, -1, -1, -2, -1, -1, 0, 0],
        light_pos: vec![0.0; 24],
        light_dir: vec![
            0.0, 0.0, 0.0, 1e-30, 0.0, 0.0, 1e-10, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0,
        ],
        light_pos0: vec![0.0; 24],
        light_poscom0: [0.25, 0.5, -0.75].repeat(8),
        light_dir0: [0.0, 2.0, 0.0].repeat(8),
    }
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn degenerate_targets_zero_mass_and_missing_light_normalization_match_warp() {
    let session = TransferSession::new(0).unwrap();
    let model = CamLightModelInput::new(zero_mass_base(), edge_fields()).unwrap();
    let plan = KinematicsPlan::with_camlight(
        &session,
        model,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut data = plan.create_data(513).unwrap();
    plan.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    for world in 0..513 {
        let c = out.camlight().world(world).unwrap();
        assert_eq!(
            &c.cam_xmat[..9],
            &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]
        );
        assert_eq!(&c.cam_xmat[9..27], &[0.0; 18]);
        assert_eq!(
            &c.cam_xmat[27..36],
            &[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]
        );
        assert_eq!(&c.cam_xpos[12..15], &[0.5, -0.25, 0.75]); // zero mass COM remains zero
        assert_eq!(&c.cam_xpos[15..18], &[1.0, 2.0, 3.0]);
        assert_eq!(
            &c.light_xdir[..9],
            &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0]
        );
        assert_eq!(
            &c.light_xdir[9..18],
            &[0.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0, 1.0, 0.0]
        );
        assert_eq!(&c.light_xdir[18..], &[0.0; 6]);
        assert_eq!(&c.light_xpos[15..18], &[0.25, 0.5, -0.75]);
    }
    data.write_world_mocap(512, &[-1.0, 0.5, 2.0], &[0.5, 0.0, 0.0, 0.5])
        .unwrap();
    plan.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    let c = out.camlight().world(512).unwrap();
    compare(&c.cam_xpos[15..18], &[-1.0, 0.5, 2.0]);
    compare(&c.light_xdir[9..15], &[0.0, 0.0, 2.0, 0.0, 0.0, 3.0]);
    assert_eq!(&c.cam_xpos[12..15], &[0.5, -0.25, 0.75]);
}

#[test]
#[ignore = "需要NVIDIA驱动与NVRTC"]
fn empty_camera_or_light_sets_preserve_periods_and_legacy_readback() {
    let session = TransferSession::new(0).unwrap();
    for (nc, nl) in [(0, 0), (6, 0), (0, 8)] {
        let mut f = edge_fields();
        f.cam_mode.truncate(nc);
        f.cam_bodyid.truncate(nc);
        f.cam_targetbodyid.truncate(nc);
        f.cam_pos.truncate(3 * nc);
        f.cam_quat.truncate(4 * nc);
        f.cam_pos0.truncate(3 * nc);
        f.cam_poscom0.truncate(3 * nc);
        f.cam_mat0.truncate(9 * nc);
        f.light_mode.truncate(nl);
        f.light_bodyid.truncate(nl);
        f.light_targetbodyid.truncate(nl);
        f.light_pos.truncate(3 * nl);
        f.light_dir.truncate(3 * nl);
        f.light_pos0.truncate(3 * nl);
        f.light_poscom0.truncate(3 * nl);
        f.light_dir0.truncate(3 * nl);
        let mut p = CamLightParameters::default();
        for field in F::ALL {
            if shared(&f, field).is_empty() {
                p.set(
                    field,
                    ParameterBatch::new(i32::MAX as usize, 0, vec![]).unwrap(),
                );
            }
        }
        let plan = KinematicsPlan::with_camlight(
            &session,
            CamLightModelInput::new(zero_mass_base(), f).unwrap(),
            KinematicsParameters::default(),
            p,
        )
        .unwrap();
        let mut data = plan.create_data(513).unwrap();
        plan.update_rigid(&mut data).unwrap();
        plan.update_attached(&mut data).unwrap();
        plan.update_com(&mut data).unwrap();
        if nc == 0 && nl == 0 {
            assert!(
                data.readback()
                    .unwrap()
                    .camlight()
                    .world(512)
                    .unwrap()
                    .cam_xmat
                    .is_empty()
            );
        } else {
            assert!(matches!(
                data.readback(),
                Err(TransferError::StageNotReady { stage: "camlight" })
            ));
        }
        plan.update_camlight(&mut data).unwrap();
        let out = data.readback().unwrap();
        assert_eq!((out.camlight().ncam(), out.camlight().nlight()), (nc, nl));
        assert!(out.camlight().world(513).is_err());
        assert_eq!(flattened(&out, 512).len(), 12 * nc + 6 * nl);
    }
}
