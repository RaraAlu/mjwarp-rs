//! 空间肌腱的site、滑轮与球柱子集。
//! 固定与空间集合各用局部编号。

use super::{
    FixedTendonModelInput, FixedTendonRows, KinematicModelInput, ParameterBatch,
    topology::JointType,
};
use crate::diagnostics::InputError;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpatialTendonFields {
    pub tendon_adr: Vec<i32>,
    pub tendon_num: Vec<i32>,
    pub wrap_type: Vec<i32>,
    pub wrap_objid: Vec<i32>,
    /// site项忽略，pulley项使用除数。
    /// 球柱项四舍五入为侧向site。
    pub wrap_prm: Vec<f32>,
    pub ten_j_rowadr: Vec<i32>,
    pub ten_j_rownnz: Vec<i32>,
    pub ten_j_colind: Vec<i32>,
}

/// 原生几何类型与独立尺寸周期。
/// 尺寸行采用三标量原生布局。
/// GPU绕行只读取首项半径。
#[derive(Clone, Debug, PartialEq)]
pub struct SpatialTendonGeometry {
    pub geom_type: Vec<i32>,
    pub geom_size: ParameterBatch,
}

/// 空间集合的只读CSR布局。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpatialTendonRows {
    csr: FixedTendonRows,
    nwrap: usize,
}
impl SpatialTendonRows {
    pub fn ntendon(&self) -> usize {
        self.csr.ntendon()
    }
    pub fn nnz(&self) -> usize {
        self.csr.nnz()
    }
    pub fn nwrap(&self) -> usize {
        self.nwrap
    }
    pub fn rowadr(&self) -> &[i32] {
        self.csr.rowadr()
    }
    pub fn rownnz(&self) -> &[i32] {
        self.csr.rownnz()
    }
    pub fn colind(&self) -> &[i32] {
        self.csr.colind()
    }
    pub fn row(&self, tendon: usize) -> Result<&[i32], InputError> {
        self.csr.row(tendon)
    }
}

