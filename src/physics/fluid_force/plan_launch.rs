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
        let worlds = layout.qfrc.worlds() as u32;
        let velocity_inputs = kernels::FluidForceInputs {
            parameters: &self.parameters,
            rigid: inputs.rigid,
            com: inputs.com,
            fourth: inputs.velocity,
            attached: inputs.attached,
        };
        self.kernels
            .launch_validate(&self.metadata, velocity_inputs, status_buffer, worlds)?;
        let status = read_output(status_buffer, worlds as usize, "fluid_force_status")?;
        if let Some(index) = status[4..4 + worlds as usize]
            .iter()
            .position(|&value| value != 1.0)
        {
            return Err(InputError::InvalidTopology {
                field: "fluid_force_source",
                index,
                reason: "guard_mapping_or_nonfinite",
            }
            .into());
        }
        self.kernels
            .launch_body(&self.metadata, velocity_inputs, body, worlds)?;
        read_output(body, layout.body.total_elements(), "fluid_force_body")?;
        let body_inputs = kernels::FluidForceInputs {
            fourth: body,
            ..velocity_inputs
        };
        self.kernels
            .launch_project(&self.metadata, body_inputs, qfrc, worlds)?;
        read_output(qfrc, layout.qfrc.total_elements(), "qfrc_fluid")?;
        Ok(())
    }
}
