//! 只接受关节项的固定肌腱。
//! 不表示完整肌腱模型。

use super::CamLightModelInput;
use crate::diagnostics::InputError;

/// 原生共享拓扑与共享系数。
/// 稀疏列保持原生DOF编号。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FixedTendonFields {
    pub tendon_adr: Vec<i32>,
    pub tendon_num: Vec<i32>,
    pub wrap_type: Vec<i32>,
    pub wrap_objid: Vec<i32>,
    pub wrap_prm: Vec<f32>,
    pub ten_j_rowadr: Vec<i32>,
    pub ten_j_rownnz: Vec<i32>,
    pub ten_j_colind: Vec<i32>,
}

/// 已检查的共享CSR行布局。
/// 额外稀疏列保存零力臂。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FixedTendonRows {
    rowadr: Vec<i32>,
    rownnz: Vec<i32>,
    colind: Vec<i32>,
}

impl FixedTendonRows {
    pub fn ntendon(&self) -> usize {
        self.rowadr.len()
    }
    pub fn nnz(&self) -> usize {
        self.colind.len()
    }
    pub fn rowadr(&self) -> &[i32] {
        &self.rowadr
    }
    pub fn rownnz(&self) -> &[i32] {
        &self.rownnz
    }
    pub fn colind(&self) -> &[i32] {
        &self.colind
    }
    pub fn row(&self, tendon: usize) -> Result<&[i32], InputError> {
        if tendon >= self.ntendon() {
            return Err(InputError::InvalidIndex {
                field: "tendon",
                index: tendon,
                limit: self.ntendon(),
            });
        }
        let start = self.rowadr[tendon] as usize;
        Ok(&self.colind[start..start + self.rownnz[tendon] as usize])
    }
}

/// 拥有已检查的固定肌腱。
/// 只允许hinge与slide引用。
/// 允许其他关节不参与肌腱。
/// 每条肌腱拒绝重复关节项。
/// 系数不支持独立参数批量。
///
/// ```compile_fail
/// fn mutate(m: &mut mjwarp_rs::model::FixedTendonModelInput) {
///     m.fields().wrap_prm[0] = 2.0;
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct FixedTendonModelInput {
    camlight: CamLightModelInput,
    fields: FixedTendonFields,
    rows: FixedTendonRows,
}

impl FixedTendonModelInput {
    pub fn new(
        camlight: CamLightModelInput,
        fields: FixedTendonFields,
    ) -> Result<Self, InputError> {
        let k = camlight.mocap().attached().rigid().kinematics();
        let nt = fields.tendon_adr.len();
        let nw = fields.wrap_type.len();
        let nnz = fields.ten_j_colind.len();
        // Bound the kernel's complete integer pack before any derived allocation.
        checked_pack(nt, nw, nnz)?;
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
        partition(
            &fields.tendon_adr,
            &fields.tendon_num,
            nw,
            "tendon_adr",
            true,
        )?;
        partition(
            &fields.ten_j_rowadr,
            &fields.ten_j_rownnz,
            nnz,
            "ten_J_rowadr",
            false,
        )?;
        for (index, &value) in fields.wrap_prm.iter().enumerate() {
            if !value.is_finite() {
                return Err(InputError::NonFinite {
                    field: "wrap_prm",
                    index,
                });
            }
        }
        for tendon in 0..nt {
            let start = fields.tendon_adr[tendon] as usize;
            let end = start + fields.tendon_num[tendon] as usize;
            let row = fields.ten_j_rowadr[tendon] as usize;
            let columns = &fields.ten_j_colind[row..row + fields.ten_j_rownnz[tendon] as usize];
            for (offset, &column) in columns.iter().enumerate() {
                if column < 0
                    || column as usize >= k.nv()
                    || (offset > 0 && columns[offset - 1] >= column)
                {
                    return Err(invalid(
                        "ten_J_colind",
                        row + offset,
                        "invalid_sorted_dof_column",
                    ));
                }
            }
            let mut seen = std::collections::BTreeSet::new();
            for index in start..end {
                if fields.wrap_type[index] != 1 {
                    return Err(invalid(
                        "wrap_type",
                        index,
                        "fixed_tendon_requires_joint_wrap",
                    ));
                }
                let joint = fields.wrap_objid[index];
                if joint < 0 || joint as usize >= k.njnt() {
                    return Err(invalid("wrap_objid", index, "joint_reference_out_of_range"));
                }
                let joint = joint as usize;
                if !matches!(k.fields().jnt_type[joint], 2 | 3) {
                    return Err(invalid(
                        "wrap_objid",
                        index,
                        "fixed_tendon_requires_scalar_joint",
                    ));
                }
                if !seen.insert(joint) {
                    return Err(invalid("wrap_objid", index, "duplicate_tendon_joint"));
                }
                if columns
                    .binary_search(&k.fields().jnt_dofadr[joint])
                    .is_err()
                {
                    return Err(invalid("ten_J_colind", row, "missing_tendon_dof_column"));
                }
            }
        }
        let rows = FixedTendonRows {
            rowadr: fields.ten_j_rowadr.clone(),
            rownnz: fields.ten_j_rownnz.clone(),
            colind: fields.ten_j_colind.clone(),
        };
        Ok(Self {
            camlight,
            fields,
            rows,
        })
    }
    pub fn camlight(&self) -> &CamLightModelInput {
        &self.camlight
    }
    pub fn fields(&self) -> &FixedTendonFields {
        &self.fields
    }
    pub fn rows(&self) -> &FixedTendonRows {
        &self.rows
    }
    pub(crate) fn into_parts(self) -> (CamLightModelInput, FixedTendonFields, FixedTendonRows) {
        (self.camlight, self.fields, self.rows)
    }
}

