//! 窄相输入校验：数组长度、世界批量与受支持几何的尺寸（退化输入显式拒绝）。

use super::{GEOM_BOX, GEOM_PLANE, GEOM_SPHERE, NarrowphaseError, PrimitiveModel, PrimitiveState};

/// 受支持几何的尺寸校验：非正/非有限尺寸属于退化输入，窄相入口显式拒绝。
///
/// plane：x/y 半径必须 >0、z 分量不得为负、三轴有限；sphere：半径 >0 且三轴有限；
/// box：三轴半尺寸 >0 且有限。其余几何类型本片不支持，尺寸不做约束。
fn validate_geom_sizes(model: &PrimitiveModel<'_>) -> Result<(), NarrowphaseError> {
    for geom in 0..model.ngeom {
        let size = model.geom_size[geom];
        for (axis, value) in size.iter().enumerate() {
            if !value.is_finite() {
                return Err(NarrowphaseError::InvalidGeomSize { geom, axis });
            }
        }
        match model.geom_type[geom] {
            GEOM_PLANE => {
                if size[0] <= 0.0 {
                    return Err(NarrowphaseError::InvalidGeomSize { geom, axis: 0 });
                }
                if size[1] <= 0.0 {
                    return Err(NarrowphaseError::InvalidGeomSize { geom, axis: 1 });
                }
                if size[2] < 0.0 {
                    return Err(NarrowphaseError::InvalidGeomSize { geom, axis: 2 });
                }
            }
            GEOM_SPHERE => {
                if size[0] <= 0.0 {
                    return Err(NarrowphaseError::InvalidGeomSize { geom, axis: 0 });
                }
            }
            GEOM_BOX => {
                for (axis, value) in size.iter().enumerate() {
                    if *value <= 0.0 {
                        return Err(NarrowphaseError::InvalidGeomSize { geom, axis });
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn validate(
    model: &PrimitiveModel<'_>,
    state: &PrimitiveState<'_>,
) -> Result<(), NarrowphaseError> {
    let ngeom = model.ngeom;
    for (field, actual) in [
        ("geom_type", model.geom_type.len()),
        ("geom_condim", model.geom_condim.len()),
        ("geom_priority", model.geom_priority.len()),
        ("geom_solmix", model.geom_solmix.len()),
        ("geom_solref", model.geom_solref.len()),
        ("geom_solimp", model.geom_solimp.len()),
        ("geom_friction", model.geom_friction.len()),
        ("geom_margin", model.geom_margin.len()),
        ("geom_gap", model.geom_gap.len()),
        ("geom_adhesion", model.geom_adhesion.len()),
        ("geom_size", model.geom_size.len()),
    ] {
        if actual != ngeom {
            return Err(NarrowphaseError::InvalidLength {
                field,
                expected: ngeom,
                actual,
            });
        }
    }
    validate_geom_sizes(model)?;
    for (field, actual) in [
        ("geom_xpos", state.geom_xpos.len()),
        ("geom_xmat", state.geom_xmat.len()),
    ] {
        if actual != state.nworld * ngeom {
            return Err(NarrowphaseError::InvalidLength {
                field,
                expected: state.nworld * ngeom,
                actual,
            });
        }
    }
    Ok(())
}
