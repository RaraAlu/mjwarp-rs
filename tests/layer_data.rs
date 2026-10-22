//! A1 设备数据归属契约的宿主验证。
//! 覆盖布局、批量、身份、生命周期与公开兼容路径。
//! 不发射内核，也不申请设备内存。

use mjwarp_rs::diagnostics::{InputError, TransferError};
use mjwarp_rs::model::BatchLayout;
use mjwarp_rs::model::data::{
    DataIdentity, DataLifecycle, DeviceBatch as DataDeviceBatch, DeviceDataLayout, DeviceModelData,
    FieldBatch, GUARD_SLOTS, LayoutIdentity, MetadataProofOwner, MetadataProofRevision,
    ModelIdentity, SessionBinding, StateField, StateFieldVersions, WorldBatch,
};
use std::sync::Arc;

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

fn layout() -> DeviceDataLayout {
    DeviceDataLayout::new(2, 3, 4).unwrap()
}

fn identity() -> DataIdentity {
    DataIdentity::new(
        ModelIdentity::of_topology(&[3, 2, 1, 6]),
        LayoutIdentity::of_parts(&[2, 6, 4]),
    )
}

#[test]
fn guarded_layout_keeps_four_slots_per_end() {
    let layout = layout();
    assert_eq!(GUARD_SLOTS, 8);
    assert_eq!(layout.guarded_span_elements().unwrap(), 8 + 6);
    assert_eq!(layout.guarded_span_bytes().unwrap(), (8 + 6) * 4);
    layout.check_guarded_elements(14).unwrap();
    assert!(matches!(
        layout.check_guarded_elements(13),
        Err(InputError::LengthMismatch { .. })
    ));
    assert!(layout.check_guarded_elements(15).is_err());
    assert_eq!(layout.guarded_world_elements(1).unwrap(), 4 + 3..4 + 6);
    assert!(layout.guarded_world_elements(2).is_err());
}

#[test]
fn wide_outputs_only_require_the_minimum_span() {
    let layout = layout();
    layout.check_minimum_elements(6).unwrap();
    layout.check_minimum_elements(64).unwrap();
    assert!(layout.check_minimum_elements(5).is_err());
    let empty = DeviceDataLayout::new(1, 0, 4).unwrap();
    assert_eq!(empty.guarded_span_elements().unwrap(), GUARD_SLOTS);
    assert_eq!(empty.guarded_world_elements(0).unwrap(), 4..4);
}

#[test]
fn shared_batches_sample_parameters_by_period() {
    let worlds = WorldBatch::new(5).unwrap();
    let parameters = FieldBatch::new(2).unwrap();
    let samples: Vec<usize> = (0..worlds.worlds())
        .map(|world| parameters.sample_index(worlds, world).unwrap())
        .collect();
    assert_eq!(samples, vec![0, 1, 0, 1, 0]);
    assert!(parameters.sample_index(worlds, 5).is_err());
    assert!(FieldBatch::new(0).is_err());
    assert!(WorldBatch::new(0).is_err());
}

#[test]
fn model_identity_separates_instances_from_topology() {
    let first = Arc::new(1u32);
    let second = Arc::new(1u32);
    let first_identity = ModelIdentity::of_model(&first);
    assert_eq!(first_identity, ModelIdentity::of_model(&Arc::clone(&first)));
    assert_ne!(first_identity, ModelIdentity::of_model(&second));
    assert_eq!(
        ModelIdentity::of_topology(&[3, 2, 1, 6]),
        ModelIdentity::of_topology(&[3, 2, 1, 6])
    );
    assert_ne!(
        ModelIdentity::of_topology(&[3, 2, 1, 6]),
        ModelIdentity::of_topology(&[3, 2, 1, 7])
    );
}

#[test]
fn layout_identity_is_distinct_from_model_identity() {
    let data = identity();
    let same_layout_other_model =
        DataIdentity::new(ModelIdentity::of_topology(&[9, 9, 9, 9]), data.layout());
    let same_model_other_layout =
        DataIdentity::new(data.model(), LayoutIdentity::of_parts(&[3, 9, 4]));
    assert!(data.matches(&identity()));
    assert!(!data.matches(&same_layout_other_model));
    assert!(!data.matches(&same_model_other_layout));
    assert!(LayoutIdentity::new(0).is_err());
}

#[test]
fn lifecycle_rejects_cross_session_and_stale_identity() {
    let mut data = DataLifecycle::new(MockSession { device: 0, id: 3 }, identity());
    data.check_session(&MockSession { device: 0, id: 3 })
        .unwrap();
    assert_eq!(data.session().device, 0);
    assert_eq!(
        data.check_session(&MockSession { device: 0, id: 4 })
            .unwrap_err(),
        TransferError::SessionMismatch
    );
    assert_eq!(
        data.check_identity(DataIdentity::new(
            ModelIdentity::of_topology(&[1, 1, 1, 1]),
            LayoutIdentity::of_parts(&[1]),
        ))
        .unwrap_err(),
        TransferError::ModelMismatch
    );
    data.state_written().unwrap();
    assert_eq!(data.check_identity(identity()).unwrap(), ());
    assert_eq!(data.state_version(), 1);
}

