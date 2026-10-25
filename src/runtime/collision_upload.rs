//! G10@P5 G5：碰撞/体积模型字段的上传布局与接触容量视图（`nworld` 维口径）。
//!
//! ## 模型静态字段：一份共享缓冲（与 world 数无关）
//!
//! G1 规格（`docs/development-evidence/g10-p5-model-fields/README.md` §6）：`hfield_data`、
//! `hfield_nrow/ncol/size/adr`、`oct_depth/child/aabb/coeff`、`mesh_octadr/octnum`、`noct`、
//! `opt_sdf_*` 都是模型静态数据，与 world 无关，**按一份共享缓冲上传**（所有 world 同一基址），
//! 不做逐 world 复制。上游 `io.py` 实测一致：这些字段的形状不带 world 维
//! （如 `hfield_size: array("nhfield", wp.vec4)`、`oct_aabb: array("noct", 2, wp.vec3)`）。
//! 本模块只负责把这些字段打包成一份共享 payload（f64→f32 转换仍走 G1 的唯一转换入口）。
//!
//! ## 接触容量：全局行编号（G1 §6 的「逐 world 分段」预留与上游实测不符）
//!
//! 上游 `io.py` 实测：`nacon: array(1)` 是**全局单计数器**，接触数组形状为 `(naconmax, ...)`，
//! 由 `wp.atomic_add(nacon, 0, 1)` 分配**全局行号**，world 归属存在 `contact_worldid` 字段；
//! `naconmax` 是全部 world 的总行数（`nconmax * nworld` 只是默认换算），**没有**
//! 「每 world 一段 + kernel 按 world 计算行偏移」的布局。`naccdmax` 同理是全局总量，
//! `nccd` 计数按 `geomgeomid` 分组、容量判定 `ccdid >= naccdmax`。
//! 本模块按上游实测口径提供容量视图（`naconmax/naccdmax` 总行、全局行号、`efc_address` 宽度
//! `nmaxpyramid = max(1, 2*(nmaxcondim-1))`），并在证据 README 记录与 G1 §6 的差异。
//!
//! 纯 CPU 侧布局/校验：不分配设备缓冲、不需要 GPU；GPU 窗口阶段的 launcher 直接消费
//! `CollisionUploadPlan::pack` 的 payload 与 `ContactCapacity` 的行号规则。

#![allow(dead_code)]

use crate::{
    diagnostics::{InputError, NativeProbeError},
    model::VolumeModelInput,
};

/// 上传标量类型（体积/SDF 模型字段只有 i32 与 f32）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UploadScalar {
    I32,
    F32,
}

impl UploadScalar {
    /// 单标量字节数。
    pub const fn bytes(self) -> usize {
        match self {
            Self::I32 | Self::F32 => 4,
        }
    }

    /// 字段名里的设备侧类型标签（u8 布局，供证据/launcher 打印）。
    pub const fn tag(self) -> &'static str {
        match self {
            Self::I32 => "i32",
            Self::F32 => "f32",
        }
    }
}

/// 共享缓冲里的一个字段。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedField {
    pub name: &'static str,
    pub scalar: UploadScalar,
    /// 标量个数（已含分量宽度）。
    pub elements: usize,
    /// 在共享 payload 中的字节偏移（16 字节对齐）。
    pub offset_bytes: usize,
}

impl SharedField {
    /// 字段字节数。
    pub fn bytes(&self) -> usize {
        self.elements * self.scalar.bytes()
    }
}

/// 字段对齐粒度。
const FIELD_ALIGN: usize = 16;

fn align_up(value: usize) -> Result<usize, NativeProbeError> {
    let mask = FIELD_ALIGN - 1;
    value
        .checked_add(mask)
        .map(|padded| padded & !mask)
        .ok_or_else(|| {
            InputError::Overflow {
                field: "upload_layout",
            }
            .into()
        })
}

fn scaled(count: usize, width: usize, field: &'static str) -> Result<usize, NativeProbeError> {
    count
        .checked_mul(width)
        .ok_or_else(|| InputError::Overflow { field }.into())
}

