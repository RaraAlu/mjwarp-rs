//! 校验后的体积与SDF模型输入：hfield、mesh octree、`opt.sdf_*`。
//! 构造时校验字段数、类型、长度、`noct`一致性与下标范围。
//! 不实装GPU内核、不分配设备缓冲；f64→f32转换函数在`model::volume`。

use super::volume::{VOLUME_FIELDS, VolumeFieldData, VolumeFields, VolumeModelCounts, num_to_f32};
use crate::diagnostics::{InputError, NativeProbeError};

/// 校验后的体积模型输入；不表示完整put_model。
#[derive(Clone, Debug, PartialEq)]
pub struct VolumeModelInput {
    counts: VolumeModelCounts,
    fields: VolumeFields,
}

impl VolumeModelInput {
    /// 校验字段数、类型、长度与下标范围。
    pub fn new(counts: VolumeModelCounts, fields: VolumeFields) -> Result<Self, NativeProbeError> {
        if fields.len() != VOLUME_FIELDS.len() {
            return Err(InputError::LengthMismatch {
                field: "volume_fields",
                expected: VOLUME_FIELDS.len(),
                actual: fields.len(),
            }
            .into());
        }
        for (index, spec) in VOLUME_FIELDS.iter().enumerate() {
            let Some(data) = fields.get(index) else {
                return Err(InputError::LengthMismatch {
                    field: spec.name,
                    expected: 1,
                    actual: 0,
                }
                .into());
            };
            if data.kind() != spec.kind {
                return Err(InputError::InvalidDimension { field: spec.name }.into());
            }
            let expected = counts.expected(spec)?;
            if data.len() != expected {
                return Err(InputError::LengthMismatch {
                    field: spec.name,
                    expected,
                    actual: data.len(),
                }
                .into());
            }
        }
        let input = Self { counts, fields };
        input.check_ranges()?;
        Ok(input)
    }

    /// 体积相关计数。
    pub fn counts(&self) -> VolumeModelCounts {
        self.counts
    }

    /// 按字段名取数据。
    pub fn field(&self, name: &str) -> Option<&VolumeFieldData> {
        self.fields.field(name)
    }

    /// 取int32字段。
    pub fn ints(&self, name: &str) -> Result<&[i32], NativeProbeError> {
        match self.field(name) {
            Some(VolumeFieldData::Int(v)) => Ok(v),
            _ => Err(InputError::InvalidDimension {
                field: "volume_kind",
            }
            .into()),
        }
    }

    /// 取double字段。
    pub fn nums(&self, name: &str) -> Result<&[f64], NativeProbeError> {
        match self.field(name) {
            Some(VolumeFieldData::Num(v)) => Ok(v),
            _ => Err(InputError::InvalidDimension {
                field: "volume_kind",
            }
            .into()),
        }
    }

    /// 取float32字段。
    pub fn floats(&self, name: &str) -> Result<&[f32], NativeProbeError> {
        match self.field(name) {
            Some(VolumeFieldData::Float(v)) => Ok(v),
            _ => Err(InputError::InvalidDimension {
                field: "volume_kind",
            }
            .into()),
        }
    }

    /// 取int64单值字段。
    pub fn scalar_size(&self, name: &str) -> Result<i64, NativeProbeError> {
        match self.field(name) {
            Some(VolumeFieldData::Size(v)) if v.len() == 1 => Ok(v[0]),
            _ => Err(InputError::InvalidDimension {
                field: "volume_kind",
            }
            .into()),
        }
    }

    /// octree节点数。
    pub fn noct(&self) -> usize {
        self.counts.noct
    }

    /// `opt.sdf_iterations`。
    pub fn sdf_iterations(&self) -> Result<i32, NativeProbeError> {
        self.ints("opt_sdf_iterations")?
            .first()
            .copied()
            .ok_or_else(|| {
                InputError::InvalidDimension {
                    field: "opt_sdf_iterations",
                }
                .into()
            })
    }

