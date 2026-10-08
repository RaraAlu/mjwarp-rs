//! 检查肌腱唤醒的共享拓扑。
//! 不实现完整休眠状态机。

use super::{
    FlexEdgeFields, FlexPositionFields, FlexPositionModelInput, ParameterBatch, TendonModelInput,
};
use crate::diagnostics::InputError;

/// 冻结上游的完全唤醒值。
pub const TREE_FULLY_AWAKE: i32 = -11;

/// 范围与边距各用独立周期。
/// 开关仅控制本子集副作用。
#[derive(Clone, Debug, PartialEq)]
pub struct TendonWakeFields {
    pub body_treeid: Vec<i32>,
    pub tendon_limited: Vec<bool>,
    pub tendon_range: ParameterBatch,
    pub tendon_margin: ParameterBatch,
    pub sleep_enabled: bool,
    pub island_disabled: bool,
}

/// 接收原生树编号与限位字段。
/// 可选柔体支持位置与边。
#[derive(Clone, Debug, PartialEq)]
pub struct TendonWakeModelInput {
    tendons: TendonModelInput,
    flex: Option<FlexPositionFields>,
    edges: Option<FlexEdgeFields>,
    pub(crate) info: TendonWakeInfo,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TendonWakeInfo {
    pub fields: TendonWakeFields,
    pub ntree: usize,
    pub tendon_adr: Vec<i32>,
    pub tendon_num: Vec<i32>,
    pub wrap_treeid: Vec<i32>,
}

impl TendonWakeModelInput {
    pub fn new(tendons: TendonModelInput, fields: TendonWakeFields) -> Result<Self, InputError> {
        Self::checked(tendons, None, None, fields)
    }
    pub fn with_flex_positions(
        model: FlexPositionModelInput,
        fields: TendonWakeFields,
    ) -> Result<Self, InputError> {
        let (tendons, flex, edges) = model.into_parts();
        Self::checked(tendons, Some(flex), edges, fields)
    }
    fn checked(
        tendons: TendonModelInput,
        flex: Option<FlexPositionFields>,
        edges: Option<FlexEdgeFields>,
        fields: TendonWakeFields,
    ) -> Result<Self, InputError> {
        let attached = tendons.spatial().fixed().camlight().mocap().attached();
        let k = attached.rigid().kinematics().fields();
        length(
            "body_treeid",
            fields.body_treeid.len(),
            k.body_parentid.len(),
        )?;
        let mut expected = vec![-1; k.body_parentid.len()];
        let mut ntree = 0;
        for b in 1..expected.len() {
            expected[b] = expected[k.body_parentid[b] as usize];
            if expected[b] < 0 && k.body_jntnum[b] > 0 {
                expected[b] = ntree as i32;
                ntree += 1;
            }
        }
        if let Some(index) = fields
            .body_treeid
            .iter()
            .zip(&expected)
            .position(|(a, b)| a != b)
        {
            return Err(InputError::InvalidTopology {
                field: "body_treeid",
                index,
                reason: "noncanonical_tree_mapping",
            });
        }
        let paths = tendons.fields();
        let nt = tendons.rows().ntendon();
        length("tendon_limited", fields.tendon_limited.len(), nt)?;
        length("tendon_range", fields.tendon_range.row_elements(), 2 * nt)?;
        length("tendon_margin", fields.tendon_margin.row_elements(), nt)?;
        for (index, pair) in fields
            .tendon_range
            .values()
            .as_chunks::<2>()
            .0
            .iter()
            .enumerate()
        {
            if pair[0] > pair[1] {
                return Err(InputError::InvalidTopology {
                    field: "tendon_range",
                    index,
                    reason: "reversed_limits",
                });
            }
        }
        if let Some(index) = fields.tendon_margin.values().iter().position(|&v| v < 0.0) {
            return Err(InputError::NegativeValue {
                field: "tendon_margin",
                index,
            });
        }
        let overflow = || InputError::Overflow {
            field: "tendon_wake_fields",
        };
        nt.checked_mul(3)
            .and_then(|n| n.checked_add(paths.wrap_type.len()))
            .and_then(|n| n.checked_add(7))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        fields
            .tendon_range
            .values()
            .len()
            .checked_add(fields.tendon_margin.values().len())
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or_else(overflow)?;
        let wrap_treeid = paths
            .wrap_type
            .iter()
            .zip(&paths.wrap_objid)
            .map(|(&kind, &id)| {
                let body = match kind {
                    1 => k.jnt_bodyid[id as usize],
                    3 => attached.fields().site_bodyid[id as usize],
                    4 | 5 => attached.fields().geom_bodyid[id as usize],
                    _ => return -1,
                };
                fields.body_treeid[body as usize]
            })
            .collect();
        let info = TendonWakeInfo {
            fields,
            ntree,
            tendon_adr: paths.tendon_adr.clone(),
            tendon_num: paths.tendon_num.clone(),
            wrap_treeid,
        };
        Ok(Self {
            tendons,
            flex,
            edges,
            info,
        })
    }
    pub fn ntree(&self) -> usize {
        self.info.ntree
    }
    pub fn fields(&self) -> &TendonWakeFields {
        &self.info.fields
    }
    pub fn tendons(&self) -> &TendonModelInput {
        &self.tendons
    }
    pub fn flex_positions(&self) -> Option<&FlexPositionFields> {
        self.flex.as_ref()
    }
    pub fn flex_edges(&self) -> Option<&FlexEdgeFields> {
        self.edges.as_ref()
    }
    pub(crate) fn into_parts(
        self,
    ) -> (
        TendonModelInput,
        Option<FlexPositionFields>,
        Option<FlexEdgeFields>,
        TendonWakeInfo,
    ) {
        (self.tendons, self.flex, self.edges, self.info)
    }
}

/// 负值代表唤醒计数。
/// 非负值连接睡眠循环。
/// 写入时按计数派生tree_awake。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SleepTreeState {
    pub tree_asleep: Vec<i32>,
    pub nbody_awake: i32,
    pub nv_awake: i32,
}
impl SleepTreeState {
    pub(crate) fn validate(&self, ntree: usize, nbody: usize, nv: usize) -> Result<(), InputError> {
        length("tree_asleep", self.tree_asleep.len(), ntree)?;
        for (field, value, limit) in [
            ("nbody_awake", self.nbody_awake, nbody),
            ("nv_awake", self.nv_awake, nv),
        ] {
            if value < 0 || value as usize > limit {
                return Err(InputError::InvalidTopology {
                    field,
                    index: 0,
                    reason: "invalid_awake_count",
                });
            }
        }
        // 每个睡眠顶点恰有一条入边。
        // 此条件保证所有分量形成循环。
        let mut incoming = vec![0u8; ntree];
        for (index, &next) in self.tree_asleep.iter().enumerate() {
            if next < 0 {
                continue;
            }
            let next = next as usize;
            if next >= ntree || self.tree_asleep[next] < 0 || incoming[next] != 0 {
                return Err(InputError::InvalidTopology {
                    field: "tree_asleep",
                    index,
                    reason: "invalid_sleep_cycle",
                });
            }
            incoming[next] = 1;
        }
        if let Some(index) = self
            .tree_asleep
            .iter()
            .zip(incoming)
            .position(|(&v, count)| v >= 0 && count != 1)
        {
            return Err(InputError::InvalidTopology {
                field: "tree_asleep",
                index,
                reason: "invalid_sleep_cycle",
            });
        }
        Ok(())
    }
    #[cfg(any(feature = "cuda-probe", test))]
    pub(crate) fn packed(&self) -> Vec<i32> {
        let mut result = self.tree_asleep.clone();
        result.extend(self.tree_asleep.iter().map(|&v| i32::from(v < 0)));
        result.extend([
            self.tree_asleep.iter().filter(|&&v| v < 0).count() as i32,
            self.nbody_awake,
            self.nv_awake,
        ]);
        result
    }
}
fn length(field: &'static str, actual: usize, expected: usize) -> Result<(), InputError> {
    if actual != expected {
        return Err(InputError::LengthMismatch {
            field,
            expected,
            actual,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_negative_counts_and_disjoint_sleep_cycles() {
        for values in [vec![-11, -17, -1], vec![1, 0, 2], vec![-11, 2, 1], vec![]] {
            let state = SleepTreeState {
                tree_asleep: values,
                nbody_awake: 1,
                nv_awake: 0,
            };
            state.validate(state.tree_asleep.len(), 4, 3).unwrap();
            let p = state.packed();
            assert_eq!(p.len(), 2 * state.tree_asleep.len() + 3);
        }
    }
    #[test]
    fn rejects_noncyclic_sleep_links_and_invalid_counts() {
        for values in [vec![1, -1, 2], vec![1, 1, 2], vec![3, 1, 2]] {
            assert!(
                SleepTreeState {
                    tree_asleep: values,
                    nbody_awake: 1,
                    nv_awake: 0
                }
                .validate(3, 4, 3)
                .is_err()
            );
        }
        for (body, dof) in [(-1, 0), (5, 0), (0, -1), (0, 4)] {
            assert!(
                SleepTreeState {
                    tree_asleep: vec![],
                    nbody_awake: body,
                    nv_awake: dof
                }
                .validate(0, 4, 3)
                .is_err()
            );
        }
        assert!(
            SleepTreeState {
                tree_asleep: vec![],
                nbody_awake: 1,
                nv_awake: 0
            }
            .validate(3, 4, 3)
            .is_err()
        );
    }
}