#[test]
fn lifecycle_invalidates_derived_results_and_proofs() {
    let mut data = DataLifecycle::new(MockSession { device: 0, id: 3 }, identity());
    assert!(data.derived_fresh());
    data.state_written().unwrap();
    assert!(!data.derived_fresh());
    assert_eq!(data.derived_published().unwrap(), 1);
    assert!(data.derived_fresh());
    data.metadata_written().unwrap();
    assert_eq!(data.metadata_generation(), 1);
    data.check_metadata_generation(1).unwrap();
    assert!(data.check_metadata_generation(0).is_err());
}

#[test]
fn tagged_state_versions_are_reported_per_field() {
    let mut data = DataLifecycle::new(MockSession { device: 0, id: 3 }, identity());
    assert_eq!(data.state_field_versions(), StateFieldVersions::default());
    assert_eq!(data.record_state_write(StateField::Qpos).unwrap(), 1);
    assert_eq!(data.record_state_write(StateField::Qpos).unwrap(), 2);
    assert_eq!(data.record_state_write(StateField::Qvel).unwrap(), 3);
    assert_eq!(data.record_state_write(StateField::Mocap).unwrap(), 4);
    assert_eq!(data.record_state_write(StateField::Sleep).unwrap(), 5);
    let versions = data.state_field_versions();
    assert_eq!(versions.qpos(), 2);
    assert_eq!(versions.get(StateField::Qvel), 1);
    assert_eq!(versions.mocap(), 1);
    assert_eq!(versions.sleep(), 1);
    assert_eq!(data.state_version(), 5);
    assert!(!data.derived_fresh());
    assert_eq!(data.record_metadata_write().unwrap(), 1);
    assert_eq!(data.metadata_generation(), 1);
    assert!(data.check_metadata_generation(0).is_err());
    data.check_metadata_generation(1).unwrap();
}

#[test]
fn shared_layout_converts_from_checked_batch_layout() {
    let batch = BatchLayout::new(3, 5, 8).unwrap();
    let shared = DeviceDataLayout::from_batch(batch);
    assert_eq!(shared.worlds(), 3);
    assert_eq!(shared.elements_per_world(), 5);
    assert_eq!(shared.total_bytes(), batch.total_bytes());
    assert_eq!(shared.guarded_span_bytes().unwrap(), 3 * 5 * 8 + 8 * 8);
}

#[test]
fn keeps_public_compatibility_paths() {
    // io 路径与 model::data 路径指向同一设备批量类型。
    fn batch_name<T>() -> &'static str {
        std::any::type_name::<T>()
    }
    assert_eq!(
        batch_name::<mjwarp_rs::io::DeviceBatch<f32>>(),
        batch_name::<DataDeviceBatch<f32>>()
    );
    assert_eq!(
        batch_name::<mjwarp_rs::model::DeviceDataLayout>(),
        batch_name::<DeviceDataLayout>()
    );
    // 计划级元数据证明的公开别名仍可在共享归属下引用。
    fn accepts_proof<M: MetadataProofOwner>() {}
    fn accepts_revision(_: MetadataProofRevision) {}
    let _ = accepts_proof::<TestProof>;
    let _ = accepts_revision;
}

struct TestProof;

impl MetadataProofOwner for TestProof {
    type Dims = ();

    const KIND: mjwarp_rs::runtime::launcher::ProofKind =
        mjwarp_rs::runtime::launcher::ProofKind("test.layer_data.proof.v1");

    fn validate(
        _values: &[i32],
        _dims: &Self::Dims,
    ) -> Result<(), mjwarp_rs::runtime::launcher::LauncherError> {
        Ok(())
    }
}

#[test]
fn device_models_expose_shared_ownership_contract() {
    fn accepts<T: DeviceModelData>() -> &'static str {
        std::any::type_name::<T>()
    }
    assert!(accepts::<mjwarp_rs::io::DeviceKinematicModel>().contains("DeviceKinematicModel"));
    assert!(accepts::<mjwarp_rs::io::DeviceInertialModel>().contains("DeviceInertialModel"));
}

#[cfg(feature = "cuda-probe")]
#[test]
#[ignore = "requires an NVIDIA GPU"]
fn real_sessions_bind_and_reject_cross_session_lifecycle() {
    use mjwarp_rs::runtime::TransferSession;
    let session = TransferSession::new(0).unwrap();
    let clone = session.clone();
    let other = TransferSession::new(0).unwrap();
    let mut data = DataLifecycle::new(session, identity());
    data.check_session(&clone).unwrap();
    assert_eq!(
        data.check_session(&other).unwrap_err(),
        TransferError::SessionMismatch
    );
    data.state_written().unwrap();
    assert!(!data.derived_fresh());
    data.check_session(&clone).unwrap();
    assert_eq!(data.session().device(), 0);
}
