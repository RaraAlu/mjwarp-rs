//! 常驻运动学的字段批量子集。
//! 拓扑仍共享，不表示完整Model。

use super::{AttachedModelInput, BatchLayout};
use crate::diagnostics::InputError;

/// 连续参数行 `[batch, scalar]`。
/// 世界索引独立采用取模。
/// 空字段也要求正批量。
#[derive(Clone, Debug, PartialEq)]
pub struct ParameterBatch {
    layout: BatchLayout,
    values: Vec<f32>,
}

impl ParameterBatch {
    pub fn new(batches: usize, row_elements: usize, values: Vec<f32>) -> Result<Self, InputError> {
        if batches > i32::MAX as usize {
            return Err(InputError::Overflow {
                field: "parameter_batches",
            });
        }
        let layout = BatchLayout::new(batches, row_elements, 4)?;
        if layout.total_elements() > i32::MAX as usize {
            return Err(InputError::Overflow {
                field: "parameter_elements",
            });
        }
        if values.len() != layout.total_elements() {
            return Err(InputError::LengthMismatch {
                field: "parameter_batch",
                expected: layout.total_elements(),
                actual: values.len(),
            });
        }
        for (index, value) in values.iter().enumerate() {
            if !value.is_finite() {
                return Err(InputError::NonFinite {
                    field: "parameter_batch",
                    index,
                });
            }
        }
        Ok(Self { layout, values })
    }

    pub fn batches(&self) -> usize {
        self.layout.worlds()
    }
    pub fn row_elements(&self) -> usize {
        self.layout.elements_per_world()
    }
    pub fn values(&self) -> &[f32] {
        &self.values
    }

    /// 不要求周期整除世界总数。
    pub fn world(&self, world: usize) -> &[f32] {
        let start = (world % self.batches()) * self.row_elements();
        &self.values[start..start + self.row_elements()]
    }
}

/// 当前三个阶段读取的浮点参数。
#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KinematicsParameter {
    Qpos0,
    BodyPos,
    BodyQuat,
    BodyIpos,
    BodyIquat,
    JointPos,
    JointAxis,
    BodyMass,
    BodyInertia,
    GeomPos,
    GeomQuat,
    SitePos,
    SiteQuat,
}

impl KinematicsParameter {
    pub const ALL: [Self; 13] = [
        Self::Qpos0,
        Self::BodyPos,
        Self::BodyQuat,
        Self::BodyIpos,
        Self::BodyIquat,
        Self::JointPos,
        Self::JointAxis,
        Self::BodyMass,
        Self::BodyInertia,
        Self::GeomPos,
        Self::GeomQuat,
        Self::SitePos,
        Self::SiteQuat,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Qpos0 => "qpos0",
            Self::BodyPos => "body_pos",
            Self::BodyQuat => "body_quat",
            Self::BodyIpos => "body_ipos",
            Self::BodyIquat => "body_iquat",
            Self::JointPos => "jnt_pos",
            Self::JointAxis => "jnt_axis",
            Self::BodyMass => "body_mass",
            Self::BodyInertia => "body_inertia",
            Self::GeomPos => "geom_pos",
            Self::GeomQuat => "geom_quat",
            Self::SitePos => "site_pos",
            Self::SiteQuat => "site_quat",
        }
    }

    pub(crate) fn shared(self, model: &AttachedModelInput) -> &[f32] {
        let k = model.rigid().kinematics().fields();
        let i = model.rigid().fields();
        let a = model.fields();
        match self {
            Self::Qpos0 => &k.qpos0,
            Self::BodyPos => &k.body_pos,
            Self::BodyQuat => &k.body_quat,
            Self::BodyIpos => &i.body_ipos,
            Self::BodyIquat => &i.body_iquat,
            Self::JointPos => &k.jnt_pos,
            Self::JointAxis => &k.jnt_axis,
            Self::BodyMass => &i.body_mass,
            Self::BodyInertia => &i.body_inertia,
            Self::GeomPos => &a.geom_pos,
            Self::GeomQuat => &a.geom_quat,
            Self::SitePos => &a.site_pos,
            Self::SiteQuat => &a.site_quat,
        }
    }
}

