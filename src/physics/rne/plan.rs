//! RNE计划构造与访问器。

use super::data::check_values;
#[cfg(feature = "cuda-probe")]
use super::kernels::RneBackwardKernel;
#[cfg(feature = "cuda-probe")]
use super::kernels::RneBackwardRequest;
#[cfg(feature = "cuda-probe")]
use super::kernels::RneCaccKernel;
#[cfg(feature = "cuda-probe")]
use super::kernels::RneCfrcKernel;
#[cfg(feature = "cuda-probe")]
use super::kernels::RneQfrcKernel;
#[cfg(feature = "cuda-probe")]
use super::metadata_buffer::RneMetadataBuffer;
use super::*;

impl RnePlan {
    pub fn new(
        session: &TransferSession,
        model: AttachedModelInput,
        gravity: [f32; 3],
    ) -> Result<Self, TransferError> {
        check_values("rne_gravity", 3, &gravity)?;
        let k = model.rigid().kinematics();
        let (nbody, nv) = (k.nbody(), k.nv());
        RneLayout::new(1, nbody, nv)?;
        #[cfg(feature = "cuda-probe")]
        let (metadata, depth) = {
            let f = k.fields();
            let mut m = crate::runtime::host_staging::<i32>(5 + 4 * nbody + nv)?;
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
                m[5 + 3 * nbody + nv + b] = m[5 + 3 * nbody + nv + f.body_parentid[b] as usize] + 1;
            }
            m[5 + 3 * nbody..5 + 3 * nbody + nv]
                .copy_from_slice(&model.rigid().fields().dof_bodyid);
            let depth = *m[5 + 3 * nbody + nv..].iter().max().unwrap() as usize;
            (
                RneMetadataBuffer::upload(session, m, nbody, nv, depth)?,
                depth,
            )
        };
        let velocity = ComVelocityPlan::new(session, model)?;
        #[cfg(feature = "cuda-probe")]
        let backward_kernel = RneBackwardKernel::compile(session)?;
        #[cfg(feature = "cuda-probe")]
        let cacc_kernel = RneCaccKernel::compile(session)?;
        #[cfg(feature = "cuda-probe")]
        let cfrc_kernel = RneCfrcKernel::compile(session)?;
        #[cfg(feature = "cuda-probe")]
        let qfrc_kernel = RneQfrcKernel::compile(session)?;
        Ok(Self {
            velocity,
            nbody,
            nv,
            #[cfg(feature = "cuda-probe")]
            depth,
            #[cfg(feature = "cuda-probe")]
            session: session.clone(),
            #[cfg(feature = "cuda-probe")]
            metadata,
            #[cfg(feature = "cuda-probe")]
            backward_kernel,
            #[cfg(feature = "cuda-probe")]
            cacc_kernel,
            #[cfg(feature = "cuda-probe")]
            cfrc_kernel,
            #[cfg(feature = "cuda-probe")]
            qfrc_kernel,
        })
    }

    pub fn device(&self) -> usize {
        self.velocity.device()
    }

    pub fn create_data(&self, worlds: usize) -> Result<RneData, TransferError> {
        let layout = RneLayout::new(worlds, self.nbody, self.nv)?;
        let velocity = self.velocity.create_data(worlds)?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (layout, velocity);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let mut qacc = crate::runtime::host_staging::<f32>(layout.qacc.total_elements() + 8)?;
            qacc.fill(-131072.0);
            qacc[4..layout.qacc.total_elements() + 4].fill(0.0);
            let mut values = crate::runtime::host_staging::<f32>(layout.guarded)?;
            values.fill(-131072.0);
            values[4..layout.guarded - 4].fill(f32::NAN);
            Ok(RneData {
                velocity,
                layout,
                ready: false,
                flg_acc: false,
                qacc: self.session.upload(&qacc)?,
                output: self.session.upload(&values)?,
            })
        }
    }

    pub fn update(&self, data: &mut RneData, flg_acc: bool) -> Result<(), TransferError> {
        self.update_velocity(data)?;
        self.update_rne(data, flg_acc)
    }

    pub fn update_velocity(&self, data: &mut RneData) -> Result<(), TransferError> {
        self.velocity.check_data(&data.velocity)?;
        data.ready = false;
        self.velocity.update(&mut data.velocity)
    }

    /// 只消费已就绪的前置结果。
    /// 本严格辅助不隐式补算前置。
    pub fn update_rne(&self, data: &mut RneData, flg_acc: bool) -> Result<(), TransferError> {
        self.velocity.check_data(&data.velocity)?;
        data.ready = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = flg_acc;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let inputs = data.velocity.rne_inputs()?;
            let worlds = data.worlds() as u32;
            let cacc_launch = self.cacc_kernel.preflight(
                &self.session,
                &self.metadata,
                inputs.qvel,
                &data.qacc,
                inputs.com,
                inputs.velocity,
                &data.output,
                flg_acc,
                worlds,
            )?;
            let cfrc_launch = self.cfrc_kernel.preflight(
                &self.session,
                &self.metadata,
                inputs.com,
                inputs.velocity,
                &data.output,
                worlds,
            )?;
            let qfrc_launch = self.qfrc_kernel.preflight(
                &self.session,
                &self.metadata,
                inputs.com,
                inputs.velocity,
                &data.output,
                worlds,
            )?;
            let mut backward_launches = Vec::with_capacity(self.depth);
            for level in (1..=self.depth).rev() {
                backward_launches.push(self.backward_kernel.preflight(
                    &self.session,
                    &self.metadata,
                    inputs.com,
                    inputs.velocity,
                    &data.output,
                    RneBackwardRequest {
                        worlds,
                        level: level as u32,
                    },
                )?);
            }
            self.cacc_kernel.launch(
                &cacc_launch,
                &self.metadata,
                inputs.qvel,
                &data.qacc,
                inputs.com,
                inputs.velocity,
                &mut data.output,
            )?;
            self.cfrc_kernel.launch(
                &cfrc_launch,
                &self.metadata,
                inputs.com,
                inputs.velocity,
                &mut data.output,
            )?;
            // 每次同步发射形成反向层级屏障。
            // 同层兄弟仍通过atomicAdd归约。
            for launch in &backward_launches {
                self.backward_kernel.launch(
                    launch,
                    &self.metadata,
                    inputs.com,
                    inputs.velocity,
                    &mut data.output,
                )?;
            }
            self.qfrc_kernel.launch(
                &qfrc_launch,
                &self.metadata,
                inputs.com,
                inputs.velocity,
                &mut data.output,
            )?;
            data.flg_acc = flg_acc;
            data.ready = true;
            Ok(())
        }
    }
}
