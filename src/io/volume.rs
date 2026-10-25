//! 原生体积与SDF模型字段读取：hfield、mesh octree、`opt.sdf_*`。
//! 复用G01字段ABI（`mjwarp_native_g01_field_info`与`mjwarp_native_copy_g01_field`），
//! 不新增ABI结构、不分配设备缓冲、不使用GPU。
//! 字段号45..58与`native/g01_fields.inc`、`model::volume::VOLUME_FIELDS`同步。

use super::fields::native_staging;
use crate::{
    diagnostics::{InputError, NativeProbeError},
    model::{
        NativeModelInfo, VolumeModelInput,
        volume::{
            VOLUME_FIELDS, VolumeFieldData, VolumeFieldKind, VolumeFields, VolumeModelCounts,
            field_index,
        },
    },
};
use std::{ffi::c_void, mem::size_of};

/// 首个新增体积字段的G01字段号。
const BASE_ID: u32 = 45;

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

fn index(name: &str) -> Result<usize, NativeProbeError> {
    field_index(name).ok_or_else(|| {
        InputError::InvalidDimension {
            field: "volume_field_table",
        }
        .into()
    })
}

fn query(owner: *const c_void, index: usize) -> Result<FieldInfo, NativeProbeError> {
    let mut info = FieldInfo::default();
    // SAFETY: the caller retains the immutable owner and DLL; the bridge writes
    // exactly one aligned DTO and returns nonzero for an unknown field ID.
    status(
        unsafe { mjwarp_native_g01_field_info(owner, BASE_ID + index as u32, &mut info) },
        "g01_volume_field_info",
    )?;
    Ok(info)
}

fn checked_count(index: usize, info: FieldInfo) -> Result<usize, NativeProbeError> {
    let spec = &VOLUME_FIELDS[index];
    if info.reserved != 0 || info.kind != spec.kind.code() || info.count < 0 {
        return Err(InputError::InvalidDimension { field: spec.name }.into());
    }
    usize::try_from(info.count).map_err(|_| InputError::Overflow { field: spec.name }.into())
}

fn capture_field<T: Copy + Default>(
    owner: *const c_void,
    index: usize,
    info: &FieldInfo,
    count: usize,
) -> Result<Vec<T>, NativeProbeError> {
    let mut values: Vec<T> = native_staging(count)?;
    // SAFETY: the buffer owns exactly `count` scalars of T, the caller keeps the
    // immutable owner live, and the bridge rechecks kind, count and source.
    status(
        unsafe {
            mjwarp_native_copy_g01_field(
                owner,
                BASE_ID + index as u32,
                info,
                values.as_mut_ptr().cast::<c_void>(),
            )
        },
        "copy_g01_volume_field",
    )?;
    Ok(values)
}

fn derive_counts(infos: &[FieldInfo], noct: usize) -> Result<VolumeModelCounts, NativeProbeError> {
    Ok(VolumeModelCounts {
        nhfield: checked_count(index("hfield_nrow")?, infos[index("hfield_nrow")?])?,
        nhfielddata: checked_count(index("hfield_data")?, infos[index("hfield_data")?])?,
        noct,
        nmesh: checked_count(index("mesh_octadr")?, infos[index("mesh_octadr")?])?,
    })
}

/// 原生体积字段的独占快照；不借用模型或DLL。
#[derive(Clone, Debug, PartialEq)]
pub struct NativeVolumeSnapshot {
    info: NativeModelInfo,
    counts: VolumeModelCounts,
    fields: VolumeFields,
}

impl NativeVolumeSnapshot {
    /// 读取同一原生模型的全部体积与SDF字段。
    ///
    /// # Safety
    /// 调用者须保证所有者与DLL在调用期间存活且不被修改。
    pub(crate) unsafe fn capture(
        owner: *const c_void,
        info: NativeModelInfo,
    ) -> Result<Self, NativeProbeError> {
        info.validate()?;
        let mut infos = [FieldInfo::default(); VOLUME_FIELDS.len()];
        for (field, info) in infos.iter_mut().enumerate() {
            *info = query(owner, field)?;
            // Kind and non-negative count are checked before any allocation.
            checked_count(field, *info)?;
        }
        let noct_index = index("noct")?;
        let noct_count = checked_count(noct_index, infos[noct_index])?;
        if noct_count != 1 {
            return Err(InputError::LengthMismatch {
                field: "noct",
                expected: 1,
                actual: noct_count,
            }
            .into());
        }
        // SAFETY: same live immutable owner; the output owns one int64.
        let noct_value = capture_field::<i64>(owner, noct_index, &infos[noct_index], 1)?;
        let noct = usize::try_from(noct_value[0])
            .map_err(|_| InputError::InvalidDimension { field: "noct" })?;
        let counts = derive_counts(&infos, noct)?;
        let mut data = Vec::with_capacity(VOLUME_FIELDS.len());
        for (field, spec) in VOLUME_FIELDS.iter().enumerate() {
            let expected = counts.expected(spec)?;
            let actual = checked_count(field, infos[field])?;
            if expected != actual {
                return Err(InputError::LengthMismatch {
                    field: spec.name,
                    expected,
                    actual,
                }
                .into());
            }
            let value = match spec.kind {
                VolumeFieldKind::Int32 => {
                    VolumeFieldData::Int(capture_field(owner, field, &infos[field], actual)?)
                }
                VolumeFieldKind::Num => {
                    VolumeFieldData::Num(capture_field(owner, field, &infos[field], actual)?)
                }
                VolumeFieldKind::Float => {
                    VolumeFieldData::Float(capture_field(owner, field, &infos[field], actual)?)
                }
                VolumeFieldKind::Size if field == noct_index => {
                    VolumeFieldData::Size(noct_value.clone())
                }
                VolumeFieldKind::Size => {
                    VolumeFieldData::Size(capture_field(owner, field, &infos[field], actual)?)
                }
            };
            data.push(value);
        }
        Ok(Self {
            info,
            counts,
            fields: VolumeFields::new(data),
        })
    }

    /// 原生ABI身份与核心计数。
    pub fn info(&self) -> NativeModelInfo {
        self.info
    }

    /// 体积相关计数。
    pub fn counts(&self) -> VolumeModelCounts {
        self.counts
    }

    /// 按字段名取数据。
    pub fn field(&self, name: &str) -> Option<&VolumeFieldData> {
        self.fields.field(name)
    }

    /// 校验并转成体积模型输入。
    pub fn into_model(self) -> Result<VolumeModelInput, NativeProbeError> {
        VolumeModelInput::new(self.counts, self.fields)
    }
}
