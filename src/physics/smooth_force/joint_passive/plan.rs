//! 关节被动计划构造、写入与更新入口。

use super::*;

impl JointPassivePlan {
    pub fn new(
        session: &TransferSession,
        model: KinematicModelInput,
        params: JointPassiveParameters,
        options: JointPassiveOptions,
    ) -> Result<Self, TransferError> {
        let i = input(model, params, options)?;
        debug_assert_eq!(i.meta.len(), 4 * i.model.nv() + 8);
        debug_assert_eq!(
            i.offsets[4] + i.params.damping_poly.values().len(),
            i.param_len
        );
        layout(i.model.nq(), i.model.nv(), 1)?;
        let sm = SmoothForceInput::new(0, vec![-1; i.model.nbody()], i.body.clone())?;
        let smooth = SmoothForcePlan::new(session, sm, SmoothForceOptions::default())?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (i, smooth);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut p = vec![-131072.0; 4];
            for f in i.params.fields() {
                p.extend(f.values());
            }
            p.extend([-131072.0; 4]);
            let periods = i.params.fields().map(|p| p.batches().to_string()).join(",");
            let offsets = i.offsets.map(|v| v.to_string()).join(",");
            let code = format!(
                "#define JP_NQ {}\n#define JP_NV {}\n#define JP_NJ {}\n#define JP_PL {}\n#define JP_S {}\n#define JP_D {}\n__device__ __constant__ unsigned JP_BATCH[5]={{{periods}}};\n__device__ __constant__ unsigned JP_OFF[5]={{{offsets}}};\n{CUDA}",
                i.model.nq(),
                i.model.nv(),
                i.model.njnt(),
                i.param_len,
                u32::from(options.disable_spring),
                u32::from(options.disable_damper)
            );
            let static_dims = launcher_abi::JointPassiveStaticDims::new(
                i.model.nq(),
                i.model.nv(),
                i.model.njnt(),
                i.param_len,
            )
            .map_err(launcher_abi::joint_passive_launch_error)?;
            let metadata = launcher_abi::JointPassiveMetadataBuffer::from_values(
                session,
                i.meta.clone(),
                static_dims,
            )?;
            Ok(Self {
                input: Arc::new(i),
                options,
                smooth,
                session: session.clone(),
                metadata,
                parameters: session.upload(&p)?,
                kernels: kernels::JointPassiveKernels::compile(session, &code)?,
            })
        }
    }
    pub fn create_data(&self, worlds: usize) -> Result<JointPassiveData, TransferError> {
        let m = &self.input.model;
        let l = layout(m.nq(), m.nv(), worlds)?;
        let smooth = self.smooth.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (l, smooth);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let alloc = |count: usize, value| {
                let mut v = crate::runtime::host_staging::<f32>(count + 8)?;
                v.fill(-131072.0);
                v[4..4 + count].fill(value);
                self.session.upload(&v)
            };
            let mut state = alloc(l.state.total_elements(), 0.0)?;
            for w in 0..worlds {
                state.write_range(4 + w * m.nq(), &m.fields().qpos0)?;
            }
            Ok(JointPassiveData {
                input: self.input.clone(),
                layout: l,
                ready: false,
                smooth,
                source: None,
                state,
                current: alloc(l.parts.total_elements(), f32::NAN)?,
                staged: alloc(l.parts.total_elements(), f32::NAN)?,
                status: alloc(worlds, f32::NAN)?,
            })
        }
    }
    pub fn options(&self) -> JointPassiveOptions {
        self.options
    }
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn shared_passive_model(&self) -> &KinematicModelInput {
        &self.input.model
    }
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn shared_passive_context(
        &self,
    ) -> (
        TransferSession,
        SmoothForceInput,
        usize,
        JointPassiveOptions,
    ) {
        (
            self.session.clone(),
            self.smooth.input().clone(),
            self.input.model.nv(),
            self.options,
        )
    }
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn write_shared_passive_base(
        &self,
        data: &JointPassiveData,
        target: &mut TransferBuffer<f32>,
        status: &mut TransferBuffer<f32>,
        worlds: usize,
    ) -> Result<(), TransferError> {
        if !Arc::ptr_eq(&self.input, &data.input)
            || data.worlds() != worlds
            || target.len() != 2 * worlds * self.input.model.nv() + 8
            || status.len() != worlds + 8
        {
            return Err(TransferError::ModelMismatch);
        }
        let mut guards = [0.0; 8];
        target.read_range_into(0, &mut guards[..4])?;
        target.read_range_into(target.len() - 4, &mut guards[4..])?;
        crate::physics::check_device_guards(&guards)?;
        let w = worlds as u32;
        self.kernels
            .launch_validate(&self.metadata, &self.parameters, &data.state, status, w)?;
        let checked = read(status, worlds, "passive_force_joint_status")?;
        if let Some(index) = checked[4..4 + worlds].iter().position(|&v| v != 1.0) {
            return Err(InputError::InvalidTopology {
                field: "passive_force_joint_source",
                index,
                reason: "nonfinite_guard_or_mapping",
            }
            .into());
        }
        self.kernels
            .launch_compute(&self.metadata, &self.parameters, &data.state, target, w)
    }
    pub fn update_joint(&self, data: &mut JointPassiveData) -> Result<(), TransferError> {
        if !Arc::ptr_eq(&self.input, &data.input) {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        data.smooth.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let w = data.worlds() as u32;
            let mut guards = [0.0; 8];
            data.current.read_range_into(0, &mut guards[..4])?;
            data.current
                .read_range_into(data.current.len() - 4, &mut guards[4..])?;
            crate::physics::check_device_guards(&guards)?;
            self.kernels.launch_validate(
                &self.metadata,
                &self.parameters,
                &data.state,
                &mut data.status,
                w,
            )?;
            let status = read(&data.status, data.worlds(), "joint_passive_status")?;
            if let Some(index) = status[4..4 + data.worlds()].iter().position(|&v| v != 1.0) {
                return Err(InputError::InvalidTopology {
                    field: "joint_passive_source",
                    index,
                    reason: "nonfinite_guard_or_mapping",
                }
                .into());
            }
            self.kernels.launch_compute(
                &self.metadata,
                &self.parameters,
                &data.state,
                &mut data.staged,
                w,
            )?;
            read(
                &data.staged,
                data.layout.parts.total_elements(),
                "joint_passive_staged",
            )?;
            std::mem::swap(&mut data.current, &mut data.staged);
            data.ready = true;
            Ok(())
        }
    }
    /// 只合并本关节子集；不冒充gravcomp/fluid等完整被动力。
    pub fn update_joint_only_smooth(
        &self,
        data: &mut JointPassiveData,
    ) -> Result<(), TransferError> {
        self.update_joint(data)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let w = data.worlds() as u32;
            self.kernels.launch_sum(
                &self.metadata,
                &data.current,
                &data.current,
                &mut data.smooth.forces,
                w,
            )?;
            self.smooth.update(&mut data.smooth)
        }
    }
}
