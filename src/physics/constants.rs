//! G27 常量计算：固定常量 `set_const_fixed` 的宿主侧阶段切片。
//!
//! 上游 `mujoco_warp.set_const_fixed` 只计算 `body_subtreemass`
//! （身体及其全部后代的 f32 质量累加）。本切片按当前代码布局只做
//! 宿主侧固定常量：不发射内核、不改模型布局、不引入 Model/Data 类型。
//!
//! 累加顺序对齐上游：按 `body_tree` 深度分组、由深到浅，
//! 组内按身体编号升序把已累加的质量加到父身体。f32 加法可交换，
//! 因此对同一模型该顺序与上游并发原子累加的数值一致。
//!
//! 未验收边界：qpos0 依赖常量（dof/body/tendon invweight0、
//! tendon_length0、cam/light pos0）、执行器常量与长度范围
//! 依赖尚未实现的 G05 字段与 G01/G02/G03 链；动态更新属 @P7。

use crate::diagnostics::InputError;

/// `set_const_fixed` 的最小输入：模型级 f32 质量与父身体索引。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FixedConstantsInput<'a> {
    /// 每个身体的质量（千克，f32）。
    pub body_mass: &'a [f32],
    /// 每个身体的父身体索引；世界身体 0 的父索引为 0。
    pub body_parentid: &'a [i32],
}

/// 固定常量结果；当前只含 `body_subtreemass`。
#[derive(Clone, Debug, PartialEq)]
pub struct FixedConstants {
    body_subtreemass: Vec<f32>,
}

impl FixedConstants {
    /// 每个身体及其全部后代的质量累加。
    pub fn body_subtreemass(&self) -> &[f32] {
        &self.body_subtreemass
    }
}

/// 计算固定常量（上游 `set_const_fixed` 的宿主侧等价实现）。
pub fn set_const_fixed(input: FixedConstantsInput<'_>) -> Result<FixedConstants, InputError> {
    let body_mass = input.body_mass;
    let body_parentid = input.body_parentid;
    let nbody = body_mass.len();
    if nbody == 0 {
        return Err(InputError::InvalidDimension { field: "nbody" });
    }
    if body_parentid.len() != nbody {
        return Err(InputError::LengthMismatch {
            field: "body_parentid",
            expected: nbody,
            actual: body_parentid.len(),
        });
    }
    for (index, value) in body_mass.iter().enumerate() {
        if !value.is_finite() {
            return Err(InputError::NonFinite {
                field: "body_mass",
                index,
            });
        }
        if *value < 0.0 {
            return Err(InputError::NegativeValue {
                field: "body_mass",
                index,
            });
        }
    }
    if body_parentid[0] != 0 {
        return Err(InputError::InvalidTopology {
            field: "body_parentid",
            index: 0,
            reason: "世界身体 0 的父身体必须是 0",
        });
    }
    let depth = body_depths(body_parentid)?;

    let mut body_subtreemass = body_mass.to_vec();
    let max_depth = depth.iter().copied().max().unwrap_or(0);
    let mut levels = vec![Vec::new(); max_depth + 1];
    for (body, level) in depth.iter().enumerate() {
        levels[*level].push(body);
    }
    for level in (1..=max_depth).rev() {
        for body in &levels[level] {
            let parent = body_parentid[*body] as usize;
            let child = body_subtreemass[*body];
            body_subtreemass[parent] += child;
        }
    }
    Ok(FixedConstants { body_subtreemass })
}