    /// `opt.sdf_initpoints`。
    pub fn sdf_initpoints(&self) -> Result<i32, NativeProbeError> {
        self.ints("opt_sdf_initpoints")?
            .first()
            .copied()
            .ok_or_else(|| {
                InputError::InvalidDimension {
                    field: "opt_sdf_initpoints",
                }
                .into()
            })
    }

    /// `hfield_data`：原生已是f32，上传链路不再转换。
    pub fn hfield_data(&self) -> Result<&[f32], NativeProbeError> {
        self.floats("hfield_data")
    }

    /// `hfield_size`（原生f64）。
    pub fn hfield_size(&self) -> Result<&[f64], NativeProbeError> {
        self.nums("hfield_size")
    }

    /// `oct_aabb`（原生f64）。
    pub fn oct_aabb(&self) -> Result<&[f64], NativeProbeError> {
        self.nums("oct_aabb")
    }

    /// `oct_coeff`（原生f64）。
    pub fn oct_coeff(&self) -> Result<&[f64], NativeProbeError> {
        self.nums("oct_coeff")
    }

    /// 唯一的f64→f32转换点：`oct_aabb`一次转换。
    pub fn oct_aabb_f32(&self) -> Result<Vec<f32>, NativeProbeError> {
        Ok(num_to_f32(self.oct_aabb()?))
    }

    /// 唯一的f64→f32转换点：`oct_coeff`一次转换。
    pub fn oct_coeff_f32(&self) -> Result<Vec<f32>, NativeProbeError> {
        Ok(num_to_f32(self.oct_coeff()?))
    }

    /// 唯一的f64→f32转换点：`hfield_size`一次转换。
    pub fn hfield_size_f32(&self) -> Result<Vec<f32>, NativeProbeError> {
        Ok(num_to_f32(self.hfield_size()?))
    }

    /// `noct`上传为int32；超出int32范围拒绝。
    pub fn noct_i32(&self) -> Result<i32, NativeProbeError> {
        i32::try_from(self.noct()).map_err(|_| InputError::Overflow { field: "noct" }.into())
    }

