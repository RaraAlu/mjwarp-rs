//! A1-1/A1-4/A1-5：设备数据生命周期、身份与证明所有权。
//! 会话所有权、模型/布局身份分开记录。
//! 状态写入使派生结果失效；元数据写入使证明代际失效。
//! 证明由统一发射器的`Checked`承担。

use crate::diagnostics::{InputError, TransferError};
use crate::model::data::layout::DeviceDataLayout;
use crate::runtime::TransferSession;
use crate::runtime::launcher::LauncherError;
use std::sync::Arc;

/// 计划级元数据证明的受检包装。
///
/// 每条证明绑定设备区间、维度与写入代际。
/// 写入须先使代际失效，发射前复核。
pub use crate::runtime::launcher::Checked as CheckedMetadata;

/// 计划级元数据证明所有权登记。
pub use crate::runtime::launcher::{
    MetadataProof as MetadataProofOwner, MetadataRevision as MetadataProofRevision,
};

/// 会话绑定契约。
///
/// 克隆共享同一设备流与隔离状态。
/// 跨会话使用一律拒绝。
pub trait SessionBinding: Clone {
    fn device(&self) -> usize;

    fn binds_to(&self, other: &Self) -> bool;
}

impl SessionBinding for TransferSession {
    fn device(&self) -> usize {
        TransferSession::device(self)
    }

    fn binds_to(&self, other: &Self) -> bool {
        #[cfg(feature = "cuda-probe")]
        {
            TransferSession::same_session(self, other)
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            // 无设备特性时匿名会话不可构造，比较退化为真。
            let _ = other;
            true
        }
    }
}

/// 模型身份。
///
/// 宿主模型按实例身份记录：同尺寸不代表同拓扑。
/// 已上传设备模型按固定拓扑尺寸记录。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelIdentity(u64);

impl ModelIdentity {
    /// 宿主模型实例身份。
    pub fn of_model<T>(model: &Arc<T>) -> Self {
        Self(pointer_hash(Arc::as_ptr(model) as *const ()))
    }