/// 由父索引计算深度；父必须在范围内、不能自指、必须最终连到身体 0。
fn body_depths(body_parentid: &[i32]) -> Result<Vec<usize>, InputError> {
    let nbody = body_parentid.len();
    let mut parent = Vec::with_capacity(nbody);
    for (index, value) in body_parentid.iter().enumerate() {
        let value = usize::try_from(*value).map_err(|_| InputError::InvalidTopology {
            field: "body_parentid",
            index,
            reason: "父身体索引非法",
        })?;
        if value >= nbody {
            return Err(InputError::InvalidTopology {
                field: "body_parentid",
                index,
                reason: "父身体索引越界",
            });
        }
        if index > 0 && value == index {
            return Err(InputError::InvalidTopology {
                field: "body_parentid",
                index,
                reason: "父身体不能是自身",
            });
        }
        parent.push(value);
    }
    let mut depth = vec![usize::MAX; nbody];
    depth[0] = 0;
    for body in 1..nbody {
        if depth[body] != usize::MAX {
            continue;
        }
        let mut path = Vec::new();
        let mut node = body;
        while depth[node] == usize::MAX {
            path.push(node);
            if path.len() > nbody {
                return Err(InputError::InvalidTopology {
                    field: "body_parentid",
                    index: body,
                    reason: "父身体链存在环",
                });
            }
            node = parent[node];
        }
        let mut current = depth[node];
        for node in path.iter().rev() {
            current += 1;
            depth[*node] = current;
        }
    }
    if let Some(index) = depth.iter().position(|value| *value == usize::MAX) {
        return Err(InputError::InvalidTopology {
            field: "body_parentid",
            index,
            reason: "身体未连接到身体 0",
        });
    }
    Ok(depth)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn fixture() -> Value {
        serde_json::from_str(include_str!("../../fixtures/set-const/g27-fixed.json")).unwrap()
    }

    fn f32_list(value: &Value, field: &str) -> Vec<f32> {
        value[field]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_f64().unwrap() as f32)
            .collect()
    }

    fn compute(value: &Value) -> FixedConstants {
        let input = value["input"].clone();
        let body_mass = f32_list(&input, "body_mass");
        let body_parentid: Vec<i32> = input["body_parentid"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_i64().unwrap() as i32)
            .collect();
        set_const_fixed(FixedConstantsInput {
            body_mass: &body_mass,
            body_parentid: &body_parentid,
        })
        .unwrap()
    }

    #[test]
    fn fixed_constants_match_frozen_reference() {
        let value = fixture();
        assert_eq!(
            value["task_id"].as_str().unwrap(),
            "2026-10-11-roadmap-g27-p2"
        );
        assert_eq!(
            value["upstream_function"].as_str().unwrap(),
            "set_const_fixed"
        );
        let nbody = value["model"]["nbody"].as_u64().unwrap() as usize;
        let constants = compute(&value);
        let product = constants.body_subtreemass();
        assert_eq!(product.len(), nbody);

        let native: Vec<f64> = value["native_reference"]["body_subtreemass_f64"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_f64().unwrap())
            .collect();
        let native_tolerance = value["tolerance"]["native_abs"].as_f64().unwrap();
        assert_eq!(native.len(), product.len());
        for (index, (actual, expected)) in product.iter().zip(native.iter()).enumerate() {
            let difference = (*actual as f64 - expected).abs();
            assert!(
                difference <= native_tolerance,
                "body_subtreemass[{index}] product={actual} native={expected} diff={difference}"
            );
        }

        let runs = value["upstream_reference"]["runs"].as_array().unwrap();
        assert!(!runs.is_empty());
        for (run_index, run) in runs.iter().enumerate() {
            let upstream: Vec<f32> = run
                .as_array()
                .unwrap()
                .iter()
                .map(|item| item.as_f64().unwrap() as f32)
                .collect();
            assert_eq!(upstream.len(), product.len());
            for (index, (actual, expected)) in product.iter().zip(upstream.iter()).enumerate() {
                assert_eq!(
                    actual.to_bits(),
                    expected.to_bits(),
                    "run {run_index} body_subtreemass[{index}] product={actual} upstream={expected}"
                );
            }
        }
    }

    #[test]
    fn fixed_constants_are_deterministic() {
        let value = fixture();
        let first = compute(&value);
        let second = compute(&value);
        assert_eq!(first, second);
    }

    #[test]
    fn accumulates_descendants_into_world_body() {
        let body_mass = [0.0_f32, 1.3, 2.7, 0.9];
        let body_parentid = [0, 0, 1, 2];
        let constants = set_const_fixed(FixedConstantsInput {
            body_mass: &body_mass,
            body_parentid: &body_parentid,
        })
        .unwrap();
        let subtree = constants.body_subtreemass();
        assert_eq!(subtree[0], 1.3_f32 + (2.7_f32 + 0.9));
        assert_eq!(subtree[1], 1.3_f32 + (2.7_f32 + 0.9));
        assert_eq!(subtree[2], 2.7_f32 + 0.9);
        assert_eq!(subtree[3], 0.9);
    }

    #[test]
    fn accepts_parent_ids_greater_than_child_ids() {
        // 身体索引不保证父在子前；仅按父链判定深度。
        let body_mass = [0.0_f32, 0.5, 1.5];
        let body_parentid = [0, 2, 0];
        let constants = set_const_fixed(FixedConstantsInput {
            body_mass: &body_mass,
            body_parentid: &body_parentid,
        })
        .unwrap();
        assert_eq!(constants.body_subtreemass(), &[2.0, 0.5, 2.0]);
    }

    #[test]
    fn rejects_invalid_mass_and_lengths() {
        let parent = [0_i32, 0];
        assert_eq!(
            set_const_fixed(FixedConstantsInput {
                body_mass: &[0.0, 1.0, 2.0],
                body_parentid: &parent,
            }),
            Err(InputError::LengthMismatch {
                field: "body_parentid",
                expected: 3,
                actual: 2,
            })
        );
        assert_eq!(
            set_const_fixed(FixedConstantsInput {
                body_mass: &[],
                body_parentid: &[],
            }),
            Err(InputError::InvalidDimension { field: "nbody" })
        );
        assert_eq!(
            set_const_fixed(FixedConstantsInput {
                body_mass: &[0.0, -1.0],
                body_parentid: &[0, 0],
            }),
            Err(InputError::NegativeValue {
                field: "body_mass",
                index: 1,
            })
        );
        assert_eq!(
            set_const_fixed(FixedConstantsInput {
                body_mass: &[0.0, f32::NAN],
                body_parentid: &[0, 0],
            }),
            Err(InputError::NonFinite {
                field: "body_mass",
                index: 1,
            })
        );
    }

    #[test]
    fn rejects_invalid_topology() {
        let mass = [0.0_f32, 1.0, 1.0];
        let cases = [[1_i32, 0, 1], [0, 3, 0], [0, 1, 1], [0, 2, 1]];
        for parent in cases {
            assert!(
                matches!(
                    set_const_fixed(FixedConstantsInput {
                        body_mass: &mass,
                        body_parentid: &parent,
                    }),
                    Err(InputError::InvalidTopology { .. })
                ),
                "parent={parent:?} must be rejected"
            );
        }
    }
}
