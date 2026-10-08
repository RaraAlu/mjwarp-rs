#![cfg(all(feature = "cuda-probe", feature = "native-model-probe"))]
//! G01完整链验收。参考采用离线原生计算。
use mjwarp_rs::{
    io::NativeModelProbe,
    model::{CamLightParameters, KinematicsParameters},
    physics::{
        G01State, KinematicsPlan, camlight, com_pos, flex, fwd_kinematics, kinematics, tendon,
    },
    runtime::TransferSession,
};
use serde_json::Value;
fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/g01/reference.json")).unwrap()
}
fn floats(v: &Value, key: &str) -> Vec<f32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap() as f32)
        .collect()
}
fn input() -> mjwarp_rs::model::TendonWakeModelInput {
    let path: std::path::PathBuf = std::env::var_os("MJWARP_MUJOCO_DLL").unwrap().into();
    // SAFETY: acceptance uses verified DLL identity and a fixed trusted MJB.
    let native = unsafe {
        NativeModelProbe::load_trusted(&path, include_bytes!("../fixtures/g01/scene.mjb"))
    }
    .unwrap();
    let snapshot = native.g01_snapshot().unwrap();
    drop(native);
    snapshot.into_model().unwrap()
}
fn plan(session: &TransferSession) -> KinematicsPlan {
    KinematicsPlan::for_g01(
        session,
        input(),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap()
}
fn state(v: &Value, worlds: usize, round: usize) -> G01State {
    let mut result = G01State::default();
    for world in 0..worlds {
        let c = &v["cases"][(world + 3 * round) % 16];
        result.qpos.extend(floats(c, "qpos"));
        result.qvel.extend(floats(c, "qvel"));
        result.mocap_pos.extend(floats(c, "mocap_pos"));
        result.mocap_quat.extend(floats(c, "mocap_quat"));
    }
    result.flex_hessian_valid = Some(vec![true; worlds * 4]);
    result
}
fn cache(v: &Value, keys: &[&str]) -> Vec<f32> {
    keys.iter().flat_map(|key| floats(v, key)).collect()
}
fn com_cache(v: &Value) -> Vec<f32> {
    // 首段存放内部子树质量。
    // 低层读者不使用此暂存段。
    let mut values = vec![0.0; floats(v, "xpos").len() / 3];
    values.extend(cache(v, &["subtree_com", "cinert", "cdof"]));
    values
}
fn compare(actual: &[f32], expected: &Value, key: &str, max: &mut f64, scalars: &mut usize) {
    let expected = expected[key].as_array().unwrap();
    assert_eq!(actual.len(), expected.len(), "{key} length");
    for (i, (&a, e)) in actual.iter().zip(expected).enumerate() {
        let e = e.as_f64().unwrap();
        let err = (f64::from(a) - e).abs();
        *max = max.max(err);
        *scalars += 1;
        assert!(
            a.is_finite() && err <= 2e-5 + 2e-5 * e.abs(),
            "G01 {key}[{i}] actual={a} expected={e} err={err}"
        );
    }
}
#[test]
fn reference_hashes_and_coverage() {
    use sha2::{Digest, Sha256};
    let manifest: Value =
        serde_json::from_str(include_str!("../fixtures/g01/manifest.json")).unwrap();
    for f in manifest["files"].as_array().unwrap() {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/g01")
            .join(f["path"].as_str().unwrap());
        assert_eq!(
            format!("{:x}", Sha256::digest(std::fs::read(p).unwrap())),
            f["sha256"].as_str().unwrap()
        );
    }
    let v = fixture();
    assert_eq!(v["cases"].as_array().unwrap().len(), 16);
    assert_eq!(
        v["model"]["jnt_type"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_i64().unwrap())
            .collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from([0, 1, 2, 3])
    );
    assert!(
        v["model"]["flex_interp"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x == -1)
    );
}
#[test]
#[ignore = "需要可信MuJoCo DLL与NVIDIA GPU"]
fn full_chain_matches_native_reference_without_freshness_ticket() {
    let v = fixture();
    let session = TransferSession::new(0).unwrap();
    let p = plan(&session);
    drop(session);
    let (mut states, mut scalars, mut max) = (0, 0, 0.0_f64);
    for nw in [1, 2, 5, 513] {
        let mut d = p.create_data(nw).unwrap();
        for round in 0..4 {
            p.import_g01_state(&mut d, &state(&v, nw, round)).unwrap();
            assert!(d.readback().is_err());
            fwd_kinematics(&p, &mut d).unwrap();
            let out = d.readback().unwrap();
            for w in 0..nw {
                let c = &v["cases"][(w + 3 * round) % 16];
                let r = out.rigid().world(w).unwrap();
                let com = out.com().world(w).unwrap();
                let a = out.attached().world(w).unwrap();
                let cam = out.camlight().world(w).unwrap();
                let flex = out.flex_positions().unwrap().world(w).unwrap();
                let edge = out.flex_edges().unwrap().world(w).unwrap();
                let face = out.flex_faces().unwrap().world(w).unwrap();
                let t = out.tendon().unwrap().world(w).unwrap();
                assert_eq!(flex.flex_hessian_valid, &[false; 4]);
                for (actual, key) in [
                    (r.xpos, "xpos"),
                    (r.xquat, "xquat"),
                    (r.xmat, "xmat"),
                    (r.xipos, "xipos"),
                    (r.ximat, "ximat"),
                    (r.xanchor, "xanchor"),
                    (r.xaxis, "xaxis"),
                    (com.subtree_com, "subtree_com"),
                    (com.cinert, "cinert"),
                    (com.cdof, "cdof"),
                    (a.geom_xpos, "geom_xpos"),
                    (a.geom_xmat, "geom_xmat"),
                    (a.site_xpos, "site_xpos"),
                    (a.site_xmat, "site_xmat"),
                    (cam.cam_xpos, "cam_xpos"),
                    (cam.cam_xmat, "cam_xmat"),
                    (cam.light_xpos, "light_xpos"),
                    (cam.light_xdir, "light_xdir"),
                    (flex.flexnode_xpos, "flexnode_xpos"),
                    (flex.flexvert_xpos, "flexvert_xpos"),
                    (edge.flexedge_length, "flexedge_length"),
                    (edge.flexedge_velocity, "flexedge_velocity"),
                    (edge.flexedge_jacobian, "flexedge_J"),
                    (t.ten_length, "ten_length"),
                    (t.ten_jacobian, "ten_J"),
                    (face.face_xpos, "face_xpos"),
                ] {
                    compare(actual, c, key, &mut max, &mut scalars);
                }
                let quats = c["face_quat"].as_array().unwrap();
                for (a, e) in face
                    .face_quat
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .zip(quats.as_chunks::<4>().0)
                {
                    let dot = a
                        .iter()
                        .zip(e)
                        .map(|(&a, e)| f64::from(a) * e.as_f64().unwrap())
                        .sum::<f64>();
                    for (&a, e) in a.iter().zip(e) {
                        let expected = e.as_f64().unwrap() * if dot < 0.0 { -1.0 } else { 1.0 };
                        let err = (f64::from(a) - expected).abs();
                        assert!(
                            err <= 2e-5 + 2e-5 * expected.abs(),
                            "face quaternion err={err}"
                        );
                        max = max.max(err);
                        scalars += 1;
                    }
                }
                let points = c["wrap_obj"].as_array().unwrap().len();
                compare(
                    &t.wrap_xpos[..3 * points],
                    c,
                    "wrap_xpos",
                    &mut max,
                    &mut scalars,
                );
                assert!(t.wrap_xpos[3 * points..].iter().all(|&x| x == 0.0));
                for (a, key) in [
                    (t.ten_wrapadr, "ten_wrapadr"),
                    (t.ten_wrapnum, "ten_wrapnum"),
                    (&t.wrap_obj[..points], "wrap_obj"),
                ] {
                    let e = c[key]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|x| x.as_i64().unwrap() as i32)
                        .collect::<Vec<_>>();
                    assert_eq!(a, e, "{key}");
                }
                let sleep = out.sleep_trees().unwrap().world(w).unwrap();
                assert!(sleep.tree_asleep.iter().all(|&x| x == -11));
                assert_eq!((sleep.nbody_awake, sleep.nv_awake), (0, 0));
                states += 1;
            }
        }
    }
    assert_eq!(states, 2084);
    println!("G01-complete states={states} scalars={scalars} max_abs_error={max}");
}