/// 体积/SDF 模型字段的共享上传布局（字段顺序固定，与 G1 字段表一致）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollisionUploadPlan {
    fields: Vec<SharedField>,
    buffer_bytes: usize,
}

impl CollisionUploadPlan {
    /// 按 G1 校验后的体积模型输入构建共享布局；`nworld` 不参与（一份共享缓冲）。
    pub fn new(volume: &VolumeModelInput) -> Result<Self, NativeProbeError> {
        let counts = volume.counts();
        let nhfield = counts.nhfield;
        let nhfielddata = counts.nhfielddata;
        let noct = counts.noct;
        let nmesh = counts.nmesh;

        let specs: [(&'static str, UploadScalar, usize); 14] = [
            ("hfield_nrow", UploadScalar::I32, nhfield),
            ("hfield_ncol", UploadScalar::I32, nhfield),
            (
                "hfield_size",
                UploadScalar::F32,
                scaled(nhfield, 4, "hfield_size")?,
            ),
            ("hfield_adr", UploadScalar::I32, nhfield),
            ("hfield_data", UploadScalar::F32, nhfielddata),
            // `noct` 上传为 int32；超出 i32 由 G1 `noct_i32` 拒绝。
            ("noct", UploadScalar::I32, 1),
            ("oct_depth", UploadScalar::I32, noct),
            (
                "oct_child",
                UploadScalar::I32,
                scaled(noct, 8, "oct_child")?,
            ),
            ("oct_aabb", UploadScalar::F32, scaled(noct, 6, "oct_aabb")?),
            (
                "oct_coeff",
                UploadScalar::F32,
                scaled(noct, 8, "oct_coeff")?,
            ),
            ("mesh_octadr", UploadScalar::I32, nmesh),
            ("mesh_octnum", UploadScalar::I32, nmesh),
            ("opt_sdf_iterations", UploadScalar::I32, 1),
            ("opt_sdf_initpoints", UploadScalar::I32, 1),
        ];

        let mut fields = Vec::with_capacity(specs.len());
        let mut offset = 0_usize;
        for (name, scalar, elements) in specs {
            offset = align_up(offset)?;
            let bytes = scaled(elements, scalar.bytes(), name)?;
            fields.push(SharedField {
                name,
                scalar,
                elements,
                offset_bytes: offset,
            });
            offset = offset
                .checked_add(bytes)
                .ok_or_else(|| NativeProbeError::from(InputError::Overflow { field: name }))?;
        }
        Ok(Self {
            fields,
            buffer_bytes: offset,
        })
    }

    /// 全部字段（顺序固定）。
    pub fn fields(&self) -> &[SharedField] {
        &self.fields
    }

    /// 按名取字段。
    pub fn field(&self, name: &str) -> Option<&SharedField> {
        self.fields.iter().find(|field| field.name == name)
    }

    /// 共享 payload 字节数。
    pub fn buffer_bytes(&self) -> usize {
        self.buffer_bytes
    }

    /// 共享行数：模型静态字段恒为 1 份（所有 world 同一基址）。
    pub const fn shared_rows(&self) -> usize {
        1
    }

    /// 打包共享 payload（小端字节；f64→f32 只走 G1 的唯一转换入口）。
    pub fn pack(&self, volume: &VolumeModelInput) -> Result<Vec<u8>, NativeProbeError> {
        let mut bytes = vec![0_u8; self.buffer_bytes];
        for field in &self.fields {
            match field.scalar {
                UploadScalar::I32 => {
                    let values = int_values(volume, field.name)?;
                    if values.len() != field.elements {
                        return Err(InputError::LengthMismatch {
                            field: field.name,
                            expected: field.elements,
                            actual: values.len(),
                        }
                        .into());
                    }
                    for (index, value) in values.iter().enumerate() {
                        let at = field.offset_bytes + index * 4;
                        bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
                    }
                }
                UploadScalar::F32 => {
                    let values = float_values(volume, field.name)?;
                    if values.len() != field.elements {
                        return Err(InputError::LengthMismatch {
                            field: field.name,
                            expected: field.elements,
                            actual: values.len(),
                        }
                        .into());
                    }
                    for (index, value) in values.iter().enumerate() {
                        let at = field.offset_bytes + index * 4;
                        bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
                    }
                }
            }
        }
        Ok(bytes)
    }
}

fn int_values<'a>(
    volume: &'a VolumeModelInput,
    name: &'static str,
) -> Result<std::borrow::Cow<'a, [i32]>, NativeProbeError> {
    match name {
        "noct" => Ok(std::borrow::Cow::Owned(vec![volume.noct_i32()?])),
        _ => Ok(std::borrow::Cow::Borrowed(volume.ints(name)?)),
    }
}

