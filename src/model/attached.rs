//! 几何与site的局部附着字段。
//! 不表示完整模型或原生ABI。

use super::InertialModelInput;
use crate::diagnostics::InputError;

/// 六项连续字段的未检查输入。
/// 参数批量长度固定为一。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AttachedFields {
    pub geom_bodyid: Vec<i32>,
    /// 体坐标系内位置，单位米。
    pub geom_pos: Vec<f32>,
    /// 体坐标系内姿态，顺序wxyz。
    pub geom_quat: Vec<f32>,
    pub site_bodyid: Vec<i32>,
    /// 体坐标系内位置，单位米。
    pub site_pos: Vec<f32>,
    /// 体坐标系内姿态，顺序wxyz。
    pub site_quat: Vec<f32>,
}

/// 拥有刚体子集与六项附着字段。
/// 严格构造要求单位四元数。
/// 等价构造仅检查有限性。
/// 验证后仅提供只读借用。
///
/// ```compile_fail
/// use mjwarp_rs::model::AttachedModelInput;
/// fn mutate(model: &mut AttachedModelInput) {
///     model.fields().geom_bodyid[0] = -1;
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct AttachedModelInput {
    rigid: InertialModelInput,
    fields: AttachedFields,
}

impl AttachedModelInput {
    pub fn new(rigid: InertialModelInput, fields: AttachedFields) -> Result<Self, InputError> {
        Self::checked(rigid, fields, true)
    }

    /// 检查布局而不要求单位姿态。
    /// G01沿用上游矩阵计算语义。
    pub fn new_equivalent(
        rigid: InertialModelInput,
        fields: AttachedFields,
    ) -> Result<Self, InputError> {
        Self::checked(rigid, fields, false)
    }

    fn checked(
        rigid: InertialModelInput,
        fields: AttachedFields,
        strict: bool,
    ) -> Result<Self, InputError> {
        for (ids, pos, quat, names) in [
            (
                &fields.geom_bodyid,
                &fields.geom_pos,
                &fields.geom_quat,
                ["geom_bodyid", "geom_pos", "geom_quat"],
            ),
            (
                &fields.site_bodyid,
                &fields.site_pos,
                &fields.site_quat,
                ["site_bodyid", "site_pos", "site_quat"],
            ),
        ] {
            check_count(ids.len())?;
            for (values, width, field) in [(pos, 3, names[1]), (quat, 4, names[2])] {
                let expected = ids.len() * width;
                if values.len() != expected {
                    return Err(InputError::LengthMismatch {
                        field,
                        expected,
                        actual: values.len(),
                    });
                }
                for (index, value) in values.iter().enumerate() {
                    if !value.is_finite() {
                        return Err(InputError::NonFinite { field, index });
                    }
                }
            }
            for (index, &body) in ids.iter().enumerate() {
                if body < 0 || body as usize >= rigid.kinematics().nbody() {
                    return Err(InputError::InvalidTopology {
                        field: names[0],
                        index,
                        reason: "body_reference_out_of_range",
                    });
                }
            }
            for (index, rotation) in quat.as_chunks::<4>().0.iter().enumerate() {
                let squared: f64 = rotation.iter().map(|&x| f64::from(x).powi(2)).sum();
                if strict && (squared - 1.0).abs() > 2e-6 {
                    return Err(InputError::InvalidTopology {
                        field: names[2],
                        index,
                        reason: "nonunit_model_rotation",
                    });
                }
            }
        }
        Ok(Self { rigid, fields })
    }

    pub fn rigid(&self) -> &InertialModelInput {
        &self.rigid
    }

    pub fn fields(&self) -> &AttachedFields {
        &self.fields
    }

    pub fn ngeom(&self) -> usize {
        self.fields.geom_bodyid.len()
    }

    pub fn nsite(&self) -> usize {
        self.fields.site_bodyid.len()
    }
}