    fn check_ranges(&self) -> Result<(), NativeProbeError> {
        let noct = self.scalar_size("noct")?;
        if noct < 0 || usize::try_from(noct).ok() != Some(self.counts.noct) {
            return Err(InputError::InvalidDimension { field: "noct" }.into());
        }
        let adr = self.ints("hfield_adr")?;
        let nrow = self.ints("hfield_nrow")?;
        let ncol = self.ints("hfield_ncol")?;
        for index in 0..self.counts.nhfield {
            let cells = i64::from(nrow[index]).checked_mul(i64::from(ncol[index]));
            let end = cells.and_then(|cells| i64::from(adr[index]).checked_add(cells));
            let ok = end.is_some_and(|end| {
                nrow[index] >= 0
                    && ncol[index] >= 0
                    && adr[index] >= 0
                    && end <= self.counts.nhfielddata as i64
            });
            if !ok {
                return Err(InputError::InvalidRange {
                    field: "hfield_data",
                    offset: usize::try_from(adr[index]).unwrap_or(0),
                    elements: usize::try_from(
                        i64::from(nrow[index]).saturating_mul(i64::from(ncol[index])),
                    )
                    .unwrap_or(usize::MAX),
                    capacity: self.counts.nhfielddata,
                }
                .into());
            }
        }
        let octadr = self.ints("mesh_octadr")?;
        let octnum = self.ints("mesh_octnum")?;
        for index in 0..self.counts.nmesh {
            let end = i64::from(octadr[index]).checked_add(i64::from(octnum[index]));
            let ok = octadr[index] >= 0
                && octnum[index] >= 0
                && end.is_some_and(|end| end <= self.counts.noct as i64);
            if !ok {
                return Err(InputError::InvalidRange {
                    field: "mesh_octadr",
                    offset: usize::try_from(octadr[index]).unwrap_or(0),
                    elements: usize::try_from(octnum[index]).unwrap_or(0),
                    capacity: self.counts.noct,
                }
                .into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> Vec<VolumeFieldData> {
        vec![
            VolumeFieldData::Int(vec![3]),
            VolumeFieldData::Int(vec![3]),
            VolumeFieldData::Num(vec![1.0, 1.0, 0.1, 0.1]),
            VolumeFieldData::Int(vec![0]),
            VolumeFieldData::Float(vec![0.0, 0.5, 0.0, 0.5, 1.0, 0.5, 0.0, 0.5, 0.0]),
            VolumeFieldData::Size(vec![0]),
            VolumeFieldData::Int(vec![]),
            VolumeFieldData::Int(vec![]),
            VolumeFieldData::Num(vec![]),
            VolumeFieldData::Num(vec![]),
            VolumeFieldData::Int(vec![0]),
            VolumeFieldData::Int(vec![0]),
            VolumeFieldData::Int(vec![10]),
            VolumeFieldData::Int(vec![40]),
        ]
    }

    fn counts() -> VolumeModelCounts {
        VolumeModelCounts {
            nhfield: 1,
            nhfielddata: 9,
            noct: 0,
            nmesh: 1,
        }
    }

    #[test]
    fn rejects_kind_and_length_mismatches() {
        assert!(VolumeModelInput::new(counts(), VolumeFields::new(data())).is_ok());
        let mut wrong_kind = data();
        wrong_kind[4] = VolumeFieldData::Num(vec![0.0; 9]);
        assert!(VolumeModelInput::new(counts(), VolumeFields::new(wrong_kind)).is_err());
        let mut wrong_len = data();
        wrong_len[4] = VolumeFieldData::Float(vec![0.0; 8]);
        assert!(VolumeModelInput::new(counts(), VolumeFields::new(wrong_len)).is_err());
        let mut short = data();
        short.pop();
        assert!(VolumeModelInput::new(counts(), VolumeFields::new(short)).is_err());
    }

    #[test]
    fn rejects_out_of_range_hfield_and_mesh_octree() {
        let mut bad_hfield = data();
        bad_hfield[3] = VolumeFieldData::Int(vec![1]);
        assert!(VolumeModelInput::new(counts(), VolumeFields::new(bad_hfield)).is_err());
        let mut bad_mesh = data();
        bad_mesh[10] = VolumeFieldData::Int(vec![-1]);
        assert!(VolumeModelInput::new(counts(), VolumeFields::new(bad_mesh)).is_err());
    }

    #[test]
    fn converts_f64_to_f32_once_with_ties_to_even() {
        let converted = num_to_f32(&[0.1, 1.0, 0.5]);
        assert_eq!(converted, vec![0.1f32, 1.0f32, 0.5f32]);
        assert_eq!(converted[0].to_bits(), 0x3DCC_CCCD);
        // 0.1的f64精确十进制展开；与0.1同一f64，仅用于确认一次转换的唯一结果。
        #[allow(clippy::excessive_precision)]
        let same = 0.1000000000000000055511151231257827f64;
        assert_eq!(num_to_f32(&[same])[0].to_bits(), 0x3DCC_CCCD);
        // 二次量化会得到不同的位型：f32->f64->f32必须幂等。
        for value in [0.1f32, 1.3f32, -2.7f32, 1.0 / 3.0] {
            let once = num_to_f32(&[f64::from(value)]);
            assert_eq!(once[0].to_bits(), value.to_bits());
        }
    }

    #[test]
    fn reports_sdf_options_and_noct_conversion() {
        let input = VolumeModelInput::new(counts(), VolumeFields::new(data())).unwrap();
        assert_eq!(input.sdf_iterations().unwrap(), 10);
        assert_eq!(input.sdf_initpoints().unwrap(), 40);
        assert_eq!(input.noct_i32().unwrap(), 0);
        assert_eq!(input.hfield_size_f32().unwrap(), vec![1.0, 1.0, 0.1, 0.1]);
        assert_eq!(input.hfield_data().unwrap().len(), 9);
    }
}
