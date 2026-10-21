//! 被动合力数据分配、发射与收尾。

use super::*;

impl<'a, 'v, 'p, 'f> PassiveForcePlan<'a, 'v, 'p, 'f> {
    pub fn create_data(&self, worlds: usize) -> Result<PassiveForceData, TransferError> {
        let layout = BatchLayout::new(worlds, self.nv, 4)?;
        let count = layout
            .total_elements()
            .checked_mul(2)
            .ok_or(InputError::Overflow {
                field: "passive_force_parts",
            })?;
        let smooth = self.smooth.create_data(worlds)?;
        let fluid = self
            .fluid
            .as_ref()
            .map(|plan| plan.create_data(worlds))
            .transpose()?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (count, smooth, fluid);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let allocate = |count: usize, value: f32| {
                let mut values = crate::runtime::host_staging::<f32>(count + 8)?;
                values.fill(-131072.0);
                values[4..4 + count].fill(value);
                self.session.upload(&values)
            };
            Ok(PassiveForceData {
                identity: self.identity.clone(),
                layout,
                ready: false,
                smooth,
                fluid,
                parts: allocate(count, f32::NAN)?,
                gravity: allocate(layout.total_elements(), 0.0)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }

    #[cfg(feature = "cuda-probe")]
    pub(super) fn write_passive(
        &self,
        data: &mut PassiveForceData,
        worlds: usize,
        work: u32,
    ) -> Result<(), TransferError> {
        let fluid = match (&self.fluid, &data.fluid) {
            (Some(plan), Some(fluid)) => Some(plan.shared_qfrc(fluid, worlds)?),
            (None, None) => None,
            _ => return Err(TransferError::ModelMismatch),
        };
        let passive = data.smooth.passive_target();
        if let Some(gravity) = &self.gravity {
            gravity.sum_shared(&data.parts, &data.gravity, passive, worlds)?;
        } else {
            // SAFETY: 分量缓冲含两个连续字段。
            // 目标仅写入平滑力的Passive字段。
            unsafe {
                self.kernels[1].launch(
                    &self.metadata,
                    &self.dummy,
                    &data.parts,
                    passive,
                    [0, self.nv as u32, worlds as u32, work],
                )?;
            }
        }
        if let Some(fluid) = fluid {
            // SAFETY: 流体缓冲与目标都含worlds*nv槽。
            // 本次加法严格位于重力补偿之后。
            unsafe {
                self.kernels[3].launch(
                    &self.metadata,
                    &self.dummy,
                    fluid,
                    passive,
                    [0, self.nv as u32, worlds as u32, work],
                )?;
            }
        }
        Ok(())
    }

    #[cfg(feature = "cuda-probe")]
    pub(super) fn finish(
        &self,
        data: &mut PassiveForceData,
        worlds: usize,
        work: u32,
        sleep: Option<&TransferBuffer<i32>>,
    ) -> Result<(), TransferError> {
        self.write_passive(data, worlds, work)?;
        self.smooth
            .update_with_sleep_source(&mut data.smooth, sleep)
    }
}
