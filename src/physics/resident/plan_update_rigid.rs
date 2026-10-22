//! 刚体与速度位置更新。

#[cfg(feature = "cuda-probe")]
use super::pack_batches::FIELD_BATCH_CUDA;
use super::*;

impl KinematicsPlan {
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn compile_velocity_position(
        &self,
    ) -> Result<VelocityPositionKernels, TransferError> {
        // FK复用计划内核，等价计划保持G01变体。
        let com = crate::physics::COM_POSITION_CUDA;
        let source = format!("{FIELD_BATCH_CUDA}\n{com}");
        Ok(VelocityPositionKernels {
            com: ResidentComKernel::compile(&self.session, &source)?,
        })
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn update_velocity_position(
        &self,
        data: &KinematicsData,
        kernels: &VelocityPositionKernels,
        rigid: &mut TransferBuffer<f32>,
        com: &mut TransferBuffer<f32>,
    ) -> Result<(), TransferError> {
        self.check_data(data)?;
        let l = data.layout;
        for (actual, expected) in [
            (rigid.len(), l.rigid.output.total_elements() + 8),
            (com.len(), l.com.output.total_elements() + 8),
        ] {
            if actual != expected {
                return Err(InputError::LengthMismatch {
                    field: "velocity_position_workspace",
                    expected,
                    actual,
                }
                .into());
            }
        }
        if self.equivalent {
            // G01变体读取body0；复制当前导入的世界位姿。
            // 内核随后覆盖其余全部FK槽位。
            rigid.copy_range_from(0, &data.rigid, 0, data.rigid.len())?;
        }
        let worlds = data.worlds() as u32;
        // 刚体发射由类型化包装核对证明、精确跨度、同会话与别名。
        self.rigid_kernel
            .launch(&self.rigid, &data.state, rigid, worlds)?;
        // 质心发射读取已同步刚体结果，写互不相交的世界输出。
        kernels
            .com
            .launch(&self.com.metadata, &self.com.parameters, rigid, com, worlds)?;
        Ok(())
    }

    pub(super) fn check_g01_data(&self, data: &KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        if !self.equivalent {
            return Err(InputError::InvalidDimension {
                field: "g01_requires_equivalent_plan",
            }
            .into());
        }
        Ok(())
    }

    pub(super) fn require_input(
        &self,
        data: &KinematicsData,
        ready: bool,
        stage: &'static str,
    ) -> Result<(), TransferError> {
        if !self.equivalent {
            data.ready.require(ready, stage)?;
        }
        Ok(())
    }

    /// 只更新七项刚体结果。
    /// 它废弃全部派生结果。
    pub fn update_rigid(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.ready = ReadyStages::default();
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let worlds = data.worlds() as u32;
            // 类型化包装核对证明、精确跨度、同会话、别名与拒绝先于写入。
            self.rigid_kernel
                .launch(&self.rigid, &data.state, &mut data.rigid, worlds)?;
            data.ready.rigid = true;
            Ok(())
        }
    }
}