/// 可选字段覆盖，缺省使用共享行。
/// 计划构造时验证模型尺寸。
/// 计划持有后不允许修改参数。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct KinematicsParameters {
    fields: [Option<ParameterBatch>; 13],
}

impl KinematicsParameters {
    /// 替换指定字段，不改变其他周期。
    pub fn set(&mut self, field: KinematicsParameter, batch: ParameterBatch) {
        self.fields[field as usize] = Some(batch);
    }

    pub fn get(&self, field: KinematicsParameter) -> Option<&ParameterBatch> {
        self.fields[field as usize].as_ref()
    }

    pub(crate) fn validate(&self, model: &AttachedModelInput) -> Result<(), InputError> {
        for field in KinematicsParameter::ALL {
            if let Some(batch) = self.get(field) {
                let expected = field.shared(model).len();
                if batch.row_elements() != expected {
                    return Err(InputError::LengthMismatch {
                        field: field.name(),
                        expected,
                        actual: batch.row_elements(),
                    });
                }
            }
        }
        Ok(())
    }

    pub(crate) fn values<'a>(
        &'a self,
        field: KinematicsParameter,
        model: &'a AttachedModelInput,
    ) -> &'a [f32] {
        self.get(field)
            .map_or_else(|| field.shared(model), ParameterBatch::values)
    }

    #[cfg(any(feature = "cuda-probe", test))]
    pub(crate) fn batches(&self, field: KinematicsParameter) -> usize {
        self.get(field).map_or(1, ParameterBatch::batches)
    }

    #[cfg(feature = "cuda-probe")]
    pub(crate) fn world<'a>(
        &'a self,
        field: KinematicsParameter,
        model: &'a AttachedModelInput,
        world: usize,
    ) -> &'a [f32] {
        self.get(field)
            .map_or_else(|| field.shared(model), |b| b.world(world))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_rows_and_independent_periods() {
        let batch = ParameterBatch::new(3, 2, vec![-0.0, 1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        assert_eq!(batch.world(4), &[2.0, 3.0]);
        assert_eq!(batch.world(usize::MAX), batch.world(usize::MAX % 3));
        assert_eq!(batch.values()[0].to_bits(), (-0.0f32).to_bits());
        let mut fields = KinematicsParameters::default();
        fields.set(KinematicsParameter::BodyPos, batch.clone());
        fields.set(
            KinematicsParameter::JointPos,
            ParameterBatch::new(2, 0, vec![]).unwrap(),
        );
        assert_eq!(fields.get(KinematicsParameter::BodyPos), Some(&batch));
        assert_eq!(fields.batches(KinematicsParameter::JointPos), 2);
        assert_eq!(fields.batches(KinematicsParameter::Qpos0), 1);
    }

    #[test]
    fn accepts_empty_fields_and_rejects_bad_dimensions() {
        assert!(
            ParameterBatch::new(i32::MAX as usize, 0, vec![])
                .unwrap()
                .world(513)
                .is_empty()
        );
        for (b, width) in [
            (0, 0),
            (0, 1),
            (usize::MAX, 0),
            (1, usize::MAX),
            (i32::MAX as usize, 2),
        ] {
            assert!(ParameterBatch::new(b, width, vec![]).is_err());
        }
        assert!(matches!(
            ParameterBatch::new(2, 3, vec![0.0; 5]),
            Err(InputError::LengthMismatch {
                expected: 6,
                actual: 5,
                ..
            })
        ));
    }

    #[test]
    fn rejects_nonfinite_values_in_unused_rows() {
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                ParameterBatch::new(3, 1, vec![0.0, 1.0, bad]),
                Err(InputError::NonFinite {
                    field: "parameter_batch",
                    index: 2
                })
            );
        }
    }
}
