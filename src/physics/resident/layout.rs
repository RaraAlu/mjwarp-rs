//! 常驻设备包装与布局规则。

use super::*;

#[cfg(feature = "cuda-probe")]
pub(super) struct DeviceFlex {
    pub(super) model: crate::physics::DeviceModel,
    pub(super) kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct DeviceFlexPositions {
    pub(super) metadata: FlexMetadataBuffer,
    pub(super) parameters: TransferBuffer<f32>,
    pub(super) kernel: FlexPositionsKernel,
    pub(super) invalidate_kernel: FlexInvalidateKernel,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct DeviceFlexHessian {
    pub(super) model: crate::physics::DeviceModel,
    pub(super) kernel: SynchronousKernel,
    pub(super) validate_kernel: SynchronousKernel,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct DeviceFlexEdges {
    pub(super) metadata: TransferBuffer<i32>,
    pub(super) kernel: SynchronousKernel,
}
#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentFlexEdges {
    pub(super) qvel: TransferBuffer<f32>,
    pub(super) values: TransferBuffer<f32>,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct DeviceTendon {
    pub(super) metadata: TendonMetadataBuffer,
    pub(super) parameters: TransferBuffer<f32>,
    pub(super) kernel: ResidentTendonKernel,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct ResidentTendon {
    pub(super) values: TransferBuffer<f32>,
    pub(super) indices: TransferBuffer<i32>,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct DeviceSpatialTendon {
    pub(super) metadata: SpatialTendonMetadataBuffer,
    pub(super) parameters: TransferBuffer<f32>,
    pub(super) kernel: ResidentSpatialTendonKernel,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct DeviceFixedTendon {
    pub(super) metadata: FixedTendonMetadataBuffer,
    pub(super) parameters: TransferBuffer<f32>,
    pub(super) kernel: FixedTendonKernel,
}

#[cfg(feature = "cuda-probe")]
pub(super) struct DeviceCamLight {
    pub(super) metadata: CamLightMetadataBuffer,
    pub(super) parameters: TransferBuffer<f32>,
    pub(super) kernel: ResidentCamLightKernel,
}

impl ResidentLayout {
    pub(super) fn new(
        model: &AttachedModelInput,
        worlds: usize,
        nmocap: usize,
    ) -> Result<Self, InputError> {
        let k = model.rigid().kinematics();
        let stride = nmocap
            .checked_mul(7)
            .and_then(|n| n.checked_add(k.nq()))
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or(InputError::Overflow {
                field: "resident_state",
            })?;
        Ok(Self {
            rigid: KinematicsLayout::new(worlds, k.nq(), k.nbody(), k.njnt())?,
            com: ComPositionLayout::new(worlds, k.nbody(), k.njnt(), k.nv())?,
            attached: AttachedLayout::new(worlds, model.ngeom(), model.nsite())?,
            camlight: CamLightLayout::new(worlds, 0, 0)?,
            fixed_tendon: FixedTendonLayout::new(worlds, 0, 0)?,
            spatial_tendon: SpatialTendonLayout::new(worlds, 0, 0, 0)?,
            tendon: None,
            flex: None,
            edges: None,
            faces: None,
            hessian: None,
            sleep: None,
            qpos: BatchLayout::new(worlds, k.nq(), 4)?,
            mocap_pos: BatchLayout::new(worlds, nmocap * 3, 4)?,
            mocap_quat: BatchLayout::new(worlds, nmocap * 4, 4)?,
            state: BatchLayout::new(worlds, stride, 4)?,
        })
    }
}

impl ReadyStages {
    pub(super) fn require(&self, ready: bool, stage: &'static str) -> Result<(), TransferError> {
        if ready {
            Ok(())
        } else {
            Err(TransferError::StageNotReady { stage })
        }
    }
    pub(super) fn require_all(&self) -> Result<(), TransferError> {
        self.require(self.rigid, "rigid")?;
        self.require(self.attached, "attached")?;
        self.require(self.com, "com")
    }
}