/// 支持site、pulley与球柱路径。
/// 每个分支至少包含两个site。
/// 它不提供原生全局肌腱编号。
///
/// ```compile_fail
/// fn mutate(m: &mut mjwarp_rs::model::SpatialTendonModelInput) {
///     m.fields().wrap_objid[0] = 2;
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct SpatialTendonModelInput {
    fixed: FixedTendonModelInput,
    fields: SpatialTendonFields,
    rows: SpatialTendonRows,
    geometry: Option<SpatialTendonGeometry>,
}
impl SpatialTendonModelInput {
    pub fn new(
        fixed: FixedTendonModelInput,
        fields: SpatialTendonFields,
    ) -> Result<Self, InputError> {
        Self::checked(fixed, fields, None)
    }
    pub fn with_geometry(
        fixed: FixedTendonModelInput,
        fields: SpatialTendonFields,
        geometry: SpatialTendonGeometry,
    ) -> Result<Self, InputError> {
        Self::checked(fixed, fields, Some(geometry))
    }
    fn checked(
        fixed: FixedTendonModelInput,
        fields: SpatialTendonFields,
        geometry: Option<SpatialTendonGeometry>,
    ) -> Result<Self, InputError> {
        let base = fixed.camlight().mocap().attached();
        let k = base.rigid().kinematics();
        let nt = fields.tendon_adr.len();
        let nw = fields.wrap_type.len();
        checked_pack(nt, nw, fields.ten_j_colind.len(), k.nbody())?;
        if let Some(g) = &geometry {
            if g.geom_type.len() != base.ngeom() || g.geom_size.row_elements() != 3 * base.ngeom() {
                return Err(InputError::InvalidDimension {
                    field: "spatial_geom_dimensions",
                });
            }
            for (index, &kind) in g.geom_type.iter().enumerate() {
                if !(0..=8).contains(&kind) {
                    return Err(invalid("geom_type", index, "unknown_geom_type"));
                }
            }
            for (index, &size) in g.geom_size.values().iter().enumerate() {
                if size < 0.0 {
                    return Err(invalid("geom_size", index, "negative_geom_size"));
                }
            }
            if nw
                .checked_add(g.geom_size.values().len())
                .is_none_or(|n| n > i32::MAX as usize)
            {
                return Err(InputError::Overflow {
                    field: "spatial_tendon_parameters",
                });
            }
        }
        for (field, actual, expected) in [
            ("tendon_num", fields.tendon_num.len(), nt),
            ("wrap_objid", fields.wrap_objid.len(), nw),
            ("wrap_prm", fields.wrap_prm.len(), nw),
            ("ten_J_rowadr", fields.ten_j_rowadr.len(), nt),
            ("ten_J_rownnz", fields.ten_j_rownnz.len(), nt),
        ] {
            if actual != expected {
                return Err(InputError::LengthMismatch {
                    field,
                    expected,
                    actual,
                });
            }
        }
        super::fixed_tendon::partition(
            &fields.tendon_adr,
            &fields.tendon_num,
            nw,
            "tendon_adr",
            true,
        )?;
        let csr = FixedTendonRows::checked(
            fields.ten_j_rowadr.clone(),
            fields.ten_j_rownnz.clone(),
            fields.ten_j_colind.clone(),
            k.nv(),
        )?;
        for index in 0..nw {
            if !fields.wrap_prm[index].is_finite() {
                return Err(InputError::NonFinite {
                    field: "wrap_prm",
                    index,
                });
            }
            match fields.wrap_type[index] {
                3 => {
                    let id = fields.wrap_objid[index];
                    if id < 0 || id as usize >= base.nsite() {
                        return Err(invalid("wrap_objid", index, "site_reference_out_of_range"));
                    }
                }
                2 => {
                    let divisor = fields.wrap_prm[index];
                    if divisor <= 0.0 || !(1.0 / divisor).is_finite() {
                        return Err(invalid("wrap_prm", index, "unusable_pulley_divisor"));
                    }
                }
                4 | 5 => {
                    let g = geometry.as_ref().ok_or(invalid(
                        "wrap_type",
                        index,
                        "missing_geometry_fields",
                    ))?;
                    let id = fields.wrap_objid[index];
                    if id < 0 || id as usize >= base.ngeom() {
                        return Err(invalid("wrap_objid", index, "geom_reference_out_of_range"));
                    }
                    let expected = if fields.wrap_type[index] == 4 { 2 } else { 5 };
                    if g.geom_type[id as usize] != expected {
                        return Err(invalid("geom_type", id as usize, "wrap_geom_type_mismatch"));
                    }
                    let side = fields.wrap_prm[index].round();
                    if side >= 0.0 && f64::from(side) >= base.nsite() as f64 {
                        return Err(invalid(
                            "wrap_prm",
                            index,
                            "sidesite_reference_out_of_range",
                        ));
                    }
                }
                _ => {
                    return Err(invalid("wrap_type", index, "unsupported_spatial_wrap_type"));
                }
            }
        }
        for tendon in 0..nt {
            let start = fields.tendon_adr[tendon] as usize;
            let end = start + fields.tendon_num[tendon] as usize;
            let mut sites = 0;
            for i in start..end {
                if fields.wrap_type[i] == 2 {
                    if i != start && sites < 2 {
                        return Err(invalid("wrap_type", i, "spatial_branch_requires_two_sites"));
                    }
                    sites = 0;
                } else if fields.wrap_type[i] == 3 {
                    sites += 1;
                    if i > start && fields.wrap_type[i - 1] == 3 {
                        let a =
                            base.fields().site_bodyid[fields.wrap_objid[i - 1] as usize] as usize;
                        let b = base.fields().site_bodyid[fields.wrap_objid[i] as usize] as usize;
                        // A common rigid transform cancels from segment length.
                        // Native CSR may omit its shared ancestor DOFs.
                        require_dofs(k, &csr, tendon, a, b)?;
                    }
                } else {
                    if i == start
                        || i + 1 == end
                        || fields.wrap_type[i - 1] != 3
                        || fields.wrap_type[i + 1] != 3
                    {
                        return Err(invalid("wrap_type", i, "geom_requires_site_geom_site"));
                    }
                    let a = base.fields().site_bodyid[fields.wrap_objid[i - 1] as usize] as usize;
                    let g = base.fields().geom_bodyid[fields.wrap_objid[i] as usize] as usize;
                    let b = base.fields().site_bodyid[fields.wrap_objid[i + 1] as usize] as usize;
                    require_dofs(k, &csr, tendon, a, g)?;
                    require_dofs(k, &csr, tendon, g, b)?;
                }
            }
            if sites < 2 {
                return Err(invalid(
                    "tendon_num",
                    tendon,
                    "spatial_branch_requires_two_sites",
                ));
            }
        }
        Ok(Self {
            fixed,
            fields,
            rows: SpatialTendonRows { csr, nwrap: nw },
            geometry,
        })
    }
    pub fn fixed(&self) -> &FixedTendonModelInput {
        &self.fixed
    }
    pub fn fields(&self) -> &SpatialTendonFields {
        &self.fields
    }
    pub fn rows(&self) -> &SpatialTendonRows {
        &self.rows
    }
    pub fn geometry(&self) -> Option<&SpatialTendonGeometry> {
        self.geometry.as_ref()
    }
    pub(crate) fn into_parts(
        self,
    ) -> (
        FixedTendonModelInput,
        SpatialTendonFields,
        SpatialTendonRows,
        Option<SpatialTendonGeometry>,
    ) {
        (self.fixed, self.fields, self.rows, self.geometry)
    }
}
fn require_dofs(
    k: &KinematicModelInput,
    csr: &FixedTendonRows,
    tendon: usize,
    a: usize,
    b: usize,
) -> Result<(), InputError> {
    let ca = chain_dofs(k, a)?;
    let cb = chain_dofs(k, b)?;
    let columns = csr.row(tendon)?;
    for dof in ca.symmetric_difference(&cb) {
        if columns.binary_search(dof).is_err() {
            return Err(invalid(
                "ten_J_colind",
                csr.rowadr()[tendon] as usize,
                "missing_spatial_dof_column",
            ));
        }
    }
    Ok(())
}
fn invalid(field: &'static str, index: usize, reason: &'static str) -> InputError {
    InputError::InvalidTopology {
        field,
        index,
        reason,
    }
}
fn chain_dofs(k: &KinematicModelInput, mut body: usize) -> Result<BTreeSet<i32>, InputError> {
    let f = k.fields();
    let mut dofs = BTreeSet::new();
    while body > 0 {
        if f.body_jntnum[body] > 0 {
            let start = f.body_jntadr[body] as usize;
            for joint in start..start + f.body_jntnum[body] as usize {
                let width = JointType::try_from(f.jnt_type[joint])?.dof_width();
                for offset in 0..width {
                    dofs.insert(f.jnt_dofadr[joint] + offset as i32);
                }
            }
        }
        body = f.body_parentid[body] as usize;
    }
    Ok(dofs)
}
fn checked_pack(nt: usize, nw: usize, nnz: usize, nb: usize) -> Result<(), InputError> {
    // Header, body parent/root/DOF descriptors, tendon rows, types and site bodies.
    let count = [(1usize, 9usize), (nb, 4), (nt, 4), (nw, 4), (nnz, 1)]
        .into_iter()
        .try_fold(0usize, |sum, (n, width)| {
            n.checked_mul(width).and_then(|n| sum.checked_add(n))
        })
        .filter(|&n| n <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "spatial_tendon_fields",
        })?;
    let _ = count;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_pack_overflow_without_allocating() {
        for sizes in [
            (usize::MAX, 0, 0, 1),
            (0, usize::MAX, 0, 1),
            (0, 0, usize::MAX, 1),
            (0, 0, 0, usize::MAX),
        ] {
            assert!(checked_pack(sizes.0, sizes.1, sizes.2, sizes.3).is_err());
        }
        checked_pack(0, 0, 0, 1).unwrap();
        assert!(SpatialTendonRows::default().row(0).is_err());
    }
}
