//! 空间肌腱的site与pulley子集。
//! 固定与空间集合各用局部编号。

use super::{FixedTendonModelInput, FixedTendonRows, KinematicModelInput, topology::JointType};
use crate::diagnostics::InputError;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpatialTendonFields {
    pub tendon_adr: Vec<i32>,
    pub tendon_num: Vec<i32>,
    pub wrap_type: Vec<i32>,
    pub wrap_objid: Vec<i32>,
    /// site项忽略参数，pulley项使用正除数。
    pub wrap_prm: Vec<f32>,
    pub ten_j_rowadr: Vec<i32>,
    pub ten_j_rownnz: Vec<i32>,
    pub ten_j_colind: Vec<i32>,
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

/// 只接受site与pulley路径。
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
}
impl SpatialTendonModelInput {
    pub fn new(
        fixed: FixedTendonModelInput,
        fields: SpatialTendonFields,
    ) -> Result<Self, InputError> {
        let base = fixed.camlight().mocap().attached();
        let k = base.rigid().kinematics();
        let nt = fields.tendon_adr.len();
        let nw = fields.wrap_type.len();
        checked_pack(nt, nw, fields.ten_j_colind.len(), k.nbody())?;
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
                _ => {
                    return Err(invalid(
                        "wrap_type",
                        index,
                        "spatial_tendon_requires_site_or_pulley",
                    ));
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
                } else {
                    sites += 1;
                    if i > start && fields.wrap_type[i - 1] == 3 {
                        let a =
                            base.fields().site_bodyid[fields.wrap_objid[i - 1] as usize] as usize;
                        let b = base.fields().site_bodyid[fields.wrap_objid[i] as usize] as usize;
                        // A common rigid transform cancels from segment length.
                        // Native CSR may omit its shared ancestor DOFs.
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
                    }
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
    pub(crate) fn into_parts(
        self,
    ) -> (
        FixedTendonModelInput,
        SpatialTendonFields,
        SpatialTendonRows,
    ) {
        (self.fixed, self.fields, self.rows)
    }
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
    let count = [(1usize, 6usize), (nb, 4), (nt, 4), (nw, 3), (nnz, 1)]
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