fn sleep_state(
    model: &mjwarp_rs::model::TendonWakeModelInput,
) -> mjwarp_rs::physics::G01SleepState {
    mjwarp_rs::physics::G01SleepState {
        tree_asleep: (0..model.ntree()).map(|i| i as i32).collect(),
        tree_awake: vec![0; model.ntree()],
        ntree_awake: 0,
        nbody_awake: 0,
        nv_awake: 0,
    }
}
fn with_sleep_flags(
    model: &mjwarp_rs::model::TendonWakeModelInput,
    enabled: bool,
    disabled: bool,
) -> mjwarp_rs::model::TendonWakeModelInput {
    use mjwarp_rs::model::{FlexPositionModelInput, TendonWakeModelInput};
    let flex = FlexPositionModelInput::new(
        model.tendons().clone(),
        model.flex_positions().unwrap().clone(),
    )
    .unwrap()
    .with_edges(model.flex_edges().unwrap().clone())
    .unwrap()
    .with_faces(model.flex_faces().unwrap().clone())
    .unwrap();
    let mut fields = model.fields().clone();
    fields.sleep_enabled = enabled;
    fields.island_disabled = disabled;
    TendonWakeModelInput::with_flex_positions(flex, fields).unwrap()
}
// Independent host state-machine oracle. It never feeds GPU physics results.
fn wake_oracle(
    model: &mjwarp_rs::model::TendonWakeModelInput,
    mut s: mjwarp_rs::physics::G01SleepState,
    lengths: &[f32],
) -> mjwarp_rs::physics::G01SleepState {
    let f = model.tendons().fields();
    let fields = model.fields();
    if !fields.sleep_enabled || fields.island_disabled || f.tendon_adr.is_empty() {
        return s;
    }
    let attached = model
        .tendons()
        .spatial()
        .fixed()
        .camlight()
        .mocap()
        .attached();
    let k = attached.rigid().kinematics().fields();
    for (t, &length) in lengths.iter().enumerate() {
        let start = f.tendon_adr[t] as usize;
        let end = start + f.tendon_num[t] as usize;
        let trees = (start..end)
            .filter_map(|i| {
                let id = f.wrap_objid[i] as usize;
                let body = match f.wrap_type[i] {
                    1 => k.jnt_bodyid[id],
                    3 => attached.fields().site_bodyid[id],
                    4 | 5 => attached.fields().geom_bodyid[id],
                    _ => return None,
                };
                let tree = fields.body_treeid[body as usize];
                (tree >= 0).then_some(tree as usize)
            })
            .collect::<Vec<_>>();
        let awake = trees
            .iter()
            .copied()
            .filter(|&tree| s.tree_awake[tree] == 1)
            .collect::<Vec<_>>();
        let range = fields.tendon_range.world(0);
        let margin = fields.tendon_margin.world(0);
        if awake.is_empty()
            || !fields.tendon_limited[t]
            || (length - range[2 * t] >= margin[t] && range[2 * t + 1] - length >= margin[t])
        {
            continue;
        }
        let wake = awake.iter().map(|&i| s.tree_asleep[i]).fold(-11, i32::min);
        for tree in trees {
            if s.tree_awake[tree] != 0 {
                continue;
            }
            if s.tree_asleep[tree] < 0 {
                s.tree_asleep[tree] = s.tree_asleep[tree].min(wake);
                continue;
            }
            let mut current = tree;
            loop {
                let next = s.tree_asleep[current] as usize;
                s.tree_asleep[current] = wake;
                current = next;
                if current == tree {
                    break;
                }
            }
        }
    }
    s.tree_awake = s.tree_asleep.iter().map(|&x| i32::from(x < 0)).collect();
    s.ntree_awake = s.tree_awake.iter().sum();
    s.nbody_awake = 0;
    s.nv_awake = 0;
    s
}
#[test]
#[ignore = "需要可信MuJoCo DLL与NVIDIA GPU"]
fn g01_wake_preserves_input_activity_until_lightweight_refresh() {
    let v = fixture();
    let native = input();
    let session = TransferSession::new(0).unwrap();
    let mut seeds = Vec::new();
    let k = native
        .tendons()
        .spatial()
        .fixed()
        .camlight()
        .mocap()
        .attached()
        .rigid()
        .kinematics();
    let hinge_body = k.fields().jnt_bodyid[0] as usize;
    let hinge = native.fields().body_treeid[hinge_body] as usize;
    let slide_joint = k
        .fields()
        .jnt_type
        .iter()
        .enumerate()
        .find(|&(j, &t)| {
            t == 2
                && native.fields().body_treeid[k.fields().jnt_bodyid[j] as usize] as usize != hinge
        })
        .unwrap()
        .0;
    let slide = native.fields().body_treeid[k.fields().jnt_bodyid[slide_joint] as usize] as usize;
    assert_ne!(hinge, slide);
    for mode in 0..4 {
        let mut s = sleep_state(&native);
        if mode == 0 || mode == 2 {
            s.tree_asleep[hinge] = if mode == 2 { -15 } else { -3 };
            s.tree_awake[hinge] = 1;
            s.ntree_awake = 1;
        }
        if mode == 1 {
            s.tree_asleep[hinge] = slide as i32;
            s.tree_asleep[slide] = hinge as i32;
        }
        if mode == 2 {
            let other = (0..native.ntree())
                .find(|&t| t != hinge && t != slide)
                .unwrap();
            s.tree_asleep[slide] = other as i32;
            s.tree_asleep[other] = slide as i32;
        }
        if mode == 3 {
            s.tree_asleep.fill(-11);
        }
        s.nbody_awake = 3;
        s.nv_awake = 2;
        seeds.push(s);
    }
    for (enabled, disabled) in [(true, false), (false, false), (true, true)] {
        let model = with_sleep_flags(&native, enabled, disabled);
        let p = KinematicsPlan::for_g01(
            &session,
            model.clone(),
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let mut d = p.create_data(4).unwrap();
        let mut state = state(&v, 4, 0);
        state.sleep = Some(seeds.clone());
        p.import_g01_state(&mut d, &state).unwrap();
        fwd_kinematics(&p, &mut d).unwrap();
        let out = d.readback().unwrap();
        for (world, seed) in seeds.iter().enumerate() {
            let expected = wake_oracle(
                &model,
                seed.clone(),
                &floats(&v["cases"][world], "ten_length"),
            );
            let actual = out.sleep_trees().unwrap().world(world).unwrap();
            assert_eq!(actual.tree_asleep, expected.tree_asleep);
            assert_eq!(actual.tree_awake, expected.tree_awake);
            assert_eq!(
                (actual.ntree_awake, actual.nbody_awake, actual.nv_awake),
                (
                    expected.ntree_awake,
                    expected.nbody_awake,
                    expected.nv_awake
                )
            );
        }
    }
}

#[test]
#[ignore = "需要可信MuJoCo DLL与NVIDIA GPU"]
fn g01_preserves_imported_world_frames_and_static_geometry_cache() {
    let v = fixture();
    let model = input();
    let attached = model
        .tendons()
        .spatial()
        .fixed()
        .camlight()
        .mocap()
        .attached();
    let mask = model
        .tendons()
        .spatial()
        .fixed()
        .camlight()
        .mocap()
        .static_geom()
        .to_vec();
    let ng = attached.ngeom();
    let ns = attached.nsite();
    let stride = 12 * (ng + ns);
    let free = attached
        .rigid()
        .kinematics()
        .fields()
        .jnt_type
        .iter()
        .position(|&t| t == 0)
        .unwrap();
    let body = attached.rigid().kinematics().fields().jnt_bodyid[free] as usize;
    let session = TransferSession::new(0).unwrap();
    let p = KinematicsPlan::for_g01(
        &session,
        model,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut d = p.create_data(3).unwrap();
    let mut state = state(&v, 3, 0);
    state.world_pose = Some(vec![[1.0, -2.0, 0.5, 1.0, 0.0, 0.0, 0.0]; 3]);
    state.attached_cache = Some(vec![777.0; stride * 3]);
    p.import_g01_state(&mut d, &state).unwrap();
    for _ in 0..2 {
        fwd_kinematics(&p, &mut d).unwrap();
        let out = d.readback().unwrap();
        for w in 0..3 {
            let r = out.rigid().world(w).unwrap();
            let a = out.attached().world(w).unwrap();
            assert_eq!(&r.xpos[..3], &[1.0, -2.0, 0.5]);
            assert_eq!(&r.xquat[..4], &[1.0, 0.0, 0.0, 0.0]);
            let c = &v["cases"][w];
            let expected = floats(c, "xpos");
            for (a, &e) in r.xpos[3 * body..3 * body + 3]
                .iter()
                .zip(&expected[3 * body..3 * body + 3])
            {
                assert!((a - e).abs() < 2e-5);
            }
            for (g, &is_static) in mask.iter().enumerate() {
                if is_static {
                    assert_eq!(&a.geom_xpos[3 * g..3 * g + 3], &[777.0; 3]);
                    assert_eq!(&a.geom_xmat[9 * g..9 * g + 9], &[777.0; 9]);
                } else {
                    assert!(a.geom_xpos[3 * g..3 * g + 3].iter().all(|&x| x != 777.0));
                }
            }
            assert!(a.site_xpos.iter().all(|&x| x != 777.0));
        }
    }
    let second = plan(&session);
    assert!(matches!(
        fwd_kinematics(&second, &mut d),
        Err(mjwarp_rs::diagnostics::TransferError::ModelMismatch)
    ));
}

#[test]
#[ignore = "需要可信MuJoCo DLL与NVIDIA GPU"]
fn g01_accepts_finite_nonunit_parameters_and_zero_state_quaternions() {
    use mjwarp_rs::model::{CamLightParameter, KinematicsParameter, ParameterBatch};
    let v = fixture();
    let model = input();
    let attached = model
        .tendons()
        .spatial()
        .fixed()
        .camlight()
        .mocap()
        .attached();
    let k = attached.rigid().kinematics();
    let fields = k.fields().clone();
    let mut params = KinematicsParameters::default();
    let values = fields
        .body_quat
        .iter()
        .map(|&q| q * 2.0)
        .collect::<Vec<_>>();
    params.set(
        KinematicsParameter::BodyQuat,
        ParameterBatch::new(1, values.len(), values).unwrap(),
    );
    let mut cams = CamLightParameters::default();
    let values = model
        .tendons()
        .spatial()
        .fixed()
        .camlight()
        .fields()
        .cam_quat
        .iter()
        .map(|&q| q * 2.0)
        .collect::<Vec<_>>();
    cams.set(
        CamLightParameter::CamQuat,
        ParameterBatch::new(1, values.len(), values).unwrap(),
    );
    let session = TransferSession::new(0).unwrap();
    assert!(
        KinematicsPlan::with_tendon_wake(&session, model.clone(), params.clone(), cams.clone())
            .is_err()
    );
    let p = KinematicsPlan::for_g01(&session, model, params, cams).unwrap();
    let mut d = p.create_data(1).unwrap();
    let mut state = state(&v, 1, 0);
    state.mocap_quat.fill(0.0);
    for (j, &kind) in fields.jnt_type.iter().enumerate() {
        if kind <= 1 {
            let start = fields.jnt_qposadr[j] as usize + if kind == 0 { 3 } else { 0 };
            state.qpos[start..start + 4].fill(0.0);
        }
    }
    p.import_g01_state(&mut d, &state).unwrap();
    fwd_kinematics(&p, &mut d).unwrap();
    let out = d.readback().unwrap();
    let r = out.rigid().world(0).unwrap();
    for (j, &kind) in fields.jnt_type.iter().enumerate() {
        if kind <= 1 {
            let b = fields.jnt_bodyid[j] as usize;
            assert_eq!(&r.xquat[4 * b..4 * b + 4], &[0.0; 4]);
        }
    }
    assert!(
        out.flex_positions()
            .unwrap()
            .world(0)
            .unwrap()
            .flex_hessian_valid
            .iter()
            .all(|&v| !v)
    );
}

#[test]
#[ignore = "需要可信MuJoCo DLL与NVIDIA GPU"]
fn g01_import_preflights_all_fields_and_preserves_previous_results_on_rejection() {
    let v = fixture();
    let model = input();
    let seed = sleep_state(&model);
    let session = TransferSession::new(0).unwrap();
    let p = KinematicsPlan::for_g01(
        &session,
        model,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    assert!(p.create_data(0).is_err());
    assert!(p.create_data(u32::MAX as usize).is_err());
    let mut d = p.create_data(2).unwrap();
    let mut state = state(&v, 2, 0);
    state.sleep = Some(vec![seed.clone(), seed]);
    state.world_pose = Some(vec![[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]; 2]);
    p.import_g01_state(&mut d, &state).unwrap();
    fwd_kinematics(&p, &mut d).unwrap();
    let previous = d.readback().unwrap();
    let a = previous.attached().world(0).unwrap();
    let row = a
        .geom_xpos
        .iter()
        .chain(a.geom_xmat)
        .chain(a.site_xpos)
        .chain(a.site_xmat)
        .copied()
        .collect::<Vec<_>>();
    state.attached_cache = Some(row.repeat(2));
    state.rigid_cache = Some(
        cache(
            &v["cases"][0],
            &[
                "xpos", "xquat", "xmat", "xipos", "ximat", "xanchor", "xaxis",
            ],
        )
        .repeat(2),
    );
    state.com_cache = Some(com_cache(&v["cases"][0]).repeat(2));
    for mutation in 0..19 {
        let mut bad = state.clone();
        match mutation {
            0 => {
                bad.qpos.pop();
            }
            1 => bad.qpos[0] = f32::NAN,
            2 => {
                bad.qvel.pop();
            }
            3 => bad.qvel[0] = f32::INFINITY,
            4 => {
                bad.mocap_pos.pop();
            }
            5 => bad.mocap_pos[0] = f32::NAN,
            6 => {
                bad.mocap_quat.pop();
            }
            7 => bad.mocap_quat[0] = f32::NAN,
            8 => {
                bad.world_pose.as_mut().unwrap().pop();
            }
            9 => bad.world_pose.as_mut().unwrap()[0][0] = f32::NAN,
            10 => {
                bad.attached_cache.as_mut().unwrap().pop();
            }
            11 => bad.attached_cache.as_mut().unwrap()[0] = f32::NAN,
            12 => {
                bad.flex_hessian_valid.as_mut().unwrap().pop();
            }
            13 => {
                bad.sleep.as_mut().unwrap()[1].tree_awake[0] = 2;
            }
            14 => {
                bad.sleep.as_mut().unwrap()[1].ntree_awake = -1;
            }
            15 => {
                bad.rigid_cache.as_mut().unwrap().pop();
            }
            16 => bad.rigid_cache.as_mut().unwrap()[0] = f32::NAN,
            17 => {
                bad.com_cache.as_mut().unwrap().pop();
            }
            _ => bad.com_cache.as_mut().unwrap()[0] = f32::INFINITY,
        }
        assert!(
            p.import_g01_state(&mut d, &bad).is_err(),
            "mutation {mutation}"
        );
        let out = d.readback().unwrap();
        for w in 0..2 {
            assert_eq!(
                out.rigid().world(w).unwrap().xpos,
                previous.rigid().world(w).unwrap().xpos
            );
            assert_eq!(
                out.attached().world(w).unwrap().geom_xpos,
                previous.attached().world(w).unwrap().geom_xpos
            );
            assert_eq!(
                out.tendon().unwrap().world(w).unwrap().ten_length,
                previous.tendon().unwrap().world(w).unwrap().ten_length
            );
            assert_eq!(
                out.sleep_trees().unwrap().world(w).unwrap().tree_asleep,
                previous
                    .sleep_trees()
                    .unwrap()
                    .world(w)
                    .unwrap()
                    .tree_asleep
            );
        }
    }
    let strict = KinematicsPlan::with_tendon_wake(
        &session,
        input(),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap();
    let mut other = strict.create_data(1).unwrap();
    assert!(fwd_kinematics(&strict, &mut other).is_err());
    for stage in [kinematics, com_pos, camlight, flex, tendon] {
        assert!(stage(&strict, &mut other).is_err());
        assert!(matches!(
            stage(&strict, &mut d),
            Err(mjwarp_rs::diagnostics::TransferError::ModelMismatch)
        ));
    }
    assert!(other.readback_g01().is_err());
}

#[test]
#[ignore = "需要可信MuJoCo DLL与NVIDIA GPU"]
fn raw_g01_stages_read_prepared_fields_without_implicit_dependencies() {
    let v = fixture();
    let model = input();
    let seed = sleep_state(&model);
    let session = TransferSession::new(0).unwrap();
    let p = plan(&session);
    let mut d = p.create_data(16).unwrap();
    let mut prepared = state(&v, 16, 0);
    // qpos故意对应另一状态。
    // 低层入口必须只读现有字段。
    prepared.qpos = (0..16)
        .flat_map(|w| floats(&v["cases"][(w + 1) % 16], "qpos"))
        .collect();
    prepared.rigid_cache = Some(
        (0..16)
            .flat_map(|w| {
                cache(
                    &v["cases"][w],
                    &[
                        "xpos", "xquat", "xmat", "xipos", "ximat", "xanchor", "xaxis",
                    ],
                )
            })
            .collect(),
    );
    prepared.com_cache = Some((0..16).flat_map(|w| com_cache(&v["cases"][w])).collect());
    prepared.attached_cache = Some(
        (0..16)
            .flat_map(|w| {
                cache(
                    &v["cases"][w],
                    &["geom_xpos", "geom_xmat", "site_xpos", "site_xmat"],
                )
            })
            .collect(),
    );
    prepared.sleep = Some(vec![seed; 16]);
    p.import_g01_state(&mut d, &prepared).unwrap();
    assert!(d.readback().is_err());
    let mut max = 0.0_f64;
    let mut scalars = 0;
    // 逆序调用拒绝隐式前置更新。
    camlight(&p, &mut d).unwrap();
    flex(&p, &mut d).unwrap();
    // 固定肌腱读取qpos，因此改回状态。
    d.write_qpos(&state(&v, 16, 0).qpos).unwrap();
    tendon(&p, &mut d).unwrap();
    assert!(d.readback().is_err());
    let out = d.readback_g01().unwrap();
    for w in 0..16 {
        let c = &v["cases"][w];
        let r = out.rigid().world(w).unwrap();
        let com = out.com().world(w).unwrap();
        let cam = out.camlight().world(w).unwrap();
        let f = out.flex_positions().unwrap().world(w).unwrap();
        let e = out.flex_edges().unwrap().world(w).unwrap();
        let t = out.tendon().unwrap().world(w).unwrap();
        for (a, key) in [
            (r.xpos, "xpos"),
            (r.xquat, "xquat"),
            (r.xmat, "xmat"),
            (com.subtree_com, "subtree_com"),
            (com.cinert, "cinert"),
            (com.cdof, "cdof"),
            (cam.cam_xpos, "cam_xpos"),
            (cam.cam_xmat, "cam_xmat"),
            (cam.light_xpos, "light_xpos"),
            (cam.light_xdir, "light_xdir"),
            (f.flexvert_xpos, "flexvert_xpos"),
            (f.flexnode_xpos, "flexnode_xpos"),
            (e.flexedge_length, "flexedge_length"),
            (e.flexedge_velocity, "flexedge_velocity"),
            (e.flexedge_jacobian, "flexedge_J"),
            (t.ten_length, "ten_length"),
            (t.ten_jacobian, "ten_J"),
        ] {
            compare(a, c, key, &mut max, &mut scalars);
        }
        assert_eq!(f.flex_hessian_valid, &[false; 4]);
        let sleep = out.sleep_trees().unwrap().world(w).unwrap();
        assert_eq!(
            sleep.tree_asleep,
            prepared.sleep.as_ref().unwrap()[w].tree_asleep
        );
        assert_eq!(
            sleep.tree_awake,
            prepared.sleep.as_ref().unwrap()[w].tree_awake
        );
    }
    com_pos(&p, &mut d).unwrap();
    let out = d.readback_g01().unwrap();
    for w in 0..16 {
        let c = &v["cases"][w];
        let com = out.com().world(w).unwrap();
        for (a, key) in [
            (com.subtree_com, "subtree_com"),
            (com.cinert, "cinert"),
            (com.cdof, "cdof"),
        ] {
            compare(a, c, key, &mut max, &mut scalars);
        }
    }
    kinematics(&p, &mut d).unwrap();
    let out = d.readback_g01().unwrap();
    for w in 0..16 {
        let r = out.rigid().world(w).unwrap();
        compare(r.xpos, &v["cases"][w], "xpos", &mut max, &mut scalars);
    }
    println!("G01 raw scalars={scalars} max_abs_error={max:e}");
}
