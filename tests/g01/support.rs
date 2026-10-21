//! G01 验收共享辅助。

use mjwarp_rs::io::NativeModelProbe;
use mjwarp_rs::model::{CamLightParameters, KinematicsParameters};
use mjwarp_rs::physics::{G01State, KinematicsPlan};
use mjwarp_rs::runtime::TransferSession;
use serde_json::Value;

use crate::two_reference::{Audit, evaluate, native_reference};

pub(crate) fn fixture() -> Value {
    serde_json::from_str(include_str!("../../fixtures/g01/reference.json")).unwrap()
}
pub(crate) fn floats(v: &Value, key: &str) -> Vec<f32> {
    v[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap() as f32)
        .collect()
}
pub(crate) fn input() -> mjwarp_rs::model::TendonWakeModelInput {
    let path: std::path::PathBuf = std::env::var_os("MJWARP_MUJOCO_DLL").unwrap().into();
    // SAFETY: acceptance uses verified DLL identity and a fixed trusted MJB.
    let native = unsafe {
        NativeModelProbe::load_trusted(&path, include_bytes!("../../fixtures/g01/scene.mjb"))
    }
    .unwrap();
    let snapshot = native.g01_snapshot().unwrap();
    drop(native);
    snapshot.into_model().unwrap()
}
pub(crate) fn plan(session: &TransferSession) -> KinematicsPlan {
    KinematicsPlan::for_g01(
        session,
        input(),
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .unwrap()
}
pub(crate) fn state(v: &Value, worlds: usize, round: usize) -> G01State {
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
pub(crate) fn cache(v: &Value, keys: &[&str]) -> Vec<f32> {
    keys.iter().flat_map(|key| floats(v, key)).collect()
}
pub(crate) fn com_cache(v: &Value) -> Vec<f32> {
    // 首段存放内部子树质量。
    // 低层读者不使用此暂存段。
    let mut values = vec![0.0; floats(v, "xpos").len() / 3];
    values.extend(cache(v, &["subtree_com", "cinert", "cdof"]));
    values
}
pub(crate) fn compare(
    actual: &[f32],
    expected: &Value,
    key: &str,
    max: &mut f64,
    scalars: &mut usize,
    audit: &mut Audit,
) {
    let values = expected[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    let identity = format!(
        "MuJoCo3.12.0:fixtures/g01/reference.json:case={}:field={key}",
        expected["id"]
    );
    let decision = evaluate(
        actual,
        Some(native_reference(identity, values, 2e-5, 2e-5)),
        || Ok(None),
    );
    *max = max.max(audit.record(
        serde_json::json!({"case": expected["id"], "field": key}),
        decision,
    ));
    *scalars += actual.len();
}

pub(crate) fn sleep_state(
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
pub(crate) fn with_sleep_flags(
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
pub(crate) fn wake_oracle(
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
