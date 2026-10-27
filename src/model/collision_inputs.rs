//! 产品碰撞输入（S1）：`CollisionGpuInputs` 32 项字段规范与计数口径。
//!
//! 本模块只定义字段规范与计数校验；原生读取、拥有式快照与 f64→f32 宿主转换位于
//! `io::collision_inputs`（无设备缓冲、无GPU、无产品接线，S3 才接线）。
//! 字段顺序与 `src/runtime/collision_gpu/types.rs::CollisionGpuInputs` 的 32 项 1:1，
//! 字段号与 `include/mjwarp_collision_inputs_probe.h`、
//! `native/model_probe_collision_inputs.cpp` 的 switch 同步。
//!
//! 状态输入只有 `qpos`：`geom_xpos`/`geom_xmat` 由调用方传入 qpos 后经一次
//! `mj_kinematics` 得到；mocap/act/ctrl 不在本片默认支持面。

use crate::diagnostics::{InputError, NativeProbeError};

/// 原生标量类型（本 probe schema 的 kind 编号）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollisionInputsFieldKind {
    /// 原生 int32（kind 1）。
    Index,
    /// 原生 double / mjtNum（kind 2）。
    Num,
    /// 原生 float32（kind 4）。
    Float,
}

impl CollisionInputsFieldKind {
    /// 原生 ABI kind 编号。
    pub const fn code(self) -> u32 {
        match self {
            Self::Index => 1,
            Self::Num => 2,
            Self::Float => 4,
        }
    }

    /// 单个标量字节数。
    pub const fn scalar_bytes(self) -> usize {
        match self {
            Self::Index => 4,
            Self::Num => 8,
            Self::Float => 4,
        }
    }

    /// 设备侧是否为 f32（`Num` 经既有唯一 f64→f32 入口收窄，`Float` 直接保留）。
    pub const fn device_is_float(self) -> bool {
        matches!(self, Self::Num | Self::Float)
    }
}

/// 字段计数的基数（单个字段的标量数 = 基数 × 分量宽度）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollisionInputsUnit {
    /// 常量为 1（本片 32 项未使用，保留给后续字段）。
    Fixed,
    Ngeom,
    Nmesh,
    Nmeshvert,
    Nmeshgraph,
    Nmeshpoly,
    Nmeshpolyvert,
    Nmeshpolymap,
    Nhfield,
    Nhfielddata,
    Noct,
}

/// 字段来源：`Model` 走静态字段读取；`Data` 走 mjData 帧快照。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollisionInputsFieldSource {
    Model,
    Data,
}

/// 单个碰撞输入字段的规范。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollisionInputsFieldSpec {
    pub name: &'static str,
    pub kind: CollisionInputsFieldKind,
    pub unit: CollisionInputsUnit,
    pub width: i64,
    pub source: CollisionInputsFieldSource,
}

/// 32 项字段表；下标即原生字段号（0..31）。
pub const COLLISION_INPUTS_FIELDS: [CollisionInputsFieldSpec; 32] = [
    field("geom_xpos", Float32, Ngeom, 3, Data),
    field("geom_xmat", Float32, Ngeom, 9, Data),
    field("geom_size", Num, Ngeom, 3, Model),
    field("geom_margin", Num, Ngeom, 1, Model),
    field("geom_gap", Num, Ngeom, 1, Model),
    field("geom_rbound", Num, Ngeom, 1, Model),
    field("geom_aabb", Num, Ngeom, 6, Model),
    field("geom_type", Index, Ngeom, 1, Model),
    field("geom_dataid", Index, Ngeom, 1, Model),
    field("mesh_vertadr", Index, Nmesh, 1, Model),
    field("mesh_vertnum", Index, Nmesh, 1, Model),
    field("mesh_graphadr", Index, Nmesh, 1, Model),
    field("mesh_octadr", Index, Nmesh, 1, Model),
    field("mesh_vert", Float32, Nmeshvert, 3, Model),
    field("mesh_graph", Index, Nmeshgraph, 1, Model),
    field("mesh_polynum", Index, Nmesh, 1, Model),
    field("mesh_polyadr", Index, Nmesh, 1, Model),
    field("mesh_polynormal", Num, Nmeshpoly, 3, Model),
    field("mesh_polyvertadr", Index, Nmeshpoly, 1, Model),
    field("mesh_polyvertnum", Index, Nmeshpoly, 1, Model),
    field("mesh_polyvert", Index, Nmeshpolyvert, 1, Model),
    field("mesh_polymapadr", Index, Nmeshvert, 1, Model),
    field("mesh_polymapnum", Index, Nmeshvert, 1, Model),
    field("mesh_polymap", Index, Nmeshpolymap, 1, Model),
    field("hfield_size", Num, Nhfield, 4, Model),
    field("hfield_nrow", Index, Nhfield, 1, Model),
    field("hfield_ncol", Index, Nhfield, 1, Model),
    field("hfield_adr", Index, Nhfield, 1, Model),
    field("hfield_data", Float32, Nhfielddata, 1, Model),
    field("oct_aabb", Num, Noct, 6, Model),
    field("oct_child", Index, Noct, 8, Model),
    field("oct_coeff", Num, Noct, 8, Model),
];