fn float_values<'a>(
    volume: &'a VolumeModelInput,
    name: &'static str,
) -> Result<std::borrow::Cow<'a, [f32]>, NativeProbeError> {
    match name {
        "hfield_size" => Ok(std::borrow::Cow::Owned(volume.hfield_size_f32()?)),
        "oct_aabb" => Ok(std::borrow::Cow::Owned(volume.oct_aabb_f32()?)),
        "oct_coeff" => Ok(std::borrow::Cow::Owned(volume.oct_coeff_f32()?)),
        "hfield_data" => Ok(std::borrow::Cow::Borrowed(volume.hfield_data()?)),
        _ => Err(InputError::InvalidDimension { field: name }.into()),
    }
}

/// 接触/CCD 容量视图（上游实测：**全局**行编号，不是逐 world 分段）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContactCapacity {
    nworld: usize,
    /// 全部 world 的接触总行数（上游 `naconmax`）。
    pub naconmax: usize,
    /// 全部 world 的 CCD/EPA 总量（上游 `naccdmax`）。
    pub naccdmax: usize,
    /// 上游 `nmaxpyramid = max(1, 2*(nmaxcondim-1))`。
    pub efc_address_width: usize,
}

impl ContactCapacity {
    /// 上游 `put_data`/`_resolve_batch_size` 口径：`nconmax`/`nccdmax` 按 world 给，
    /// `naconmax`/`naccdmax` 是换算出的**全局总量**。
    pub fn from_per_world(
        nworld: usize,
        nconmax: usize,
        nccdmax: Option<usize>,
        nmaxcondim: usize,
    ) -> Result<Self, NativeProbeError> {
        if nworld == 0 {
            return Err(InputError::InvalidDimension { field: "nworld" }.into());
        }
        if nmaxcondim == 0 {
            return Err(InputError::InvalidDimension {
                field: "nmaxcondim",
            }
            .into());
        }
        let naconmax = nconmax
            .checked_mul(nworld)
            .ok_or_else(|| NativeProbeError::from(InputError::Overflow { field: "naconmax" }))?;
        let per_world_ccd = nccdmax.unwrap_or(nconmax);
        if per_world_ccd > nconmax {
            return Err(InputError::InvalidDimension { field: "nccdmax" }.into());
        }
        let naccdmax = per_world_ccd
            .checked_mul(nworld)
            .ok_or_else(|| NativeProbeError::from(InputError::Overflow { field: "naccdmax" }))?;
        if naccdmax > naconmax {
            return Err(InputError::InvalidDimension { field: "naccdmax" }.into());
        }
        let efc_address_width = (nmaxcondim - 1)
            .checked_mul(2)
            .ok_or_else(|| {
                NativeProbeError::from(InputError::Overflow {
                    field: "nmaxpyramid",
                })
            })?
            .max(1);
        Ok(Self {
            nworld,
            naconmax,
            naccdmax,
            efc_address_width,
        })
    }

    /// world 数。
    pub const fn nworld(&self) -> usize {
        self.nworld
    }

