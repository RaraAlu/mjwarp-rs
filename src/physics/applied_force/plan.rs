//! 计划构造与访问器。

#[cfg(feature = "cuda-probe")]
use super::data::guards;
#[cfg(feature = "cuda-probe")]
use super::data::read;
#[cfg(feature = "cuda-probe")]
use super::kernels::AppliedForceDeltaKernel;
#[cfg(feature = "cuda-probe")]
use super::kernels::AppliedForcePublishKernel;
#[cfg(feature = "cuda-probe")]
use super::kernels::AppliedForceValidateKernel;
#[cfg(feature = "cuda-probe")]
use super::metadata_buffer::AppliedForceMetadataBuffer;
use super::*;

impl AppliedForcePlan {
    pub fn new(
        session: &TransferSession,
        body_parentid: Vec<i32>,
        body_rootid: Vec<i32>,
        dof_bodyid: Vec<i32>,
    ) -> Result<Self, TransferError> {
        let input = topology(body_parentid, body_rootid, dof_bodyid)?;
        layout(1, input.parent.len(), input.body.len())?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = session;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut meta = vec![-131072; 4];
            meta.extend(&input.parent);
            meta.extend(&input.root);
            meta.extend(&input.body);
            meta.extend([-131072; 4]);
            let nb = u32::try_from(input.parent.len()).map_err(|_| InputError::Overflow {
                field: "applied_force_nb",
            })?;
            let n = u32::try_from(input.body.len()).map_err(|_| InputError::Overflow {
                field: "applied_force_n",
            })?;
            Ok(Self {
                input: Arc::new(input),
                session: session.clone(),
                metadata: AppliedForceMetadataBuffer::upload(session, meta, [nb, n])?,
                validate: AppliedForceValidateKernel::compile(session)?,
                delta: AppliedForceDeltaKernel::compile(session)?,
                publish: AppliedForcePublishKernel::compile(session)?,
            })
        }
    }
    pub fn create_data(&self, worlds: usize) -> Result<AppliedForceData, TransferError> {
        let l = layout(worlds, self.input.parent.len(), self.input.body.len())?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = l;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let alloc = |count, initial| {
                let mut v = crate::runtime::host_staging::<f32>(count + 8)?;
                v.fill(-131072.0);
                v[4..4 + count].fill(initial);
                self.session.upload(&v)
            };
            Ok(AppliedForceData {
                input: self.input.clone(),
                layout: l,
                ready: false,
                geometry: alloc(l.geometry_len, 0.0)?,
                ft: alloc(l.ft.total_elements(), 0.0)?,
                current: alloc(l.vector.total_elements(), 0.0)?,
                delta: alloc(l.vector.total_elements(), f32::NAN)?,
                staged: alloc(l.vector.total_elements(), f32::NAN)?,
                status: alloc(worlds, f32::NAN)?,
            })
        }
    }
    /// U075语义：每次调用累加到当前广义力。
    pub fn accumulate(&self, data: &mut AppliedForceData) -> Result<(), TransferError> {
        self.apply(data, true)
    }
    pub fn replace(&self, data: &mut AppliedForceData) -> Result<(), TransferError> {
        self.apply(data, false)
    }
    pub(super) fn apply(
        &self,
        data: &mut AppliedForceData,
        add: bool,
    ) -> Result<(), TransferError> {
        if !Arc::ptr_eq(&self.input, &data.input) {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = add;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let nb = self.input.parent.len() as u32;
            let n = self.input.body.len() as u32;
            let worlds = data.worlds() as u32;
            let work = data.layout.vector.total_elements() as u32;
            self.validate.launch(
                &self.metadata,
                &data.geometry,
                &data.ft,
                &mut data.status,
                [nb, n, worlds, worlds],
            )?;
            let status = read(&data.status, worlds as usize, "applied_force_status")?;
            if let Some(index) = status[4..4 + worlds as usize]
                .iter()
                .position(|&v| v != 1.0)
            {
                return Err(InputError::InvalidTopology {
                    field: "applied_force_source",
                    index,
                    reason: "nonfinite_guard_or_mapping",
                }
                .into());
            }
            if add {
                read(
                    &data.current,
                    data.layout.vector.total_elements(),
                    "qfrc_current",
                )?;
            } else {
                guards(&data.current)?;
            }
            self.delta.launch(
                &self.metadata,
                &data.geometry,
                &data.ft,
                &mut data.delta,
                [nb, n, worlds, work],
            )?;
            read(
                &data.delta,
                data.layout.vector.total_elements(),
                "applied_force_delta",
            )?;
            self.publish.launch(
                self.metadata.as_buffer(),
                &data.current,
                &data.delta,
                &mut data.staged,
                [n, worlds, u32::from(add), work],
            )?;
            read(
                &data.staged,
                data.layout.vector.total_elements(),
                "qfrc_staged",
            )?;
            // 只有完整有限结果才替换当前缓冲。
            std::mem::swap(&mut data.current, &mut data.staged);
            data.ready = true;
            Ok(())
        }
    }
}