use CollisionInputsFieldKind::{Float as Float32, Index, Num};
use CollisionInputsFieldSource::{Data, Model};
use CollisionInputsUnit::{
    Ngeom, Nhfield, Nhfielddata, Nmesh, Nmeshgraph, Nmeshpoly, Nmeshpolymap, Nmeshpolyvert,
    Nmeshvert, Noct,
};

const fn field(
    name: &'static str,
    kind: CollisionInputsFieldKind,
    unit: CollisionInputsUnit,
    width: i64,
    source: CollisionInputsFieldSource,
) -> CollisionInputsFieldSpec {
    CollisionInputsFieldSpec {
        name,
        kind,
        unit,
        width,
        source,
    }
}

/// 原生模型计数；全部非负，非法计数在读取前拒绝。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CollisionInputsCounts {
    pub nq: i64,
    pub ngeom: i64,
    pub nmesh: i64,
    pub nmeshvert: i64,
    pub nmeshgraph: i64,
    pub nmeshpoly: i64,
    pub nmeshpolyvert: i64,
    pub nmeshpolymap: i64,
    pub nhfield: i64,
    pub nhfielddata: i64,
    pub noct: i64,
}

impl CollisionInputsCounts {
    /// 校验全部计数非负。
    pub fn validate(self) -> Result<(), NativeProbeError> {
        for (field, value) in [
            ("nq", self.nq),
            ("ngeom", self.ngeom),
            ("nmesh", self.nmesh),
            ("nmeshvert", self.nmeshvert),
            ("nmeshgraph", self.nmeshgraph),
            ("nmeshpoly", self.nmeshpoly),
            ("nmeshpolyvert", self.nmeshpolyvert),
            ("nmeshpolymap", self.nmeshpolymap),
            ("nhfield", self.nhfield),
            ("nhfielddata", self.nhfielddata),
            ("noct", self.noct),
        ] {
            if value < 0 {
                return Err(InputError::InvalidDimension { field }.into());
            }
        }
        Ok(())
    }

    fn count(self, unit: CollisionInputsUnit) -> Result<usize, NativeProbeError> {
        let value = match unit {
            CollisionInputsUnit::Fixed => 1,
            CollisionInputsUnit::Ngeom => self.ngeom,
            CollisionInputsUnit::Nmesh => self.nmesh,
            CollisionInputsUnit::Nmeshvert => self.nmeshvert,
            CollisionInputsUnit::Nmeshgraph => self.nmeshgraph,
            CollisionInputsUnit::Nmeshpoly => self.nmeshpoly,
            CollisionInputsUnit::Nmeshpolyvert => self.nmeshpolyvert,
            CollisionInputsUnit::Nmeshpolymap => self.nmeshpolymap,
            CollisionInputsUnit::Nhfield => self.nhfield,
            CollisionInputsUnit::Nhfielddata => self.nhfielddata,
            CollisionInputsUnit::Noct => self.noct,
        };
        if value < 0 {
            return Err(InputError::InvalidDimension { field: "counts" }.into());
        }
        usize::try_from(value).map_err(|_| InputError::Overflow { field: "counts" }.into())
    }

    /// 单个字段的期望标量数。
    pub fn expected(self, spec: &CollisionInputsFieldSpec) -> Result<usize, NativeProbeError> {
        self.validate()?;
        let width = usize::try_from(spec.width)
            .map_err(|_| InputError::InvalidDimension { field: spec.name })?;
        self.count(spec.unit)?
            .checked_mul(width)
            .ok_or_else(|| InputError::Overflow { field: spec.name }.into())
    }
}
