#![cfg(feature = "cuda-probe")]
//! G01字段批量的解析与独立参考。
//! 测试不生成原生参考。

use mjwarp_rs::{
    model::{
        AttachedFields, AttachedModelInput, InertialFields, InertialModelInput, KinematicFields,
        KinematicModelInput, KinematicsParameter as F, KinematicsParameters, ParameterBatch,
    },
    physics::{KinematicsPlan, KinematicsSnapshot},
    runtime::TransferSession,
};
use serde_json::Value;

#[path = "support/rigid_reference.rs"]
mod reference;

const ID: [f32; 4] = [1.0, 0.0, 0.0, 0.0];
const ROTATIONS: [[f32; 4]; 3] = [ID, [0.0, 1.0, 0.0, 0.0], [0.5; 4]];
const MATRIX_ID: [f64; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
const PERIODS: [usize; 13] = [3, 1, 3, 5, 4, 7, 2, 3, 5, 2, 4, 7, 3];
const ABS: f64 = 2e-5;
const REL: f64 = 2e-5;

fn slide_model() -> AttachedModelInput {
    let k = KinematicModelInput::new(
        1,
        KinematicFields {
            qpos0: vec![0.0],
            body_parentid: vec![0, 0],
            body_jntadr: vec![-1, 0],
            body_jntnum: vec![0, 1],
            body_pos: vec![0.0; 6],
            body_quat: ID.repeat(2),
            jnt_type: vec![2],
            jnt_bodyid: vec![1],
            jnt_qposadr: vec![0],
            jnt_dofadr: vec![0],
            jnt_pos: vec![0.0; 3],
            jnt_axis: vec![1.0, 0.0, 0.0],
        },
    )
    .unwrap();
    let i = InertialModelInput::new(
        k,
        InertialFields {
            body_ipos: vec![0.0; 6],
            body_iquat: ID.repeat(2),
            body_mass: vec![0.0, 2.0],
            body_inertia: vec![0.0, 0.0, 0.0, 0.5, 0.75, 1.0],
            dof_bodyid: vec![1],
            dof_jntid: vec![0],
            dof_parentid: vec![-1],
            dof_armature: vec![0.0],
            dof_damping: vec![0.0],
        },
    )
    .unwrap();
    AttachedModelInput::new(
        i,
        AttachedFields {
            geom_bodyid: vec![0, 1],
            geom_pos: vec![0.0; 6],
            geom_quat: ID.repeat(2),
            site_bodyid: vec![1],
            site_pos: vec![0.0; 3],
            site_quat: ID.to_vec(),
        },
    )
    .unwrap()
}

fn parameter_row(field: F, row: usize) -> Vec<f32> {
    let r = row as f32;
    match field {
        F::Qpos0 => vec![0.25 * r],
        F::BodyPos => vec![0.0, 0.0, 0.0, 0.05 * r, -0.1 * r, 0.25],
        F::BodyQuat | F::BodyIquat => [ID.as_slice(), &ROTATIONS[row % 3]].concat(),
        F::BodyIpos => vec![0.0, 0.0, 0.0, 0.125 + r * 0.05, -0.25, 0.5],
        F::JointPos => vec![-0.125 * r, 0.25, 0.375],
        F::JointAxis => {
            let mut axis = vec![0.0; 3];
            axis[row % 3] = 1.0;
            axis
        }
        F::BodyMass => vec![0.0, 2.0 + r],
        F::BodyInertia => vec![0.0, 0.0, 0.0, 0.5 + r * 0.1, 0.75 + r * 0.2, 1.0 + r * 0.3],
        F::GeomPos => vec![1.0 + r, 0.25, -0.125, 0.25, -0.5 + r * 0.1, 0.75],
        F::GeomQuat => [ROTATIONS[row % 3].as_slice(), &ROTATIONS[(row + 1) % 3]].concat(),
        F::SitePos => vec![-0.25, 0.5, -0.75 + r * 0.1],
        F::SiteQuat => ROTATIONS[row % 3].to_vec(),
    }
}

fn parameters(worlds: usize) -> KinematicsParameters {
    let mut parameters = KinematicsParameters::default();
    for (index, field) in F::ALL.into_iter().enumerate() {
        let period = if field == F::BodyPos {
            worlds
        } else {
            PERIODS[index]
        };
        let width = parameter_row(field, 0).len();
        let values = (0..period).flat_map(|r| parameter_row(field, r)).collect();
        parameters.set(field, ParameterBatch::new(period, width, values).unwrap());
    }
    parameters
}

// Known axis permutations give independent closed-form rotation matrices.
// This oracle never invokes GPU kernels or copies their quaternion formulas.
fn matrix(q: &[f32]) -> [f64; 9] {
    match q {
        [1.0, 0.0, 0.0, 0.0] => MATRIX_ID,
        [0.0, 1.0, 0.0, 0.0] => [1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0],
        [0.5, 0.5, 0.5, 0.5] => [0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        _ => panic!("unknown analytic rotation"),
    }
}

fn rotate(m: &[f64; 9], v: &[f32]) -> [f64; 3] {
    std::array::from_fn(|i| (0..3).map(|j| m[3 * i + j] * f64::from(v[j])).sum())
}
fn multiply(a: &[f64; 9], b: &[f64; 9]) -> [f64; 9] {
    std::array::from_fn(|i| (0..3).map(|k| a[3 * (i / 3) + k] * b[3 * k + i % 3]).sum())
}
fn translate(p: &[f64; 3], m: &[f64; 9], local: &[f32]) -> [f64; 3] {
    let offset = rotate(m, local);
    std::array::from_fn(|i| p[i] + offset[i])
}

fn assert_fields(fields: Vec<(&str, &[f32], Vec<f64>)>, world: usize, kind: &str) {
    let mut largest = 0.0f64;
    for (field, actual, expected) in fields {
        assert_eq!(actual.len(), expected.len());
        for (i, (&a, &e)) in actual.iter().zip(&expected).enumerate() {
            let error = (f64::from(a) - e).abs();
            largest = largest.max(error);
            assert!(
                a.is_finite() && error <= ABS + REL * e.abs(),
                "G01 field batches {kind} {field} w={world} i={i}: {a} != {e}; error={error}"
            );
        }
    }
    eprintln!("G01-field-batches kind={kind} world={world} max_abs_error={largest:e}");
}

fn check_analytic(out: &KinematicsSnapshot, parameters: &KinematicsParameters, qpos: &[f32]) {
    for (w, &q) in qpos.iter().enumerate() {
        let field = |f| parameters.get(f).unwrap().world(w);
        let r = matrix(&field(F::BodyQuat)[4..]);
        let ir = multiply(&r, &matrix(&field(F::BodyIquat)[4..]));
        let axis = rotate(&r, field(F::JointAxis));
        let pos: [f64; 3] = std::array::from_fn(|i| {
            f64::from(field(F::BodyPos)[3 + i])
                + axis[i] * (f64::from(q) - f64::from(field(F::Qpos0)[0]))
        });
        let ipos = translate(&pos, &r, &field(F::BodyIpos)[3..]);
        let anchor = translate(&pos, &r, field(F::JointPos));
        // The slide anchor precedes the slide displacement.
        let anchor: Vec<_> = (0..3)
            .map(|i| anchor[i] - axis[i] * (f64::from(q) - f64::from(field(F::Qpos0)[0])))
            .collect();
        let mass = f64::from(field(F::BodyMass)[1]);
        let inertia = &field(F::BodyInertia)[3..];
        let tensor: [f64; 9] = std::array::from_fn(|i| {
            (0..3)
                .map(|a| ir[3 * (i / 3) + a] * f64::from(inertia[a]) * ir[3 * (i % 3) + a])
                .sum()
        });
        let mut ci = vec![0.0; 10];
        ci.extend([
            tensor[0], tensor[4], tensor[8], tensor[1], tensor[2], tensor[5], 0.0, 0.0, 0.0, mass,
        ]);
        let rig = out.rigid().world(w).unwrap();
        let com = out.com().world(w).unwrap();
        let attached = out.attached().world(w).unwrap();
        let gp = field(F::GeomPos);
        let gm = field(F::GeomQuat);
        assert_fields(
            vec![
                ("xpos", rig.xpos, [vec![0.0; 3], pos.to_vec()].concat()),
                (
                    "xquat",
                    rig.xquat,
                    field(F::BodyQuat).iter().map(|&x| f64::from(x)).collect(),
                ),
                ("xmat", rig.xmat, [MATRIX_ID.as_slice(), &r].concat()),
                ("xipos", rig.xipos, [vec![0.0; 3], ipos.to_vec()].concat()),
                ("ximat", rig.ximat, [MATRIX_ID.as_slice(), &ir].concat()),
                ("xanchor", rig.xanchor, anchor),
                ("xaxis", rig.xaxis, axis.to_vec()),
                ("subtree_mass", com.subtree_mass, vec![mass, mass]),
                ("subtree_com", com.subtree_com, ipos.repeat(2)),
                ("cinert", com.cinert, ci),
                ("cdof", com.cdof, [vec![0.0; 3], axis.to_vec()].concat()),
                (
                    "geom_xpos",
                    attached.geom_xpos,
                    [
                        gp[..3].iter().map(|&x| f64::from(x)).collect(),
                        translate(&pos, &r, &gp[3..]).to_vec(),
                    ]
                    .concat(),
                ),
                (
                    "geom_xmat",
                    attached.geom_xmat,
                    [
                        matrix(&gm[..4]).as_slice(),
                        &multiply(&r, &matrix(&gm[4..])),
                    ]
                    .concat(),
                ),
                (
                    "site_xpos",
                    attached.site_xpos,
                    translate(&pos, &r, field(F::SitePos)).to_vec(),
                ),
                (
                    "site_xmat",
                    attached.site_xmat,
                    multiply(&r, &matrix(field(F::SiteQuat))).to_vec(),
                ),
            ],
            w,
            "analytic",
        );
    }
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn independent_field_periods_match_closed_form_across_blocks() {
    let session = TransferSession::new(0).unwrap();
    for worlds in [1, 2, 5, 513] {
        let parameters = parameters(worlds);
        let plan =
            KinematicsPlan::with_parameters(&session, slide_model(), parameters.clone()).unwrap();
        assert_eq!(plan.parameters(), &parameters);
        let mut data = plan.create_data(worlds).unwrap();
        let mut qpos: Vec<_> = (0..worlds)
            .map(|w| parameters.get(F::Qpos0).unwrap().world(w)[0])
            .collect();
        plan.update(&mut data).unwrap();
        check_analytic(&data.readback().unwrap(), &parameters, &qpos);
        qpos = (0..worlds).map(|w| 0.3 - (w % 9) as f32 * 0.01).collect();
        data.write_qpos(&qpos).unwrap();
        plan.update(&mut data).unwrap();
        check_analytic(&data.readback().unwrap(), &parameters, &qpos);
        qpos[worlds - 1] = -0.75;
        data.write_world_qpos(worlds - 1, &[-0.75]).unwrap();
        plan.update_rigid(&mut data).unwrap();
        plan.update_com(&mut data).unwrap();
        plan.update_attached(&mut data).unwrap();
        check_analytic(&data.readback().unwrap(), &parameters, &qpos);
    }
    session.upload(&[1.0f32]).unwrap();
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn qpos0_periods_preserve_independent_native_joint_references() {
    let v: Value =
        serde_json::from_str(include_str!("../fixtures/kinematics/mixed-joints.json")).unwrap();
    let model = reference::model(&v);
    let k = model.kinematics();
    let shared = &k.fields().qpos0;
    let mut rows = shared.repeat(3);
    for row in 0..3 {
        for j in 0..k.njnt() {
            if k.fields().jnt_type[j] >= 2 {
                rows[row * k.nq() + k.fields().jnt_qposadr[j] as usize] += row as f32 * 0.125;
            }
        }
    }
    let mut parameters = KinematicsParameters::default();
    parameters.set(
        F::Qpos0,
        ParameterBatch::new(3, k.nq(), rows.clone()).unwrap(),
    );
    let session = TransferSession::new(0).unwrap();
    let plan = KinematicsPlan::with_parameters(
        &session,
        AttachedModelInput::new(model.clone(), AttachedFields::default()).unwrap(),
        parameters,
    )
    .unwrap();
    let cases = v["cases"].as_array().unwrap();
    let mut data = plan.create_data(513).unwrap();
    for round in 0..2 {
        let mut qpos = vec![];
        for w in 0..513 {
            let mut q = reference::floats(&cases[(w + round) % cases.len()], "qpos");
            for j in 0..k.njnt() {
                if k.fields().jnt_type[j] >= 2 {
                    let a = k.fields().jnt_qposadr[j] as usize;
                    q[a] += rows[(w % 3) * k.nq() + a] - shared[a];
                }
            }
            qpos.extend(q);
        }
        data.write_qpos(&qpos).unwrap();
        plan.update(&mut data).unwrap();
        let out = data.readback().unwrap();
        for w in 0..513 {
            let r = out.rigid().world(w).unwrap();
            let c = &cases[(w + round) % cases.len()];
            assert_fields(
                [
                    ("xpos", r.xpos),
                    ("xquat", r.xquat),
                    ("xmat", r.xmat),
                    ("xipos", r.xipos),
                    ("ximat", r.ximat),
                    ("xanchor", r.xanchor),
                    ("xaxis", r.xaxis),
                ]
                .into_iter()
                .map(|(name, actual)| {
                    (
                        name,
                        actual,
                        c[name]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|x| x.as_f64().unwrap())
                            .collect(),
                    )
                })
                .collect(),
                w,
                "native",
            );
        }
    }
    session.upload(&[1.0f32]).unwrap();
}

#[test]
#[ignore = "requires NVIDIA driver and NVRTC"]
fn empty_fields_keep_positive_long_periods_without_world_expansion() {
    let v: Value =
        serde_json::from_str(include_str!("../fixtures/kinematics/zero-dof.json")).unwrap();
    let rigid = reference::model(&v);
    let mut parameters = KinematicsParameters::default();
    for field in [
        F::Qpos0,
        F::JointPos,
        F::JointAxis,
        F::GeomPos,
        F::GeomQuat,
        F::SitePos,
        F::SiteQuat,
    ] {
        parameters.set(
            field,
            ParameterBatch::new(i32::MAX as usize, 0, vec![]).unwrap(),
        );
    }
    let session = TransferSession::new(0).unwrap();
    let plan = KinematicsPlan::with_parameters(
        &session,
        AttachedModelInput::new(rigid, AttachedFields::default()).unwrap(),
        parameters,
    )
    .unwrap();
    let mut data = plan.create_data(513).unwrap();
    data.write_qpos(&[]).unwrap();
    plan.update(&mut data).unwrap();
    let out = data.readback().unwrap();
    let c = &v["cases"][0];
    for w in 0..513 {
        let r = out.rigid().world(w).unwrap();
        assert_fields(
            [("xpos", r.xpos), ("xmat", r.xmat), ("xipos", r.xipos)]
                .into_iter()
                .map(|(name, actual)| {
                    (
                        name,
                        actual,
                        c[name]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|x| x.as_f64().unwrap())
                            .collect(),
                    )
                })
                .collect(),
            w,
            "empty",
        );
        assert!(out.attached().world(w).unwrap().geom_xpos.is_empty());
        assert!(out.attached().world(w).unwrap().site_xpos.is_empty());
        assert!(out.com().world(w).unwrap().cdof.is_empty());
    }
    session.upload(&[1.0f32]).unwrap();
}