    /// 接触行号：上游行号是**全局** `cid`（`nacon` 原子计数返回值），
    /// 不存在「world × 每 world 容量 + cid」的逐 world 偏移；world 归属存 `contact_worldid`。
    pub fn contact_row_for_world(&self, worldid: usize, cid: usize) -> Option<usize> {
        (worldid < self.nworld && cid < self.naconmax).then_some(cid)
    }

    /// `efc_address` 元素下标：`cid * efc_address_width + dim`。
    pub fn efc_address_index(&self, cid: usize, dim: usize) -> Option<usize> {
        if cid < self.naconmax && dim < self.efc_address_width {
            Some(cid * self.efc_address_width + dim)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{VolumeFieldData, VolumeFields, VolumeModelCounts};

    fn volume_counts() -> VolumeModelCounts {
        VolumeModelCounts {
            nhfield: 1,
            nhfielddata: 9,
            noct: 2,
            nmesh: 1,
        }
    }

    fn volume_data(noct: usize) -> Vec<VolumeFieldData> {
        vec![
            VolumeFieldData::Int(vec![3]),
            VolumeFieldData::Int(vec![3]),
            VolumeFieldData::Num(vec![1.0, 1.0, 0.1, 0.1]),
            VolumeFieldData::Int(vec![0]),
            VolumeFieldData::Float(vec![0.5; 9]),
            VolumeFieldData::Size(vec![noct as i64]),
            VolumeFieldData::Int((0..noct).map(|value| value as i32).collect()),
            VolumeFieldData::Int((0..8 * noct).map(|value| value as i32).collect()),
            VolumeFieldData::Num((0..6 * noct).map(|value| value as f64 * 0.25).collect()),
            VolumeFieldData::Num((0..8 * noct).map(|value| value as f64 * -0.5).collect()),
            VolumeFieldData::Int(vec![0]),
            VolumeFieldData::Int(vec![noct as i32]),
            VolumeFieldData::Int(vec![10]),
            VolumeFieldData::Int(vec![40]),
        ]
    }

    fn volume() -> VolumeModelInput {
        let counts = volume_counts();
        VolumeModelInput::new(counts, VolumeFields::new(volume_data(counts.noct))).unwrap()
    }

    #[test]
    fn shared_layout_is_independent_of_world_count() {
        let volume = volume();
        let plan = CollisionUploadPlan::new(&volume).unwrap();
        // 模型静态字段只有 1 份共享行（不存在逐 world 复制）。
        assert_eq!(plan.shared_rows(), 1);
        // world 数只影响容量视图，不影响共享 payload 布局。
        let two = ContactCapacity::from_per_world(2, 64, None, 3).unwrap();
        let eight = ContactCapacity::from_per_world(8, 64, None, 3).unwrap();
        assert_eq!(two.naconmax, 128);
        assert_eq!(eight.naconmax, 512);
        let payload = plan.pack(&volume).unwrap();
        assert_eq!(payload.len(), plan.buffer_bytes());
    }

    #[test]
    fn contact_rows_are_global_not_per_world_segmented() {
        let capacity = ContactCapacity::from_per_world(3, 10, None, 3).unwrap();
        assert_eq!(capacity.nworld(), 3);
        assert_eq!(capacity.naconmax, 30);
        assert_eq!(capacity.naccdmax, 30);
        // 同一全局 cid 在任意 world 都映射到同一行（world 归属存 `contact_worldid` 字段）；
        // 上游没有 world 分段偏移。
        for worldid in 0..3 {
            assert_eq!(capacity.contact_row_for_world(worldid, 7), Some(7));
        }
        assert_eq!(capacity.contact_row_for_world(3, 7), None);
        assert_eq!(capacity.contact_row_for_world(0, 30), None);
        // `efc_address` 宽度 = max(1, 2*(nmaxcondim-1))；nmaxcondim=3 → 4。
        assert_eq!(capacity.efc_address_width, 4);
        assert_eq!(capacity.efc_address_index(0, 3), Some(3));
        assert_eq!(capacity.efc_address_index(0, 4), None);
    }

    #[test]
    fn derives_capacity_and_rejects_invalid_configurations() {
        let capacity = ContactCapacity::from_per_world(4, 64, Some(16), 6).unwrap();
        assert_eq!(capacity.naconmax, 256);
        assert_eq!(capacity.naccdmax, 64);
        // 上游 `nmaxpyramid = max(1, 2*(nmaxcondim-1))`：nmaxcondim=6 → 10。
        assert_eq!(capacity.efc_address_width, 10);
        assert_eq!(
            ContactCapacity::from_per_world(1, 8, None, 1)
                .unwrap()
                .efc_address_width,
            1
        );

        assert!(ContactCapacity::from_per_world(0, 64, None, 3).is_err());
        assert!(ContactCapacity::from_per_world(1, 64, None, 0).is_err());
        // 上游约束：nccdmax <= nconmax。
        assert!(ContactCapacity::from_per_world(2, 64, Some(65), 3).is_err());
        // 乘法溢出必须报错而不是回绕。
        assert!(ContactCapacity::from_per_world(usize::MAX, 2, None, 3).is_err());
    }

    #[test]
    fn packs_shared_payload_from_g1_conversion_points() {
        let volume = volume();
        let plan = CollisionUploadPlan::new(&volume).unwrap();
        let payload = plan.pack(&volume).unwrap();

        let field = plan.field("hfield_size").unwrap();
        let expected = volume.hfield_size_f32().unwrap();
        for (index, value) in expected.iter().enumerate() {
            let at = field.offset_bytes + index * 4;
            let mut bits = [0_u8; 4];
            bits.copy_from_slice(&payload[at..at + 4]);
            assert_eq!(f32::from_le_bytes(bits).to_bits(), value.to_bits());
        }

        let field = plan.field("oct_aabb").unwrap();
        let expected = volume.oct_aabb_f32().unwrap();
        for (index, value) in expected.iter().enumerate() {
            let at = field.offset_bytes + index * 4;
            let mut bits = [0_u8; 4];
            bits.copy_from_slice(&payload[at..at + 4]);
            assert_eq!(f32::from_le_bytes(bits).to_bits(), value.to_bits());
        }

        let field = plan.field("noct").unwrap();
        assert_eq!(field.elements, 1);
        let mut bits = [0_u8; 4];
        bits.copy_from_slice(&payload[field.offset_bytes..field.offset_bytes + 4]);
        assert_eq!(i32::from_le_bytes(bits), volume.noct_i32().unwrap());

        // 偏移单调、16 字节对齐、互不重叠，且 payload 覆盖到最后一个字段末尾。
        let mut previous_end = 0_usize;
        for field in plan.fields() {
            assert_eq!(field.offset_bytes % 16, 0, "{}", field.name);
            assert!(field.offset_bytes >= previous_end, "{}", field.name);
            previous_end = field.offset_bytes + field.bytes();
        }
        assert_eq!(plan.buffer_bytes(), previous_end);
        assert_eq!(payload.len(), previous_end);
    }

    #[test]
    fn covers_octree_field_widths() {
        // 合法输入下布局层再验证各字段宽度（`VolumeModelInput` 已拒绝计数溢出的输入）。
        let volume = volume();
        let plan = CollisionUploadPlan::new(&volume).unwrap();
        assert_eq!(plan.field("oct_child").unwrap().elements, 16);
        assert_eq!(plan.field("oct_aabb").unwrap().elements, 12);
        assert_eq!(plan.field("oct_coeff").unwrap().elements, 16);
        assert_eq!(plan.field("noct").unwrap().elements, 1);
    }

    /// 上传布局里出现的字段名必须都能在 G1 字段表中找到同名项。
    #[test]
    fn every_shared_field_maps_to_g1_field_spec() {
        let volume = volume();
        let plan = CollisionUploadPlan::new(&volume).unwrap();
        for field in plan.fields() {
            assert!(
                crate::model::volume::field_index(field.name).is_some(),
                "{} 不在 G1 字段表",
                field.name
            );
        }
    }
}
