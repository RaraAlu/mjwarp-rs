//! 流体系数独立周期与椭球输入测试。

use super::cuda::CUDA;
use super::spec::spec;
use super::tests_support::*;
use super::*;
use crate::model::KinematicsParameter;

#[test]
fn host_preserves_independent_periods_and_ellipsoid_inputs() {
    let simple_parameters = simple_kinematics_parameters();
    let simple = spec(
        Arc::new(simple_model()),
        simple_parameters
            .get(KinematicsParameter::BodyMass)
            .unwrap()
            .clone(),
        simple_parameters
            .get(KinematicsParameter::BodyInertia)
            .unwrap()
            .clone(),
        simple_fluid(),
    )
    .unwrap();
    assert_eq!(simple.offsets, [0, 6, 9, 14, 35, 134]);
    assert_eq!(simple.parameter_len, 134);
    assert_eq!(&simple.metadata[4..7], &[0, 0, 1]);
    assert_eq!(&simple.metadata[7..10], &[0, 1, 1]);
    assert_eq!(&simple.metadata[10..11], &[1]);

    let model = Arc::new(model());
    let parameters = kinematics_parameters();
    let mass = parameters
        .get(KinematicsParameter::BodyMass)
        .unwrap()
        .clone();
    let inertia = parameters
        .get(KinematicsParameter::BodyInertia)
        .unwrap()
        .clone();
    let value = spec(model.clone(), mass.clone(), inertia.clone(), fluid()).unwrap();
    assert_eq!(value.offsets, [0, 6, 9, 14, 49, 214]);
    assert_eq!(value.parameter_len, 334);
    assert_eq!(&value.metadata[4..9], &[0, 0, 1, 2, 1]);
    assert_eq!(&value.metadata[9..14], &[0, 1, 1, 1, 1]);
    assert_eq!(&value.metadata[14..16], &[1, 2]);
    assert_eq!(
        [
            value.input.wind.batches(),
            value.input.density.batches(),
            value.input.viscosity.batches(),
            value.body_mass.batches(),
            value.body_inertia.batches(),
            value.input.geom_size.batches(),
        ],
        [2, 3, 5, 7, 11, 4]
    );
    let mut ellipsoid = ellipsoid_fluid();
    let accepted = spec(
        model.clone(),
        mass.clone(),
        inertia.clone(),
        ellipsoid.clone(),
    )
    .unwrap();
    assert_eq!(
        accepted.input.body_fluid_ellipsoid,
        [false, true, true, false, true]
    );
    ellipsoid.geom_fluid[0] = f32::NAN;
    assert!(matches!(
        spec(model.clone(), mass.clone(), inertia.clone(), ellipsoid),
        Err(InputError::NonFinite {
            field: "geom_fluid",
            index: 0
        })
    ));
    let mut invalid = fluid();
    invalid.geom_type.push(4);
    assert!(matches!(
        spec(model, mass, inertia, invalid),
        Err(InputError::LengthMismatch {
            field: "geom_type",
            ..
        })
    ));
    assert!(layout(13, 5, 2).is_ok());
    assert!(layout(0, 5, 2).is_err());
    assert!(layout(1, usize::MAX, 1).is_err());
}

#[test]
fn viscosity_torque_keeps_frozen_left_associative_rounding() {
    let scale = 6.0f32 / 6.0;
    let box0 = ((4.0f32 + 3.0 - 5.0).max(1.0e-15) * scale).sqrt();
    let box1 = ((5.0f32 + 3.0 - 4.0).max(1.0e-15) * scale).sqrt();
    let box2 = ((5.0f32 + 4.0 - 3.0).max(1.0e-15) * scale).sqrt();
    let diameter = (box0 + box1 + box2) / 3.0;
    let diameter_pow3 = diameter.powf(3.0);
    let left_associative = ((-0.2f32 * diameter_pow3) * std::f32::consts::PI) * 0.375;
    let regrouped = -0.2f32 * ((diameter_pow3 * std::f32::consts::PI) * 0.375);
    assert_eq!(left_associative.to_bits(), 0xbfe1_33de);
    assert_eq!(regrouped.to_bits(), 0xbfe1_33df);
}

// 四项字面量是冻结源码的逐位十进制展开。
// 只豁免本测试，禁止缩短或替换。
#[allow(clippy::excessive_precision)]
#[test]
fn ellipsoid_constants_and_projection_order_match_frozen_source() {
    assert_eq!(4.18879032135009765625f32.to_bits(), 0x4086_0a92);
    assert_eq!(1.6755161285400390625f32.to_bits(), 0x3fd6_7750);
    assert_eq!(0.666666686534881591796875f32.to_bits(), 0x3f2a_aaab);
    assert_eq!(9.424777984619140625f32.to_bits(), 0x4116_cbe4);
    for needle in [
        "float sq=x*x;return sq*sq",
        "float unit0_sq=unit[0]*unit[0]",
        "float proj_num=s12_sq*unit0_sq+s20_sq*unit1_sq+s01_sq*unit2_sq",
        "if(coef<=0.0f)continue",
        "if(has_density&&fluid[4]!=0.0f&&lin_speed>FF_MINVAL)",
        "force_global[i]+=world_force[i];torque_global[i]+=world_torque[i]",
    ] {
        assert!(CUDA.contains(needle), "missing frozen expression: {needle}");
    }
    assert!(!CUDA.contains("powf(dmax,4.0f)"));
}
