//! 被动合力更新、清零与休眠过滤入口。

use super::data::check_shared_source_generation;
use super::*;

impl<'a, 'v, 'p, 'f> PassiveForcePlan<'a, 'v, 'p, 'f> {
    /// 调用方须同步全部qpos与qvel副本。
    /// 本入口拒绝不同常驻源与不同世界数。
    /// 启用休眠时借用同一常驻源。
    pub fn update<'b, 's>(
        &'b self,
        joint: &'b JointPassiveData,
        tendon_velocity: &'b TendonVelocityView<'s, 'p>,
        flex_source: &'b KinematicsData,
        data: &'b mut PassiveForceData,
    ) -> Result<PassiveForceView<'b, 's, 'p>, TransferError> {
        if self.disable_spring && self.disable_damper {
            return Err(InputError::InvalidDimension {
                field: "passive_force_disabled_path",
            }
            .into());
        }
        if !Arc::ptr_eq(&self.identity, &data.identity)
            || data.worlds() != joint.worlds()
            || data.worlds() != tendon_velocity.worlds()
            || data.worlds() != flex_source.worlds()
            || !self.tendon.accepts_shared_velocity(tendon_velocity)
            || !self.flex_edge.accepts_shared_source(flex_source)
            || self
                .gravity
                .as_ref()
                .is_some_and(|gravity| !gravity.accepts_source(flex_source))
            || self.fluid.is_some() != data.fluid.is_some()
            || self.fluid.as_ref().is_some_and(|fluid| {
                !fluid.accepts_resident_source(self.flex_edge.shared_source_plan())
            })
            || !std::ptr::eq(tendon_velocity.source_data(), flex_source)
        {
            return Err(TransferError::ModelMismatch);
        }
        // 同代核对先于任何设备写入：声明来源必须与常驻源字段版本一致。
        check_shared_source_generation(
            joint.source_state(),
            tendon_velocity.published_state(),
            flex_source.source_state(),
        )?;
        data.ready = false;
        data.smooth.invalidate();
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = joint;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let worlds = data.worlds();
            self.joint.write_shared_passive_base(
                joint,
                &mut data.parts,
                &mut data.status,
                worlds,
            )?;
            self.tendon.accumulate_shared_passive(
                tendon_velocity,
                &mut data.parts,
                &mut data.status,
                worlds,
            )?;
            self.flex_edge.accumulate_shared_passive(
                flex_source,
                &mut data.parts,
                &mut data.status,
                worlds,
            )?;
            if let Some(gravity) = &self.gravity {
                gravity.write_shared(flex_source, &mut data.gravity, &mut data.status, worlds)?;
                read_parts(&data.gravity, data.layout.total_elements())?;
            }
            if let Some(fluid) = &self.fluid {
                fluid.update_resident(
                    flex_source,
                    data.fluid.as_mut().ok_or(TransferError::ModelMismatch)?,
                )?;
            }
            let sleep = if self.smooth.options().enable_sleep() {
                Some(self.flex_edge.shared_sleep_source(flex_source)?)
            } else {
                None
            };
            read_parts(&data.parts, 2 * data.layout.total_elements())?;
            self.finish(data, worlds, sleep)?;
            data.ready = true;
            Ok(PassiveForceView {
                data,
                _joint: joint,
                _tendon_velocity: tendon_velocity,
                _flex_source: flex_source,
            })
        }
    }

    /// 双禁用时仅清零共享分量。
    /// 本入口不读取任何生产者。
    pub fn clear_disabled<'b>(
        &'b self,
        data: &'b mut PassiveForceData,
    ) -> Result<ClearedPassiveForceView<'b>, TransferError> {
        if !Arc::ptr_eq(&self.identity, &data.identity)
            || !self.disable_spring
            || !self.disable_damper
        {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        data.smooth.invalidate();
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let worlds = data.worlds();
            let w = worlds as u32;
            self.kernels
                .launch_clear(&self.metadata, &self.dummy, &mut data.parts, w)?;
            self.kernels
                .launch_clear_gravity(&self.metadata, &self.dummy, &mut data.gravity, w)?;
            if let Some(fluid) = &self.fluid {
                fluid.clear_shared(
                    data.fluid.as_mut().ok_or(TransferError::ModelMismatch)?,
                    worlds,
                )?;
            }
            read_parts(&data.parts, 2 * data.layout.total_elements())?;
            self.write_passive(data, worlds)?;
            if !self.smooth.options().enable_sleep() {
                self.smooth.update(&mut data.smooth)?;
            }
            data.ready = true;
            Ok(ClearedPassiveForceView { data })
        }
    }

    /// 双禁用时只借用树活动状态。
    /// 本入口不读取物理生产者。
    pub fn clear_disabled_with_sleep<'b>(
        &'b self,
        sleep_source: &'b KinematicsData,
        data: &'b mut PassiveForceData,
    ) -> Result<SleepClearedPassiveForceView<'b>, TransferError> {
        if !Arc::ptr_eq(&self.identity, &data.identity)
            || !self.disable_spring
            || !self.disable_damper
            || !self.smooth.options().enable_sleep()
            || data.worlds() != sleep_source.worlds()
            || !self.flex_edge.accepts_shared_source(sleep_source)
        {
            return Err(TransferError::ModelMismatch);
        }
        data.ready = false;
        data.smooth.invalidate();
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let worlds = data.worlds();
            let w = worlds as u32;
            self.kernels
                .launch_clear(&self.metadata, &self.dummy, &mut data.parts, w)?;
            self.kernels
                .launch_clear_gravity(&self.metadata, &self.dummy, &mut data.gravity, w)?;
            if let Some(fluid) = &self.fluid {
                fluid.clear_shared(
                    data.fluid.as_mut().ok_or(TransferError::ModelMismatch)?,
                    worlds,
                )?;
            }
            read_parts(&data.parts, 2 * data.layout.total_elements())?;
            let sleep = self.flex_edge.shared_sleep_source(sleep_source)?;
            self.finish(data, worlds, Some(sleep))?;
            data.ready = true;
            Ok(SleepClearedPassiveForceView {
                data,
                _sleep_source: sleep_source,
            })
        }
    }
}
