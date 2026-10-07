//! 只读运动学字段子集。
//! 不表示完整Model或ModelInput。

use super::BatchLayout;
use crate::diagnostics::InputError;

/// 保留原生四种关节值。
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JointType {
    Free = 0,
    Ball = 1,
    Slide = 2,
    Hinge = 3,
}

impl TryFrom<i32> for JointType {
    type Error = InputError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Free),
            1 => Ok(Self::Ball),
            2 => Ok(Self::Slide),
            3 => Ok(Self::Hinge),
            _ => Err(InputError::InvalidDimension { field: "jnt_type" }),
        }
    }
}

impl JointType {
    pub const fn qpos_width(self) -> usize {
        match self {
            Self::Free => 7,
            Self::Ball => 4,
            Self::Slide | Self::Hinge => 1,
        }
    }
    pub const fn dof_width(self) -> usize {
        match self {
            Self::Free => 6,
            Self::Ball => 3,
            Self::Slide | Self::Hinge => 1,
        }
    }
}

/// 连续字段的未检查输入。
/// vec3与四元数按标量展开。
/// 参数批量长度固定为一。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct KinematicFields {
    pub qpos0: Vec<f32>,
    pub body_parentid: Vec<i32>,
    pub body_jntadr: Vec<i32>,
    pub body_jntnum: Vec<i32>,
    /// 父体坐标系内位置，单位米。
    pub body_pos: Vec<f32>,
    /// 父体坐标系内姿态，顺序wxyz。
    pub body_quat: Vec<f32>,
    pub jnt_type: Vec<i32>,
    pub jnt_bodyid: Vec<i32>,
    pub jnt_qposadr: Vec<i32>,
    pub jnt_dofadr: Vec<i32>,
    /// 体坐标系内锚点，单位米。
    pub jnt_pos: Vec<f32>,
    /// 体坐标系内轴向量。
    pub jnt_axis: Vec<f32>,
}

/// 拥有已检查的运动学字段。
/// 验证后仅提供只读借用。
/// 不验证质量、资产或物理组合。
///
/// ```compile_fail
/// use mjwarp_rs::model::{KinematicFields, KinematicModelInput};
/// let model = KinematicModelInput::new(0, KinematicFields::default()).unwrap();
/// model.fields().body_parentid[0] = 1;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct KinematicModelInput {
    nv: usize,
    fields: KinematicFields,
}

impl KinematicModelInput {
    pub fn new(nv: usize, fields: KinematicFields) -> Result<Self, InputError> {
        validate_fields(nv, &fields)?;
        Ok(Self { nv, fields })
    }
    pub fn nq(&self) -> usize {
        self.fields.qpos0.len()
    }
    pub fn nv(&self) -> usize {
        self.nv
    }
    pub fn nbody(&self) -> usize {
        self.fields.body_parentid.len()
    }
    pub fn njnt(&self) -> usize {
        self.fields.jnt_type.len()
    }
    pub fn fields(&self) -> &KinematicFields {
        &self.fields
    }
}

fn field_len(count: usize, width: usize) -> Result<usize, InputError> {
    let elements = count.checked_mul(width).ok_or(InputError::Overflow {
        field: "kinematic_field",
    })?;
    BatchLayout::new(1, elements, 4)?;
    Ok(elements)
}

fn error(field: &'static str, index: usize, reason: &'static str) -> InputError {
    InputError::InvalidTopology {
        field,
        index,
        reason,
    }
}

