//! 共享mocap映射与静态几何检查。
//! 不表示完整原生模型。

use super::AttachedModelInput;
use crate::diagnostics::InputError;

/// 拥有附着子集与mocap映射。
/// mocap体必须直接连接世界。
/// mocap体自身不允许关节。
/// 编号允许任意连续排列。
#[derive(Clone, Debug, PartialEq)]
pub struct MocapModelInput {
    attached: AttachedModelInput,
    body_mocapid: Vec<i32>,
    nmocap: usize,
    static_geom: Vec<bool>,
}

impl MocapModelInput {
    pub fn new(attached: AttachedModelInput, body_mocapid: Vec<i32>) -> Result<Self, InputError> {
        let k = attached.rigid().kinematics();
        if body_mocapid.len() != k.nbody() {
            return Err(InputError::LengthMismatch {
                field: "body_mocapid",
                expected: k.nbody(),
                actual: body_mocapid.len(),
            });
        }
        let nmocap = body_mocapid.iter().filter(|&&id| id >= 0).count();
        let mut seen = vec![false; nmocap];
        let mut static_body = vec![true; k.nbody()];
        let f = k.fields();
        for (body, &id) in body_mocapid.iter().enumerate() {
            let invalid = |reason| InputError::InvalidTopology {
                field: "body_mocapid",
                index: body,
                reason,
            };
            if id < -1 || (id >= 0 && id as usize >= nmocap) {
                return Err(invalid("mocap_reference_out_of_range"));
            }
            if id >= 0 {
                if body == 0 || f.body_parentid[body] != 0 || f.body_jntnum[body] != 0 {
                    return Err(invalid("mocap_body_requires_fixed_world_parent"));
                }
                if std::mem::replace(&mut seen[id as usize], true) {
                    return Err(invalid("duplicate_mocap_reference"));
                }
            }
            if body != 0 {
                static_body[body] = static_body[f.body_parentid[body] as usize]
                    && f.body_jntnum[body] == 0
                    && id == -1;
            }
        }
        // Parent-before-child validation makes this equivalent to the frozen
        // world-weld predicate, including its mocap-root exception.
        let static_geom = attached
            .fields()
            .geom_bodyid
            .iter()
            .map(|&body| static_body[body as usize])
            .collect();
        Ok(Self {
            attached,
            body_mocapid,
            nmocap,
            static_geom,
        })
    }

    pub fn attached(&self) -> &AttachedModelInput {
        &self.attached
    }
    pub fn body_mocapid(&self) -> &[i32] {
        &self.body_mocapid
    }
    pub fn nmocap(&self) -> usize {
        self.nmocap
    }
    /// true表示世界焊接静态几何。
    /// mocap后代始终返回false。
    pub fn static_geom(&self) -> &[bool] {
        &self.static_geom
    }

    pub(crate) fn into_parts(self) -> (AttachedModelInput, Vec<i32>, usize, Vec<bool>) {
        (
            self.attached,
            self.body_mocapid,
            self.nmocap,
            self.static_geom,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        AttachedFields, InertialFields, InertialModelInput, KinematicFields, KinematicModelInput,
    };

    fn tree() -> AttachedModelInput {
        let k = KinematicModelInput::new(
            1,
            KinematicFields {
                qpos0: vec![0.0],
                body_parentid: vec![0, 0, 1, 0, 3, 4, 0],
                body_jntadr: vec![-1, -1, -1, -1, -1, 0, -1],
                body_jntnum: vec![0, 0, 0, 0, 0, 1, 0],
                body_pos: vec![0.0; 21],
                body_quat: [1.0, 0.0, 0.0, 0.0].repeat(7),
                jnt_type: vec![2],
                jnt_bodyid: vec![5],
                jnt_qposadr: vec![0],
                jnt_dofadr: vec![0],
                jnt_pos: vec![0.0; 3],
                jnt_axis: vec![1.0, 0.0, 0.0],
            },
        )
        .unwrap();
        let r = InertialModelInput::new(
            k,
            InertialFields {
                body_ipos: vec![0.0; 21],
                body_iquat: [1.0, 0.0, 0.0, 0.0].repeat(7),
                body_mass: vec![0.0; 7],
                body_inertia: vec![0.0; 21],
                dof_bodyid: vec![5],
                dof_jntid: vec![0],
                dof_parentid: vec![-1],
                dof_armature: vec![0.0],
                dof_damping: vec![0.0],
            },
        )
        .unwrap();
        AttachedModelInput::new(
            r,
            AttachedFields {
                geom_bodyid: (0..7).collect(),
                geom_pos: vec![0.0; 21],
                geom_quat: [1.0, 0.0, 0.0, 0.0].repeat(7),
                ..Default::default()
            },
        )
        .unwrap()
    }

    #[test]
    fn accepts_permuted_ids_and_classifies_mocap_descendants() {
        let m = MocapModelInput::new(tree(), vec![-1, -1, -1, 1, -1, -1, 0]).unwrap();
        assert_eq!(m.nmocap(), 2);
        assert_eq!(m.body_mocapid(), [-1, -1, -1, 1, -1, -1, 0]);
        assert_eq!(
            m.static_geom(),
            [true, true, true, false, false, false, false]
        );
        assert_eq!(m.clone(), m);
        let m = MocapModelInput::new(tree(), vec![-1; 7]).unwrap();
        assert_eq!(m.nmocap(), 0);
        assert_eq!(m.static_geom(), [true, true, true, true, true, false, true]);
    }

    #[test]
    fn rejects_invalid_ids_and_nonfixed_mocap_bodies() {
        assert!(matches!(
            MocapModelInput::new(tree(), vec![-1; 6]),
            Err(InputError::LengthMismatch {
                field: "body_mocapid",
                ..
            })
        ));
        for ids in [
            vec![-2, -1, -1, -1, -1, -1, -1],
            vec![-1, -1, -1, 0, -1, -1, 2],
            vec![-1, -1, -1, 0, -1, -1, 0],
            vec![0, -1, -1, -1, -1, -1, -1],
            vec![-1, -1, 0, -1, -1, -1, -1],
            vec![-1, -1, -1, -1, -1, 0, -1],
        ] {
            assert!(matches!(
                MocapModelInput::new(tree(), ids),
                Err(InputError::InvalidTopology {
                    field: "body_mocapid",
                    ..
                })
            ));
        }
    }
}
