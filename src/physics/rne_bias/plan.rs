//! 偏置力计划构造、重力检查与更新入口。

use super::*;

pub(super) fn check_gravity(gravity: &[f32; 3]) -> Result<(), InputError> {
    if let Some(index) = gravity.iter().position(|v| !v.is_finite()) {
        return Err(InputError::NonFinite {
            field: "rne_bias_gravity",
            index,
        });
    }
    Ok(())
}

impl RneBiasPlan {
    /// 调用方显式提供共享重力。
    /// 本入口不转换完整原生Option。
    pub fn new(
        session: &TransferSession,
        model: AttachedModelInput,
        gravity: [f32; 3],
    ) -> Result<Self, TransferError> {
        check_gravity(&gravity)?;
        let k = model.rigid().kinematics();
        let (nbody, nv) = (k.nbody(), k.nv());
        RneBiasLayout::new(1, nbody, nv)?;
        #[cfg(feature = "cuda-probe")]
        let values = {
            let f = k.fields();
            let mut m = crate::runtime::host_staging::<i32>(5 + 3 * nbody + nv)?;
            m[0] = nbody as i32;
            m[1] = nv as i32;
            for i in 0..3 {
                m[2 + i] = gravity[i].to_bits() as i32;
            }
            m[5..5 + nbody].copy_from_slice(&f.body_parentid);
            for b in 1..nbody {
                let start = f.body_jntadr[b] as usize;
                let count = f.body_jntnum[b] as usize;
                if count != 0 {
                    m[5 + 2 * nbody + b] = f.jnt_dofadr[start];
                    m[5 + nbody + b] = f.jnt_type[start..start + count]
                        .iter()
                        .map(|&t| match t {
                            0 => 6,
                            1 => 3,
                            _ => 1,
                        })
                        .sum();
                }
            }
            m[5 + 3 * nbody..].copy_from_slice(&model.rigid().fields().dof_bodyid);
            m
        };
        let velocity = ComVelocityPlan::new(session, model)?;
        #[cfg(feature = "cuda-probe")]
        let metadata = RneBiasMetadataBuffer::upload(session, values, nbody, nv)?;
        #[cfg(feature = "cuda-probe")]
        let kernel = RneBiasKernel::compile(session)?;
        Ok(Self {
            velocity,
            nbody,
            nv,
            #[cfg(feature = "cuda-probe")]
            session: session.clone(),
            #[cfg(feature = "cuda-probe")]
            metadata,
            #[cfg(feature = "cuda-probe")]
            kernel,
        })
    }

    pub fn device(&self) -> usize {
        self.velocity.device()
    }

    pub fn create_data(&self, worlds: usize) -> Result<RneBiasData, TransferError> {
        let layout = RneBiasLayout::new(worlds, self.nbody, self.nv)?;
        let velocity = self.velocity.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (layout, velocity);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut values = crate::runtime::host_staging::<f32>(layout.guarded)?;
            values.fill(-131072.0);
            values[4..layout.guarded - 4].fill(f32::NAN);
            Ok(RneBiasData {
                velocity,
                layout,
                ready: false,
                output: self.session.upload(&values)?,
            })
        }
    }

    /// GPU更新位置、质心、速度与偏置力。
    /// 阶段间不回读宿主中间量。
    pub fn update(&self, data: &mut RneBiasData) -> Result<(), TransferError> {
        self.update_velocity(data)?;
        self.update_bias(data)
    }

    /// 显式准备本子集前置阶段。
    /// 本调用废弃已有偏置力。
    pub fn update_velocity(&self, data: &mut RneBiasData) -> Result<(), TransferError> {
        self.velocity.check_data(&data.velocity)?;
        data.ready = false;
        self.velocity.update(&mut data.velocity)
    }

    /// 只读取已有空间速度与质心。
    /// 不隐式重算任何前置阶段。
    /// 本严格子集要求新鲜度。
    /// 它不替代完整等价rne入口。
    pub fn update_bias(&self, data: &mut RneBiasData) -> Result<(), TransferError> {
        self.velocity.check_data(&data.velocity)?;
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let velocity_inputs = data.velocity.rne_inputs()?;
            let inputs = RneBiasInputs {
                metadata: &self.metadata,
                qvel: velocity_inputs.qvel,
                com: velocity_inputs.com,
                velocity: velocity_inputs.velocity,
            };
            let launch = self.kernel.preflight(
                &self.session,
                &inputs,
                &data.output,
                data.layout.output.worlds() as u32,
            )?;
            self.kernel.launch(&launch, &inputs, &mut data.output)?;
            data.ready = true;
            Ok(())
        }
    }
}
