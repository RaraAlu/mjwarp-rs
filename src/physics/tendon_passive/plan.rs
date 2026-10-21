//! 肌腱被动力计划与更新。

#[cfg(feature = "cuda-probe")]
use super::cuda::CUDA;
use super::*;
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::TransferError;
#[cfg(feature = "cuda-probe")]
use crate::runtime::SynchronousKernel;

impl<'v, 'p> TendonPassivePlan<'v, 'p> {
    pub fn new(
        velocity: &'v TendonVelocityPlan<'p>,
        parameters: TendonPassiveParameters,
        options: TendonPassiveOptions,
    ) -> Result<Self, TransferError> {
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (velocity, parameters, options);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let (session, nv, nt, nnz, stride, max_row) = velocity.tendon_passive_context();
            let spec = spec((nv, nt, nnz, stride, max_row), parameters)?;
            layout(1, nv)?;
            let mut packed = vec![-131072.0; 4];
            for field in spec.parameters.fields() {
                packed.extend(field.values());
            }
            packed.extend([-131072.0; 4]);
            let periods = spec
                .parameters
                .fields()
                .map(|field| field.batches().to_string())
                .join(",");
            let offsets = spec.offsets.map(|offset| offset.to_string()).join(",");
            let code = format!(
                "#define TP_NV {nv}\n#define TP_NT {nt}\n#define TP_NNZ {nnz}\n#define TP_STRIDE {stride}\n#define TP_MAX_ROW {max_row}\n#define TP_PL {}\n#define TP_S {}\n#define TP_D {}\n__device__ __constant__ unsigned TP_BATCH[5]={{{periods}}};\n__device__ __constant__ unsigned TP_OFF[5]={{{offsets}}};\n{CUDA}",
                spec.parameter_len,
                u32::from(options.disable_spring),
                u32::from(options.disable_damper),
            );
            Ok(Self {
                velocity,
                spec: Arc::new(spec),
                options,
                parameters: session.upload(&packed)?,
                kernels: [
                    SynchronousKernel::compile(&session, &code, "tendon_passive_validate")?,
                    SynchronousKernel::compile(&session, &code, "tendon_passive_clear")?,
                    SynchronousKernel::compile(&session, &code, "tendon_passive_compute")?,
                ],
                session,
            })
        }
    }

    pub fn create_data(&self, worlds: usize) -> Result<TendonPassiveData, TransferError> {
        let layout = layout(worlds, self.spec.nv)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = layout;
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
            let count = 2 * layout.total_elements();
            Ok(TendonPassiveData {
                spec: self.spec.clone(),
                layout,
                ready: false,
                current: allocate(count, f32::NAN)?,
                staged: allocate(count, f32::NAN)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }

    pub fn options(&self) -> TendonPassiveOptions {
        self.options
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn shared_passive_context(
        &self,
    ) -> (TransferSession, usize, TendonPassiveOptions) {
        (self.session.clone(), self.spec.nv, self.options)
    }

    pub(in crate::physics) fn accepts_shared_velocity(
        &self,
        velocity: &TendonVelocityView<'_, 'p>,
    ) -> bool {
        velocity.belongs_to(self.velocity)
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn accumulate_shared_passive(
        &self,
        velocity: &TendonVelocityView<'_, 'p>,
        target: &mut TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        let input = velocity.tendon_passive_device()?;
        if !velocity.belongs_to(self.velocity)
            || input.worlds != worlds
            || target.len() != 2 * worlds * self.spec.nv + 8
            || status.len() != worlds + 8
        {
            return Err(TransferError::ModelMismatch);
        }
        let w = worlds as u32;
        // SAFETY: 速度视图锁定真实源。
        // 共享状态缓冲仅接收验证结果。
        unsafe {
            self.kernels[0].launch_tendon_passive(
                input.metadata,
                &self.parameters,
                input.source,
                input.velocity,
                status,
                [self.spec.nt as u32, self.spec.nv as u32, w, w],
            )?;
        }
        let checked = read(status, worlds, "passive_force_tendon_status")?;
        if let Some(index) = checked[4..4 + worlds].iter().position(|&v| v != 1.0) {
            return Err(InputError::InvalidTopology {
                field: "passive_force_tendon_source",
                index,
                reason: "guard_mapping_or_nonfinite",
            }
            .into());
        }
        let work = w
            .checked_mul(self.spec.nt as u32)
            .and_then(|n| n.checked_mul(self.spec.max_row as u32))
            .ok_or(InputError::Overflow {
                field: "passive_force_tendon_work",
            })?;
        // SAFETY: 每线程处理一个稀疏槽。
        // 目标先含关节贡献。
        unsafe {
            self.kernels[2].launch_tendon_passive(
                input.metadata,
                &self.parameters,
                input.source,
                input.velocity,
                target,
                [self.spec.nt as u32, self.spec.nv as u32, w, work],
            )
        }
    }

    pub fn update<'a>(
        &'a self,
        velocity: &'a TendonVelocityView<'a, 'p>,
        data: &'a mut TendonPassiveData,
    ) -> Result<TendonPassiveView<'a, 'v, 'p>, TransferError> {
        if !Arc::ptr_eq(&self.spec, &data.spec) {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = velocity;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let input = velocity.tendon_passive_device()?;
            if !velocity.belongs_to(self.velocity) || input.worlds != data.worlds() {
                return Err(TransferError::ModelMismatch);
            }
            let worlds = data.worlds() as u32;
            let mut guards = [0.0; 8];
            data.current.read_range_into(0, &mut guards[..4])?;
            data.current
                .read_range_into(data.current.len() - 4, &mut guards[4..])?;
            crate::physics::check_device_guards(&guards)?;
            // SAFETY: 速度视图锁定真实源。
            // 常量限定全部读取跨度。
            unsafe {
                self.kernels[0].launch_tendon_passive(
                    input.metadata,
                    &self.parameters,
                    input.source,
                    input.velocity,
                    &mut data.status,
                    [self.spec.nt as u32, self.spec.nv as u32, worlds, worlds],
                )?;
            }
            let status = read(&data.status, data.worlds(), "tendon_passive_status")?;
            if let Some(index) = status[4..4 + data.worlds()]
                .iter()
                .position(|&value| value != 1.0)
            {
                return Err(InputError::InvalidTopology {
                    field: "tendon_passive_source",
                    index,
                    reason: "guard_mapping_or_nonfinite",
                }
                .into());
            }
            let slots = worlds * self.spec.nv as u32;
            // SAFETY: 清零线程各自独占结果槽。
            unsafe {
                self.kernels[1].launch_tendon_passive(
                    input.metadata,
                    &self.parameters,
                    input.source,
                    input.velocity,
                    &mut data.staged,
                    [0, 0, worlds, 2 * slots],
                )?;
            }
            let work = worlds
                .checked_mul(self.spec.nt as u32)
                .and_then(|n| n.checked_mul(self.spec.max_row as u32))
                .ok_or(InputError::Overflow {
                    field: "tendon_passive_work",
                })?;
            // SAFETY: 每线程处理一个稀疏槽。
            // 共享DOF严格使用设备原子加。
            unsafe {
                self.kernels[2].launch_tendon_passive(
                    input.metadata,
                    &self.parameters,
                    input.source,
                    input.velocity,
                    &mut data.staged,
                    [self.spec.nt as u32, self.spec.nv as u32, worlds, work],
                )?;
            }
            read(
                &data.staged,
                2 * data.layout.total_elements(),
                "tendon_passive_staged",
            )?;
            std::mem::swap(&mut data.current, &mut data.staged);
            data.ready = true;
            Ok(TendonPassiveView {
                plan: self,
                velocity,
                data,
            })
        }
    }
}