fn invalid(field: &'static str, index: usize, reason: &'static str) -> InputError {
    InputError::InvalidTopology {
        field,
        index,
        reason,
    }
}
fn checked_pack(nt: usize, nw: usize, nnz: usize) -> Result<(), InputError> {
    nt.checked_mul(2)
        .and_then(|n| nw.checked_mul(2).and_then(|w| n.checked_add(w)))
        .and_then(|n| n.checked_add(1))
        .filter(|&n| n <= i32::MAX as usize && nnz <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "fixed_tendon_fields",
        })?;
    Ok(())
}
fn partition(
    adr: &[i32],
    num: &[i32],
    capacity: usize,
    field: &'static str,
    nonempty: bool,
) -> Result<(), InputError> {
    let mut cursor = 0usize;
    for (index, (&start, &count)) in adr.iter().zip(num).enumerate() {
        if start < 0 || start as usize != cursor || count < i32::from(nonempty) {
            return Err(invalid(field, index, "invalid_contiguous_partition"));
        }
        cursor = cursor
            .checked_add(count as usize)
            .filter(|&end| end <= capacity)
            .ok_or_else(|| invalid(field, index, "partition_out_of_range"))?;
    }
    if cursor != capacity {
        return Err(invalid(field, adr.len(), "unpartitioned_elements"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_partition_gaps_overlaps_and_unclaimed_elements() {
        for (adr, num, capacity) in [
            (vec![1], vec![1], 2),
            (vec![-1], vec![1], 1),
            (vec![0, 0], vec![1, 1], 2),
            (vec![0], vec![-1], 1),
            (vec![0], vec![2], 1),
            (vec![0], vec![0], 1),
            (vec![], vec![], 1),
        ] {
            assert!(partition(&adr, &num, capacity, "test", false).is_err());
        }
        assert!(partition(&[0], &[0], 0, "test", true).is_err());
        partition(&[0, 2], &[2, 1], 3, "test", true).unwrap();
        partition(&[], &[], 0, "test", true).unwrap();
    }
    #[test]
    fn checks_pack_overflow_without_allocating() {
        for values in [
            (usize::MAX, 0, 0),
            (0, usize::MAX, 0),
            (0, 0, usize::MAX),
            (i32::MAX as usize / 2, 1, 0),
        ] {
            assert!(checked_pack(values.0, values.1, values.2).is_err());
        }
        checked_pack(0, 0, 0).unwrap();
        assert!(FixedTendonRows::default().row(0).is_err());
    }
}
