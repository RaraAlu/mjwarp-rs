//! 九项惯性字段的只读子集。
//! 不计算质量矩阵或动力学。

use super::{BatchLayout, KinematicModelInput, topology::JointType};
use crate::diagnostics::InputError;

/// 九项连续字段的未检查输入。
/// 参数批量长度固定为一。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InertialFields {
    /// 体坐标系内质心，单位米。
    pub body_ipos: Vec<f32>,
    /// 惯性系相对体姿态，顺序wxyz。
    pub body_iquat: Vec<f32>,
    /// 质量，单位千克。
    pub body_mass: Vec<f32>,
    /// 惯性系主惯量，单位kg*m^2。
    pub body_inertia: Vec<f32>,
    pub dof_bodyid: Vec<i32>,
    pub dof_jntid: Vec<i32>,
    /// 最近祖先自由度，无祖先用-1。
    pub dof_parentid: Vec<i32>,
    /// 平移用kg，转动用kg*m^2。
    pub dof_armature: Vec<f32>,
    /// 原生阻尼；不叠加执行器阻尼。
    /// 平移用kg/s，转动用kg*m^2/s。
    pub dof_damping: Vec<f32>,
}

/// 拥有二十一项已检查字段。
/// 本辅助接口拒绝负惯性参数。
/// 不校验惯量三角或物理组合。
/// 保留有限姿态，不执行归一化。
/// 不替代完整ModelInput。
///
/// ```compile_fail
/// use mjwarp_rs::model::InertialModelInput;
/// fn mutate(input: &mut InertialModelInput) {
///     input.fields().body_mass[0] = 1.0;
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct InertialModelInput {
    kinematics: KinematicModelInput,
    fields: InertialFields,
}

impl InertialModelInput {
    pub fn new(
        kinematics: KinematicModelInput,
        fields: InertialFields,
    ) -> Result<Self, InputError> {
        validate_fields(&kinematics, &fields)?;
        Ok(Self { kinematics, fields })
    }
    pub fn kinematics(&self) -> &KinematicModelInput {
        &self.kinematics
    }
    pub fn fields(&self) -> &InertialFields {
        &self.fields
    }
}

fn field_len(count: usize, width: usize) -> Result<usize, InputError> {
    let elements = count.checked_mul(width).ok_or(InputError::Overflow {
        field: "inertial_field",
    })?;
    BatchLayout::new(1, elements, 4)?;
    Ok(elements)
}

