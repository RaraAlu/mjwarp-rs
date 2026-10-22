use super::spec::*;
#[cfg(feature = "cuda-probe")]
use super::validation::*;
use super::*;

pub struct MassMatrixEnergyPlan<'plan, 'matrix> {
    pub(super) multiply: &'plan MassMatrixMultiplyPlan<'matrix>,
    pub(super) spec: Arc<EnergySpec>,
    #[cfg(feature = "cuda-probe")]
    pub(super) session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    pub(super) metadata: KineticEnergyMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    pub(super) source_validator: DeviceEnergySourceValidationKernel,
    #[cfg(feature = "cuda-probe")]
    pub(super) scalar_validator: KineticValidationKernel,
    #[cfg(feature = "cuda-probe")]
    pub(super) reducer: KineticEnergy32Kernel,
}
pub struct MassMatrixEnergyData {
    pub(super) spec: Arc<EnergySpec>,
    pub(super) layout: Layout,
    #[cfg(feature = "cuda-probe")]
    pub(super) pair: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    pub(super) current: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    pub(super) staged: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    pub(super) status: TransferBuffer<f32>,
}
/// 同时借用原乘积视图与能量数据，不认证未来状态。
/// 当前态证明仅覆盖动能；势能由调用方提供。
pub struct MassMatrixEnergyView<'energy, 'source> {
    pub(super) input: &'energy MassMatrixMultiplyView<'source>,
    pub(super) data: &'energy MassMatrixEnergyData,
}
#[derive(Clone, Debug)]
pub struct MassMatrixEnergySnapshot {
    pub(super) layout: BatchLayout,
    pub(super) values: Vec<f32>,
}
impl MassMatrixEnergySnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    /// 返回历史`[势能,动能]`，不携带伪造状态标签。
    pub fn world(&self, world: usize) -> Result<&[f32], InputError> {
        let r = self.layout.world_elements(world)?;
        Ok(&self.values[4 + r.start..4 + r.end])
    }
}
impl MassMatrixEnergyView<'_, '_> {
    pub fn worlds(&self) -> usize {
        debug_assert_eq!(self.input.worlds(), self.data.worlds());
        self.input.worlds()
    }
    pub fn snapshot(&self) -> Result<MassMatrixEnergySnapshot, TransferError> {
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            Ok(MassMatrixEnergySnapshot {
                layout: self.data.layout.energy,
                values: super::super::read(
                    &self.data.current,
                    self.data.layout.energy.total_elements(),
                    "device_energy",
                )?,
            })
        }
    }
}
impl<'plan, 'matrix> MassMatrixEnergyPlan<'plan, 'matrix> {
    pub fn new(multiply: &'plan MassMatrixMultiplyPlan<'matrix>) -> Result<Self, TransferError> {
        let n = multiply.spec.nv;
        layout(n, 1)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let session = multiply.session.clone();
            Ok(Self {
                multiply,
                spec: Arc::new(EnergySpec {
                    mass: multiply.spec.clone(),
                }),
                metadata: KineticEnergyMetadataBuffer::upload(&session, n as u32)?,
                source_validator: DeviceEnergySourceValidationKernel::compile(&session, n as u32)?,
                scalar_validator: KineticValidationKernel::compile(&session, n as u32)?,
                reducer: KineticEnergy32Kernel::compile(&session, n as u32)?,
                session,
            })
        }
    }
    pub fn create_data(&self, worlds: usize) -> Result<MassMatrixEnergyData, TransferError> {
        let l = layout(self.spec.mass.nv, worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = l;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let alloc = |count: usize, initial| {
                let mut v = crate::runtime::host_staging::<f32>(count + 8)?;
                v.fill(-131072.0);
                v[4..4 + count].fill(initial);
                self.session.upload(&v)
            };
            let mut current = alloc(l.energy.total_elements(), f32::NAN)?;
            for w in 0..worlds {
                current.write_range(4 + 2 * w, &[0.0])?;
            }
            Ok(MassMatrixEnergyData {
                spec: self.spec.clone(),
                layout: l,
                pair: alloc(l.pair.total_elements(), 0.0)?,
                current,
                staged: alloc(l.energy.total_elements(), f32::NAN)?,
                status: alloc(worlds, f32::NAN)?,
            })
        }
    }
    pub fn update<'energy, 'source>(
        &self,
        data: &'energy mut MassMatrixEnergyData,
        input: &'energy MassMatrixMultiplyView<'source>,
    ) -> Result<MassMatrixEnergyView<'energy, 'source>, TransferError> {
        if !binding(&self.spec, &data.spec, &input.data.spec) {
            return Err(TransferError::ModelMismatch);
        }
        self.multiply.matrix.check_data(input.source)?;
        if input.worlds() != data.worlds() || input.computed.len() != data.worlds() {
            return Err(InputError::InvalidDimension {
                field: "device_energy_worlds",
            }
            .into());
        }
        all_current(&input.computed)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            // Recheck the actual source ticket; historical solver or energy
            // snapshots never substitute for the live matrix/source borrow.
            input.source.matrix_input()?;
            let n = self.spec.mass.nv;
            let worlds = data.worlds();
            if input.data.vector.len() != worlds * n + 8
                || input.data.current.len() != worlds * n + 8
            {
                return Err(TransferError::ModelMismatch);
            }
            let dims = [n as u32, worlds as u32, 0, worlds as u32];
            self.source_validator.launch(
                &self.metadata,
                &input.data.vector,
                &input.data.current,
                &mut data.status,
                dims,
            )?;
            status(&data.status, worlds)?;
            // These are explicit GPU-to-GPU copies, not a zero-copy pair and
            // never a download/upload round trip of physical values.
            for w in 0..worlds {
                data.pair
                    .copy_range_from(4 + 2 * w * n, &input.data.vector, 4 + w * n, n)?;
                data.pair.copy_range_from(
                    4 + (2 * w + 1) * n,
                    &input.data.current,
                    4 + w * n,
                    n,
                )?;
            }
            self.scalar_validator.launch(
                &self.metadata,
                &data.pair,
                &data.current,
                &mut data.status,
                dims,
            )?;
            status(&data.status, worlds)?;
            self.reducer.launch(
                &self.metadata,
                &data.pair,
                &data.current,
                &mut data.staged,
                dims,
            )?;
            super::super::read(
                &data.staged,
                data.layout.energy.total_elements(),
                "device_energy_staged",
            )?;
            std::mem::swap(&mut data.current, &mut data.staged);
            Ok(MassMatrixEnergyView { input, data })
        }
    }
}
impl MassMatrixEnergyData {
    pub fn worlds(&self) -> usize {
        debug_assert_eq!(self.layout.pair.worlds(), self.layout.energy.worlds());
        self.layout.energy.worlds()
    }
    pub fn write_potential(&mut self, values: &[f32]) -> Result<(), TransferError> {
        super::super::checked(values, self.worlds(), "device_energy_potential")?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            for (w, &v) in values.iter().enumerate() {
                self.current.write_range(4 + 2 * w, &[v])?;
            }
            Ok(())
        }
    }
    pub fn write_world_potential(&mut self, world: usize, value: f32) -> Result<(), TransferError> {
        self.layout.energy.world_elements(world)?;
        super::super::checked(&[value], 1, "device_energy_potential")?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.current.write_range(4 + 2 * world, &[value])
        }
    }
}
#[cfg(feature = "cuda-probe")]
pub(super) fn status(buffer: &TransferBuffer<f32>, worlds: usize) -> Result<(), TransferError> {
    let values = super::super::read(buffer, worlds, "device_energy_status")?;
    if let Some(index) = values[4..4 + worlds].iter().position(|&v| v != 1.0) {
        return Err(InputError::InvalidTopology {
            field: "device_energy_source",
            index,
            reason: "nonfinite_guard_or_shape",
        }
        .into());
    }
    Ok(())
}
#[cfg(feature = "cuda-probe")]
pub(super) const SOURCE_VALIDATE: &str = r#"
extern "C" __global__ void device_energy_source_validate(const int* meta,const float* q,const float* mq,float* status,
 unsigned n,unsigned worlds,unsigned unused,unsigned work){
 unsigned w=blockIdx.x*blockDim.x+threadIdx.x;if(w>=work)return;
 bool valid=n==ENERGY_NV&&meta[0]==ENERGY_NV;
 for(unsigned j=0;j<4;++j)valid=valid&&q[j]==-131072.0f&&mq[j]==-131072.0f&&q[4+1ull*n*worlds+j]==-131072.0f&&mq[4+1ull*n*worlds+j]==-131072.0f;
 for(unsigned j=0;j<n;++j)valid=valid&&isfinite(q[4+1ull*w*n+j])&&isfinite(mq[4+1ull*w*n+j]);
 status[4+w]=valid?1.0f:0.0f;
}
"#;
