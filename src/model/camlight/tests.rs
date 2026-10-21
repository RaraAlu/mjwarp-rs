use super::*;
use crate::model::{
    AttachedFields, AttachedModelInput, InertialFields, InertialModelInput, KinematicFields,
    KinematicModelInput,
};
use crate::model::{MocapModelInput, ParameterBatch};

fn base() -> MocapModelInput {
    let k = KinematicModelInput::new(
        0,
        KinematicFields {
            body_parentid: vec![0],
            body_jntadr: vec![-1],
            body_jntnum: vec![0],
            body_pos: vec![0.0; 3],
            body_quat: vec![1.0, 0.0, 0.0, 0.0],
            ..Default::default()
        },
    )
    .unwrap();
    let i = InertialModelInput::new(
        k,
        InertialFields {
            body_ipos: vec![0.0; 3],
            body_iquat: vec![1.0, 0.0, 0.0, 0.0],
            body_mass: vec![0.0],
            body_inertia: vec![0.0; 3],
            ..Default::default()
        },
    )
    .unwrap();
    MocapModelInput::new(
        AttachedModelInput::new(i, AttachedFields::default()).unwrap(),
        vec![-1],
    )
    .unwrap()
}
fn fields() -> CamLightFields {
    CamLightFields {
        cam_mode: vec![0, 1, 2, 3, 4],
        cam_bodyid: vec![0; 5],
        cam_targetbodyid: vec![-2; 5],
        cam_pos: vec![-0.0; 15],
        cam_quat: [1.0, 0.0, 0.0, 0.0].repeat(5),
        cam_pos0: vec![0.0; 15],
        cam_poscom0: vec![0.0; 15],
        cam_mat0: vec![0.0; 45],
        light_mode: vec![0, 1, 2, 3, 4],
        light_bodyid: vec![0; 5],
        light_targetbodyid: vec![-2; 5],
        light_pos: vec![0.0; 15],
        light_dir: [0.0, 0.0, 2.0].repeat(5),
        light_pos0: vec![0.0; 15],
        light_poscom0: vec![0.0; 15],
        light_dir0: vec![0.0; 15],
    }
}
fn field_mut(fields: &mut CamLightFields, f: CamLightParameter) -> &mut Vec<f32> {
    match f {
        CamLightParameter::CamPos => &mut fields.cam_pos,
        CamLightParameter::CamQuat => &mut fields.cam_quat,
        CamLightParameter::CamPos0 => &mut fields.cam_pos0,
        CamLightParameter::CamPoscom0 => &mut fields.cam_poscom0,
        CamLightParameter::CamMat0 => &mut fields.cam_mat0,
        CamLightParameter::LightPos => &mut fields.light_pos,
        CamLightParameter::LightDir => &mut fields.light_dir,
        CamLightParameter::LightPos0 => &mut fields.light_pos0,
        CamLightParameter::LightPoscom0 => &mut fields.light_poscom0,
        CamLightParameter::LightDir0 => &mut fields.light_dir0,
    }
}
#[test]
fn preserves_all_modes_negative_targets_and_independent_periods() {
    let fields = fields();
    let model = CamLightModelInput::new(base(), fields.clone()).unwrap();
    assert_eq!((model.ncam(), model.nlight()), (5, 5));
    assert_eq!(model.fields(), &fields);
    assert_eq!(model.fields().cam_pos[0].to_bits(), (-0.0f32).to_bits());
    assert_eq!(model.clone(), model);
    let mut p = CamLightParameters::default();
    for (i, f) in CamLightParameter::ALL.into_iter().enumerate() {
        let shared = f.shared(&fields);
        p.set(
            f,
            ParameterBatch::new(i + 2, shared.len(), shared.repeat(i + 2)).unwrap(),
        );
        assert_eq!(p.get(f).unwrap().world(513), shared);
    }
    p.validate(&model).unwrap();
    let empty = CamLightModelInput::new(base(), CamLightFields::default()).unwrap();
    for f in CamLightParameter::ALL {
        p.set(
            f,
            ParameterBatch::new(i32::MAX as usize, 0, vec![]).unwrap(),
        );
    }
    p.validate(&empty).unwrap();
}
#[test]
fn rejects_every_float_length_nonfinite_and_nonunit_rotation() {
    for f in CamLightParameter::ALL {
        let mut short = fields();
        field_mut(&mut short, f).pop();
        assert!(
            matches!(CamLightModelInput::new(base(),short),Err(InputError::LengthMismatch{field,..}) if field==f.name())
        );
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut fields = fields();
            *field_mut(&mut fields, f).last_mut().unwrap() = bad;
            assert!(
                matches!(CamLightModelInput::new(base(),fields),Err(InputError::NonFinite{field,..}) if field==f.name())
            );
        }
    }
    for bad in [0.0, 2.0] {
        let mut fields = fields();
        fields.cam_quat[0] = bad;
        assert!(matches!(
            CamLightModelInput::new(base(), fields),
            Err(InputError::InvalidTopology {
                field: "cam_quat",
                ..
            })
        ));
    }
    let model = CamLightModelInput::new(base(), fields()).unwrap();
    let mut p = CamLightParameters::default();
    for f in CamLightParameter::ALL {
        p.set(f, ParameterBatch::new(1, 0, vec![]).unwrap());
        assert!(p.validate(&model).is_err());
        p = CamLightParameters::default();
    }
    let mut q = model.fields().cam_quat.repeat(7);
    *q.last_mut().unwrap() = 2.0;
    p.set(
        CamLightParameter::CamQuat,
        ParameterBatch::new(7, 20, q).unwrap(),
    );
    assert!(p.validate(&model).is_err());
}
#[test]
fn equivalent_rotation_still_requires_strict_shared_or_override_validation() {
    let mut fields = fields();
    fields.cam_quat[16] = 2.0;
    let model = CamLightModelInput::new_equivalent(base(), fields.clone()).unwrap();
    assert_eq!(model.fields(), &fields);
    assert!(CamLightModelInput::new(base(), fields).is_err());
    let mut parameters = CamLightParameters::default();
    assert!(matches!(
        parameters.validate(&model),
        Err(InputError::InvalidTopology {
            field: "cam_quat",
            index: 4,
            reason: "nonunit_model_rotation"
        })
    ));
    parameters.validate_mode(&model, false).unwrap();
    parameters.set(
        CamLightParameter::CamQuat,
        ParameterBatch::new(1, 20, [1.0, 0.0, 0.0, 0.0].repeat(5)).unwrap(),
    );
    parameters.validate(&model).unwrap();
    let mut nonfinite = model.fields().clone();
    nonfinite.cam_quat[16] = f32::NAN;
    assert!(matches!(
        CamLightModelInput::new_equivalent(base(), nonfinite),
        Err(InputError::NonFinite {
            field: "cam_quat",
            index: 16
        })
    ));
}
#[test]
fn rejects_topology_lengths_modes_and_unsafe_references() {
    for column in 0..6 {
        let mut f = fields();
        let v = match column {
            0 => &mut f.cam_mode,
            1 => &mut f.cam_targetbodyid,
            2 => &mut f.light_mode,
            3 => &mut f.light_targetbodyid,
            4 => &mut f.cam_bodyid,
            _ => &mut f.light_bodyid,
        };
        v.pop();
        assert!(CamLightModelInput::new(base(), f).is_err());
    }
    for column in 0..6 {
        for bad in [-1, 5, i32::MAX] {
            let mut f = fields();
            let v = match column {
                0 => &mut f.cam_mode,
                1 => &mut f.light_mode,
                2 => &mut f.cam_bodyid,
                3 => &mut f.light_bodyid,
                4 => &mut f.cam_targetbodyid,
                _ => &mut f.light_targetbodyid,
            };
            v[0] = bad;
            assert_eq!(
                CamLightModelInput::new(base(), f).is_err(),
                column < 4 || bad >= 0
            );
        }
    }
}