    /// 已上传/派生模型身份：由固定拓扑与字段尺寸确定。
    pub fn of_topology(parts: &[usize]) -> Self {
        Self(fnv_parts(parts).max(1))
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

/// 布局身份。
///
/// 由世界数、字段步长与字段集合确定。
/// 与模型身份分开记录。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutIdentity(u64);

impl LayoutIdentity {
    /// 序列式布局身份；零保留给未初始化。
    pub fn new(serial: u64) -> Result<Self, InputError> {
        if serial == 0 {
            return Err(InputError::InvalidDimension {
                field: "layout_identity",
            });
        }
        Ok(Self(serial))
    }

    /// 由布局尺寸摘要生成身份。
    pub fn of_parts(parts: &[usize]) -> Self {
        Self(fnv_parts(parts).max(1))
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

/// 数据身份：模型身份与布局身份的组合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataIdentity {
    model: ModelIdentity,
    layout: LayoutIdentity,
}

impl DataIdentity {
    pub fn new(model: ModelIdentity, layout: LayoutIdentity) -> Self {
        Self { model, layout }
    }

    pub fn model(&self) -> ModelIdentity {
        self.model
    }

    pub fn layout(&self) -> LayoutIdentity {
        self.layout
    }

    pub fn matches(&self, other: &Self) -> bool {
        self == other
    }
}

/// 阶段设备数据的生命周期记录。
///
/// 只记录会话、身份与版本。
/// 不持有设备缓冲；缓冲随所属`Data`类型释放。
/// 计划只借用`Data`，不取得所有权。
#[derive(Clone, Debug)]
pub struct DataLifecycle<S: SessionBinding> {
    session: S,
    identity: DataIdentity,
    state_version: u64,
    derived_version: u64,
    metadata_generation: u64,
}

impl<S: SessionBinding> DataLifecycle<S> {
    pub fn new(session: S, identity: DataIdentity) -> Self {
        Self {
            session,
            identity,
            state_version: 0,
            derived_version: 0,
            metadata_generation: 0,
        }
    }

    pub fn session(&self) -> &S {
        &self.session
    }

    pub fn identity(&self) -> DataIdentity {
        self.identity
    }

    pub fn state_version(&self) -> u64 {
        self.state_version
    }

    pub fn derived_version(&self) -> u64 {
        self.derived_version
    }

    pub fn metadata_generation(&self) -> u64 {
        self.metadata_generation
    }

    /// 派生结果是否仍对应当前状态。
    pub fn derived_fresh(&self) -> bool {
        self.derived_version == self.state_version
    }

    /// 状态写入后废弃派生结果。
    pub fn state_written(&mut self) -> Result<u64, TransferError> {
        self.state_version = self
            .state_version
            .checked_add(1)
            .ok_or(InputError::Overflow {
                field: "state_version",
            })?;
        Ok(self.state_version)
    }

    /// 派生结果发布后对齐状态版本。
    pub fn derived_published(&mut self) -> Result<u64, TransferError> {
        if !self.derived_fresh() {
            self.derived_version = self.state_version;
        }
        Ok(self.derived_version)
    }

    /// 元数据写入后使既有证明代际失效。
    pub fn metadata_written(&mut self) -> Result<u64, TransferError> {
        self.metadata_generation =
            self.metadata_generation
                .checked_add(1)
                .ok_or(InputError::Overflow {
                    field: "metadata_generation",
                })?;
        Ok(self.metadata_generation)
    }

    /// 跨会话使用一律拒绝。
    pub fn check_session(&self, session: &S) -> Result<(), TransferError> {
        if self.session.binds_to(session) {
            Ok(())
        } else {
            Err(TransferError::SessionMismatch)
        }
    }

    /// 身份不符即拒；同尺寸不代表同拓扑。
    pub fn check_identity(&self, identity: DataIdentity) -> Result<(), TransferError> {
        if self.identity.matches(&identity) {
            Ok(())
        } else {
            Err(TransferError::ModelMismatch)
        }
    }

    /// 发射前复核证明代际。
    pub fn check_metadata_generation(&self, generation: u64) -> Result<(), LauncherError> {
        if generation == self.metadata_generation {
            Ok(())
        } else {
            Err(LauncherError::StaleMetadata)
        }
    }
}

/// io设备模型的公开兼容转发面。
///
/// 只新增共享契约访问器，不改变旧签名。
/// 兼容层不复制设备资源。
pub trait DeviceModelData {
    /// 模型身份：由固定拓扑与字段尺寸确定。
    fn model_identity(&self) -> ModelIdentity;

    /// 主字段的共享批量布局。
    fn primary_layout(&self) -> DeviceDataLayout;
}

fn pointer_hash(pointer: *const ()) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in (pointer as usize).to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash.max(1)
}

fn fnv_parts(parts: &[usize]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for part in parts {
        for byte in part.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct MockSession {
        device: usize,
        id: u64,
    }

    impl SessionBinding for MockSession {
        fn device(&self) -> usize {
            self.device
        }

        fn binds_to(&self, other: &Self) -> bool {
            self.device == other.device && self.id == other.id
        }
    }

    fn identity() -> DataIdentity {
        DataIdentity::new(
            ModelIdentity::of_topology(&[4, 3, 2]),
            LayoutIdentity::of_parts(&[2, 12, 4]),
        )
    }

    fn lifecycle() -> DataLifecycle<MockSession> {
        DataLifecycle::new(MockSession { device: 0, id: 7 }, identity())
    }

    #[test]
    fn separates_model_and_layout_identity() {
        let identity = identity();
        let other_model =
            DataIdentity::new(ModelIdentity::of_topology(&[4, 3, 2, 1]), identity.layout());
        let other_layout = DataIdentity::new(identity.model(), LayoutIdentity::new(9).unwrap());
        assert!(identity.matches(&identity));
        assert!(!identity.matches(&other_model));
        assert!(!identity.matches(&other_layout));
        assert_eq!(
            identity.model().value(),
            ModelIdentity::of_topology(&[4, 3, 2]).value()
        );
        assert!(LayoutIdentity::new(0).is_err());
    }

    #[test]
    fn rejects_cross_session_and_foreign_identity() {
        let data = lifecycle();
        let same = MockSession { device: 0, id: 7 };
        let other = MockSession { device: 0, id: 8 };
        assert_eq!(data.session().device(), 0);
        data.check_session(&same).unwrap();
        assert_eq!(
            data.check_session(&other).unwrap_err(),
            TransferError::SessionMismatch
        );
        data.check_identity(identity()).unwrap();
        let foreign = DataIdentity::new(
            ModelIdentity::of_topology(&[9, 9, 9]),
            LayoutIdentity::of_parts(&[1, 1, 4]),
        );
        assert_eq!(
            data.check_identity(foreign).unwrap_err(),
            TransferError::ModelMismatch
        );
    }

    #[test]
    fn state_writes_invalidate_derived_results() {
        let mut data = lifecycle();
        assert!(data.derived_fresh());
        assert_eq!(data.state_version(), 0);
        assert_eq!(data.state_written().unwrap(), 1);
        assert!(!data.derived_fresh());
        assert_eq!(data.derived_version(), 0);
        data.check_metadata_generation(0).unwrap();
        assert_eq!(data.derived_published().unwrap(), 1);
        assert!(data.derived_fresh());
        assert_eq!(data.state_written().unwrap(), 2);
        assert!(!data.derived_fresh());
    }

    #[test]
    fn metadata_writes_invalidate_proof_generation() {
        let mut data = lifecycle();
        data.check_metadata_generation(0).unwrap();
        assert_eq!(data.metadata_written().unwrap(), 1);
        assert_eq!(
            data.check_metadata_generation(0).unwrap_err(),
            LauncherError::StaleMetadata
        );
        data.check_metadata_generation(1).unwrap();
        assert!(data.derived_fresh());
    }

    #[test]
    fn rejects_counter_exhaustion() {
        let mut data = lifecycle();
        data.state_version = u64::MAX;
        assert!(data.state_written().is_err());
        data.metadata_generation = u64::MAX;
        assert!(data.metadata_written().is_err());
    }
}
