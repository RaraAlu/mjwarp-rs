//! 原生碰撞输入读取（S1）：32 项字段快照、mjData 帧快照与 f64→f32 宿主转换。
//! 只读：不分配设备缓冲、不使用GPU、不做产品接线（S3 才接线）。
//! 原生ABI见 `include/mjwarp_collision_inputs_probe.h`；字段规范见 `model::collision_inputs`。

use super::fields::native_staging;
use super::{NativeModelProbe, convert_f64_to_f32_into};
use crate::{
    diagnostics::{InputError, NativeProbeError},
    model::{
        NativeModelInfo,
        collision_inputs::{
            COLLISION_INPUTS_FIELDS, CollisionInputsCounts, CollisionInputsFieldKind,
            CollisionInputsFieldSource, CollisionInputsFieldSpec,
        },
    },
};
use std::{ffi::c_void, marker::PhantomData, mem::size_of, ptr::NonNull, rc::Rc};

/// 首个模型字段号；0/1 是 mjData 字段（geom_xpos/geom_xmat）。
const MODEL_FIELD_BASE: u32 = 2;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct FieldInfo {
    kind: u32,
    reserved: u32,
    count: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct InfoData {
    schema: u32,
    reserved: u32,
    nq: i64,
    ngeom: i64,
    nmesh: i64,
    nmeshvert: i64,
    nmeshgraph: i64,
    nmeshpoly: i64,
    nmeshpolyvert: i64,
    nmeshpolymap: i64,
    nhfield: i64,
    nhfielddata: i64,
    noct: i64,
}

const _: () = {
    assert!(size_of::<FieldInfo>() == 16);
    assert!(align_of::<FieldInfo>() == 8);
    assert!(std::mem::offset_of!(FieldInfo, count) == 8);
    assert!(size_of::<InfoData>() == 96);
    assert!(align_of::<InfoData>() == 8);
    assert!(std::mem::offset_of!(InfoData, nq) == 8);
    assert!(std::mem::offset_of!(InfoData, noct) == 88);
};

unsafe extern "C" {
    fn mjwarp_native_collision_inputs_info(owner: *const c_void, info: *mut InfoData) -> i32;
    fn mjwarp_native_collision_inputs_field_info(
        owner: *const c_void,
        id: u32,
        info: *mut FieldInfo,
    ) -> i32;
    fn mjwarp_native_copy_collision_inputs_field(
        owner: *const c_void,
        id: u32,
        info: *const FieldInfo,
        target: *mut c_void,
    ) -> i32;
    fn mjwarp_native_collision_inputs_create(owner: *const c_void, output: *mut *mut c_void)
    -> i32;
    fn mjwarp_native_collision_inputs_set_qpos(
        owner: *const c_void,
        data: *mut c_void,
        qpos: *const f64,
        nq: u64,
    ) -> i32;
    fn mjwarp_native_collision_inputs_copy_xpos_xmat(
        owner: *const c_void,
        data: *const c_void,
        geom_xpos: *mut f32,
        geom_xmat: *mut f32,
        ngeom: u64,
    ) -> i32;
    fn mjwarp_native_collision_inputs_destroy(owner: *const c_void, data: *mut c_void) -> i32;
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

fn counts_from(data: InfoData) -> Result<CollisionInputsCounts, NativeProbeError> {
    if data.schema != 1 || data.reserved != 0 {
        return Err(NativeProbeError::AbiMismatch);
    }
    let counts = CollisionInputsCounts {
        nq: data.nq,
        ngeom: data.ngeom,
        nmesh: data.nmesh,
        nmeshvert: data.nmeshvert,
        nmeshgraph: data.nmeshgraph,
        nmeshpoly: data.nmeshpoly,
        nmeshpolyvert: data.nmeshpolyvert,
        nmeshpolymap: data.nmeshpolymap,
        nhfield: data.nhfield,
        nhfielddata: data.nhfielddata,
        noct: data.noct,
    };
    counts.validate()?;
    Ok(counts)
}

fn query_field(owner: *const c_void, id: u32) -> Result<FieldInfo, NativeProbeError> {
    let mut info = FieldInfo::default();
    // SAFETY: the caller keeps the immutable owner and DLL alive; the bridge
    // writes one aligned DTO and rejects unknown field IDs.
    status(
        unsafe { mjwarp_native_collision_inputs_field_info(owner, id, &mut info) },
        "collision_inputs_field_info",
    )?;
    Ok(info)
}

fn checked_count(
    spec: &CollisionInputsFieldSpec,
    counts: CollisionInputsCounts,
    info: FieldInfo,
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

fn capture<T: Copy + Default>(
    owner: *const c_void,
    id: u32,
    info: &FieldInfo,
    count: usize,
) -> Result<Vec<T>, NativeProbeError> {
    let mut values: Vec<T> = native_staging(count)?;
    // SAFETY: the buffer owns exactly `count` scalars of T, the caller keeps the
    // immutable owner live, and the bridge rechecks kind, count and source.
    status(
        unsafe {
            mjwarp_native_copy_collision_inputs_field(
                owner,
                id,
                info,
                values.as_mut_ptr().cast::<c_void>(),
            )
        },
        "copy_collision_inputs_field",
    )?;
    Ok(values)
}

/// 单个字段的原始原生快照。
#[derive(Clone, Debug, PartialEq)]
pub enum CollisionInputsFieldData {
    /// 原生int32字段。
    Index(Vec<i32>),
    /// 原生double字段。
    Num(Vec<f64>),
    /// 原生float32字段。
    Float(Vec<f32>),
}

impl CollisionInputsFieldData {
    /// 标量个数。
    pub fn len(&self) -> usize {
        match self {
            Self::Index(values) => values.len(),
            Self::Num(values) => values.len(),
            Self::Float(values) => values.len(),
        }
    }

    /// 是否为空字段。
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// 单个字段的宿主侧（设备就绪）值：整数或f32。
#[derive(Clone, Debug, PartialEq)]
pub enum CollisionInputsFieldValue {
    Index(Vec<i32>),
    Float(Vec<f32>),
}

/// 模型静态字段的宿主输入；顺序与字段号 2..31 一致。
#[derive(Clone, Debug, PartialEq)]
pub struct CollisionInputsModelInput {
    counts: CollisionInputsCounts,
    fields: Vec<CollisionInputsFieldValue>,
}

impl CollisionInputsModelInput {
    /// 模型计数。
    pub fn counts(&self) -> CollisionInputsCounts {
        self.counts
    }

    /// 按原生字段号取字段值（0/1 属于 mjData，返回 None）。
    pub fn field(&self, id: u32) -> Option<&CollisionInputsFieldValue> {
        let index = id.checked_sub(MODEL_FIELD_BASE)?;
        self.fields.get(index as usize)
    }
}

/// 原生模型字段的独占快照；不借用模型或DLL。
#[derive(Clone, Debug, PartialEq)]
pub struct NativeCollisionInputsSnapshot {
    info: NativeModelInfo,
    counts: CollisionInputsCounts,
    fields: Vec<CollisionInputsFieldData>,
}

impl NativeCollisionInputsSnapshot {
    /// 读取同一原生模型的31项模型字段。
    ///
    /// # Safety
    /// 调用者须保证所有者与DLL在调用期间存活且不被修改。
    pub(crate) unsafe fn capture(
        owner: *const c_void,
        info: NativeModelInfo,
    ) -> Result<Self, NativeProbeError> {
        info.validate()?;
        let mut raw = InfoData::default();
        // SAFETY: the caller keeps the immutable owner and DLL alive.
        status(
            unsafe { mjwarp_native_collision_inputs_info(owner, &mut raw) },
            "collision_inputs_info",
        )?;
        let counts = counts_from(raw)?;
        let mut fields = Vec::with_capacity(COLLISION_INPUTS_FIELDS.len() - 2);
        for id in MODEL_FIELD_BASE..u32::try_from(COLLISION_INPUTS_FIELDS.len()).unwrap_or(u32::MAX)
        {
            let spec = &COLLISION_INPUTS_FIELDS[id as usize];
            if spec.source != CollisionInputsFieldSource::Model {
                return Err(InputError::InvalidDimension { field: spec.name }.into());
            }
            let info = query_field(owner, id)?;
            let count = checked_count(spec, counts, info)?;
            let data = match spec.kind {
                CollisionInputsFieldKind::Index => {
                    CollisionInputsFieldData::Index(capture(owner, id, &info, count)?)
                }
                CollisionInputsFieldKind::Num => {
                    CollisionInputsFieldData::Num(capture(owner, id, &info, count)?)
                }
                CollisionInputsFieldKind::Float => {
                    CollisionInputsFieldData::Float(capture(owner, id, &info, count)?)
                }
            };
            fields.push(data);
        }
        Ok(Self {
            info,
            counts,
            fields,
        })
    }

    /// 原生ABI身份与核心计数。
    pub fn info(&self) -> NativeModelInfo {
        self.info
    }

    /// 32项字段使用的模型计数。
    pub fn counts(&self) -> CollisionInputsCounts {
        self.counts
    }

    /// 按原生字段号取原始快照（0/1 属于 mjData，返回 None）。
    pub fn field(&self, id: u32) -> Option<&CollisionInputsFieldData> {
        let index = id.checked_sub(MODEL_FIELD_BASE)?;
        self.fields.get(index as usize)
    }

    /// 校验全部字段并收窄为宿主输入；f64→f32 走既有唯一转换入口。
    pub fn into_inputs(self) -> Result<CollisionInputsModelInput, NativeProbeError> {
        self.info.validate()?;
        self.counts.validate()?;
        let mut fields = Vec::with_capacity(self.fields.len());
        for (index, data) in self.fields.into_iter().enumerate() {
            let id = MODEL_FIELD_BASE + index as u32;
            let spec = &COLLISION_INPUTS_FIELDS[id as usize];
            if spec.kind.device_is_float() {
                let values = match data {
                    CollisionInputsFieldData::Num(values) => convert(spec.name, &values)?,
                    CollisionInputsFieldData::Float(values) => values,
                    CollisionInputsFieldData::Index(_) => {
                        return Err(InputError::InvalidDimension { field: spec.name }.into());
                    }
                };
                fields.push(CollisionInputsFieldValue::Float(values));
            } else {
                match data {
                    CollisionInputsFieldData::Index(values) => {
                        fields.push(CollisionInputsFieldValue::Index(values))
                    }
                    _ => return Err(InputError::InvalidDimension { field: spec.name }.into()),
                }
            }
        }
        Ok(CollisionInputsModelInput {
            counts: self.counts,
            fields,
        })
    }
}

fn convert(field: &'static str, values: &[f64]) -> Result<Vec<f32>, NativeProbeError> {
    let bytes = values
        .len()
        .checked_mul(size_of::<f32>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .ok_or(InputError::Overflow { field })?;
    let mut target = Vec::new();
    target
        .try_reserve_exact(values.len())
        .map_err(|_| NativeProbeError::HostAllocation { bytes })?;
    target.resize(values.len(), 0.0_f32);
    convert_f64_to_f32_into(values, &mut target).map_err(|error| match error {
        InputError::NonFinite { index, .. } => InputError::NonFinite { field, index },
        InputError::ScalarOverflow { index, .. } => InputError::ScalarOverflow { field, index },
        other => other,
    })?;
    Ok(target)
}

/// 一次 mjData 帧快照：geom_xpos（ngeom×3）与 geom_xmat（ngeom×9），f32。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CollisionInputsFrame {
    pub geom_xpos: Vec<f32>,
    pub geom_xmat: Vec<f32>,
}

/// 独立 mjData 句柄；生命周期受 `NativeModelProbe` 借用约束，必须先于所有者关闭。
pub struct CollisionInputsData<'a> {
    owner: *const c_void,
    handle: NonNull<c_void>,
    info: NativeModelInfo,
    _probe: PhantomData<&'a NativeModelProbe>,
    _thread: PhantomData<Rc<()>>,
}

impl<'a> CollisionInputsData<'a> {
    /// 创建mjData并缓存；同一次创建只重跑运动学。
    ///
    /// # Safety
    /// 调用者须保证 `probe` 的拥有者与DLL在句柄存活期间不解引用已释放内存。
    pub(crate) unsafe fn create(probe: &'a NativeModelProbe) -> Result<Self, NativeProbeError> {
        let mut handle = std::ptr::null_mut();
        // SAFETY: the borrow keeps the immutable owner and its DLL alive.
        status(
            unsafe { mjwarp_native_collision_inputs_create(probe.owner_ptr(), &mut handle) },
            "collision_inputs_create",
        )?;
        let handle = NonNull::new(handle).ok_or(NativeProbeError::AbiMismatch)?;
        Ok(Self {
            owner: probe.owner_ptr(),
            handle,
            info: probe.info(),
            _probe: PhantomData,
            _thread: PhantomData,
        })
    }

    /// 原生模型状态维度nq。
    pub fn nq(&self) -> i64 {
        self.info.nq
    }

    /// 原生模型几何数ngeom。
    pub fn ngeom(&self) -> i64 {
        self.info.ngeom
    }

    /// 写入调用方qpos并重跑mj_kinematics；长度必须等于nq。
    pub fn set_qpos(&mut self, qpos: &[f64]) -> Result<(), NativeProbeError> {
        if self.info.nq < 0 {
            return Err(InputError::InvalidDimension { field: "nq" }.into());
        }
        let expected =
            usize::try_from(self.info.nq).map_err(|_| InputError::Overflow { field: "nq" })?;
        if expected != qpos.len() {
            return Err(InputError::LengthMismatch {
                field: "qpos",
                expected,
                actual: qpos.len(),
            }
            .into());
        }
        if qpos.len() > u64::MAX as usize {
            return Err(InputError::Overflow { field: "qpos" }.into());
        }
        // SAFETY: borrowed handle, caller-kept owner, exact qpos length checked.
        status(
            unsafe {
                mjwarp_native_collision_inputs_set_qpos(
                    self.owner,
                    self.handle.as_ptr(),
                    qpos.as_ptr(),
                    qpos.len() as u64,
                )
            },
            "collision_inputs_set_qpos",
        )
    }

    /// 复制当前mjData的geom_xpos/geom_xmat为f32。
    pub fn capture_frame(&mut self) -> Result<CollisionInputsFrame, NativeProbeError> {
        if self.info.ngeom < 0 {
            return Err(InputError::InvalidDimension { field: "ngeom" }.into());
        }
        let ngeom = usize::try_from(self.info.ngeom)
            .map_err(|_| InputError::Overflow { field: "ngeom" })?;
        let positions = ngeom
            .checked_mul(3)
            .ok_or(InputError::Overflow { field: "geom_xpos" })?;
        let matrices = ngeom
            .checked_mul(9)
            .ok_or(InputError::Overflow { field: "geom_xmat" })?;
        let mut frame = CollisionInputsFrame {
            geom_xpos: native_staging(positions)?,
            geom_xmat: native_staging(matrices)?,
        };
        // SAFETY: both buffers own exact counts, the handle and owner are live,
        // and the bridge preflights ngeom and every pointer before writing.
        status(
            unsafe {
                mjwarp_native_collision_inputs_copy_xpos_xmat(
                    self.owner,
                    self.handle.as_ptr(),
                    frame.geom_xpos.as_mut_ptr(),
                    frame.geom_xmat.as_mut_ptr(),
                    ngeom as u64,
                )
            },
            "collision_inputs_copy_xpos_xmat",
        )?;
        Ok(frame)
    }
}

impl Drop for CollisionInputsData<'_> {
    fn drop(&mut self) {
        // SAFETY: this guard owns exactly one successful create; the bridge
        // checks the owner and never deletes mjData through a foreign owner.
        unsafe {
            mjwarp_native_collision_inputs_destroy(self.owner, self.handle.as_ptr());
        }
    }
}
