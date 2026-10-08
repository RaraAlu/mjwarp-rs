#![cfg(feature = "native-model-probe")]
//! G01原生转换与组合验收。
//! 测试只读取冻结参考。
use mjwarp_rs::io::NativeModelProbe;
fn load(bytes: &[u8]) -> NativeModelProbe {
    let path: std::path::PathBuf = std::env::var_os("MJWARP_MUJOCO_DLL").unwrap().into();
    // SAFETY: acceptance supplies a verified DLL and fixed trusted MJB fixtures.
    unsafe { NativeModelProbe::load_trusted(&path, bytes) }.unwrap()
}
const MODELS: &[(&str, &[u8])] = &[
    (
        "empty",
        include_bytes!("../fixtures/native-probe/zero-dof.mjb"),
    ),
    (
        "rigid",
        include_bytes!("../fixtures/native-probe/mixed-joints.mjb"),
    ),
    (
        "flex",
        include_bytes!("../fixtures/flex-position/flex-tree.mjb"),
    ),
    ("g01", include_bytes!("../fixtures/g01/scene.mjb")),
];
#[test]
#[ignore = "需要可信MuJoCo DLL"]
fn converts_complete_native_g01_fields_after_owner_drop() {
    for &(name, bytes) in MODELS {
        let owner = load(bytes);
        let snapshot = owner.g01_snapshot().unwrap();
        assert_eq!(snapshot.info(), owner.info());
        drop(owner);
        let input = snapshot.into_model().unwrap();
        let camlight = input.tendons().spatial().fixed().camlight();
        assert!(input.flex_positions().is_some(), "{name}");
        assert!(input.flex_edges().is_some(), "{name}");
        assert!(input.flex_faces().is_some(), "{name}");
        assert_eq!(
            camlight.mocap().attached().rigid().kinematics().nbody(),
            input.fields().body_treeid.len()
        );
        if name == "g01" {
            check_native_fields(&input);
        }
    }
}

fn check_native_fields(input: &mjwarp_rs::model::TendonWakeModelInput) {
    let reference: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/g01/reference.json")).unwrap();
    let expected = &reference["model"];
    let mut count = 0;
    let mut check = |key: &str, values: Vec<f32>| {
        let native: Vec<f32> = expected[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap() as f32)
            .collect();
        assert_eq!(values, native, "G01 native {key}");
        count += 1;
    };
    macro_rules! fields {
        ($f:expr; $($name:ident),+ $(,)?) => {
            $(check(&stringify!($name).replace("_j_", "_J_"), $f.$name.iter().map(|&x| x as f32).collect());)+
        };
    }
    let tendons = input.tendons();
    let spatial = tendons.spatial();
    let camera = spatial.fixed().camlight();
    let mocap = camera.mocap();
    let attached = mocap.attached();
    let rigid = attached.rigid();
    fields!(rigid.kinematics().fields(); qpos0, body_parentid, body_jntadr, body_jntnum, body_pos, body_quat, jnt_type, jnt_bodyid, jnt_qposadr, jnt_dofadr, jnt_pos, jnt_axis);
    fields!(rigid.fields(); body_ipos, body_iquat, body_mass, body_inertia, dof_bodyid, dof_jntid, dof_parentid, dof_armature, dof_damping);
    fields!(attached.fields(); geom_bodyid, geom_pos, geom_quat, site_bodyid, site_pos, site_quat);
    check(
        "body_mocapid",
        mocap.body_mocapid().iter().map(|&x| x as f32).collect(),
    );
    fields!(camera.fields(); cam_mode, cam_bodyid, cam_targetbodyid, cam_pos, cam_quat, cam_poscom0, cam_pos0, cam_mat0, light_mode, light_bodyid, light_targetbodyid, light_pos, light_dir, light_poscom0, light_pos0, light_dir0);
    fields!(tendons.fields(); tendon_adr, tendon_num, wrap_type, wrap_objid, wrap_prm, ten_j_rowadr, ten_j_rownnz, ten_j_colind);
    let geometry = spatial.geometry().unwrap();
    fields!(geometry; geom_type);
    check("geom_size", geometry.geom_size.values().to_vec());
    let positions = input.flex_positions().unwrap();
    fields!(positions; flex_interp, flex_cellnum, flex_nodeadr, flex_nodenum, flex_vertadr, flex_vertnum, flex_nodebodyid, flex_vertbodyid, flex_node, flex_vert, flex_vert0);
    check(
        "flex_centered",
        positions
            .flex_centered
            .iter()
            .map(|&x| u8::from(x) as f32)
            .collect(),
    );
    fields!(input.flex_edges().unwrap(); flex_edgeadr, flex_edgenum, flex_edge, flexedge_j_rowadr, flexedge_j_rownnz, flexedge_j_colind);
    let sleep = input.fields();
    fields!(sleep; body_treeid);
    check(
        "tendon_limited",
        sleep
            .tendon_limited
            .iter()
            .map(|&x| u8::from(x) as f32)
            .collect(),
    );
    check("tendon_range", sleep.tendon_range.values().to_vec());
    check("tendon_margin", sleep.tendon_margin.values().to_vec());
    check(
        "enableflags",
        vec![if sleep.sleep_enabled { 1 << 4 } else { 0 } as f32],
    );
    check(
        "disableflags",
        vec![if sleep.island_disabled { 1 << 18 } else { 0 } as f32],
    );
    assert_eq!(count, 78);
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "需要可信MuJoCo DLL与NVIDIA GPU"]
fn executes_complete_native_g01_chain() {
    use mjwarp_rs::{
        model::{CamLightParameters, KinematicsParameters},
        physics::{KinematicsPlan, fwd_kinematics},
        runtime::TransferSession,
    };
    let session = TransferSession::new(0).unwrap();
    for &(name, bytes) in MODELS {
        let input = load(bytes).g01_snapshot().unwrap().into_model().unwrap();
        let plan = KinematicsPlan::for_g01(
            &session,
            input,
            KinematicsParameters::default(),
            CamLightParameters::default(),
        )
        .unwrap();
        let mut data = plan.create_data(5).unwrap();
        fwd_kinematics(&plan, &mut data).unwrap();
        let snapshot = data.readback().unwrap();
        assert_eq!(snapshot.rigid().worlds(), 5, "{name}");
        assert!(snapshot.flex_positions().is_some());
        assert!(snapshot.flex_edges().is_some());
        assert!(snapshot.flex_faces().is_some());
    }
}
