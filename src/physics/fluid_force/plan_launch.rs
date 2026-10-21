//! 流体计划内核发射与缓冲绑定。

use super::*;

impl<'p> FluidForcePlan<'p> {
    #[cfg(feature = "cuda-probe")]
    pub(super) fn launch(
        &self,
        inputs: FluidInputs<'_>,
        data: &mut FluidForceData,
    ) -> Result<(), TransferError> {
        let layout = data.layout;
        self.launch_buffers(
            inputs,
            layout,
            &mut data.body,
            &mut data.qfrc,
            &mut data.status,
        )
    }

    #[cfg(feature = "cuda-probe")]
    pub(super) fn launch_buffers(
        &self,
        inputs: FluidInputs<'_>,
        layout: Layout,
        body: &mut TransferBuffer<f32>,
        qfrc: &mut TransferBuffer<f32>,
        status_buffer: &mut TransferBuffer<f32>,
    ) -> Result<(), TransferError> {
        check_guards(body)?;
        check_guards(qfrc)?;
        let worlds = layout.qfrc.worlds();
        let body_work = worlds
            .checked_mul(self.spec.nb)
            .ok_or(InputError::Overflow {
                field: "fluid_force_work",
            })?;
        let qfrc_work = worlds
            .checked_mul(self.spec.nv)
            .ok_or(InputError::Overflow {
                field: "fluid_force_work",
            })?;
        let attached = inputs.attached.unwrap_or(&self.parameters);
        // SAFETY: 输入来自同一已就绪速度票据。
        // 验证核先检查守卫、映射与参数。
        unsafe {
            self.kernels[0].launch_fluid_force(
                &self.metadata,
                [
                    &self.parameters,
                    inputs.rigid,
                    inputs.com,
                    inputs.velocity,
                    attached,
                ],
                status_buffer,
                [
                    self.spec.nb as u32,
                    self.spec.nv as u32,
                    worlds as u32,
                    worlds as u32,
                ],
            )?;
        }
        let status = read_output(status_buffer, worlds, "fluid_force_status")?;
        if let Some(index) = status[4..4 + worlds].iter().position(|&value| value != 1.0) {
            return Err(InputError::InvalidTopology {
                field: "fluid_force_source",
                index,
                reason: "guard_mapping_or_nonfinite",
            }
            .into());
        }
        // SAFETY: 每线程独占一个刚体空间力槽。
        unsafe {
            self.kernels[1].launch_fluid_force(
                &self.metadata,
                [
                    &self.parameters,
                    inputs.rigid,
                    inputs.com,
                    inputs.velocity,
                    attached,
                ],
                body,
                [
                    self.spec.nb as u32,
                    self.spec.nv as u32,
                    worlds as u32,
                    body_work as u32,
                ],
            )?;
        }
        read_output(body, layout.body.total_elements(), "fluid_force_body")?;
        // SAFETY: 每线程按bodyid升序投影自由度。
        unsafe {
            self.kernels[2].launch_fluid_force(
                &self.metadata,
                [&self.parameters, inputs.rigid, inputs.com, body, attached],
                qfrc,
                [
                    self.spec.nb as u32,
                    self.spec.nv as u32,
                    worlds as u32,
                    qfrc_work as u32,
                ],
            )?;
        }
        read_output(qfrc, layout.qfrc.total_elements(), "qfrc_fluid")?;
        Ok(())
    }
}
