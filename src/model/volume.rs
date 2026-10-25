//! 体积与SDF模型字段的规范层：hfield、mesh octree、`opt.sdf_*`。
//! 本模块只定义字段规范、原生计数、数据面与唯一的f64→f32转换函数；
//! 校验后的输入见`model::volume_input`，原生读取在`io::volume`。
//! 字段表与`native/g01_fields.inc`的45..58号逐项一致。

use crate::diagnostics::{InputError, NativeProbeError};

/// 原生标量类型；编号与`native/g01_fields.inc`的kind一致。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VolumeFieldKind {
    /// 原生int32。
    Int32,
    /// 原生double（mjtNum）。
    Num,
    /// 原生float32（`hfield_data`）。
    Float,
    /// 原生int64（mjtSize，如`noct`）。
    Size,
}

impl VolumeFieldKind {
    /// 原生ABI kind编号。
    pub const fn code(self) -> u32 {
        match self {
            Self::Int32 => 1,
            Self::Num => 2,
            Self::Float => 4,
            Self::Size => 5,
        }
    }

    /// 单个标量的字节数。
    pub const fn scalar_bytes(self) -> usize {
        match self {
            Self::Int32 | Self::Float => 4,
            Self::Num | Self::Size => 8,
        }
    }
}

/// 字段计数的基数；`Fixed`表示常量个标量。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VolumeCountUnit {
    Fixed,
    Nhfield,
    Nhfielddata,
    Noct,
    Nmesh,
}

use VolumeCountUnit::{Fixed as Fx, Nhfield as Hf, Nhfielddata as Hd, Nmesh as Me, Noct as Oc};
use VolumeFieldKind::{Float as F, Int32 as I, Num as D, Size as S};

/// 单个体积字段的规范；期望标量数=基数×分量宽度。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VolumeFieldSpec {
    pub name: &'static str,
    pub kind: VolumeFieldKind,
    pub unit: VolumeCountUnit,
    pub width: usize,
}

/// 字段顺序必须与`native/g01_fields.inc`的45..58号完全一致。
pub const VOLUME_FIELDS: [VolumeFieldSpec; 14] = [
    VolumeFieldSpec {
        name: "hfield_nrow",
        kind: I,
        unit: Hf,
        width: 1,
    },
    VolumeFieldSpec {
        name: "hfield_ncol",
        kind: I,
        unit: Hf,
        width: 1,
    },
    VolumeFieldSpec {
        name: "hfield_size",
        kind: D,
        unit: Hf,
        width: 4,
    },
    VolumeFieldSpec {
        name: "hfield_adr",
        kind: I,
        unit: Hf,
        width: 1,
    },
    VolumeFieldSpec {
        name: "hfield_data",
        kind: F,
        unit: Hd,
        width: 1,
    },
    VolumeFieldSpec {
        name: "noct",
        kind: S,
        unit: Fx,
        width: 1,
    },
    VolumeFieldSpec {
        name: "oct_depth",
        kind: I,
        unit: Oc,
        width: 1,
    },
    VolumeFieldSpec {
        name: "oct_child",
        kind: I,
        unit: Oc,
        width: 8,
    },
    VolumeFieldSpec {
        name: "oct_aabb",
        kind: D,
        unit: Oc,
        width: 6,
    },
    VolumeFieldSpec {
        name: "oct_coeff",
        kind: D,
        unit: Oc,
        width: 8,
    },
    VolumeFieldSpec {
        name: "mesh_octadr",
        kind: I,
        unit: Me,
        width: 1,
    },
    VolumeFieldSpec {
        name: "mesh_octnum",
        kind: I,
        unit: Me,
        width: 1,
    },
    VolumeFieldSpec {
        name: "opt_sdf_iterations",
        kind: I,
        unit: Fx,
        width: 1,
    },
    VolumeFieldSpec {
        name: "opt_sdf_initpoints",
        kind: I,
        unit: Fx,
        width: 1,
    },
];

/// 按字段名查规范表下标。
pub fn field_index(name: &str) -> Option<usize> {
    VOLUME_FIELDS.iter().position(|spec| spec.name == name)
}

/// 体积相关原生计数；全部非负。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VolumeModelCounts {
    pub nhfield: usize,
    pub nhfielddata: usize,
    pub noct: usize,
    pub nmesh: usize,
}

impl VolumeModelCounts {
    /// 规范字段的期望标量数。
    pub fn expected(&self, spec: &VolumeFieldSpec) -> Result<usize, NativeProbeError> {
        let unit = match spec.unit {
            VolumeCountUnit::Fixed => 1,
            VolumeCountUnit::Nhfield => self.nhfield,
            VolumeCountUnit::Nhfielddata => self.nhfielddata,
            VolumeCountUnit::Noct => self.noct,
            VolumeCountUnit::Nmesh => self.nmesh,
        };
        unit.checked_mul(spec.width)
            .ok_or_else(|| InputError::Overflow { field: spec.name }.into())
    }
}

/// 一个体积字段的独占数据。
#[derive(Clone, Debug, PartialEq)]
pub enum VolumeFieldData {
    Int(Vec<i32>),
    Num(Vec<f64>),
    Float(Vec<f32>),
    Size(Vec<i64>),
}

impl VolumeFieldData {
    /// 数据类型。
    pub fn kind(&self) -> VolumeFieldKind {
        match self {
            Self::Int(_) => VolumeFieldKind::Int32,
            Self::Num(_) => VolumeFieldKind::Num,
            Self::Float(_) => VolumeFieldKind::Float,
            Self::Size(_) => VolumeFieldKind::Size,
        }
    }

    /// 标量个数。
    pub fn len(&self) -> usize {
        match self {
            Self::Int(v) => v.len(),
            Self::Num(v) => v.len(),
            Self::Float(v) => v.len(),
            Self::Size(v) => v.len(),
        }
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// 全部体积字段数据；顺序与`VOLUME_FIELDS`一致。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VolumeFields {
    data: Vec<VolumeFieldData>,
}

impl VolumeFields {
    /// 由字段数据构造；长度与顺序由上层校验。
    pub fn new(data: Vec<VolumeFieldData>) -> Self {
        Self { data }
    }

    /// 字段个数。
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// 是否没有字段。
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 按下标取字段。
    pub fn get(&self, index: usize) -> Option<&VolumeFieldData> {
        self.data.get(index)
    }

    /// 按字段名取数据。
    pub fn field(&self, name: &str) -> Option<&VolumeFieldData> {
        self.data.get(field_index(name)?)
    }
}

/// f64→f32一次性转换：就近舍入（ties-to-even）、不二次量化。
/// 这是`oct_aabb`/`oct_coeff`/`hfield_size`在上传路径上的唯一转换函数。
pub fn num_to_f32(values: &[f64]) -> Vec<f32> {
    values.iter().map(|&value| value as f32).collect()
}
