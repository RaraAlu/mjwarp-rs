//! 计划构造与访问器。

#[cfg(feature = "cuda-probe")]
use super::kernels::SmoothForceComputeKernel;
#[cfg(feature = "cuda-probe")]
use super::kernels::SmoothForceValidateKernel;
#[cfg(feature = "cuda-probe")]
use super::launcher_abi::SmoothForceValidateDims;
#[cfg(feature = "cuda-probe")]
use super::metadata_buffer::SmoothForceMetadataBuffer;
use super::*;

impl SmoothForcePlan {
    pub fn new(
        session: &TransferSession,
        input: SmoothForceInput,
        options: SmoothForceOptions,
    ) -> Result<Self, TransferError> {
        layout(1, input.nbody(), input.nv(), input.ntree())?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (session, options);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let source = format!(
                "#define SMOOTH_SLEEP {}\n{}",
                i32::from(options.enable_sleep()),
                CUDA
            );
            Ok(Self {
                input: Arc::new(input),
                options,
                session: session.clone(),
                validate: SmoothForceValidateKernel::compile(
                    session,
                    &source,
                    options.enable_sleep(),
                )?,
                compute: SmoothForceComputeKernel::compile(
                    session,
                    &source,
                    options.enable_sleep(),
                )?,
            })
        }
    }
    pub fn input(&self) -> &SmoothForceInput {
        &self.input
    }
    pub fn options(&self) -> SmoothForceOptions {
        self.options
    }
    pub fn create_data(&self, worlds: usize) -> Result<SmoothForceData, TransferError> {
        let l = layout(
            worlds,
            self.input.nbody(),
            self.input.nv(),
            self.input.ntree(),
        )?;
        // 整数守卫包围静态映射与逐世界树状态。
        let mut meta = crate::runtime::host_staging::<i32>(l.meta_len)?;
        meta.fill(-131072);
        let body_end = 4 + self.input.nbody();
        let awake_start = body_end + self.input.nv();
        meta[4..body_end].copy_from_slice(self.input.body_treeid());
        meta[body_end..awake_start].copy_from_slice(self.input.dof_bodyid());
        meta[awake_start..l.meta_len - 4].fill(1);
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = meta;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let allocate = |count, initial| {
                let mut v = crate::runtime::host_staging::<f32>(count + 8)?;
                v.fill(-131072.0);
                v[4..4 + count].fill(initial);
                self.session.upload(&v)
            };
            Ok(SmoothForceData {
                input: self.input.clone(),
                layout: l,
                ready: false,
                metadata: SmoothForceMetadataBuffer::upload(
                    &self.session,
                    meta,
                    SmoothForceValidateDims {
                        abi: [
                            self.input.nbody() as u32,
                            self.input.nv() as u32,
                            self.input.ntree() as u32,
                            worlds as u32,
                        ],
                        sleep_enabled: self.options.enable_sleep(),
                    },
                )?,
                forces: allocate(l.force_len, 0.0)?,
                output: allocate(l.vector.total_elements(), f32::NAN)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }
    pub fn update(&self, data: &mut SmoothForceData) -> Result<(), TransferError> {
        #[cfg(feature = "cuda-probe")]
        {
            self.update_with_sleep_source(data, None)
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            if !Arc::ptr_eq(&self.input, &data.input) {
                return Err(TransferError::ModelMismatch);
            }
            data.ready = false;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn update_with_sleep_source(
        &self,
        data: &mut SmoothForceData,
        sleep: Option<&TransferBuffer<i32>>,
    ) -> Result<(), TransferError> {
        if !Arc::ptr_eq(&self.input, &data.input) {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        let nb = self.input.nbody() as u32;
        let n = self.input.nv() as u32;
        let nt = self.input.ntree() as u32;
        let worlds = data.worlds() as u32;
        if self.options.enable_sleep()
            && let Some(source) = sleep
        {
            let expected = data
                .worlds()
                .checked_mul(
                    self.input
                        .ntree()
                        .checked_mul(2)
                        .and_then(|n| n.checked_add(3))
                        .ok_or(InputError::Overflow {
                            field: "smooth_force_sleep_source",
                        })?,
                )
                .and_then(|n| n.checked_add(8))
                .ok_or(InputError::Overflow {
                    field: "smooth_force_sleep_source",
                })?;
            if source.len() != expected {
                return Err(InputError::LengthMismatch {
                    field: "smooth_force_sleep_source",
                    expected,
                    actual: source.len(),
                }
                .into());
            }
        }
        let sleep = sleep.unwrap_or(data.metadata.as_buffer());
        self.validate.launch(
            &data.metadata,
            &data.forces,
            sleep,
            &mut data.status,
            [nb, n, nt, worlds],
        )?;
        let mut status = crate::runtime::host_staging::<f32>(data.worlds() + 8)?;
        data.status.read_range_into(0, &mut status)?;
        crate::physics::check_device_values(&status, "smooth_force_status")?;
        if let Some(index) = status[4..4 + data.worlds()].iter().position(|&v| v != 1.0) {
            return Err(InputError::InvalidTopology {
                field: "smooth_force_source",
                index,
                reason: "nonfinite_guard_or_mapping",
            }
            .into());
        }
        self.compute.launch(
            &data.metadata,
            &data.forces,
            sleep,
            &mut data.output,
            [nb, n, nt, data.layout.vector.total_elements() as u32],
        )?;
        data.checked_values()?;
        data.ready = true;
        Ok(())
    }
}
