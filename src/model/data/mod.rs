//! A1：设备数据归属根。
//! 按层属于计算物理层；目录位置不变。
//! 只放共享契约类型，不整体搬迁目录。
//! 布局见`layout`，批量见`batch`，生命周期见`lifecycle`。

pub mod batch;
pub mod layout;
pub mod lifecycle;

pub use batch::{DeviceBatch, FieldBatch, WorldBatch};
pub use layout::{DeviceDataLayout, GUARD_SLOTS, GUARD_SLOTS_PER_END};
pub use lifecycle::{
    CheckedMetadata, DataIdentity, DataLifecycle, DeviceModelData, LayoutIdentity,
    MetadataProofOwner, MetadataProofRevision, ModelIdentity, SessionBinding,
};
