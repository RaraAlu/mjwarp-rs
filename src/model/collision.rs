//! 平面与球的宿主字段小原型。
//! 不提供碰撞执行或设备上传。

use crate::diagnostics::InputError;

/// 冻结模型的十三项几何字段。
/// 浮点字段只承载单份参数。
/// 本子集不包含网格或显式碰撞对。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PrimitiveGeometryFields {
    pub geom_type: Vec<i32>,
    pub geom_bodyid: Vec<i32>,
    /// 保留有符号整数的全部掩码位。
    pub geom_contype: Vec<i32>,
    pub geom_conaffinity: Vec<i32>,
    pub geom_condim: Vec<i32>,
    pub geom_priority: Vec<i32>,
    pub geom_solmix: Vec<f32>,
    pub geom_solref: Vec<f32>,
    pub geom_solimp: Vec<f32>,
    pub geom_size: Vec<f32>,
    pub geom_friction: Vec<f32>,
    pub geom_margin: Vec<f32>,
    pub geom_gap: Vec<f32>,
}

/// 独占已检查的宿主几何字段。
/// 本类型不证明完整物理参数有效。
#[derive(Clone, Debug, PartialEq)]
pub struct PrimitiveGeometryInput {
    nbody: usize,
    fields: PrimitiveGeometryFields,
}

impl PrimitiveGeometryInput {
    pub fn new(nbody: usize, fields: PrimitiveGeometryFields) -> Result<Self, InputError> {
        if nbody == 0 {
            return Err(InputError::InvalidDimension { field: "nbody" });
        }
        if nbody > i32::MAX as usize || fields.geom_type.len() > i32::MAX as usize / 5 {
            return Err(InputError::Overflow {
                field: "primitive_geometry",
            });
        }
        let n = fields.geom_type.len();
        check_lengths(
            n,
            [
                ("geom_bodyid", fields.geom_bodyid.len(), 1),
                ("geom_contype", fields.geom_contype.len(), 1),
                ("geom_conaffinity", fields.geom_conaffinity.len(), 1),
                ("geom_condim", fields.geom_condim.len(), 1),
                ("geom_priority", fields.geom_priority.len(), 1),
                ("geom_solmix", fields.geom_solmix.len(), 1),
                ("geom_solref", fields.geom_solref.len(), 2),
                ("geom_solimp", fields.geom_solimp.len(), 5),
                ("geom_size", fields.geom_size.len(), 3),
                ("geom_friction", fields.geom_friction.len(), 3),
                ("geom_margin", fields.geom_margin.len(), 1),
                ("geom_gap", fields.geom_gap.len(), 1),
            ],
        )?;
        for (field, values) in [
            ("geom_solmix", &fields.geom_solmix),
            ("geom_solref", &fields.geom_solref),
            ("geom_solimp", &fields.geom_solimp),
            ("geom_size", &fields.geom_size),
            ("geom_friction", &fields.geom_friction),
            ("geom_margin", &fields.geom_margin),
            ("geom_gap", &fields.geom_gap),
        ] {
            if let Some(index) = values.iter().position(|v| !v.is_finite()) {
                return Err(InputError::NonFinite { field, index });
            }
        }
        for (index, &kind) in fields.geom_type.iter().enumerate() {
            if !matches!(kind, 0 | 2) {
                return Err(InputError::InvalidTopology {
                    field: "geom_type",
                    index,
                    reason: "prototype_requires_plane_or_sphere",
                });
            }
        }
        for (index, &body) in fields.geom_bodyid.iter().enumerate() {
            if body < 0 || body as usize >= nbody {
                return Err(InputError::InvalidTopology {
                    field: "geom_bodyid",
                    index,
                    reason: "body_reference_out_of_range",
                });
            }
        }
        for (index, &dim) in fields.geom_condim.iter().enumerate() {
            if !matches!(dim, 1 | 3 | 4 | 6) {
                return Err(InputError::InvalidTopology {
                    field: "geom_condim",
                    index,
                    reason: "invalid_contact_dimension",
                });
            }
        }
        Ok(Self { nbody, fields })
    }

    pub fn fields(&self) -> &PrimitiveGeometryFields {
        &self.fields
    }
    pub fn nbody(&self) -> usize {
        self.nbody
    }
    pub fn ngeom(&self) -> usize {
        self.fields.geom_type.len()
    }
}

pub(crate) fn check_lengths(
    ngeom: usize,
    fields: impl IntoIterator<Item = (&'static str, usize, usize)>,
) -> Result<(), InputError> {
    for (field, actual, width) in fields {
        let expected = ngeom
            .checked_mul(width)
            .ok_or(InputError::Overflow { field })?;
        if actual != expected {
            return Err(InputError::LengthMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    Ok(())
}
