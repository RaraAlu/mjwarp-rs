#![allow(dead_code)]
//! S8 端到端（产品 camlight → 渲染）共享夹具（纯主机，测试期不启动 Python）。
//!
//! 产品输入来自 `fixtures/render-bvh/scene-e2e/camlight-model.json`：authoring 期由
//! `scripts/authoring/render-bvh-s7b/capture_e2e_camlight.py` 从冻结 `scene-315.mjb`
//! 提取的模型字段（double -> f32 一次转换，与产品上传 dtype 相同）。
//!
//! 参考 data 位姿不在本模块重派生：直接读 S7b1 已冻结的同配方夹具
//! （`s7b/depth_seg_fast_math_false.json`）的 `camera_data`，夹具只用于**比对**，
//! 不作为渲染输入（S8 硬口径：渲染输入的位姿必须来自 `physics::camlight`）。
//!
//! 产品链（调用链即证据）：
//! `KinematicsPlan::with_camlight` → `create_data` → `write_qpos`（qpos0）→
//! `plan.update_rigid`（产品 f32 FK）→ `plan.update_attached` → `plan.update_com` →
//! `plan.update_camlight`（`src/physics/camlight.rs` 的 GPU 段）→ `readback`
//! → `KinematicsSnapshot::camlight().world(0)` 的 `cam_xpos`/`cam_xmat`。

use mjwarp_rs::model::{
    AttachedFields, AttachedModelInput, CamLightFields, CamLightModelInput, CamLightParameters,
    InertialFields, InertialModelInput, KinematicFields, KinematicModelInput, KinematicsParameters,
    MocapModelInput,
};
use mjwarp_rs::physics::KinematicsPlan;
use mjwarp_rs::render::kernel::cast::CameraPose;
use mjwarp_rs::runtime::TransferSession;
use serde_json::Value;

use super::s7b2;

pub const CAMLIGHT_MODEL: &str =
    include_str!("../../../fixtures/render-bvh/scene-e2e/camlight-model.json");

fn array(value: &Value) -> &[Value] {
    value.as_array().expect("数组")
}

fn ints(value: &Value) -> Vec<i32> {
    array(value)
        .iter()
        .map(|item| item.as_i64().expect("i32") as i32)
        .collect()
}

fn floats(value: &Value) -> Vec<f32> {
    array(value)
        .iter()
        .map(|item| item.as_f64().expect("f32") as f32)
        .collect()
}

