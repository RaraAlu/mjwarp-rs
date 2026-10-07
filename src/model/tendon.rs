//! 检查混合肌腱的原生全局编号。
//! GPU仍分别计算两类子集。

use super::{
    CamLightModelInput, FixedTendonFields, FixedTendonModelInput, FixedTendonRows,
    SpatialTendonFields, SpatialTendonGeometry, SpatialTendonModelInput,
};
use crate::diagnostics::InputError;

/// 原生顺序的共享拓扑与CSR。
/// 同一条路径不混用关节与site。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TendonFields {
    pub tendon_adr: Vec<i32>,
    pub tendon_num: Vec<i32>,
    pub wrap_type: Vec<i32>,
    pub wrap_objid: Vec<i32>,
    pub wrap_prm: Vec<f32>,
    pub ten_j_rowadr: Vec<i32>,
    pub ten_j_rownnz: Vec<i32>,
    pub ten_j_colind: Vec<i32>,
}

/// 全局行对应的局部编号。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TendonSubset {
    Fixed(usize),
    Spatial(usize),
}

/// 保留原生全局CSR与包裹容量。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TendonRows {
    csr: FixedTendonRows,
    subsets: Vec<TendonSubset>,
    nwrap: usize,
}
impl TendonRows {
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
    pub fn subset(&self, tendon: usize) -> Result<TendonSubset, InputError> {
        self.row(tendon)?;
        Ok(self.subsets[tendon])
    }
}

/// 接收已编译模型的混合字段。
/// 拆分时保持各子集相对顺序。
/// 子集校验报告局部错误索引。
/// flex与休眠需要可选包装。
///
/// ```compile_fail
/// fn mutate(m: &mut mjwarp_rs::model::TendonModelInput) {
///     m.rows().rownnz()[0] = 0;
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct TendonModelInput {
    spatial: SpatialTendonModelInput,
    rows: TendonRows,
    fields: TendonFields,
}

impl TendonModelInput {
    pub fn new(camlight: CamLightModelInput, fields: TendonFields) -> Result<Self, InputError> {
        Self::checked(camlight, fields, None)
    }
    pub fn with_geometry(
        camlight: CamLightModelInput,
        fields: TendonFields,
        geometry: SpatialTendonGeometry,
    ) -> Result<Self, InputError> {
        Self::checked(camlight, fields, Some(geometry))
    }
    fn checked(
        camlight: CamLightModelInput,
        fields: TendonFields,
        geometry: Option<SpatialTendonGeometry>,
    ) -> Result<Self, InputError> {
        let nt = fields.tendon_adr.len();
        let nw = fields.wrap_type.len();
        nt.checked_mul(5)
            .and_then(|n| n.checked_add(8))
            .filter(|&n| {
                n <= i32::MAX as usize
                    && nw <= i32::MAX as usize
                    && fields.ten_j_colind.len() <= i32::MAX as usize
            })
            .ok_or(InputError::Overflow {
                field: "tendon_fields",
            })?;
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
        let nv = camlight.mocap().attached().rigid().kinematics().nv();
        let csr = FixedTendonRows::checked(
            fields.ten_j_rowadr.clone(),
            fields.ten_j_rownnz.clone(),
            fields.ten_j_colind.clone(),
            nv,
        )?;
        let mut fixed = FixedTendonFields::default();
        let mut spatial = SpatialTendonFields::default();
        let mut subsets = Vec::with_capacity(nt);
        for t in 0..nt {
            let start = fields.tendon_adr[t] as usize;
            let end = start + fields.tendon_num[t] as usize;
            let columns = csr.row(t)?;
            let (adr, num, types, ids, prm, rowadr, rownnz, colind) =
                if fields.wrap_type[start] == 1 {
                    subsets.push(TendonSubset::Fixed(fixed.tendon_adr.len()));
                    (
                        &mut fixed.tendon_adr,
                        &mut fixed.tendon_num,
                        &mut fixed.wrap_type,
                        &mut fixed.wrap_objid,
                        &mut fixed.wrap_prm,
                        &mut fixed.ten_j_rowadr,
                        &mut fixed.ten_j_rownnz,
                        &mut fixed.ten_j_colind,
                    )
                } else {
                    subsets.push(TendonSubset::Spatial(spatial.tendon_adr.len()));
                    (
                        &mut spatial.tendon_adr,
                        &mut spatial.tendon_num,
                        &mut spatial.wrap_type,
                        &mut spatial.wrap_objid,
                        &mut spatial.wrap_prm,
                        &mut spatial.ten_j_rowadr,
                        &mut spatial.ten_j_rownnz,
                        &mut spatial.ten_j_colind,
                    )
                };
            adr.push(types.len() as i32);
            num.push((end - start) as i32);
            types.extend_from_slice(&fields.wrap_type[start..end]);
            ids.extend_from_slice(&fields.wrap_objid[start..end]);
            prm.extend_from_slice(&fields.wrap_prm[start..end]);
            rowadr.push(colind.len() as i32);
            rownnz.push(columns.len() as i32);
            colind.extend_from_slice(columns);
        }
        let fixed = FixedTendonModelInput::new(camlight, fixed)?;
        let spatial = match geometry {
            Some(g) => SpatialTendonModelInput::with_geometry(fixed, spatial, g)?,
            None => SpatialTendonModelInput::new(fixed, spatial)?,
        };
        Ok(Self {
            spatial,
            rows: TendonRows {
                csr,
                subsets,
                nwrap: nw,
            },
            fields,
        })
    }
    pub fn spatial(&self) -> &SpatialTendonModelInput {
        &self.spatial
    }
    pub fn rows(&self) -> &TendonRows {
        &self.rows
    }
    /// 保留检查后的原生路径。
    pub fn fields(&self) -> &TendonFields {
        &self.fields
    }
    pub(crate) fn into_parts(self) -> (SpatialTendonModelInput, TendonRows) {
        (self.spatial, self.rows)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn global_rows_preserve_zero_nnz_rows_and_reject_invalid_indices() {
        let rows = TendonRows {
            csr: FixedTendonRows::checked(vec![0, 0, 2], vec![0, 2, 0], vec![0, 1], 2).unwrap(),
            subsets: vec![
                TendonSubset::Spatial(0),
                TendonSubset::Fixed(0),
                TendonSubset::Spatial(1),
            ],
            nwrap: 7,
        };
        assert_eq!((rows.ntendon(), rows.nnz(), rows.nwrap()), (3, 2, 7));
        assert!(rows.row(0).unwrap().is_empty());
        assert_eq!(rows.row(1).unwrap(), [0, 1]);
        assert_eq!(rows.subset(2).unwrap(), TendonSubset::Spatial(1));
        for index in [3, usize::MAX] {
            assert!(matches!(
                rows.subset(index),
                Err(InputError::InvalidIndex {
                    field: "tendon",
                    ..
                })
            ));
        }
        assert!(TendonRows::default().subset(0).is_err());
    }
}
