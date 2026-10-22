//! 刚体运动学包装的GPU逐位对比测试。
//! 覆盖普通探针、严格常驻、G01等价与速度位置路径。
//! 每个NVRTC编译使用独立空捕获目录，PTX须等于冻结基线。

use super::super::{KINEMATICS_CUDA, check_kinematics, kinematics_device};
use super::*;
use crate::diagnostics::InputError;
use crate::model::{
    AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, CamLightParameters,
    FixedTendonFields, FixedTendonModelInput, InertialFields, InertialModelInput, KinematicFields,
    KinematicModelInput, KinematicsParameters, MocapModelInput, SpatialTendonFields,
    SpatialTendonModelInput,
};
use crate::physics::kinematics::upload_kinematics_model;
use crate::runtime::SynchronousKernel;
use crate::runtime::launcher::LauncherError;

// 冻结基线：plain、mocap与mocap+G01三种变体。
const PLAIN_PTX: &str = "4b5e948bdaec8080a70c74ab301c5a183cf5590dd6f586557c9da9fc24d71405";
const MOCAP_PTX: &str = "0bbc1f9a3d4dadffa52d905636e80c6fa19a7140aca654934ace6c7753d7cc64";
const G01_PTX: &str = "442bcf3380006ae5127255cc5ba102a7b56cba8d383f4f6f87f2c4fd0a186992";

/// 独立空捕获目录；未设置根变量时只执行不取证。
struct Capture {
    root: Option<std::ffi::OsString>,
    dir: Option<std::path::PathBuf>,
}

impl Capture {
    fn new(label: &str) -> Self {
        let root = std::env::var_os("MJWARP_PHYSICS_PTX_DIR");
        let dir = crate::runtime::capture_child(module_path!(), label);
        if let Some(dir) = &dir {
            assert!(!dir.exists(), "捕获目录必须不存在：{}", dir.display());
            // SAFETY: 测试单线程运行，环境在Drop中恢复。
            unsafe { std::env::set_var("MJWARP_PHYSICS_PTX_DIR", dir) };
        }
        Self { root, dir }
    }

    fn expect(&self, hash: &str) {
        if let Some(dir) = &self.dir {
            let path = dir.join(format!("rigid_kinematics-{hash}.ptx"));
            assert!(path.is_file(), "缺少冻结PTX捕获：{}", path.display());
        }
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        // SAFETY: 测试单线程运行，恢复调用方原始环境。
        unsafe {
            match &self.root {
                Some(root) => std::env::set_var("MJWARP_PHYSICS_PTX_DIR", root),
                None => std::env::remove_var("MJWARP_PHYSICS_PTX_DIR"),
            }
        }
    }
}

fn bits(buffer: &TransferBuffer<f32>) -> Vec<u32> {
    let mut values = vec![0.0_f32; buffer.len()];
    buffer.read_range_into(0, &mut values).unwrap();
    values.iter().map(|value| value.to_bits()).collect()
}

fn guarded(session: &TransferSession, length: usize) -> TransferBuffer<f32> {
    let mut values = vec![f32::NAN; length];
    values[..4].fill(-131072.0);
    values[length - 4..].fill(-131072.0);
    session.upload(&values).unwrap()
}

/// 三体链：体1为自由关节，体2为固定体用于mocap映射。
fn free_body_model() -> InertialModelInput {
    let kinematics = KinematicFields {
        qpos0: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
        body_parentid: vec![0, 0, 0],
        body_jntadr: vec![-1, 0, -1],
        body_jntnum: vec![0, 1, 0],
        body_pos: vec![0.0, 0.0, 0.0, 0.1, 0.2, 0.3, 1.0, 2.0, 3.0],
        body_quat: [1.0, 0.0, 0.0, 0.0].repeat(3),
        jnt_type: vec![0],
        jnt_bodyid: vec![1],
        jnt_qposadr: vec![0],
        jnt_dofadr: vec![0],
        jnt_pos: vec![0.0, 0.0, 0.0],
        jnt_axis: vec![0.0, 0.0, 1.0],
    };
    let inertial = InertialFields {
        body_ipos: vec![0.0; 9],
        body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(3),
        body_mass: vec![0.0, 1.0, 0.5],
        body_inertia: vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.5, 0.5, 0.5],
        dof_bodyid: vec![1; 6],
        dof_jntid: vec![0; 6],
        dof_parentid: (0..6).map(|dof| dof - 1).collect(),
        dof_armature: vec![0.0; 6],
        dof_damping: vec![0.0; 6],
    };
    InertialModelInput::new(KinematicModelInput::new(6, kinematics).unwrap(), inertial).unwrap()
}

