//! 流体计划共享qfrc访问与清零入口。

use super::*;

impl<'p> FluidForcePlan<'p> {
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn shared_qfrc<'a>(
        &self,
        data: &'a FluidForceData,
        worlds: usize,
    ) -> Result<&'a TransferBuffer<f32>, TransferError> {
        if !Arc::ptr_eq(&self.spec, &data.spec) || data.worlds() != worlds {
            return Err(TransferError::ModelMismatch);
        }
        if !data.ready {
            return Err(TransferError::StageNotReady {
                stage: "fluid_force",
            });
        }
        Ok(&data.qfrc)
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn clear_shared(
        &self,
        data: &mut FluidForceData,
        worlds: usize,
    ) -> Result<(), TransferError> {
        if !matches!(&self.source, FluidForceSource::Resident(_))
            || !Arc::ptr_eq(&self.spec, &data.spec)
            || data.worlds() != worlds
            || data.resident_velocity.is_none()
        {
            return Err(TransferError::ModelMismatch);
        }
        let layout = data.layout;
        let body_work = clear_work(layout.body.total_elements())?;
        let qfrc_work = clear_work(layout.qfrc.total_elements())?;
        let status_work = clear_work(worlds)?;
        data.ready = false;
        data.resident_velocity
            .as_mut()
            .ok_or(TransferError::ModelMismatch)?
            .invalidate();
        check_guards(&data.body)?;
        check_guards(&data.qfrc)?;
        check_guards(&data.status)?;
        for (output, work) in [
            (&mut data.body, body_work),
            (&mut data.qfrc, qfrc_work),
            (&mut data.status, status_work),
        ] {
            // SAFETY: 每线程清零一个独占槽。
            // 五个参数缓冲只满足私有ABI。
            unsafe {
                self.kernels[3].launch_fluid_force(
                    &self.metadata,
                    [
                        &self.parameters,
                        &self.parameters,
                        &self.parameters,
                        &self.parameters,
                        &self.parameters,
                    ],
                    output,
                    [
                        self.spec.nb as u32,
                        self.spec.nv as u32,
                        worlds as u32,
                        work,
                    ],
                )?;
            }
        }
        read_output(&data.body, layout.body.total_elements(), "fluid_force_body")?;
        read_output(&data.qfrc, layout.qfrc.total_elements(), "qfrc_fluid")?;
        read_output(&data.status, worlds, "fluid_force_status")?;
        data.ready = true;
        Ok(())
    }
}