fn validate_fields(nv: usize, f: &KinematicFields) -> Result<(), InputError> {
    let nq = f.qpos0.len();
    let nb = f.body_parentid.len();
    let nj = f.jnt_type.len();
    if nb == 0 {
        return Err(InputError::InvalidDimension { field: "nbody" });
    }
    for (field, count) in [("nq", nq), ("nv", nv), ("nbody", nb), ("njnt", nj)] {
        if count > i32::MAX as usize {
            return Err(InputError::Overflow { field });
        }
    }
    for (field, actual, count, width) in [
        ("qpos0", nq, nq, 1),
        ("body_parentid", nb, nb, 1),
        ("body_jntadr", f.body_jntadr.len(), nb, 1),
        ("body_jntnum", f.body_jntnum.len(), nb, 1),
        ("body_pos", f.body_pos.len(), nb, 3),
        ("body_quat", f.body_quat.len(), nb, 4),
        ("jnt_bodyid", f.jnt_bodyid.len(), nj, 1),
        ("jnt_qposadr", f.jnt_qposadr.len(), nj, 1),
        ("jnt_dofadr", f.jnt_dofadr.len(), nj, 1),
        ("jnt_pos", f.jnt_pos.len(), nj, 3),
        ("jnt_axis", f.jnt_axis.len(), nj, 3),
    ] {
        let expected = field_len(count, width)?;
        if expected != actual {
            return Err(InputError::LengthMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    field_len(nv, 1)?;
    for (field, values) in [
        ("qpos0", &f.qpos0),
        ("body_pos", &f.body_pos),
        ("body_quat", &f.body_quat),
        ("jnt_pos", &f.jnt_pos),
        ("jnt_axis", &f.jnt_axis),
    ] {
        for (index, value) in values.iter().enumerate() {
            if !value.is_finite() {
                return Err(InputError::NonFinite { field, index });
            }
        }
    }
    // Preserve finite quaternions and axes; do not normalize or repair inputs.
    let mut joint_cursor = 0;
    for body in 0..nb {
        let parent = f.body_parentid[body];
        if (body == 0 && parent != 0) || (body > 0 && (parent < 0 || parent as usize >= body)) {
            return Err(error("body_parentid", body, "parent_must_precede_child"));
        }
        let count = usize::try_from(f.body_jntnum[body])
            .map_err(|_| error("body_jntnum", body, "negative_count"))?;
        if body == 0 && count != 0 {
            return Err(error("body_jntnum", body, "world_has_joint"));
        }
        if count == 0 {
            if f.body_jntadr[body] != -1 {
                return Err(error("body_jntadr", body, "empty_requires_minus_one"));
            }
            continue;
        }
        if f.body_jntadr[body] < 0 || f.body_jntadr[body] as usize != joint_cursor {
            return Err(error("body_jntadr", body, "non_contiguous_joint_range"));
        }
        let end = joint_cursor
            .checked_add(count)
            .filter(|end| *end <= nj)
            .ok_or_else(|| error("body_jntnum", body, "joint_range_exceeds_capacity"))?;
        for joint in joint_cursor..end {
            if f.jnt_bodyid[joint] < 0 || f.jnt_bodyid[joint] as usize != body {
                return Err(error("jnt_bodyid", joint, "joint_owner_mismatch"));
            }
            if f.jnt_type[joint] == JointType::Free as i32 && (count != 1 || parent != 0) {
                return Err(error("jnt_type", joint, "free_requires_single_root_joint"));
            }
        }
        joint_cursor = end;
    }
    if joint_cursor != nj {
        return Err(error("body_jntnum", nb - 1, "unassigned_joint"));
    }
    let mut q = 0_usize;
    let mut v = 0_usize;
    for joint in 0..nj {
        let kind = JointType::try_from(f.jnt_type[joint])
            .map_err(|_| error("jnt_type", joint, "unknown_joint_type"))?;
        for (field, actual, expected) in [
            ("jnt_qposadr", f.jnt_qposadr[joint], q),
            ("jnt_dofadr", f.jnt_dofadr[joint], v),
        ] {
            if actual < 0 || actual as usize != expected {
                return Err(error(field, joint, "non_contiguous_coordinate_range"));
            }
        }
        q = q
            .checked_add(kind.qpos_width())
            .ok_or(InputError::Overflow { field: "nq" })?;
        v = v
            .checked_add(kind.dof_width())
            .ok_or(InputError::Overflow { field: "nv" })?;
    }
    for (field, expected, actual) in [("qpos0", q, nq), ("nv", v, nv)] {
        if expected != actual {
            return Err(InputError::LengthMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn mixed() -> KinematicFields {
        KinematicFields {
            qpos0: vec![0.0; 13],
            body_parentid: vec![0, 0, 1, 2, 3, 0],
            body_jntadr: vec![-1, 0, 1, 2, 3, -1],
            body_jntnum: vec![0, 1, 1, 1, 1, 0],
            body_pos: vec![0.0; 18],
            body_quat: [1.0, 0.0, 0.0, 0.0].repeat(6),
            jnt_type: vec![0, 1, 2, 3],
            jnt_bodyid: vec![1, 2, 3, 4],
            jnt_qposadr: vec![0, 7, 11, 12],
            jnt_dofadr: vec![0, 6, 9, 10],
            jnt_pos: vec![0.0; 12],
            jnt_axis: vec![0.0; 12],
        }
    }
    #[test]
    fn preserves_joint_values_and_variable_coordinate_widths() {
        for (raw, q, v) in [(0, 7, 6), (1, 4, 3), (2, 1, 1), (3, 1, 1)] {
            let kind = JointType::try_from(raw).unwrap();
            assert_eq!(kind as i32, raw);
            assert_eq!((kind.qpos_width(), kind.dof_width()), (q, v));
        }
        for raw in [-1, 4, i32::MAX] {
            assert!(JointType::try_from(raw).is_err());
        }
    }
    #[test]
    fn validates_mixed_joints_and_static_body() {
        let f = mixed();
        let m = KinematicModelInput::new(11, f.clone()).unwrap();
        assert_eq!((m.nq(), m.nv(), m.nbody(), m.njnt()), (13, 11, 6, 4));
        assert_eq!(m.fields(), &f);
        assert_eq!(m.clone(), m);
    }
    #[test]
    fn accepts_zero_dof_and_preserves_finite_nonunit_bits() {
        let f = KinematicFields {
            body_parentid: vec![0],
            body_jntadr: vec![-1],
            body_jntnum: vec![0],
            body_pos: vec![-0.0, 0.0, 0.0],
            body_quat: vec![2.0, 0.0, 0.0, 0.0],
            ..Default::default()
        };
        let m = KinematicModelInput::new(0, f).unwrap();
        assert_eq!(m.fields().body_pos[0].to_bits(), (-0.0_f32).to_bits());
        assert_eq!(m.fields().body_quat[0], 2.0);
    }
    #[test]
    fn rejects_missing_world_and_capacity_overflow() {
        assert!(KinematicModelInput::new(0, KinematicFields::default()).is_err());
        assert!(KinematicModelInput::new(i32::MAX as usize + 1, mixed()).is_err());
        assert!(field_len(usize::MAX, 4).is_err());
        assert!(field_len(isize::MAX as usize, 1).is_err());
    }
    #[test]
    fn rejects_all_field_length_mismatches() {
        for case in 0..10 {
            let mut f = mixed();
            match case {
                0 => {
                    f.body_jntadr.pop();
                }
                1 => {
                    f.body_jntnum.pop();
                }
                2 => {
                    f.body_pos.pop();
                }
                3 => {
                    f.body_quat.pop();
                }
                4 => {
                    f.jnt_bodyid.pop();
                }
                5 => {
                    f.jnt_qposadr.pop();
                }
                6 => {
                    f.jnt_dofadr.pop();
                }
                7 => {
                    f.jnt_pos.pop();
                }
                8 => {
                    f.jnt_axis.pop();
                }
                _ => {
                    f.qpos0.pop();
                }
            }
            assert!(matches!(
                KinematicModelInput::new(11, f),
                Err(InputError::LengthMismatch { .. })
            ));
        }
        assert!(matches!(
            KinematicModelInput::new(10, mixed()),
            Err(InputError::LengthMismatch { field: "nv", .. })
        ));
    }
    #[test]
    fn rejects_each_nonfinite_field_with_its_index() {
        for case in 0..5 {
            let mut f = mixed();
            let (name, values) = match case {
                0 => ("qpos0", &mut f.qpos0),
                1 => ("body_pos", &mut f.body_pos),
                2 => ("body_quat", &mut f.body_quat),
                3 => ("jnt_pos", &mut f.jnt_pos),
                _ => ("jnt_axis", &mut f.jnt_axis),
            };
            values[1] = f32::NAN;
            assert_eq!(
                KinematicModelInput::new(11, f),
                Err(InputError::NonFinite {
                    field: name,
                    index: 1
                })
            );
        }
    }
    #[test]
    fn rejects_parent_cycles_forward_edges_and_negative_indices() {
        for (body, parent) in [(0, 1), (1, 1), (2, 3), (2, -1), (2, 6)] {
            let mut f = mixed();
            f.body_parentid[body] = parent;
            assert!(matches!(
                KinematicModelInput::new(11, f),
                Err(InputError::InvalidTopology {
                    field: "body_parentid",
                    ..
                })
            ));
        }
    }
    #[test]
    fn rejects_joint_partition_and_owner_errors() {
        for case in 0..8 {
            let mut f = mixed();
            match case {
                0 => f.body_jntadr[0] = 0,
                1 => f.body_jntnum[1] = -1,
                2 => f.body_jntnum[0] = 1,
                3 => f.body_jntnum[4] = 2,
                4 => f.body_jntadr[2] = 0,
                5 => f.jnt_bodyid[2] = 1,
                6 => f.body_jntnum[4] = 0,
                _ => f.body_jntadr[1] = -1,
            }
            assert!(matches!(
                KinematicModelInput::new(11, f),
                Err(InputError::InvalidTopology { .. })
            ));
        }
    }
    #[test]
    fn rejects_joint_types_address_gaps_and_width_mismatch() {
        for case in 0..6 {
            let mut f = mixed();
            match case {
                0 => f.jnt_type[1] = 4,
                1 => f.jnt_qposadr[1] = 6,
                2 => f.jnt_dofadr[1] = 7,
                3 => f.jnt_dofadr[0] = -1,
                4 => f.jnt_qposadr[0] = -1,
                _ => f.jnt_type[1] = 0,
            }
            assert!(matches!(
                KinematicModelInput::new(11, f),
                Err(InputError::InvalidTopology { .. })
            ));
        }
        let mut f = mixed();
        f.jnt_type[3] = 1;
        assert!(matches!(
            KinematicModelInput::new(11, f),
            Err(InputError::LengthMismatch { .. })
        ));
    }
}
