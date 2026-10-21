//! 输入检查。

use super::*;

#[cfg(test)]
pub(super) fn check_mocap(
    np: usize,
    nq: usize,
    pos: &[f32],
    quat: &[f32],
) -> Result<(), InputError> {
    check_mocap_mode(np, nq, pos, quat, true)
}

pub(super) fn check_size(
    field: &'static str,
    expected: usize,
    actual: usize,
) -> Result<(), InputError> {
    if actual == expected {
        Ok(())
    } else {
        Err(InputError::LengthMismatch {
            field,
            expected,
            actual,
        })
    }
}
pub(super) fn check_finite_field(
    field: &'static str,
    expected: usize,
    values: &[f32],
) -> Result<(), InputError> {
    check_size(field, expected, values.len())?;
    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(InputError::NonFinite { field, index });
        }
    }
    Ok(())
}
pub(super) fn check_g01_qpos(
    model: &crate::model::InertialModelInput,
    worlds: usize,
    qpos: &[f32],
) -> Result<KinematicsLayout, InputError> {
    let k = model.kinematics();
    let l = KinematicsLayout::new(worlds, k.nq(), k.nbody(), k.njnt())?;
    let expected = worlds.checked_mul(k.nq()).ok_or(InputError::Overflow {
        field: "kinematics_qpos",
    })?;
    check_finite_field("kinematics_qpos", expected, qpos)?;
    Ok(l)
}
pub(super) fn check_mocap_mode(
    np: usize,
    nq: usize,
    pos: &[f32],
    quat: &[f32],
    strict: bool,
) -> Result<(), InputError> {
    for (field, expected, values) in [("mocap_pos", np, pos), ("mocap_quat", nq, quat)] {
        if values.len() != expected {
            return Err(InputError::LengthMismatch {
                field,
                expected,
                actual: values.len(),
            });
        }
        for (index, v) in values.iter().enumerate() {
            if !v.is_finite() {
                return Err(InputError::NonFinite { field, index });
            }
        }
    }
    for (index, rotation) in quat.as_chunks::<4>().0.iter().enumerate() {
        let squared: f64 = rotation.iter().map(|&x| f64::from(x).powi(2)).sum();
        if strict && !(1e-12..=1e12).contains(&squared) {
            return Err(InputError::InvalidTopology {
                field: "mocap_quat",
                index: 4 * index,
                reason: "unusable_state_quaternion",
            });
        }
    }
    Ok(())
}

pub(super) fn check_parameters(
    model: &AttachedModelInput,
    parameters: &KinematicsParameters,
) -> Result<(), InputError> {
    use KinematicsParameter as F;
    parameters.validate(model)?;
    let k = model.rigid().kinematics();
    for field in F::ALL {
        let width = field.shared(model).len();
        if width == 0 {
            continue;
        }
        for (row, values) in parameters
            .values(field, model)
            .chunks_exact(width)
            .enumerate()
        {
            let fail = |index, reason| InputError::InvalidTopology {
                field: field.name(),
                index,
                reason,
            };
            let canonical: Option<&[f32]> = match field {
                F::BodyPos | F::BodyIpos => Some(&[0.0; 3]),
                F::BodyQuat | F::BodyIquat => Some(&[1.0, 0.0, 0.0, 0.0]),
                F::BodyMass => Some(&[0.0]),
                F::BodyInertia => Some(&[0.0; 3]),
                _ => None,
            };
            if let Some(required) = canonical
                && &values[..required.len()] != required
            {
                let reason = if matches!(field, F::BodyMass | F::BodyInertia) {
                    "nonzero_world_inertia"
                } else {
                    "noncanonical_world"
                };
                return Err(fail(row * width, reason));
            }
            match field {
                F::BodyQuat | F::BodyIquat | F::GeomQuat | F::SiteQuat => {
                    for (index, q) in values.as_chunks::<4>().0.iter().enumerate() {
                        if (crate::physics::norm_squared(q) - 1.0).abs() > 2e-6 {
                            return Err(fail(row * width / 4 + index, "nonunit_model_rotation"));
                        }
                    }
                }
                F::BodyMass | F::BodyInertia => {
                    for (index, &v) in values.iter().enumerate() {
                        if v < 0.0 {
                            return Err(InputError::NegativeValue {
                                field: field.name(),
                                index: row * width + index,
                            });
                        }
                    }
                }
                F::JointAxis => {
                    for (j, axis) in values.as_chunks::<3>().0.iter().enumerate() {
                        if k.fields().jnt_type[j] >= 2
                            && (crate::physics::norm_squared(axis) - 1.0).abs() > 2e-6
                        {
                            return Err(fail(row * k.njnt() + j, "nonunit_joint_axis"));
                        }
                    }
                }
                F::Qpos0 => {
                    for j in 0..k.njnt() {
                        let ty = k.fields().jnt_type[j];
                        if ty <= 1 {
                            let start =
                                k.fields().jnt_qposadr[j] as usize + if ty == 0 { 3 } else { 0 };
                            if !(1e-12..=1e12)
                                .contains(&crate::physics::norm_squared(&values[start..start + 4]))
                            {
                                return Err(fail(row * width + start, "unusable_state_quaternion"));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}
