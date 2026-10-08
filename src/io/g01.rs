//! 原生G01模型输入转换。
//! 不调用模型编译或CPU物理。
use super::{
    NativeFlexPositionSnapshot, NativeInertialSnapshot,
    fields::native_staging,
    kinematic::native::{check_info, convert, elements},
};
use crate::{
    diagnostics::{InputError, NativeProbeError},
    model::*,
};
use std::ffi::c_void;

/// 独占G01所需原生模型字段。
/// 快照不借用模型与DLL。
/// 它不表示完整put_model。
#[derive(Clone, Debug, PartialEq)]
pub struct NativeG01Snapshot {
    inertial: NativeInertialSnapshot,
    positions: NativeFlexPositionSnapshot,
    fields: Vec<NativeField>,
}
#[derive(Clone, Debug, PartialEq)]
enum NativeField {
    Int(Vec<i32>),
    Num(Vec<f64>),
    Bool(Vec<u8>),
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct FieldInfo {
    kind: u32,
    reserved: u32,
    count: i64,
}
const _: () = {
    assert!(size_of::<FieldInfo>() == 16);
    assert!(align_of::<FieldInfo>() == 8);
    assert!(std::mem::offset_of!(FieldInfo, count) == 8);
};
unsafe extern "C" {
    fn mjwarp_native_g01_field_info(owner: *const c_void, id: u32, info: *mut FieldInfo) -> i32;
    fn mjwarp_native_copy_g01_field(
        owner: *const c_void,
        id: u32,
        info: *const FieldInfo,
        target: *mut c_void,
    ) -> i32;
}
// Keep IDs equal to native/g01_fields.inc. Types are checked before allocation.
const FIELDS: [(&str, u32); 45] = [
    ("body_mocapid", 1),
    ("geom_bodyid", 1),
    ("site_bodyid", 1),
    ("cam_mode", 1),
    ("cam_bodyid", 1),
    ("cam_targetbodyid", 1),
    ("light_mode", 1),
    ("light_bodyid", 1),
    ("light_targetbodyid", 1),
    ("geom_type", 1),
    ("tendon_adr", 1),
    ("tendon_num", 1),
    ("wrap_type", 1),
    ("wrap_objid", 1),
    ("ten_J_rowadr", 1),
    ("ten_J_rownnz", 1),
    ("ten_J_colind", 1),
    ("body_treeid", 1),
    ("flex_edgeadr", 1),
    ("flex_edgenum", 1),
    ("flex_edge", 1),
    ("flexedge_J_rowadr", 1),
    ("flexedge_J_rownnz", 1),
    ("flexedge_J_colind", 1),
    ("geom_pos", 2),
    ("geom_quat", 2),
    ("site_pos", 2),
    ("site_quat", 2),
    ("cam_pos", 2),
    ("cam_quat", 2),
    ("cam_poscom0", 2),
    ("cam_pos0", 2),
    ("cam_mat0", 2),
    ("light_pos", 2),
    ("light_dir", 2),
    ("light_poscom0", 2),
    ("light_pos0", 2),
    ("light_dir0", 2),
    ("geom_size", 2),
    ("wrap_prm", 2),
    ("tendon_range", 2),
    ("tendon_margin", 2),
    ("tendon_limited", 3),
    ("enableflags", 1),
    ("disableflags", 1),
];
fn status(code: i32, stage: &'static str) -> Result<(), NativeProbeError> {
    if code == 0 {
        Ok(())
    } else {
        Err(NativeProbeError::Native {
            stage,
            code: code as u32,
        })
    }
}
fn count(info: FieldInfo, kind: u32) -> Result<usize, NativeProbeError> {
    if info.reserved != 0 || info.kind != kind || info.count < 0 {
        return Err(InputError::InvalidDimension {
            field: "native_g01_field",
        }
        .into());
    }
    Ok(elements(info.count, 1)?)
}
impl NativeG01Snapshot {
    pub(crate) unsafe fn capture(
        owner: *const c_void,
        inertial: NativeInertialSnapshot,
        positions: NativeFlexPositionSnapshot,
    ) -> Result<Self, NativeProbeError> {
        let mut infos = [FieldInfo::default(); FIELDS.len()];
        let mut total = 0usize;
        // Query every count before allocating any of the new staging buffers.
        for (id, ((_, kind), info)) in FIELDS.iter().zip(&mut infos).enumerate() {
            // SAFETY: The caller retains the immutable owner and library.
            // This query writes exactly one aligned, initialized DTO.
            status(
                unsafe { mjwarp_native_g01_field_info(owner, id as u32, info) },
                "g01_field_info",
            )?;
            total = total
                .checked_add(count(*info, *kind)?)
                .filter(|&n| n <= i32::MAX as usize)
                .ok_or(InputError::Overflow {
                    field: "native_g01_fields",
                })?;
        }
        let mut fields = Vec::with_capacity(FIELDS.len());
        for (id, ((_, kind), info)) in FIELDS.iter().zip(&infos).enumerate() {
            let n = count(*info, *kind)?;
            let mut field = match kind {
                1 => NativeField::Int(native_staging(n)?),
                2 => NativeField::Num(native_staging(n)?),
                3 => NativeField::Bool(native_staging(n)?),
                _ => unreachable!("checked scalar kind"),
            };
            let target = match &mut field {
                NativeField::Int(v) => v.as_mut_ptr().cast(),
                NativeField::Num(v) => v.as_mut_ptr().cast(),
                NativeField::Bool(v) => v.as_mut_ptr().cast(),
            };
            // SAFETY: The checked kind selects an aligned owned buffer with the
            // exact scalar capacity. It cannot alias the retained native model.
            // The bridge rechecks all counts, kinds and source pointers.
            status(
                unsafe { mjwarp_native_copy_g01_field(owner, id as u32, info, target) },
                "copy_g01_field",
            )?;
            fields.push(field);
        }
        Ok(Self {
            inertial,
            positions,
            fields,
        })
    }
    pub fn info(&self) -> NativeModelInfo {
        self.inertial.kinematics.info
    }
    fn ints(&self, id: usize) -> Result<Vec<i32>, NativeProbeError> {
        match self.fields.get(id) {
            Some(NativeField::Int(v)) => Ok(v.clone()),
            _ => Err(InputError::InvalidDimension {
                field: "native_g01_field_kind",
            }
            .into()),
        }
    }
    fn nums(&self, id: usize) -> Result<Vec<f32>, NativeProbeError> {
        match self.fields.get(id) {
            Some(NativeField::Num(v)) => convert(FIELDS[id].0, v),
            _ => Err(InputError::InvalidDimension {
                field: "native_g01_field_kind",
            }
            .into()),
        }
    }
    fn bools(&self, id: usize) -> Result<Vec<bool>, NativeProbeError> {
        let Some(NativeField::Bool(v)) = self.fields.get(id) else {
            return Err(InputError::InvalidDimension {
                field: "native_g01_field_kind",
            }
            .into());
        };
        v.iter()
            .enumerate()
            .map(|(index, &value)| match value {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(InputError::InvalidTopology {
                    field: FIELDS[id].0,
                    index,
                    reason: "noncanonical_boolean",
                }
                .into()),
            })
            .collect()
    }
    /// 转换同一原生模型的全部G01输入。
    /// 自动派生线性壳体面映射。
    /// 保留原生肌腱全局编号。
    pub fn into_model(self) -> Result<TendonWakeModelInput, NativeProbeError> {
        let info = self.info();
        check_info(info)?;
        if info != self.positions.info {
            return Err(InputError::InvalidDimension {
                field: "native_g01_model_identity",
            }
            .into());
        }
        let attached_fields = AttachedFields {
            geom_bodyid: self.ints(1)?,
            geom_pos: self.nums(24)?,
            geom_quat: self.nums(25)?,
            site_bodyid: self.ints(2)?,
            site_pos: self.nums(26)?,
            site_quat: self.nums(27)?,
        };
        if attached_fields.geom_bodyid.len() != info.ngeom as usize {
            return Err(InputError::InvalidDimension {
                field: "native_g01_ngeom",
            }
            .into());
        }
        let body_mocapid = self.ints(0)?;
        let camlight_fields = CamLightFields {
            cam_mode: self.ints(3)?,
            cam_bodyid: self.ints(4)?,
            cam_targetbodyid: self.ints(5)?,
            cam_pos: self.nums(28)?,
            cam_quat: self.nums(29)?,
            cam_poscom0: self.nums(30)?,
            cam_pos0: self.nums(31)?,
            cam_mat0: self.nums(32)?,
            light_mode: self.ints(6)?,
            light_bodyid: self.ints(7)?,
            light_targetbodyid: self.ints(8)?,
            light_pos: self.nums(33)?,
            light_dir: self.nums(34)?,
            light_poscom0: self.nums(35)?,
            light_pos0: self.nums(36)?,
            light_dir0: self.nums(37)?,
        };
        let tendon_fields = TendonFields {
            tendon_adr: self.ints(10)?,
            tendon_num: self.ints(11)?,
            wrap_type: self.ints(12)?,
            wrap_objid: self.ints(13)?,
            wrap_prm: self.nums(39)?,
            ten_j_rowadr: self.ints(14)?,
            ten_j_rownnz: self.ints(15)?,
            ten_j_colind: self.ints(16)?,
        };
        let nt = tendon_fields.tendon_adr.len();
        let geometry = SpatialTendonGeometry {
            geom_type: self.ints(9)?,
            geom_size: ParameterBatch::new(1, info.ngeom as usize * 3, self.nums(38)?)?,
        };
        let edges = FlexEdgeFields {
            flex_edgeadr: self.ints(18)?,
            flex_edgenum: self.ints(19)?,
            flex_edge: self.ints(20)?,
            flexedge_j_rowadr: self.ints(21)?,
            flexedge_j_rownnz: self.ints(22)?,
            flexedge_j_colind: self.ints(23)?,
        };
        let enable = self.ints(43)?;
        let disable = self.ints(44)?;
        super::check_length("enableflags", 1, enable.len())?;
        super::check_length("disableflags", 1, disable.len())?;
        let sleep = TendonWakeFields {
            body_treeid: self.ints(17)?,
            tendon_limited: self.bools(42)?,
            tendon_range: ParameterBatch::new(1, nt * 2, self.nums(40)?)?,
            tendon_margin: ParameterBatch::new(1, nt, self.nums(41)?)?,
            sleep_enabled: enable[0] & (1 << 4) != 0,
            island_disabled: disable[0] & (1 << 18) != 0,
        };
        let rigid = self.inertial.into_model()?;
        let attached = AttachedModelInput::new_equivalent(rigid, attached_fields)?;
        let mocap = MocapModelInput::new(attached, body_mocapid)?;
        let camlight = CamLightModelInput::new_equivalent(mocap, camlight_fields)?;
        let tendons = TendonModelInput::with_geometry(camlight, tendon_fields, geometry)?;
        let positions = self
            .positions
            .into_model(tendons)?
            .with_edges(edges)?
            .with_derived_faces()?;
        Ok(TendonWakeModelInput::with_flex_positions(positions, sleep)?)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_scalar_info_before_allocation() {
        for info in [
            FieldInfo {
                kind: 2,
                count: 1,
                reserved: 0,
            },
            FieldInfo {
                kind: 1,
                count: -1,
                reserved: 0,
            },
            FieldInfo {
                kind: 1,
                count: 1,
                reserved: 1,
            },
            FieldInfo {
                kind: 1,
                count: i64::MAX,
                reserved: 0,
            },
        ] {
            assert!(count(info, 1).is_err());
        }
        assert_eq!(
            count(
                FieldInfo {
                    kind: 1,
                    count: 0,
                    reserved: 0
                },
                1
            )
            .unwrap(),
            0
        );
        assert_eq!(FIELDS.len(), 45);
    }
}
