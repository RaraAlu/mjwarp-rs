//! 重力补偿计划、数据与视图实现。

#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::TransferError;
#[cfg(feature = "cuda-probe")]
use crate::runtime::{SynchronousKernel, TransferBuffer};
use std::sync::Arc;

#[cfg(feature = "cuda-probe")]
use super::cuda::CUDA;
use super::*;

impl<'p> GravityCompensationPlan<'p> {
    pub fn new(
        source: &'p KinematicsPlan,
        parameters: GravityCompensationParameters,
        options: GravityCompensationOptions,
    ) -> Result<Self, TransferError> {
        let (session, model, body_mass) = source.gravity_compensation_context()?;
        let spec = Arc::new(spec(model, body_mass, parameters, options)?);
        layout(1, spec.nv)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (session, spec);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut packed = vec![-131072.0; 4];
            for field in [
                &spec.parameters.gravity,
                &spec.body_mass,
                &spec.parameters.body_gravcomp,
            ] {
                packed.extend(field.values());
            }
            packed.extend([-131072.0; 4]);
            let batches = [
                spec.parameters.gravity.batches(),
                spec.body_mass.batches(),
                spec.parameters.body_gravcomp.batches(),
            ]
            .map(|value| value.to_string())
            .join(",");
            let offsets = spec.offsets.map(|value| value.to_string()).join(",");
            let rigid_stride = spec.nb * 28 + spec.nj * 6;
            let com_stride = spec.nb * 14 + spec.nv * 6;
            let code = format!(
                "#define GC_NB {}\n#define GC_NJ {}\n#define GC_NV {}\n#define GC_PL {}\n#define GC_RS {}\n#define GC_CS {}\n#define GC_ENABLED {}\n__device__ __constant__ unsigned GC_BATCH[3]={{{batches}}};\n__device__ __constant__ unsigned GC_OFF[3]={{{offsets}}};\n{CUDA}",
                spec.nb,
                spec.nj,
                spec.nv,
                spec.parameter_len,
                rigid_stride,
                com_stride,
                u32::from(!options.disable_gravity),
            );
            Ok(Self {
                source,
                spec: spec.clone(),
                options,
                metadata: session.upload(&spec.metadata)?,
                parameters: session.upload(&packed)?,
                kernels: [
                    SynchronousKernel::compile(&session, &code, "gravity_compensation_validate")?,
                    SynchronousKernel::compile(&session, &code, "gravity_compensation_clear")?,
                    SynchronousKernel::compile(&session, &code, "gravity_compensation_compute")?,
                    SynchronousKernel::compile(&session, &code, "gravity_compensation_sum")?,
                ],
                session,
            })
        }
    }

    pub fn options(&self) -> GravityCompensationOptions {
        self.options
    }

    pub fn create_data(&self, worlds: usize) -> Result<GravityCompensationData, TransferError> {
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
            Ok(GravityCompensationData {
                spec: self.spec.clone(),
                layout,
                ready: false,
                current: allocate(layout.total_elements(), f32::NAN)?,
                staged: allocate(layout.total_elements(), f32::NAN)?,
                status: allocate(worlds, f32::NAN)?,
            })
        }
    }

    pub fn update<'a>(
        &'a self,
        source: &'a KinematicsData,
        data: &'a mut GravityCompensationData,
    ) -> Result<GravityCompensationView<'a>, TransferError> {
        if !Arc::ptr_eq(&self.spec, &data.spec)
            || data.worlds() != source.worlds()
            || self.source.check_data(source).is_err()
        {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = source;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let worlds = data.worlds();
            self.write(source, &mut data.staged, &mut data.status, worlds)?;
            read_output(
                &data.staged,
                data.layout.total_elements(),
                "gravity_compensation_staged",
            )?;
            std::mem::swap(&mut data.current, &mut data.staged);
            data.ready = true;
            Ok(GravityCompensationView {
                data,
                _source: source,
            })
        }
    }

    pub(in crate::physics) fn accepts_source(&self, source: &KinematicsData) -> bool {
        source.worlds() > 0 && self.source.check_data(source).is_ok()
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    pub(in crate::physics) fn test_write_root(
        &mut self,
        body: usize,
        value: i32,
    ) -> Result<(), TransferError> {
        self.metadata.write_range(4 + body, &[value])
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    pub(in crate::physics) fn test_write_ancestor(
        &mut self,
        body: usize,
        dof: usize,
        value: i32,
    ) -> Result<(), TransferError> {
        self.metadata
            .write_range(4 + self.spec.nb + body * self.spec.nv + dof, &[value])
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    pub(in crate::physics) fn test_write_dof_joint(
        &mut self,
        dof: usize,
        value: i32,
    ) -> Result<(), TransferError> {
        self.metadata.write_range(
            4 + self.spec.nb + self.spec.nb * self.spec.nv + dof,
            &[value],
        )
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn write_shared(
        &self,
        source: &KinematicsData,
        target: &mut TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        if source.worlds() != worlds
            || self.source.check_data(source).is_err()
            || target.len() != worlds * self.spec.nv + 8
            || status.len() != worlds + 8
        {
            return Err(TransferError::ModelMismatch);
        }
        self.write(source, target, status, worlds)
    }

    #[cfg(feature = "cuda-probe")]
    fn write(
        &self,
        source: &KinematicsData,
        target: &mut TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        let slots = worlds
            .checked_mul(self.spec.nv)
            .ok_or(InputError::Overflow {
                field: "gravity_compensation_work",
            })?;
        let mut guards = [0.0; 8];
        target.read_range_into(0, &mut guards[..4])?;
        target.read_range_into(target.len() - 4, &mut guards[4..])?;
        crate::physics::check_device_guards(&guards)?;
        // SAFETY: 每线程独占一个输出槽。
        unsafe {
            self.kernels[1].launch(
                &self.metadata,
                &self.parameters,
                &self.parameters,
                target,
                [
                    self.spec.nb as u32,
                    self.spec.nv as u32,
                    worlds as u32,
                    slots as u32,
                ],
            )?;
        }
        if !self.spec.active {
            return Ok(());
        }
        let (rigid, com) = self.source.gravity_compensation_source(source)?;
        // SAFETY: 校验线程只写本世界状态。
        unsafe {
            self.kernels[0].launch_gravity_compensation(
                &self.metadata,
                &self.parameters,
                rigid,
                com,
                status,
                [
                    self.spec.nb as u32,
                    self.spec.nv as u32,
                    worlds as u32,
                    worlds as u32,
                ],
            )?;
        }
        let checked = read_output(status, worlds, "gravity_compensation_status")?;
        if let Some(index) = checked[4..4 + worlds]
            .iter()
            .position(|&value| value != 1.0)
        {
            return Err(InputError::InvalidTopology {
                field: "gravity_compensation_source",
                index,
                reason: "guard_mapping_or_nonfinite",
            }
            .into());
        }
        let work = worlds
            .checked_mul(self.spec.nb - 1)
            .and_then(|value| value.checked_mul(self.spec.nv))
            .filter(|&value| value <= (u32::MAX - 255) as usize)
            .ok_or(InputError::Overflow {
                field: "gravity_compensation_work",
            })?;
        // SAFETY: 每线程处理一个刚体自由度对。
        // 共享自由度继续使用设备原子加。
        unsafe {
            self.kernels[2].launch_gravity_compensation(
                &self.metadata,
                &self.parameters,
                rigid,
                com,
                target,
                [
                    self.spec.nb as u32,
                    self.spec.nv as u32,
                    worlds as u32,
                    work as u32,
                ],
            )
        }
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn sum_shared(
        &self,
        parts: &TransferBuffer<f32>,
        gravity: &TransferBuffer<f32>,
        output: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        let work = worlds
            .checked_mul(self.spec.nv)
            .ok_or(InputError::Overflow {
                field: "gravity_compensation_sum",
            })?;
        // SAFETY: 每线程独占一个最终被动力槽。
        unsafe {
            self.kernels[3].launch_gravity_compensation(
                &self.metadata,
                parts,
                gravity,
                gravity,
                output,
                [
                    self.spec.nb as u32,
                    self.spec.nv as u32,
                    worlds as u32,
                    work as u32,
                ],
            )
        }
    }
}

impl GravityCompensationData {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }

    fn snapshot(&self) -> Result<GravityCompensationSnapshot, TransferError> {
        if !self.ready {
            return Err(TransferError::StageNotReady {
                stage: "gravity_compensation",
            });
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(GravityCompensationSnapshot {
                layout: self.layout,
                values: read_output(
                    &self.current,
                    self.layout.total_elements(),
                    "gravity_compensation_output",
                )?,
            })
        }
    }
}

impl GravityCompensationView<'_> {
    pub fn snapshot(&self) -> Result<GravityCompensationSnapshot, TransferError> {
        self.data.snapshot()
    }
}

#[cfg(feature = "cuda-probe")]
fn read_output(
    buffer: &TransferBuffer<f32>,
    count: usize,
    field: &'static str,
) -> Result<Vec<f32>, TransferError> {
    let mut values = crate::runtime::host_staging::<f32>(count + 8)?;
    buffer.read_range_into(0, &mut values)?;
    crate::physics::check_device_values(&values, field)?;
    Ok(values)
}
