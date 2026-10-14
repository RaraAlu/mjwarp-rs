//! 四力合并所需的体、自由度与树映射。
//! 本子集只检查索引，不证明完整原生拓扑。
use crate::diagnostics::InputError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmoothForceInput {
    ntree: usize,
    body_treeid: Vec<i32>,
    dof_bodyid: Vec<i32>,
}
impl SmoothForceInput {
    pub fn new(
        ntree: usize,
        body_treeid: Vec<i32>,
        dof_bodyid: Vec<i32>,
    ) -> Result<Self, InputError> {
        if body_treeid.is_empty() || dof_bodyid.is_empty() {
            return Err(InputError::InvalidDimension {
                field: "smooth_force_topology",
            });
        }
        body_treeid
            .len()
            .checked_add(dof_bodyid.len())
            .and_then(|v| v.checked_add(8))
            .filter(|&v| v <= i32::MAX as usize)
            .ok_or(InputError::Overflow {
                field: "smooth_force_topology",
            })?;
        if ntree > i32::MAX as usize {
            return Err(InputError::Overflow {
                field: "smooth_force_trees",
            });
        }
        for (index, &tree) in body_treeid.iter().enumerate() {
            if tree < -1 || (tree >= 0 && tree as usize >= ntree) {
                return Err(InputError::InvalidTopology {
                    field: "body_treeid",
                    index,
                    reason: "tree_reference_out_of_range",
                });
            }
        }
        for (index, &body) in dof_bodyid.iter().enumerate() {
            if body < 0 || body as usize >= body_treeid.len() {
                return Err(InputError::InvalidTopology {
                    field: "dof_bodyid",
                    index,
                    reason: "body_reference_out_of_range",
                });
            }
        }
        Ok(Self {
            ntree,
            body_treeid,
            dof_bodyid,
        })
    }
    pub fn ntree(&self) -> usize {
        self.ntree
    }
    pub fn nbody(&self) -> usize {
        self.body_treeid.len()
    }
    pub fn nv(&self) -> usize {
        self.dof_bodyid.len()
    }
    pub fn body_treeid(&self) -> &[i32] {
        &self.body_treeid
    }
    pub fn dof_bodyid(&self) -> &[i32] {
        &self.dof_bodyid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_static_repeated_and_nonidentity_body_tree_mapping() {
        let m = SmoothForceInput::new(2, vec![-1, 1, 0, -1], vec![2, 0, 1, 2, 3]).unwrap();
        assert_eq!((m.ntree(), m.nbody(), m.nv()), (2, 4, 5));
        assert_eq!(m.body_treeid(), &[-1, 1, 0, -1]);
        assert_eq!(m.dof_bodyid(), &[2, 0, 1, 2, 3]);
        assert_eq!(m.clone(), m);
        assert!(SmoothForceInput::new(0, vec![-1, -1], vec![1, 0]).is_ok());
    }
    #[test]
    fn rejects_empty_out_of_range_and_oversized_tree_counts() {
        assert!(SmoothForceInput::new(0, vec![], vec![0]).is_err());
        assert!(SmoothForceInput::new(0, vec![-1], vec![]).is_err());
        assert!(SmoothForceInput::new(usize::MAX, vec![-1], vec![0]).is_err());
        for tree in [-2, 2, i32::MAX] {
            assert!(SmoothForceInput::new(2, vec![-1, tree], vec![1]).is_err());
        }
        for body in [-1, 2, i32::MAX] {
            assert!(SmoothForceInput::new(1, vec![-1, 0], vec![body]).is_err());
        }
        assert!(SmoothForceInput::new(0, vec![0], vec![0]).is_err());
    }
}