fn validate_fields(k: &KinematicModelInput, f: &InertialFields) -> Result<(), InputError> {
    let nb = k.nbody();
    let nv = k.nv();
    for (field, actual, count, width) in [
        ("body_ipos", f.body_ipos.len(), nb, 3),
        ("body_iquat", f.body_iquat.len(), nb, 4),
        ("body_mass", f.body_mass.len(), nb, 1),
        ("body_inertia", f.body_inertia.len(), nb, 3),
        ("dof_bodyid", f.dof_bodyid.len(), nv, 1),
        ("dof_jntid", f.dof_jntid.len(), nv, 1),
        ("dof_parentid", f.dof_parentid.len(), nv, 1),
        ("dof_armature", f.dof_armature.len(), nv, 1),
        ("dof_damping", f.dof_damping.len(), nv, 1),
    ] {
        let expected = field_len(count, width)?;
        if actual != expected {
            return Err(InputError::LengthMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    for (field, values, nonnegative) in [
        ("body_ipos", &f.body_ipos, false),
        ("body_iquat", &f.body_iquat, false),
        ("body_mass", &f.body_mass, true),
        ("body_inertia", &f.body_inertia, true),
        ("dof_armature", &f.dof_armature, true),
        ("dof_damping", &f.dof_damping, true),
    ] {
        for (index, &value) in values.iter().enumerate() {
            if !value.is_finite() {
                return Err(InputError::NonFinite { field, index });
            }
            if nonnegative && value < 0.0 {
                return Err(InputError::NegativeValue { field, index });
            }
        }
    }
    let t = k.fields();
    // KinematicModelInput already proves body/joint partitions and acyclic ancestry.
    // Static bodies inherit the nearest ancestor DOF, not the preceding global DOF.
    for joint in 0..k.njnt() {
        let body = t.jnt_bodyid[joint] as usize;
        let start = t.jnt_dofadr[joint] as usize;
        let width = JointType::try_from(t.jnt_type[joint])?.dof_width();
        let mut parent = if joint == t.body_jntadr[body] as usize {
            let mut ancestor = t.body_parentid[body] as usize;
            while ancestor != 0 && t.body_jntnum[ancestor] == 0 {
                ancestor = t.body_parentid[ancestor] as usize;
            }
            if t.body_jntnum[ancestor] == 0 {
                -1
            } else {
                let last = (t.body_jntadr[ancestor] + t.body_jntnum[ancestor] - 1) as usize;
                t.jnt_dofadr[last] + JointType::try_from(t.jnt_type[last])?.dof_width() as i32 - 1
            }
        } else {
            start as i32 - 1
        };
        for dof in start..start + width {
            for (field, actual, expected, reason) in [
                (
                    "dof_bodyid",
                    f.dof_bodyid[dof],
                    body as i32,
                    "dof_body_mismatch",
                ),
                (
                    "dof_jntid",
                    f.dof_jntid[dof],
                    joint as i32,
                    "dof_joint_mismatch",
                ),
                (
                    "dof_parentid",
                    f.dof_parentid[dof],
                    parent,
                    "dof_ancestor_mismatch",
                ),
            ] {
                if actual != expected {
                    return Err(InputError::InvalidTopology {
                        field,
                        index: dof,
                        reason,
                    });
                }
            }
            parent = dof as i32;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::KinematicFields;

    fn tree() -> (KinematicModelInput, InertialFields) {
        let k = KinematicModelInput::new(
            13,
            KinematicFields {
                qpos0: vec![0.0; 15],
                body_parentid: vec![0, 0, 1, 1, 3, 1, 0, 6],
                body_jntadr: vec![-1, 0, 1, -1, 2, 4, -1, 5],
                body_jntnum: vec![0, 1, 1, 0, 2, 1, 0, 1],
                body_pos: vec![0.0; 24],
                body_quat: [1.0, 0.0, 0.0, 0.0].repeat(8),
                jnt_type: vec![0, 1, 2, 3, 3, 2],
                jnt_bodyid: vec![1, 2, 4, 4, 5, 7],
                jnt_qposadr: vec![0, 7, 11, 12, 13, 14],
                jnt_dofadr: vec![0, 6, 9, 10, 11, 12],
                jnt_pos: vec![0.0; 18],
                jnt_axis: vec![0.0; 18],
            },
        )
        .unwrap();
        let f = InertialFields {
            body_ipos: vec![0.0; 24],
            body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(8),
            body_mass: vec![0.0; 8],
            body_inertia: vec![0.0; 24],
            dof_bodyid: vec![1, 1, 1, 1, 1, 1, 2, 2, 2, 4, 4, 5, 7],
            dof_jntid: vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 2, 3, 4, 5],
            dof_parentid: vec![-1, 0, 1, 2, 3, 4, 5, 6, 7, 5, 9, 5, -1],
            dof_armature: vec![0.0; 13],
            dof_damping: vec![0.0; 13],
        };
        (k, f)
    }
    fn floats(f: &mut InertialFields) -> [(&'static str, &mut Vec<f32>); 6] {
        [
            ("body_ipos", &mut f.body_ipos),
            ("body_iquat", &mut f.body_iquat),
            ("body_mass", &mut f.body_mass),
            ("body_inertia", &mut f.body_inertia),
            ("dof_armature", &mut f.dof_armature),
            ("dof_damping", &mut f.dof_damping),
        ]
    }
    #[test]
    fn validates_branches_static_ancestors_and_multi_dof_joints() {
        let (k, f) = tree();
        let m = InertialModelInput::new(k.clone(), f.clone()).unwrap();
        assert_eq!(m.kinematics(), &k);
        assert_eq!(m.fields(), &f);
        assert_eq!(m.clone(), m);
    }
    #[test]
    fn accepts_zero_dof_and_preserves_finite_nonunit_bits() {
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
        let f = InertialFields {
            body_ipos: vec![-0.0, -1.0, 0.0],
            body_iquat: vec![2.0, -1.0, 0.0, 0.0],
            body_mass: vec![-0.0],
            body_inertia: vec![-0.0; 3],
            ..Default::default()
        };
        let m = InertialModelInput::new(k, f).unwrap();
        assert_eq!(m.fields().body_ipos[0].to_bits(), (-0.0_f32).to_bits());
        assert_eq!(m.fields().body_mass[0].to_bits(), (-0.0_f32).to_bits());
        assert_eq!(m.fields().body_iquat, [2.0, -1.0, 0.0, 0.0]);
    }
    #[test]
    fn rejects_all_float_length_mismatches() {
        let (k, base) = tree();
        for case in 0..6 {
            let mut f = base.clone();
            let (field, values) = &mut floats(&mut f)[case];
            let expected = values.len();
            values.pop();
            let field = *field;
            assert_eq!(
                InertialModelInput::new(k.clone(), f),
                Err(InputError::LengthMismatch {
                    field,
                    expected,
                    actual: expected - 1
                })
            );
        }
    }
    #[test]
    fn rejects_all_index_length_mismatches() {
        let (k, base) = tree();
        for case in 0..3 {
            let mut f = base.clone();
            let (field, values) = match case {
                0 => ("dof_bodyid", &mut f.dof_bodyid),
                1 => ("dof_jntid", &mut f.dof_jntid),
                _ => ("dof_parentid", &mut f.dof_parentid),
            };
            values.pop();
            assert_eq!(
                InertialModelInput::new(k.clone(), f),
                Err(InputError::LengthMismatch {
                    field,
                    expected: 13,
                    actual: 12
                })
            );
        }
    }
    #[test]
    fn rejects_all_nonfinite_float_fields() {
        let (k, base) = tree();
        for case in 0..6 {
            for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
                let mut f = base.clone();
                let (field, values) = &mut floats(&mut f)[case];
                values[1] = value;
                let field = *field;
                assert_eq!(
                    InertialModelInput::new(k.clone(), f),
                    Err(InputError::NonFinite { field, index: 1 })
                );
            }
        }
    }
    #[test]
    fn rejects_negative_mass_inertia_armature_and_damping() {
        let (k, base) = tree();
        for case in 2..6 {
            let mut f = base.clone();
            let (field, values) = &mut floats(&mut f)[case];
            values[1] = -0.25;
            let field = *field;
            assert_eq!(
                InertialModelInput::new(k.clone(), f),
                Err(InputError::NegativeValue { field, index: 1 })
            );
        }
    }
    #[test]
    fn rejects_wrong_dof_owners_and_nonancestor_parents() {
        let (k, base) = tree();
        for (field, index, wrong) in [
            ("dof_bodyid", 6, -1),
            ("dof_bodyid", 6, 8),
            ("dof_bodyid", 6, 1),
            ("dof_jntid", 6, -1),
            ("dof_jntid", 6, 6),
            ("dof_jntid", 6, 0),
            ("dof_parentid", 0, -2),
            ("dof_parentid", 0, 0),
            ("dof_parentid", 6, -1),
            ("dof_parentid", 6, 7),
            ("dof_parentid", 9, 8),
            ("dof_parentid", 10, 5),
            ("dof_parentid", 11, 10),
            ("dof_parentid", 12, 11),
        ] {
            let mut f = base.clone();
            match field {
                "dof_bodyid" => f.dof_bodyid[index] = wrong,
                "dof_jntid" => f.dof_jntid[index] = wrong,
                _ => f.dof_parentid[index] = wrong,
            }
            assert!(
                matches!(InertialModelInput::new(k.clone(), f), Err(InputError::InvalidTopology { field: name, index: at, .. }) if name == field && at == index)
            );
        }
    }
    #[test]
    fn rejects_expanded_field_capacity_overflow() {
        assert!(field_len(usize::MAX, 4).is_err());
        assert!(field_len(isize::MAX as usize, 1).is_err());
        assert_eq!(field_len(0, 4).unwrap(), 0);
    }
}
