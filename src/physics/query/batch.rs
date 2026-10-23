//! G28@P8 多世界批量射线查询：每世界不同冻结 geom 位姿 × 共享射线列表。
//!
//! 复用 P7 的单世界 `ray`（原语求交、最近命中、无命中语义），按世界展开
//! `nworld × nray` 结果。未包含 BVH/RenderContext、mesh/hfield/flex/SDF、
//! geomgroup/flg_static/bodyexclude 过滤与设备内核。

use super::{GeomPrimitive, QueryError, Ray, RayHit, ray};

/// 多世界冻结 geom 位姿与尺寸。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldGeoms<'a> {
    pub nworld: usize,
    pub ngeom: usize,
    pub geom_type: &'a [i32],
    pub size: &'a [f32],
    pub pos: &'a [f32],
    pub mat: &'a [f32],
}

/// 批量射线查询：返回 `worlds[world][ray]` 的命中结果。
pub fn rays_batch(geoms: WorldGeoms<'_>, rays: &[Ray]) -> Result<Vec<Vec<RayHit>>, QueryError> {
    if geoms.nworld == 0 || geoms.ngeom == 0 {
        return Err(QueryError::DegenerateSize { index: 0 });
    }
    for (field, actual, expected) in [
        ("geom_type", geoms.geom_type.len(), geoms.ngeom),
        ("size", geoms.size.len(), geoms.ngeom * 3),
        ("pos", geoms.pos.len(), geoms.nworld * geoms.ngeom * 3),
        ("mat", geoms.mat.len(), geoms.nworld * geoms.ngeom * 9),
    ] {
        if actual != expected {
            return Err(QueryError::InvalidLength {
                field,
                expected,
                actual,
            });
        }
    }
    let mut output = Vec::with_capacity(geoms.nworld);
    for world in 0..geoms.nworld {
        let mut primitives = Vec::with_capacity(geoms.ngeom);
        for geom in 0..geoms.ngeom {
            let pos_base = (world * geoms.ngeom + geom) * 3;
            let mat_base = (world * geoms.ngeom + geom) * 9;
            primitives.push(GeomPrimitive {
                geom_type: geoms.geom_type[geom],
                pos: [
                    geoms.pos[pos_base],
                    geoms.pos[pos_base + 1],
                    geoms.pos[pos_base + 2],
                ],
                mat: geoms.mat[mat_base..mat_base + 9].try_into().unwrap(),
                size: [
                    geoms.size[geom * 3],
                    geoms.size[geom * 3 + 1],
                    geoms.size[geom * 3 + 2],
                ],
            });
        }
        let mut hits = Vec::with_capacity(rays.len());
        for query in rays {
            hits.push(ray(&primitives, *query)?);
        }
        output.push(hits);
    }
    Ok(output)
}

#[cfg(test)]
mod tests;
