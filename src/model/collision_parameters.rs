//! 碰撞几何参数的独立宿主批量。
//! 不分配设备缓冲或执行物理。
use super::{ParameterBatch, PrimitiveGeometryFields, PrimitiveGeometryInput};
use crate::diagnostics::InputError;

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveGeometryParameter {
    Solmix,
    Solref,
    Solimp,
    Size,
    Friction,
    Margin,
    Gap,
}
impl PrimitiveGeometryParameter {
    pub const ALL: [Self; 7] = [
        Self::Solmix,
        Self::Solref,
        Self::Solimp,
        Self::Size,
        Self::Friction,
        Self::Margin,
        Self::Gap,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Solmix => "geom_solmix",
            Self::Solref => "geom_solref",
            Self::Solimp => "geom_solimp",
            Self::Size => "geom_size",
            Self::Friction => "geom_friction",
            Self::Margin => "geom_margin",
            Self::Gap => "geom_gap",
        }
    }
    pub fn width(self) -> usize {
        match self {
            Self::Solref => 2,
            Self::Solimp => 5,
            Self::Size | Self::Friction => 3,
            _ => 1,
        }
    }
    fn shared(self, fields: &PrimitiveGeometryFields) -> &[f32] {
        match self {
            Self::Solmix => &fields.geom_solmix,
            Self::Solref => &fields.geom_solref,
            Self::Solimp => &fields.geom_solimp,
            Self::Size => &fields.geom_size,
            Self::Friction => &fields.geom_friction,
            Self::Margin => &fields.geom_margin,
            Self::Gap => &fields.geom_gap,
        }
    }
}

/// 可选覆盖行；各字段独立取模。
/// 未覆盖字段保留原共享输入。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PrimitiveGeometryParameters {
    fields: [Option<ParameterBatch>; 7],
}
impl PrimitiveGeometryParameters {
    pub fn set(&mut self, field: PrimitiveGeometryParameter, batch: ParameterBatch) {
        self.fields[field as usize] = Some(batch);
    }
    pub fn get(&self, field: PrimitiveGeometryParameter) -> Option<&ParameterBatch> {
        self.fields[field as usize].as_ref()
    }
    pub fn batches(&self, field: PrimitiveGeometryParameter) -> usize {
        self.get(field).map_or(1, ParameterBatch::batches)
    }
}

/// 独占模型与参数；发布后只读。
/// 周期可以大于世界数，也不必整除。
#[derive(Clone, Debug, PartialEq)]
pub struct PrimitiveGeometryHostBatch {
    model: PrimitiveGeometryInput,
    parameters: PrimitiveGeometryParameters,
    worlds: usize,
}
impl PrimitiveGeometryHostBatch {
    pub fn new(
        model: PrimitiveGeometryInput,
        worlds: usize,
        parameters: PrimitiveGeometryParameters,
    ) -> Result<Self, InputError> {
        if worlds == 0 {
            return Err(InputError::InvalidDimension { field: "worlds" });
        }
        // Frozen consumers use signed 32-bit world IDs, independently of periods.
        if worlds > i32::MAX as usize {
            return Err(InputError::Overflow {
                field: "collision_worlds",
            });
        }
        for field in PrimitiveGeometryParameter::ALL {
            if let Some(batch) = parameters.get(field) {
                let expected = field.shared(model.fields()).len();
                if batch.row_elements() != expected {
                    return Err(InputError::LengthMismatch {
                        field: field.name(),
                        expected,
                        actual: batch.row_elements(),
                    });
                }
            }
        }
        Ok(Self {
            model,
            parameters,
            worlds,
        })
    }
    pub fn model(&self) -> &PrimitiveGeometryInput {
        &self.model
    }
    pub fn parameters(&self) -> &PrimitiveGeometryParameters {
        &self.parameters
    }
    pub fn worlds(&self) -> usize {
        self.worlds
    }
    pub fn world(&self, world: usize) -> Result<PrimitiveGeometryWorld<'_>, InputError> {
        if world >= self.worlds {
            return Err(InputError::InvalidIndex {
                field: "world",
                index: world,
                limit: self.worlds,
            });
        }
        let f = self.model.fields();
        let values = PrimitiveGeometryParameter::ALL.map(|field| {
            self.parameters
                .get(field)
                .map_or_else(|| field.shared(f), |batch| batch.world(world))
        });
        Ok(PrimitiveGeometryWorld {
            topology: PrimitiveGeometryTopology {
                geom_type: &f.geom_type,
                geom_bodyid: &f.geom_bodyid,
                geom_contype: &f.geom_contype,
                geom_conaffinity: &f.geom_conaffinity,
                geom_condim: &f.geom_condim,
                geom_priority: &f.geom_priority,
            },
            values,
        })
    }
}

/// 六项共享拓扑；所有世界借用同源。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimitiveGeometryTopology<'a> {
    pub geom_type: &'a [i32],
    pub geom_bodyid: &'a [i32],
    pub geom_contype: &'a [i32],
    pub geom_conaffinity: &'a [i32],
    pub geom_condim: &'a [i32],
    pub geom_priority: &'a [i32],
}

/// 世界只读视图，不能越过批量拥有者。
#[derive(Clone, Copy, Debug)]
pub struct PrimitiveGeometryWorld<'a> {
    topology: PrimitiveGeometryTopology<'a>,
    values: [&'a [f32]; 7],
}
impl<'a> PrimitiveGeometryWorld<'a> {
    pub fn topology(self) -> PrimitiveGeometryTopology<'a> {
        self.topology
    }
    pub fn values(self, field: PrimitiveGeometryParameter) -> &'a [f32] {
        self.values[field as usize]
    }
    pub fn geometry_values(
        self,
        field: PrimitiveGeometryParameter,
        geometry: usize,
    ) -> Result<&'a [f32], InputError> {
        let limit = self.topology.geom_type.len();
        if geometry >= limit {
            return Err(InputError::InvalidIndex {
                field: "geometry",
                index: geometry,
                limit,
            });
        }
        // The checked model bounds ngeom * every width before this view exists.
        let start = geometry * field.width();
        Ok(&self.values(field)[start..start + field.width()])
    }
}
