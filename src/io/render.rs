//! 原生渲染字段读取与宿主校验后的渲染模型输入。
//! 只读：不分配设备缓冲、不创建纹理对象、不建BVH、不使用GPU。
//! 原生ABI见 `include/mjwarp_render_probe.h`；字段规范见 `model::render`。

use super::fields::native_staging;
use crate::{
    diagnostics::{InputError, NativeProbeError},
    model::{
        NativeModelInfo,
        render::{
            FieldData, FieldSpec, RENDER_FIELDS, RenderFieldKind, RenderFields, RenderModelCounts,
            RenderObject,
        },
    },
};
use std::{ffi::c_void, mem::size_of};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RenderInfoData {
    schema: u32,
    reserved: u32,
    nbody: i64,
    njnt: i64,
    ngeom: i64,
    nsite: i64,
    ncam: i64,
    nlight: i64,
    nmat: i64,
    ntex: i64,
    ntexdata: i64,
    nmesh: i64,
    nmeshvert: i64,
    nmeshnormal: i64,
    nmeshtexcoord: i64,
    nmeshface: i64,
    nmeshgraph: i64,
    nmeshpoly: i64,
    nmeshpolyvert: i64,
    nmeshpolymap: i64,
    nnames: i64,
}

const _: () = {
    assert!(size_of::<RenderInfoData>() == 160);
    assert!(align_of::<RenderInfoData>() == 8);
    assert!(std::mem::offset_of!(RenderInfoData, nbody) == 8);
    assert!(std::mem::offset_of!(RenderInfoData, nnames) == 152);
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct RenderFieldInfo {
    kind: u32,
    reserved: u32,
    count: i64,
}

const _: () = {
    assert!(size_of::<RenderFieldInfo>() == 16);
    assert!(align_of::<RenderFieldInfo>() == 8);
    assert!(std::mem::offset_of!(RenderFieldInfo, count) == 8);
};

unsafe extern "C" {
    fn mjwarp_native_render_info(owner: *const c_void, info: *mut RenderInfoData) -> i32;
    fn mjwarp_native_render_field_info(
        owner: *const c_void,
        id: u32,
        info: *mut RenderFieldInfo,
    ) -> i32;
    fn mjwarp_native_copy_render_field(
        owner: *const c_void,
        id: u32,
        info: *const RenderFieldInfo,
        target: *mut c_void,
    ) -> i32;
    fn mjwarp_native_render_name(
        owner: *const c_void,
        objtype: i32,
        id: i64,
        buffer: *mut std::ffi::c_char,
        capacity: i64,
        length: *mut i64,
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

fn counts_from(data: RenderInfoData) -> Result<RenderModelCounts, NativeProbeError> {
    if data.schema != 1 || data.reserved != 0 {
        return Err(NativeProbeError::AbiMismatch);
    }
    Ok(RenderModelCounts {
        nbody: data.nbody,
        njnt: data.njnt,
        ngeom: data.ngeom,
        nsite: data.nsite,
        ncam: data.ncam,
        nlight: data.nlight,
        nmat: data.nmat,
        ntex: data.ntex,
        ntexdata: data.ntexdata,
        nmesh: data.nmesh,
        nmeshvert: data.nmeshvert,
        nmeshnormal: data.nmeshnormal,
        nmeshtexcoord: data.nmeshtexcoord,
        nmeshface: data.nmeshface,
        nmeshgraph: data.nmeshgraph,
        nmeshpoly: data.nmeshpoly,
        nmeshpolyvert: data.nmeshpolyvert,
        nmeshpolymap: data.nmeshpolymap,
        nnames: data.nnames,
    })
}

fn checked_count(
    spec: &FieldSpec,
    counts: &RenderModelCounts,
    info: RenderFieldInfo,
) -> Result<usize, NativeProbeError> {
    if info.reserved != 0 || info.kind != spec.kind.code() || info.count < 0 {
        return Err(InputError::InvalidDimension { field: spec.name }.into());
    }
    let expected = counts.expected(spec)?;
    if i64::try_from(expected).ok() != Some(info.count) {
        return Err(InputError::LengthMismatch {
            field: spec.name,
            expected,
            actual: usize::try_from(info.count).unwrap_or(usize::MAX),
        }
        .into());
    }
    Ok(expected)
}

fn capture_field<T: Copy + Default>(
    owner: *const c_void,
    id: u32,
    info: &RenderFieldInfo,
    count: usize,
) -> Result<Vec<T>, NativeProbeError> {
    let mut values: Vec<T> = native_staging(count)?;
    status(
        unsafe {
            mjwarp_native_copy_render_field(owner, id, info, values.as_mut_ptr().cast::<c_void>())
        },
        "copy_render_field",
    )?;
    Ok(values)
}

/// 原生渲染字段的独占快照；不借用模型或DLL。
#[derive(Clone, Debug, PartialEq)]
pub struct NativeRenderSnapshot {
    info: NativeModelInfo,
    counts: RenderModelCounts,
    fields: RenderFields,
}

impl NativeRenderSnapshot {
    /// 读取同一原生模型的全部渲染字段。
    ///
    /// # Safety
    /// 调用者须保证所有者与DLL在调用期间存活且不被修改。
    pub(crate) unsafe fn capture(
        owner: *const c_void,
        info: NativeModelInfo,
    ) -> Result<Self, NativeProbeError> {
        info.validate()?;
        let mut raw = RenderInfoData::default();
        // SAFETY: the caller keeps the immutable owner live; one aligned DTO is written.
        status(
            unsafe { mjwarp_native_render_info(owner, &mut raw) },
            "render_info",
        )?;
        let counts = counts_from(raw)?;
        counts.validate()?;
        let mut data = Vec::with_capacity(RENDER_FIELDS.len());
        for (id, spec) in RENDER_FIELDS.iter().enumerate() {
            let mut field = RenderFieldInfo::default();
            status(
                unsafe { mjwarp_native_render_field_info(owner, id as u32, &mut field) },
                "render_field_info",
            )?;
            let count = checked_count(spec, &counts, field)?;
            let value = match spec.kind {
                RenderFieldKind::Index => {
                    FieldData::Index(capture_field(owner, id as u32, &field, count)?)
                }
                RenderFieldKind::Num => {
                    FieldData::Num(capture_field(owner, id as u32, &field, count)?)
                }
                RenderFieldKind::Flag => {
                    FieldData::Flag(capture_field(owner, id as u32, &field, count)?)
                }
                RenderFieldKind::Size => {
                    FieldData::Size(capture_field(owner, id as u32, &field, count)?)
                }
                RenderFieldKind::Float => {
                    FieldData::Float(capture_field(owner, id as u32, &field, count)?)
                }
            };
            data.push(value);
        }
        Ok(Self {
            info,
            counts,
            fields: RenderFields::new(data),
        })
    }

    pub fn into_model(self) -> Result<RenderModelInput, NativeProbeError> {
        RenderModelInput::new(self.info, self.counts, self.fields)
    }
}

/// 渲染模型输入；构造时校验计数、长度、范围与名称表。
/// 不表示完整 put_model。
#[derive(Clone, Debug, PartialEq)]
pub struct RenderModelInput {
    info: NativeModelInfo,
    counts: RenderModelCounts,
    fields: RenderFields,
}

impl RenderModelInput {
    /// 校验并构造渲染模型输入。
    pub fn new(
        info: NativeModelInfo,
        counts: RenderModelCounts,
        fields: RenderFields,
    ) -> Result<Self, NativeProbeError> {
        info.validate()?;
        counts.validate()?;
        if fields.len() != RENDER_FIELDS.len() {
            return Err(InputError::LengthMismatch {
                field: "render_fields",
                expected: RENDER_FIELDS.len(),
                actual: fields.len(),
            }
            .into());
        }
        for (index, spec) in RENDER_FIELDS.iter().enumerate() {
            let Some(data) = fields.get(index) else {
                return Err(InputError::LengthMismatch {
                    field: "render_fields",
                    expected: RENDER_FIELDS.len(),
                    actual: fields.len(),
                }
                .into());
            };
            let expected = counts.expected(spec)?;
            if data.kind() != spec.kind {
                return Err(InputError::InvalidDimension { field: spec.name }.into());
            }
            if data.len() != expected {
                return Err(InputError::LengthMismatch {
                    field: spec.name,
                    expected,
                    actual: data.len(),
                }
                .into());
            }
        }
        let input = Self {
            info,
            counts,
            fields,
        };
        crate::model::validate_render_tables(input.counts, &input.fields)?;
        input.check_mesh_ranges()?;
        Ok(input)
    }

    /// 原生ABI身份与核心计数。
    pub fn info(&self) -> NativeModelInfo {
        self.info
    }

    /// 渲染相关计数。
    pub fn counts(&self) -> RenderModelCounts {
        self.counts
    }

    /// 按字段名取数据。
    pub fn field(&self, name: &str) -> Option<&FieldData> {
        self.fields.field(name)
    }

    /// 名称字节；语义与 `mj_id2name` 一致。
    pub fn name_bytes(&self, object: RenderObject, id: i64) -> Option<&[u8]> {
        self.fields.name_bytes(object, id)
    }

    /// UTF-8名称；非UTF-8名称返回 `None`。
    pub fn name(&self, object: RenderObject, id: i64) -> Option<&str> {
        self.fields.name(object, id)
    }

    fn ints(&self, name: &str) -> Result<&[i32], NativeProbeError> {
        match self.fields.field(name) {
            Some(FieldData::Index(v)) => Ok(v),
            _ => Err(InputError::InvalidDimension {
                field: "render_field_kind",
            }
            .into()),
        }
    }

    fn check_mesh_ranges(&self) -> Result<(), NativeProbeError> {
        let ranges = [
            (
                "mesh_vert",
                "mesh_vertadr",
                "mesh_vertnum",
                self.counts.nmeshvert,
            ),
            (
                "mesh_face",
                "mesh_faceadr",
                "mesh_facenum",
                self.counts.nmeshface,
            ),
            (
                "mesh_normal",
                "mesh_normaladr",
                "mesh_normalnum",
                self.counts.nmeshnormal,
            ),
            (
                "mesh_texcoord",
                "mesh_texcoordadr",
                "mesh_texcoordnum",
                self.counts.nmeshtexcoord,
            ),
            (
                "mesh_poly",
                "mesh_polyadr",
                "mesh_polynum",
                self.counts.nmeshpoly,
            ),
            (
                "mesh_polyvert",
                "mesh_polyvertadr",
                "mesh_polyvertnum",
                self.counts.nmeshpolyvert,
            ),
            (
                "mesh_polymap",
                "mesh_polymapadr",
                "mesh_polymapnum",
                self.counts.nmeshpolymap,
            ),
        ];
        for (field, start_field, count_field, capacity) in ranges {
            let starts = self.ints(start_field)?;
            let counts = self.ints(count_field)?;
            for (&start, &count) in starts.iter().zip(counts) {
                let start_ok = if field == "mesh_texcoord" && count == 0 {
                    start >= -1
                } else {
                    start >= 0
                };
                let end = i64::from(start).checked_add(i64::from(count));
                if !start_ok || count < 0 || end.is_none_or(|end| end > capacity) {
                    return Err(InputError::InvalidRange {
                        field,
                        offset: usize::try_from(start).unwrap_or(0),
                        elements: usize::try_from(count).unwrap_or(0),
                        capacity: usize::try_from(capacity).unwrap_or(usize::MAX),
                    }
                    .into());
                }
            }
        }
        Ok(())
    }
}

/// `mj_id2name` 语义的独立对照；返回名称字节或 `None`。
///
/// # Safety
/// 调用者须保证所有者与DLL在调用期间存活且不被修改。
pub(crate) unsafe fn native_name(
    owner: *const c_void,
    objtype: i32,
    id: i64,
) -> Result<Option<Vec<u8>>, NativeProbeError> {
    let mut raw = RenderInfoData::default();
    // SAFETY: same live immutable owner as the caller.
    status(
        unsafe { mjwarp_native_render_info(owner, &mut raw) },
        "render_info",
    )?;
    let counts = counts_from(raw)?;
    if counts.nnames < 0 {
        return Err(InputError::InvalidDimension { field: "nnames" }.into());
    }
    let capacity = counts
        .nnames
        .checked_add(1)
        .ok_or(InputError::Overflow { field: "nnames" })?;
    let mut buffer = native_staging::<u8>(
        usize::try_from(capacity).map_err(|_| InputError::Overflow { field: "nnames" })?,
    )?;
    let mut length = 0i64;
    // SAFETY: the buffer owns capacity bytes; the bridge writes at most capacity.
    let code = unsafe {
        mjwarp_native_render_name(
            owner,
            objtype,
            id,
            buffer.as_mut_ptr().cast(),
            capacity,
            &mut length,
        )
    };
    match code {
        0 => {
            let length = usize::try_from(length).map_err(|_| InputError::Overflow {
                field: "render_name",
            })?;
            if length >= buffer.len() {
                return Err(InputError::LengthMismatch {
                    field: "render_name",
                    expected: length + 1,
                    actual: buffer.len(),
                }
                .into());
            }
            buffer.truncate(length);
            Ok(Some(buffer))
        }
        1 => Ok(None),
        2 => Err(InputError::LengthMismatch {
            field: "render_name",
            expected: usize::try_from(length).unwrap_or(usize::MAX),
            actual: buffer.len(),
        }
        .into()),
        other => status(other, "render_name").map(|_| None),
    }
}
