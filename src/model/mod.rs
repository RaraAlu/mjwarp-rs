//! 连续批量字段的宿主布局。
//! 不分配设备缓冲。

use crate::diagnostics::{InputError, NativeProbeError};

pub mod render;
pub use render::{
    CountUnit, FieldData, FieldSpec, RENDER_FIELDS, RENDER_OBJECTS, RenderFieldKind, RenderFields,
    RenderModelCounts, RenderObject,
};
pub mod topology;
pub use topology::{KinematicFields, KinematicModelInput};
pub mod volume;
pub use volume::{
    VOLUME_FIELDS, VolumeFieldData, VolumeFieldKind, VolumeFieldSpec, VolumeFields,
    VolumeModelCounts,
};
pub mod volume_input;
pub use volume_input::VolumeModelInput;
pub mod inertial;
pub use inertial::{
    InertialFields, InertialModelInput, SparseMassBlock, SparseMassFields, SparseMassInput,
    SparseMassRoute,
};
pub mod attached;
pub use attached::{AttachedFields, AttachedModelInput};
pub mod collision;
pub use collision::{PrimitiveGeometryFields, PrimitiveGeometryInput};
pub mod collision_parameters;
pub use collision_parameters::{
    PrimitiveGeometryHostBatch, PrimitiveGeometryParameter, PrimitiveGeometryParameters,
    PrimitiveGeometryTopology, PrimitiveGeometryWorld,
};
pub mod mocap;
pub use mocap::MocapModelInput;
pub mod parameters;
pub use parameters::{KinematicsParameter, KinematicsParameters, ParameterBatch};
pub mod camlight;
pub use camlight::{CamLightFields, CamLightModelInput, CamLightParameter, CamLightParameters};
pub mod fixed_tendon;
pub use fixed_tendon::{FixedTendonFields, FixedTendonModelInput, FixedTendonRows};
pub mod spatial_tendon;
pub use spatial_tendon::{
    SpatialTendonFields, SpatialTendonGeometry, SpatialTendonModelInput, SpatialTendonRows,
};
pub mod tendon;
pub use tendon::{TendonFields, TendonModelInput, TendonRows, TendonSubset};
pub mod flex;
pub use flex::{
    FlexEdgeFields, FlexFaceFields, FlexHessianFields, FlexPositionFields, FlexPositionModelInput,
};
pub mod data;
pub use data::{DeviceBatch, DeviceDataLayout, FieldBatch, GUARD_SLOTS, WorldBatch};
pub mod sleep;
pub use sleep::{SleepTreeState, TREE_FULLY_AWAKE, TendonWakeFields, TendonWakeModelInput};
pub mod smooth_force;
pub use smooth_force::SmoothForceInput;

/// Windows原生探针的小型DTO。
/// 计数采用原生有符号64位。
/// 本类型不表示完整模型布局。
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NativeModelInfo {
    pub schema: u32,
    pub native_version: i32,
    pub pointer_bytes: u32,
    pub num_bytes: u32,
    pub index_bytes: u32,
    pub size_bytes: u32,
    pub native_model_bytes: u64,
    pub nq: i64,
    pub nv: i64,
    pub nu: i64,
    pub na: i64,
    pub nbody: i64,
    pub njnt: i64,
    pub ngeom: i64,
    pub nsensordata: i64,
}

const _: () = {
    assert!(size_of::<NativeModelInfo>() == 96);
    assert!(align_of::<NativeModelInfo>() == 8);
    assert!(std::mem::offset_of!(NativeModelInfo, native_model_bytes) == 24);
    assert!(std::mem::offset_of!(NativeModelInfo, nq) == 32);
    assert!(std::mem::offset_of!(NativeModelInfo, nbody) == 64);
};

impl NativeModelInfo {
    pub const VERSION: i32 = 3_012_000;

    /// 仅验证候选ABI与计数容量。
    /// 不验证完整字段与引用。
    pub fn validate(self) -> Result<(), NativeProbeError> {
        if self.schema != 1
            || self.native_version != Self::VERSION
            || (
                self.pointer_bytes,
                self.num_bytes,
                self.index_bytes,
                self.size_bytes,
            ) != (8, 8, 4, 8)
            || self.native_model_bytes == 0
        {
            return Err(NativeProbeError::AbiMismatch);
        }
        for (field, count) in [
            ("nq", self.nq),
            ("nv", self.nv),
            ("nu", self.nu),
            ("na", self.na),
            ("nbody", self.nbody),
            ("njnt", self.njnt),
            ("ngeom", self.ngeom),
            ("nsensordata", self.nsensordata),
        ] {
            if count < 0 || (field == "nbody" && count == 0) {
                return Err(InputError::InvalidDimension { field }.into());
            }
            let count = usize::try_from(count).map_err(|_| InputError::Overflow { field })?;
            BatchLayout::new(1, count, 8)?;
        }
        Ok(())
    }
}

/// 连续布局 `[world, element]`。
/// 不描述共享字段或原生ABI。
///
/// A2：批量布局归属`model::data::layout`。
/// 本路径保留为兼容导出。
pub use data::layout::BatchLayout;

