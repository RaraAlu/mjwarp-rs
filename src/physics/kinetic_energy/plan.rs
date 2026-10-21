//! 动能计划构造、写入与更新入口。

#[cfg(feature = "cuda-probe")]
use super::data::read;
use super::*;

impl KineticEnergyPlan {
    pub fn new(session: &TransferSession, model_id: u64, nv: usize) -> Result<Self, TransferError> {
        let s = spec(model_id, nv)?;
        layout(1, nv)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (session, s);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let nv = nv as u32;
            Ok(Self {
                spec: Arc::new(s),
                session: session.clone(),
                metadata: KineticEnergyMetadataBuffer::upload(session, nv)?,
                validator: KineticValidationKernel::compile(session, nv)?,
                reducer: KineticEnergy32Kernel::compile(session, nv)?,
            })
        }
    }
    pub fn create_data(&self, worlds: usize) -> Result<KineticEnergyData, TransferError> {
        let l = layout(worlds, self.spec.nv)?;
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
            Ok(KineticEnergyData {
                spec: self.spec.clone(),
                layout: l,
                states: vec![None; worlds],
                ready: false,
                pair: alloc(l.pair.total_elements(), 0.0)?,
                current,
                staged: alloc(l.energy.total_elements(), f32::NAN)?,
                status: alloc(worlds, f32::NAN)?,
            })
        }
    }
    pub fn update(&self, d: &mut KineticEnergyData) -> Result<(), TransferError> {
        if !Arc::ptr_eq(&self.spec, &d.spec) {
            return Err(TransferError::ModelMismatch);
        }
        if d.states.iter().any(Option::is_none) {
            return Err(TransferError::StageNotReady {
                stage: "kinetic_energy_pair",
            });
        }
        d.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let dims = [self.spec.nv as u32, d.worlds() as u32, 0, d.worlds() as u32];
            self.validator
                .launch(&self.metadata, &d.pair, &d.current, &mut d.status, dims)?;
            let status = read(&d.status, d.worlds(), "kinetic_energy_status")?;
            if let Some(index) = status[4..4 + d.worlds()].iter().position(|&v| v != 1.0) {
                return Err(InputError::InvalidTopology {
                    field: "kinetic_energy_source",
                    index,
                    reason: "nonfinite_guard_or_shape",
                }
                .into());
            }
            self.reducer
                .launch(&self.metadata, &d.pair, &d.current, &mut d.staged, dims)?;
            read(&d.staged, d.layout.energy.total_elements(), "energy_staged")?;
            std::mem::swap(&mut d.current, &mut d.staged);
            d.ready = true;
            Ok(())
        }
    }
}