fn check_count(count: usize) -> Result<(), InputError> {
    if count > i32::MAX as usize / 7 {
        return Err(InputError::Overflow {
            field: "attached_fields",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{InertialFields, KinematicFields, KinematicModelInput};

    fn rigid() -> InertialModelInput {
        InertialModelInput::new(
            KinematicModelInput::new(
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
            .unwrap(),
            InertialFields {
                body_ipos: vec![0.0; 3],
                body_iquat: vec![1.0, 0.0, 0.0, 0.0],
                body_mass: vec![0.0],
                body_inertia: vec![0.0; 3],
                ..Default::default()
            },
        )
        .unwrap()
    }

    fn fields() -> AttachedFields {
        AttachedFields {
            geom_bodyid: vec![0],
            geom_pos: vec![-0.0, 1.0, -2.0],
            geom_quat: vec![-1.0, -0.0, 0.0, 0.0],
            site_bodyid: vec![0, 0],
            site_pos: vec![0.0; 6],
            site_quat: vec![0.5; 8],
        }
    }

    #[test]
    fn accepts_world_attachments_and_preserves_input_bits() {
        let f = fields();
        let m = AttachedModelInput::new(rigid(), f.clone()).unwrap();
        assert_eq!((m.ngeom(), m.nsite()), (1, 2));
        assert_eq!(m.rigid(), &rigid());
        assert_eq!(m.fields(), &f);
        assert_eq!(m.fields().geom_pos[0].to_bits(), (-0.0f32).to_bits());
        assert_eq!(m.fields().geom_quat[1].to_bits(), (-0.0f32).to_bits());
        assert_eq!(m.clone(), m);
    }

    #[test]
    fn accepts_empty_and_single_kind_attachments() {
        for (ng, ns) in [(0, 0), (0, 2), (1, 0)] {
            let mut f = fields();
            f.geom_bodyid.truncate(ng);
            f.geom_pos.truncate(3 * ng);
            f.geom_quat.truncate(4 * ng);
            f.site_bodyid.truncate(ns);
            f.site_pos.truncate(3 * ns);
            f.site_quat.truncate(4 * ns);
            let m = AttachedModelInput::new(rigid(), f).unwrap();
            assert_eq!((m.ngeom(), m.nsite()), (ng, ns));
        }
    }

    #[test]
    fn rejects_all_float_length_mismatches() {
        for field in 0..4 {
            let mut f = fields();
            let values = match field {
                0 => &mut f.geom_pos,
                1 => &mut f.geom_quat,
                2 => &mut f.site_pos,
                _ => &mut f.site_quat,
            };
            values.pop();
            assert!(matches!(
                AttachedModelInput::new(rigid(), f),
                Err(InputError::LengthMismatch { .. })
            ));
        }
        let mut f = fields();
        f.geom_bodyid.clear();
        assert!(matches!(
            AttachedModelInput::new(rigid(), f),
            Err(InputError::LengthMismatch {
                field: "geom_pos",
                ..
            })
        ));
    }

    #[test]
    fn rejects_nonfinite_fields_and_invalid_body_references() {
        for field in 0..4 {
            for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
                let mut f = fields();
                let values = match field {
                    0 => &mut f.geom_pos,
                    1 => &mut f.geom_quat,
                    2 => &mut f.site_pos,
                    _ => &mut f.site_quat,
                };
                values[0] = bad;
                assert!(matches!(
                    AttachedModelInput::new(rigid(), f),
                    Err(InputError::NonFinite { index: 0, .. })
                ));
            }
        }
        for site in [false, true] {
            for bad in [-1, 1, i32::MAX] {
                let mut f = fields();
                if site {
                    f.site_bodyid[0] = bad;
                } else {
                    f.geom_bodyid[0] = bad;
                }
                assert!(matches!(
                    AttachedModelInput::new(rigid(), f),
                    Err(InputError::InvalidTopology {
                        index: 0,
                        reason: "body_reference_out_of_range",
                        ..
                    })
                ));
            }
        }
    }

    #[test]
    fn rejects_nonunit_rotations_and_excessive_capacity() {
        for site in [false, true] {
            for bad in [0.0, 0.5, 2.0, f32::MAX] {
                let mut f = fields();
                let q = if site {
                    &mut f.site_quat
                } else {
                    &mut f.geom_quat
                };
                q[..4].copy_from_slice(&[bad, 0.0, 0.0, 0.0]);
                assert!(matches!(
                    AttachedModelInput::new(rigid(), f),
                    Err(InputError::InvalidTopology {
                        reason: "nonunit_model_rotation",
                        ..
                    })
                ));
            }
        }
        assert!(check_count(i32::MAX as usize / 7).is_ok());
        assert!(check_count(i32::MAX as usize / 7 + 1).is_err());
        assert!(check_count(usize::MAX).is_err());
    }
}