/// 渲染模型的名称表/纹理/网格地址前置校验。
/// 在分配任何设备缓冲之前由 `io::render` 调用。
pub fn validate_render_tables(
    counts: RenderModelCounts,
    fields: &RenderFields,
) -> Result<(), NativeProbeError> {
    let names = match fields.field("names") {
        Some(FieldData::Flag(values)) => values.as_slice(),
        _ => return Err(InputError::InvalidDimension { field: "names" }.into()),
    };
    if names.len() as i64 != counts.nnames || names.is_empty() {
        return Err(InputError::InvalidDimension { field: "names" }.into());
    }
    for object in RENDER_OBJECTS {
        let table = fields
            .ints(object.table())
            .map_err(NativeProbeError::from)?;
        if table.len() as i64 != object.count(counts) {
            return Err(InputError::InvalidDimension {
                field: object.table(),
            }
            .into());
        }
        for (index, &adr) in table.iter().enumerate() {
            if adr < 0 || adr as usize >= names.len() {
                return Err(InputError::InvalidTopology {
                    field: object.table(),
                    index,
                    reason: "name_address_out_of_range",
                }
                .into());
            }
        }
    }
    if names.last() != Some(&0) {
        return Err(InputError::InvalidTopology {
            field: "names",
            index: names.len() - 1,
            reason: "missing_name_terminator",
        }
        .into());
    }
    for (index, &size) in fields
        .ints("cam_resolution")
        .map_err(NativeProbeError::from)?
        .iter()
        .enumerate()
    {
        if size < 0 {
            return Err(InputError::NegativeValue {
                field: "cam_resolution",
                index,
            }
            .into());
        }
    }
    for (field, limit) in [
        ("light_texid", counts.ntex),
        ("mat_texid", counts.ntex),
        ("geom_matid", counts.nmat),
    ] {
        for (index, &value) in fields
            .ints(field)
            .map_err(NativeProbeError::from)?
            .iter()
            .enumerate()
        {
            if value < -1 || i64::from(value) >= limit {
                return Err(InputError::InvalidIndex {
                    field,
                    index,
                    limit: usize::try_from(limit).unwrap_or(usize::MAX),
                }
                .into());
            }
        }
    }
    for &adr in fields.sizes("tex_adr").map_err(NativeProbeError::from)? {
        if adr < 0 || adr > counts.ntexdata {
            return Err(InputError::InvalidRange {
                field: "tex_adr",
                offset: usize::try_from(adr.max(0)).unwrap_or(usize::MAX),
                elements: 1,
                capacity: usize::try_from(counts.ntexdata).unwrap_or(usize::MAX),
            }
            .into());
        }
    }
    for field in ["mesh_bvhadr", "mesh_octadr", "mesh_pathadr"] {
        for (index, &value) in fields
            .ints(field)
            .map_err(NativeProbeError::from)?
            .iter()
            .enumerate()
        {
            if value < -1 {
                return Err(InputError::InvalidIndex {
                    field,
                    index,
                    limit: 1usize << 31,
                }
                .into());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn native_info() -> NativeModelInfo {
        NativeModelInfo {
            schema: 1,
            native_version: NativeModelInfo::VERSION,
            pointer_bytes: 8,
            num_bytes: 8,
            index_bytes: 4,
            size_bytes: 8,
            native_model_bytes: 1,
            nbody: 1,
            ..Default::default()
        }
    }

    #[test]
    fn checks_native_dto_layout_and_zero_dof() {
        assert_eq!(size_of::<NativeModelInfo>(), 96);
        assert_eq!(std::mem::offset_of!(NativeModelInfo, nsensordata), 88);
        native_info().validate().unwrap();
        let mut info = native_info();
        info.nu = 2; // nu counts control inputs, not actuators.
        info.validate().unwrap();
    }

    #[test]
    fn rejects_native_schema_version_and_scalar_widths() {
        let original = native_info();
        for info in [
            NativeModelInfo {
                schema: 2,
                ..original
            },
            NativeModelInfo {
                native_version: 123,
                ..original
            },
            NativeModelInfo {
                num_bytes: 4,
                ..original
            },
            NativeModelInfo {
                index_bytes: 8,
                ..original
            },
            NativeModelInfo {
                size_bytes: 4,
                ..original
            },
            NativeModelInfo {
                pointer_bytes: 4,
                ..original
            },
            NativeModelInfo {
                native_model_bytes: 0,
                ..original
            },
        ] {
            assert_eq!(info.validate(), Err(NativeProbeError::AbiMismatch));
        }
    }

    #[test]
    fn rejects_native_negative_and_overflowing_counts() {
        let original = native_info();
        for info in [
            NativeModelInfo { nq: -1, ..original },
            NativeModelInfo { nv: -1, ..original },
            NativeModelInfo { nu: -1, ..original },
            NativeModelInfo { na: -1, ..original },
            NativeModelInfo {
                nbody: 0,
                ..original
            },
            NativeModelInfo {
                njnt: -1,
                ..original
            },
            NativeModelInfo {
                ngeom: -1,
                ..original
            },
            NativeModelInfo {
                nsensordata: -1,
                ..original
            },
            NativeModelInfo {
                nq: i64::MAX,
                ..original
            },
        ] {
            assert!(matches!(info.validate(), Err(NativeProbeError::Input(_))));
        }
    }
}