/// 产品 camlight 的模型输入 + 默认姿态（全部来自冻结 mjb 抽取夹具）。
pub struct CamLightModelFixture {
    pub model: CamLightModelInput,
    pub qpos0: Vec<f32>,
    pub ncam: usize,
    pub nlight: usize,
    pub counts: Vec<(&'static str, usize)>,
}

/// 从夹具 JSON 构造产品 `CamLightModelInput`（拓扑/惯性/附着/mocap/camlight 子集）。
pub fn camlight_model_fixture() -> CamLightModelFixture {
    let document: Value = serde_json::from_str(CAMLIGHT_MODEL).expect("camlight 模型夹具 JSON");
    let model = &document["model"];
    let counts = &document["counts"];
    let count = |name: &str| counts[name].as_u64().expect("计数") as usize;
    let nv = count("nv");
    let kinematics = KinematicModelInput::new(
        nv,
        KinematicFields {
            qpos0: floats(&model["qpos0"]),
            body_parentid: ints(&model["body_parentid"]),
            body_jntadr: ints(&model["body_jntadr"]),
            body_jntnum: ints(&model["body_jntnum"]),
            body_pos: floats(&model["body_pos"]),
            body_quat: floats(&model["body_quat"]),
            jnt_type: ints(&model["jnt_type"]),
            jnt_bodyid: ints(&model["jnt_bodyid"]),
            jnt_qposadr: ints(&model["jnt_qposadr"]),
            jnt_dofadr: ints(&model["jnt_dofadr"]),
            jnt_pos: floats(&model["jnt_pos"]),
            jnt_axis: floats(&model["jnt_axis"]),
        },
    )
    .expect("运动学模型输入");
    let inertial = InertialModelInput::new(
        kinematics,
        InertialFields {
            body_ipos: floats(&model["body_ipos"]),
            body_iquat: floats(&model["body_iquat"]),
            body_mass: floats(&model["body_mass"]),
            body_inertia: floats(&model["body_inertia"]),
            dof_bodyid: ints(&model["dof_bodyid"]),
            dof_jntid: ints(&model["dof_jntid"]),
            dof_parentid: ints(&model["dof_parentid"]),
            dof_armature: floats(&model["dof_armature"]),
            dof_damping: floats(&model["dof_damping"]),
        },
    )
    .expect("惯性模型输入");
    let attached = AttachedModelInput::new(
        inertial,
        AttachedFields {
            geom_bodyid: ints(&model["geom_bodyid"]),
            geom_pos: floats(&model["geom_pos"]),
            geom_quat: floats(&model["geom_quat"]),
            site_bodyid: ints(&model["site_bodyid"]),
            site_pos: floats(&model["site_pos"]),
            site_quat: floats(&model["site_quat"]),
        },
    )
    .expect("附着模型输入");
    let mocap =
        MocapModelInput::new(attached, ints(&model["body_mocapid"])).expect("mocap 模型输入");
    let camlight = CamLightModelInput::new(
        mocap,
        CamLightFields {
            cam_mode: ints(&model["cam_mode"]),
            cam_bodyid: ints(&model["cam_bodyid"]),
            cam_targetbodyid: ints(&model["cam_targetbodyid"]),
            cam_pos: floats(&model["cam_pos"]),
            cam_quat: floats(&model["cam_quat"]),
            cam_poscom0: floats(&model["cam_poscom0"]),
            cam_pos0: floats(&model["cam_pos0"]),
            cam_mat0: floats(&model["cam_mat0"]),
            light_mode: ints(&model["light_mode"]),
            light_bodyid: ints(&model["light_bodyid"]),
            light_targetbodyid: ints(&model["light_targetbodyid"]),
            light_pos: floats(&model["light_pos"]),
            light_dir: floats(&model["light_dir"]),
            light_poscom0: floats(&model["light_poscom0"]),
            light_pos0: floats(&model["light_pos0"]),
            light_dir0: floats(&model["light_dir0"]),
        },
    )
    .expect("camlight 模型输入");
    CamLightModelFixture {
        qpos0: floats(&model["qpos0"]),
        ncam: count("ncam"),
        nlight: count("nlight"),
        counts: vec![
            ("nq", count("nq")),
            ("nv", count("nv")),
            ("nbody", count("nbody")),
            ("njnt", count("njnt")),
            ("ngeom", count("ngeom")),
            ("nsite", count("nsite")),
            ("nmocap", count("nmocap")),
            ("ncam", count("ncam")),
            ("nlight", count("nlight")),
        ],
        model: camlight,
    }
}

/// 产品 camlight 输出：按模型相机 id 索引的 data 位姿 + 灯光位姿。
pub struct ProductCamLight {
    pub poses: Vec<CameraPose>,
    pub light_xpos: Vec<[f32; 3]>,
    pub light_xdir: Vec<[f32; 3]>,
    /// 产品 FK 的 body 位姿（`xpos`/`xquat`，按 body id）——用于证据追溯。
    pub body_xpos: Vec<[f32; 3]>,
    pub body_xquat: Vec<[f32; 4]>,
}

/// 跑产品链：产品 FK（`update_rigid`）→ 产品 camlight（`update_camlight`）→ 回读。
///
/// 这是 S8「渲染输入位姿来源」的唯一入口：返回的 `poses` 直接喂给渲染内核，
/// 夹具 `camera_data` 只在本函数之外用于比对。
pub fn product_camlight(session: &TransferSession) -> ProductCamLight {
    let fixture = camlight_model_fixture();
    let ncam = fixture.ncam;
    let nlight = fixture.nlight;
    let plan = KinematicsPlan::with_camlight(
        session,
        fixture.model,
        KinematicsParameters::default(),
        CamLightParameters::default(),
    )
    .expect("产品 camlight 计划（模型字段已校验）");
    let mut data = plan.create_data(1).expect("world=1 数据");
    data.write_qpos(&fixture.qpos0).expect("qpos0 写入");
    plan.update_rigid(&mut data).expect("产品刚体位姿");
    plan.update_attached(&mut data).expect("附着位姿");
    plan.update_com(&mut data).expect("质心");
    plan.update_camlight(&mut data).expect("产品 camlight");
    let snapshot = data.readback().expect("kinematics 回读");
    let rigid = snapshot.rigid().world(0).expect("刚体 world 0");
    let world = snapshot.camlight().world(0).expect("camlight world 0");
    let poses = (0..ncam)
        .map(|camera| {
            let mut xmat = [0.0f32; 9];
            xmat.copy_from_slice(&world.cam_xmat[9 * camera..9 * camera + 9]);
            CameraPose {
                xpos: [
                    world.cam_xpos[3 * camera],
                    world.cam_xpos[3 * camera + 1],
                    world.cam_xpos[3 * camera + 2],
                ],
                xmat,
            }
        })
        .collect();
    let light_xpos = (0..nlight)
        .map(|light| {
            [
                world.light_xpos[3 * light],
                world.light_xpos[3 * light + 1],
                world.light_xpos[3 * light + 2],
            ]
        })
        .collect();
    let light_xdir = (0..nlight)
        .map(|light| {
            [
                world.light_xdir[3 * light],
                world.light_xdir[3 * light + 1],
                world.light_xdir[3 * light + 2],
            ]
        })
        .collect();
    ProductCamLight {
        poses,
        light_xpos,
        light_xdir,
        body_xpos: rigid.xpos.chunks(3).map(|c| [c[0], c[1], c[2]]).collect(),
        body_xquat: rigid
            .xquat
            .chunks(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .collect(),
    }
}

/// 位型视图：把产品 data 位姿转成 `u32` 位（与夹具位型直接比较）。
pub fn pose_bits(pose: &CameraPose) -> ([u32; 3], [u32; 9]) {
    (pose.xpos.map(f32::to_bits), pose.xmat.map(f32::to_bits))
}

/// 与参考 data 位姿的逐位比较结果。
#[derive(Clone, Debug, Default)]
pub struct PoseBitDiff {
    /// 每相机 origin 位不等的个数（0..=3），以及 xmat 位不等的个数（0..=9）。
    pub origin_mismatches: Vec<usize>,
    pub xmat_mismatches: Vec<usize>,
    /// 最大 f32 绝对差（非位等时的定位参考）。
    pub max_origin_abs: f32,
    pub max_xmat_abs: f32,
}

/// 产品 data 位姿 vs 参考 data 位姿逐位比较。
pub fn compare_pose_bits(
    product: &[CameraPose],
    reference: &[mjwarp_rs::render::kernel::output::CameraData],
) -> PoseBitDiff {
    let mut diff = PoseBitDiff::default();
    for (index, expected) in reference.iter().enumerate() {
        let pose = product.get(index).copied().unwrap_or(CameraPose {
            xpos: [f32::NAN; 3],
            xmat: [f32::NAN; 9],
        });
        let (origin_bits, xmat_bits) = pose_bits(&pose);
        let origin_mismatch = origin_bits
            .iter()
            .zip(expected.origin_bits.iter())
            .filter(|(a, b)| a != b)
            .count();
        let xmat_mismatch = xmat_bits
            .iter()
            .zip(expected.xmat_bits.iter())
            .filter(|(a, b)| a != b)
            .count();
        for (actual, bits) in origin_bits.iter().zip(expected.origin_bits.iter()) {
            diff.max_origin_abs = diff
                .max_origin_abs
                .max((f32::from_bits(*actual) - f32::from_bits(*bits)).abs());
        }
        for (actual, bits) in xmat_bits.iter().zip(expected.xmat_bits.iter()) {
            diff.max_xmat_abs = diff
                .max_xmat_abs
                .max((f32::from_bits(*actual) - f32::from_bits(*bits)).abs());
        }
        diff.origin_mismatches.push(origin_mismatch);
        diff.xmat_mismatches.push(xmat_mismatch);
    }
    diff
}

// ---- S8 端到端验收公共断言（`tests/render_e2e.rs` 的用例只按结论调用） ----

/// 渲染一帧并读回三通道：`(rgb packed u32, depth f32, seg 对)`。
pub fn render_read_all(
    renderer: &mjwarp_rs::render::kernel::MegakernelRenderer,
    context: &mjwarp_rs::render::RenderContext,
    fields: &mjwarp_rs::model::render::RenderFields,
    model: &mjwarp_rs::render::CameraModel,
    poses: &[CameraPose],
    rays: &mjwarp_rs::render::RayBuffer,
) -> (Vec<u32>, Vec<f32>, Vec<[i32; 2]>) {
    renderer
        .render(context, fields, model, poses, rays)
        .expect("S8 渲染发射");
    let outputs = context.outputs();
    let rgb = match outputs
        .read(mjwarp_rs::render::ImageOutput::Rgb)
        .expect("rgb 读回")
    {
        mjwarp_rs::render::OutputValues::Rgb(values) => values,
        other => panic!("rgb 类型 {other:?}"),
    };
    let depth = match outputs
        .read(mjwarp_rs::render::ImageOutput::Depth)
        .expect("depth 读回")
    {
        mjwarp_rs::render::OutputValues::Depth(values) => values,
        other => panic!("depth 类型 {other:?}"),
    };
    let seg = match outputs
        .read(mjwarp_rs::render::ImageOutput::Segmentation)
        .expect("seg 读回")
    {
        mjwarp_rs::render::OutputValues::Seg(values) => values,
        other => panic!("seg 类型 {other:?}"),
    };
    (rgb, depth, seg)
}

/// depth/seg 对同配方参考：depth ≤4 ULP 且 seg 逐像素全等（本夹具期望 0/0）；
/// 返回 `(depth_mismatch, max_ulp, seg_mismatch)`。
pub fn assert_depth_seg_same_recipe(
    reference: &mjwarp_rs::render::kernel::output::DepthSegReference,
    depth: &[f32],
    seg: &[[i32; 2]],
) -> (usize, i64, usize) {
    use mjwarp_rs::render::kernel::output::{MAX_DEPTH_ULP, diff_depth, diff_seg};
    let (mut depth_mismatch, mut max_ulp, mut seg_mismatch) = (0usize, 0i64, 0usize);
    for camera in &reference.cameras {
        let base = camera.depth_adr as usize;
        let seg_base = camera.seg_adr as usize;
        let diff = diff_depth(&depth[base..base + camera.pixels], camera).expect("depth diff");
        let seg_diff =
            diff_seg(&seg[seg_base..seg_base + camera.pixels], camera).expect("seg diff");
        println!(
            "s8[e2e] cam{} depth: compared={} mismatch={} max_ulp={} seg_mismatch={}",
            camera.index, camera.pixels, diff.mismatched, diff.max_ulp, seg_diff.mismatched
        );
        assert_eq!(
            diff.mismatched, 0,
            "cam{} depth 必须与同配方参考位等（first={:?}）",
            camera.index, diff.first
        );
        assert!(diff.max_ulp <= MAX_DEPTH_ULP);
        assert_eq!(
            seg_diff.mismatched, 0,
            "cam{} seg 必须与同配方参考全等",
            camera.index
        );
        depth_mismatch += diff.mismatched;
        seg_mismatch += seg_diff.mismatched;
        max_ulp = max_ulp.max(diff.max_ulp);
    }
    (depth_mismatch, max_ulp, seg_mismatch)
}

/// RGB（packed u32）对同配方参考：解包后每通道 ≤1 LSB（byte 差 ≤1，期望 0）。
///
/// 打印逐相机直方图与按 geom 分解；返回超 1 LSB 像素总数（应为 0）。
pub fn assert_rgb_same_recipe(
    reference: &s7b2::RgbReference,
    recipe: &mjwarp_rs::render::kernel::output::DepthSegReference,
    rgb_adr: &[i32],
    rgb: &[u32],
) -> usize {
    let mut beyond = 0usize;
    for camera in &reference.cameras {
        let start = rgb_adr[camera.index] as usize;
        let actual = &rgb[start..start + camera.pixels];
        let stats = s7b2::diff_rgb(actual, camera);
        let (histogram, by_geom) =
            s7b2::rgb_diff_histogram(actual, camera, &recipe.cameras[camera.index].seg_pairs);
        println!(
            "s8[e2e] cam{} rgb: pixels={} mismatched={} max_byte_diff={} beyond_1_lsb={} \
             first={:?} histogram(0,1,2,3,>3)={:?} beyond_by_geom={:?}",
            camera.index,
            stats.compared,
            stats.mismatched_pixels,
            stats.max_byte_diff,
            stats.beyond_one_lsb_pixels,
            stats.first,
            histogram,
            by_geom
        );
        assert_eq!(
            stats.beyond_one_lsb_pixels,
            0,
            "cam{} RGB 超 1 LSB（同配方参考）：{} / {}，max_byte_diff={} first={:?}",
            camera.index,
            stats.beyond_one_lsb_pixels,
            stats.compared,
            stats.max_byte_diff,
            stats.first
        );
        beyond += stats.beyond_one_lsb_pixels;
    }
    beyond
}

/// 跨配方（S0b fast-math）偏差记录：只打印、不作通过依据。
pub fn print_cross_recipe(
    reference: &s7b2::RgbReference,
    same_recipe: &s7b2::RgbReference,
    rgb_adr: &[i32],
    rgb: &[u32],
) {
    for camera in &reference.cameras {
        let start = rgb_adr[camera.index] as usize;
        let actual = &rgb[start..start + camera.pixels];
        let stats = s7b2::diff_rgb(actual, camera);
        println!(
            "s8[e2e][cross-recipe-record] cam{}: beyond_1_lsb={} max_byte_diff={} \
             fixture_recorded={:?}",
            camera.index,
            stats.beyond_one_lsb_pixels,
            stats.max_byte_diff,
            same_recipe.cross_recipe_histograms.get(camera.index)
        );
    }
}

/// 灯位姿一致性：产品 camlight 输出 == 渲染器 `light_data_poses` == pinned 3.15 参考位型。
pub fn assert_light_poses(
    product: &ProductCamLight,
    fields: &mjwarp_rs::model::render::RenderFields,
) {
    let poses = mjwarp_rs::render::kernel::light_data_poses(fields).expect("light_data_poses");
    let checked = s7b2::check_light_data(&s7b2::parse_rgb_reference(), fields);
    assert_eq!(
        (poses.len(), product.light_xpos.len(), checked.len()),
        (3, 3, 3),
        "scene-a 三盏灯"
    );
    for (index, pose) in poses.iter().enumerate() {
        assert_eq!(
            product.light_xpos[index].map(f32::to_bits),
            pose.xpos.map(f32::to_bits),
            "light{index} xpos：产品 camlight 必须与渲染器灯位姿逐位一致"
        );
        assert_eq!(
            product.light_xdir[index].map(f32::to_bits),
            pose.xdir.map(f32::to_bits),
            "light{index} xdir：产品 camlight 必须与渲染器灯位姿逐位一致"
        );
    }
}

/// 篡改比对副本的夹具 data 位姿（identity + 原点），用于「渲染不读夹具」反证。
pub fn tampered_camera_data(
    reference: &mjwarp_rs::render::kernel::output::DepthSegReference,
) -> mjwarp_rs::render::kernel::output::DepthSegReference {
    let mut tampered = reference.clone();
    for data in &mut tampered.camera_data {
        data.origin_bits = [0u32; 3];
        data.xmat_bits = [
            1.0f32.to_bits(),
            0,
            0,
            0,
            1.0f32.to_bits(),
            0,
            0,
            0,
            1.0f32.to_bits(),
        ];
    }
    tampered
}

/// 产品位姿注入扰动：camera1 绕 y 轴旋转 `angle` 并将 `xpos` 平移 `(dx, 0, dz)`。
pub fn perturb_camera1(poses: &[CameraPose], angle: f32, dx: f32, dz: f32) -> Vec<CameraPose> {
    let mut perturbed = poses.to_vec();
    let (rotation, sine) = (angle.cos(), angle.sin());
    let m = perturbed[1].xmat;
    perturbed[1].xmat = [
        rotation * m[0] + sine * m[6],
        rotation * m[1] + sine * m[7],
        rotation * m[2] + sine * m[8],
        m[3],
        m[4],
        m[5],
        -sine * m[0] + rotation * m[6],
        -sine * m[1] + rotation * m[7],
        -sine * m[2] + rotation * m[8],
    ];
    perturbed[1].xpos[0] += dx;
    perturbed[1].xpos[2] += dz;
    perturbed
}