fn plan_from_model(
    session: &TransferSession,
    model: InertialModelInput,
    mocap: Vec<i32>,
    equivalent: bool,
) -> KinematicsPlan {
    let attached = AttachedModelInput::new(model, AttachedFields::default()).unwrap();
    let mocap = MocapModelInput::new(attached, mocap).unwrap();
    if !equivalent {
        return KinematicsPlan::with_mocap(session, mocap, KinematicsParameters::default())
            .unwrap();
    }
    let camlight = CamLightModelInput::new_equivalent(mocap, CamLightFields::default()).unwrap();
    let fixed = FixedTendonModelInput::new(camlight, FixedTendonFields::default()).unwrap();
    let model = SpatialTendonModelInput::new(fixed, SpatialTendonFields::default()).unwrap();
    KinematicsPlan::with_tendon_subsets(
        session,
        model,
        KinematicsParameters::default(),
        CamLightParameters::default(),
        OptionalSubsets {
            equivalent: true,
            ..Default::default()
        },
    )
    .unwrap()
}

/// 每世界qpos行：零自由度、单自由度、球关节与自由关节。
fn row(nq: usize, world: usize) -> Vec<f32> {
    match nq {
        0 => vec![],
        1 => vec![0.0625 * world as f32 - 0.5],
        4 => vec![0.5, 0.5, -0.5, 0.5],
        _ => vec![0.0625 * world as f32, -0.5, 0.75, 0.5, 0.5, -0.5, 0.5],
    }
}

