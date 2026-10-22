//! 质心、附着、相机与肌腱更新。

use super::*;

impl KinematicsPlan {
    /// 读取已有刚体设备结果。
    /// 严格计划要求刚体就绪。
    pub fn update_com(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        self.require_input(data, data.ready.rigid, "rigid")?;
        data.ready.com = false;
        data.ready.edges = false;
        data.ready.camlight = false;
        data.ready.spatial_tendon = false;
        data.ready.tendon = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let worlds = data.worlds() as u32;
            // 类型化包装核对证明、精确跨度、同会话与别名。
            self.com_kernel.launch(
                &self.com.metadata,
                &self.com.parameters,
                &data.rigid,
                &mut data.com,
                worlds,
            )?;
            data.ready.com = true;
            Ok(())
        }
    }

    /// 读取已有刚体设备结果。
    /// 跳过静态geom缓存。
    /// 每次重算全部site。
    pub fn update_attached(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        self.require_input(data, data.ready.rigid, "rigid")?;
        data.ready.attached = false;
        data.ready.spatial_tendon = false;
        data.ready.tendon = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let worlds = data.worlds() as u32;
            // 类型化包装核对证明、精确跨度、同会话与别名。
            self.attached_kernel.launch(
                &self.attached.metadata,
                &self.attached.parameters,
                &data.rigid,
                &mut data.attached,
                worlds,
            )?;
            data.ready.attached = true;
            Ok(())
        }
    }

    /// 复用刚体与质心设备结果。
    /// 两段执行不回读中间结果。
    pub fn update_camlight(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        self.require_input(data, data.ready.rigid, "rigid")?;
        self.require_input(data, data.ready.com, "com")?;
        data.ready.camlight = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let Some(device) = &self.camlight {
                let worlds = data.worlds() as u32;
                // 类型化包装核对证明、精确跨度、同会话与别名。
                device.kernel.launch_rigid(
                    &device.metadata,
                    &device.parameters,
                    &data.rigid,
                    &mut data.camlight,
                    worlds,
                )?;
                device.kernel.launch_com(
                    &device.metadata,
                    &device.parameters,
                    &data.com,
                    &mut data.camlight,
                    worlds,
                )?;
            }
            data.ready.camlight = true;
            Ok(())
        }
    }

    /// 只读取常驻qpos状态。
    /// 本子集不依赖刚体或质心。
    pub fn update_fixed_tendons(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.ready.fixed_tendon = false;
        data.ready.sleep = false;
        data.ready.tendon = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let Some(device) = &self.fixed_tendon {
                let worlds = data.worlds() as u32;
                // 类型化包装核对证明、精确跨度、同会话与别名。
                device.kernel.launch(
                    &device.metadata,
                    &device.parameters,
                    &data.state,
                    &mut data.fixed_tendon,
                    worlds,
                )?;
            }
            data.ready.fixed_tendon = true;
            Ok(())
        }
    }

    /// 复用site与质心设备结果。
    /// 三次执行不回读中间结果。
    pub fn update_spatial_tendons(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        self.require_input(data, data.ready.attached, "attached")?;
        self.require_input(data, data.ready.com, "com")?;
        data.ready.spatial_tendon = false;
        data.ready.sleep = false;
        data.ready.tendon = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let Some(device) = &self.spatial_tendon {
                let worlds = data.worlds() as u32;
                // 类型化包装核对证明、精确跨度、同会话与别名；同步发射保留顺序。
                device.kernel.launch_site(
                    &device.metadata,
                    &device.parameters,
                    &data.attached,
                    &mut data.spatial_tendon,
                    worlds,
                )?;
                device.kernel.launch_moment(
                    &device.metadata,
                    &device.parameters,
                    &data.com,
                    &mut data.spatial_tendon,
                    worlds,
                )?;
                device.kernel.launch_wrap(
                    &device.metadata,
                    &device.parameters,
                    &data.spatial_tendon,
                    &mut data.spatial_wrap,
                    worlds,
                )?;
            }
            data.ready.spatial_tendon = true;
            Ok(())
        }
    }

    /// 重算两类并合并全局结果。
    /// 空间集合需要附着与质心。
    pub fn update_tendons(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.update_fixed_tendons(data)?;
        self.update_spatial_tendons(data)?;
        self.assemble_tendons(data)
    }
}