/// qpos与mocap位姿随世界变化，显存覆盖多个发射块。
fn state(worlds: usize, nq: usize, nmocap: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut qpos = Vec::new();
    let mut mocap_pos = Vec::new();
    let mut mocap_quat = Vec::new();
    for world in 0..worlds {
        qpos.extend(row(nq, world));
        for mocap in 0..nmocap {
            mocap_pos.extend([
                1.0 + 0.25 * ((world + mocap) % 3) as f32,
                -2.0 + 0.25 * world as f32,
                0.5,
            ]);
            mocap_quat.extend([0.0, 0.0, 1.0, 0.0]);
        }
    }
    (qpos, mocap_pos, mocap_quat)
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn plain_probe_path_is_bitwise_stable() {
    let session = TransferSession::new(0).unwrap();
    for joint in [Some(3), Some(0), None] {
        let model = super::super::tests::test_model(joint);
        let kinematics = model.kinematics();
        let worlds = 513;
        let (qpos, ..) = state(worlds, kinematics.nq(), 0);
        let layout = check_kinematics(&model, worlds, &qpos).unwrap();
        let capture = Capture::new(&format!("probe-{joint:?}"));
        let (output, values) = kinematics_device(&session, &model, layout, &qpos).unwrap();
        capture.expect(PLAIN_PTX);
        assert_eq!(values.len(), output.len());
        let typed = bits(&output);

        let device = upload_kinematics_model(&session, &model, layout).unwrap();
        let state = session
            .upload(if qpos.is_empty() { &[0.0] } else { &qpos })
            .unwrap();
        let mut raw_output = guarded(&session, layout.guarded);
        let capture = Capture::new(&format!("probe-raw-{joint:?}"));
        let kernel = SynchronousKernel::compile(&session, KINEMATICS_CUDA, "rigid_kinematics")
            .map(|kernel| super::super::rigid_kinematics::RigidKernel::new(&session, kernel))
            .unwrap();
        capture.expect(PLAIN_PTX);
        kernel
            .launch_raw(&device, &state, &mut raw_output, worlds as u32)
            .unwrap();
        assert_eq!(typed, bits(&raw_output), "joint={joint:?}");
    }
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn strict_plan_matches_raw_and_recovers_after_metadata_write() {
    let session = TransferSession::new(0).unwrap();
    for joint in [Some(0), None] {
        let (model, mocap, nq) = match joint {
            Some(_) => (free_body_model(), vec![-1, -1, 0], 7),
            None => (super::super::tests::test_model(None), vec![-1, 0], 0),
        };
        let worlds = 513;
        let (qpos, mocap_pos, mocap_quat) = state(worlds, nq, 1);
        let capture = Capture::new(&format!("batch-{joint:?}"));
        let mut plan = plan_from_model(&session, model, mocap, false);
        capture.expect(MOCAP_PTX);
        let mut data = plan.create_data(worlds).unwrap();
        data.write_qpos(&qpos).unwrap();
        data.write_mocap(&mocap_pos, &mocap_quat).unwrap();
        let kinematics = plan.model.rigid().kinematics();
        let (nbody, njnt) = (kinematics.nbody(), kinematics.njnt());
        let result = 4 + worlds * (28 * nbody + 6 * njnt) + 4;

        let before = vec![7.5_f32; result];
        data.rigid.write_range(0, &before).unwrap();
        plan.rigid_kernel
            .launch_raw(&plan.rigid, &data.state, &mut data.rigid, worlds as u32)
            .unwrap();
        let raw = bits(&data.rigid);
        data.rigid.write_range(0, &before).unwrap();
        plan.update_rigid(&mut data).unwrap();
        assert_eq!(raw, bits(&data.rigid), "nq={nq}");

        // 写入真实坏元数据后拒绝，输出与ready保持原样。
        plan.rigid.overwrite_for_test(0, 1).unwrap();
        let rejected = bits(&data.rigid);
        assert!(plan.update_rigid(&mut data).is_err());
        assert!(matches!(
            data.readback(),
            Err(TransferError::StageNotReady { stage: "rigid" })
        ));
        assert_eq!(rejected, bits(&data.rigid));
        plan.rigid.restore_for_test().unwrap();
        plan.update_rigid(&mut data).unwrap();
        assert_eq!(raw, bits(&data.rigid), "nq={nq}");
    }
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn equivalent_plan_matches_raw_after_seeded_body_zero() {
    let session = TransferSession::new(0).unwrap();
    let model = free_body_model();
    let kinematics = model.kinematics();
    let (nq, nbody, njnt) = (kinematics.nq(), kinematics.nbody(), kinematics.njnt());
    let worlds = 513;
    let (qpos, mocap_pos, mocap_quat) = state(worlds, nq, 1);
    let capture = Capture::new("equivalent");
    let plan = plan_from_model(&session, model, vec![-1, -1, 0], true);
    capture.expect(G01_PTX);
    let mut data = plan.create_data(worlds).unwrap();
    data.write_qpos(&qpos).unwrap();
    data.write_mocap(&mocap_pos, &mocap_quat).unwrap();
    let result = 4 + worlds * (28 * nbody + 6 * njnt) + 4;
    let seed: Vec<f32> = (0..result)
        .map(|index| (index as f32 - 4096.0) * 0.000_244_140_63)
        .collect();
    data.rigid.write_range(0, &seed).unwrap();
    plan.rigid_kernel
        .launch_raw(&plan.rigid, &data.state, &mut data.rigid, worlds as u32)
        .unwrap();
    let raw = bits(&data.rigid);
    data.rigid.write_range(0, &seed).unwrap();
    plan.update_rigid(&mut data).unwrap();
    assert_eq!(raw, bits(&data.rigid));
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn velocity_position_matches_rigid_and_rejects_workspaces() {
    let session = TransferSession::new(0).unwrap();
    let worlds = 513;
    for equivalent in [false, true] {
        let model = free_body_model();
        let kinematics = model.kinematics();
        let (nq, nbody, njnt) = (kinematics.nq(), kinematics.nbody(), kinematics.njnt());
        let (qpos, mocap_pos, mocap_quat) = state(worlds, nq, 1);
        let mut plan = plan_from_model(&session, model, vec![-1, -1, 0], equivalent);
        let mut data = plan.create_data(worlds).unwrap();
        data.write_qpos(&qpos).unwrap();
        data.write_mocap(&mocap_pos, &mocap_quat).unwrap();
        plan.update_rigid(&mut data).unwrap();
        let expected = bits(&data.rigid);
        let rigid_length = 4 + worlds * (28 * nbody + 6 * njnt) + 4;
        let com_length = data.layout.com.output.total_elements() + 8;
        let kernels = plan.compile_velocity_position().unwrap();
        let mut rigid = guarded(&session, rigid_length);
        let mut com = guarded(&session, com_length);
        plan.update_velocity_position(&data, &kernels, &mut rigid, &mut com)
            .unwrap();
        assert_eq!(expected, bits(&rigid));

        // 外部工作区长度不符时拒绝且不改写。
        let mut short = guarded(&session, rigid_length - 1);
        let untouched = bits(&short);
        assert!(matches!(
            plan.update_velocity_position(&data, &kernels, &mut short, &mut com),
            Err(TransferError::Input(InputError::LengthMismatch {
                field: "velocity_position_workspace",
                ..
            }))
        ));
        assert_eq!(untouched, bits(&short));

        // 异会话工作区由联合绑定拒绝。
        let foreign = TransferSession::new(0).unwrap();
        let mut foreign_rigid = guarded(&foreign, rigid_length);
        let untouched = bits(&foreign_rigid);
        assert!(matches!(
            plan.update_velocity_position(&data, &kernels, &mut foreign_rigid, &mut com),
            Err(TransferError::SessionMismatch)
        ));
        assert_eq!(untouched, bits(&foreign_rigid));

        // 状态槽伪造为结果跨度：可写槽重叠即别名拒绝。
        let span = rigid.launcher_span().unwrap();
        let rejected = plan.rigid_kernel.plan_with_state_span_for_test(
            &plan.rigid,
            &data.state,
            &rigid,
            worlds as u32,
            span,
        );
        assert!(matches!(
            rejected,
            Err(LauncherError::Alias {
                first: 2,
                second: 3
            })
        ));

        // 坏元数据在发射前拒绝，工作区字节保持原样。
        let before = bits(&rigid);
        plan.rigid.overwrite_for_test(0, 1).unwrap();
        assert!(
            plan.update_velocity_position(&data, &kernels, &mut rigid, &mut com)
                .is_err()
        );
        assert_eq!(before, bits(&rigid));
        plan.rigid.restore_for_test().unwrap();
        plan.update_rigid(&mut data).unwrap();
        plan.update_velocity_position(&data, &kernels, &mut rigid, &mut com)
            .unwrap();
        assert_eq!(bits(&data.rigid), bits(&rigid), "equivalent={equivalent}");
    }
}
