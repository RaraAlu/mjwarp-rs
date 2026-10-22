//! 柔体、休眠与全阶段更新。

use super::*;

impl KinematicsPlan {
    pub(super) fn assemble_tendons(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        data.ready.tendon = false;
        data.ready.sleep = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let (Some(device), Some(output), Some(layout)) =
                (&self.tendon, &mut data.tendon, data.layout.tendon)
                && !layout.is_empty()
            {
                let worlds = data.layout.qpos.worlds() as u32;
                // 类型化包装核对证明、精确跨度、同会话与别名。
                device.kernel.launch_values(
                    &device.metadata,
                    &data.spatial_wrap,
                    &data.fixed_tendon,
                    &mut output.values,
                    worlds,
                    0,
                )?;
                device.kernel.launch_values(
                    &device.metadata,
                    &data.spatial_wrap,
                    &data.spatial_tendon,
                    &mut output.values,
                    worlds,
                    1,
                )?;
                device.kernel.launch_indices(
                    &device.metadata,
                    &device.parameters,
                    &data.spatial_wrap,
                    &mut output.indices,
                    worlds,
                )?;
            }
            data.ready.tendon = true;
            Ok(())
        }
    }

    /// 只更新柔体位置子集。
    /// 它只依赖刚体结果。
    /// 先清除每世界的柔体标志。
    /// 随后计算节点与顶点位置。
    /// 严格入口先检查刚体就绪。
    /// 本辅助不计算Hessian矩阵。
    pub fn update_flex_positions(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        self.require_input(data, data.ready.rigid, "rigid")?;
        data.ready.flex = false;
        data.ready.edges = false;
        data.ready.faces = false;
        data.ready.hessian = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let (Some(device), Some(out)) = (&self.flex, &mut data.flex) {
                let worlds = data.layout.qpos.worlds() as u32;
                // 类型化包装核对证明、精确跨度、同会话与别名；先失效再算位置。
                device.invalidate_kernel.launch(
                    &device.metadata,
                    &device.parameters,
                    &data.rigid,
                    data.flex_validity.as_mut().expect("checked flex validity"),
                    worlds,
                )?;
                device.kernel.launch(
                    &device.metadata,
                    &device.parameters,
                    &data.rigid,
                    out,
                    worlds,
                )?;
            }
            data.ready.flex = true;
            Ok(())
        }
    }

    /// 显式缓存拉伸投影矩阵。
    /// 本阶段不执行矩阵乘法。
    pub fn update_flex_hessian(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        if self.hessian_fields.is_some() {
            data.ready.require(data.ready.flex, "flex_positions")?;
            data.ready.require(data.ready.edges, "flex_edges")?;
        }
        data.ready.hessian = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let (Some(device), Some(output)) = (&self.hessian, &mut data.hessian) {
                let worlds = data.layout.qpos.worlds() as u32;
                // 类型化包装核对证明、精确跨度、同会话与别名。
                device.kernel.launch(
                    &device.metadata,
                    &device.parameters,
                    (
                        data.flex.as_ref().expect("checked flex positions"),
                        &data.edges.as_ref().expect("checked flex edges").values,
                    ),
                    data.flex_validity.as_ref().expect("checked flex validity"),
                    output,
                    worlds,
                )?;
                device.validate_kernel.launch(
                    &device.metadata,
                    &device.parameters,
                    output,
                    data.flex_validity.as_mut().expect("checked flex validity"),
                    worlds,
                )?;
            }
            data.ready.hessian = true;
            Ok(())
        }
    }

    /// 唤醒后只刷新树活动标记。
    /// 它不推进自动休眠计数。
    pub fn update_tendon_wake(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        if self.sleep_info.is_some() {
            data.ready.require(data.ready.tendon, "tendon")?;
        }
        data.ready.sleep = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let (Some(device), Some(out), Some(tendon)) =
                (&self.sleep, &mut data.sleep, &data.tendon)
            {
                let worlds = data.layout.qpos.worlds() as u32;
                let lengths_stride = data
                    .layout
                    .tendon
                    .expect("checked tendon layout")
                    .output
                    .elements_per_world() as u64;
                let scratch_stride = data
                    .layout
                    .sleep
                    .expect("checked sleep layout")
                    .output
                    .elements_per_world() as u64;
                // 类型化包装核对证明、精确跨度、同会话与别名。
                device.copy_kernel.launch(
                    &device.metadata,
                    &device.parameters,
                    &out.state,
                    &mut out.scratch,
                    worlds,
                )?;
                device.wake_kernel.launch(
                    &device.metadata,
                    &device.parameters,
                    &tendon.values,
                    &mut out.scratch,
                    TendonWakeSpans {
                        lengths_stride,
                        scratch_stride,
                    },
                    worlds,
                )?;
                std::mem::swap(&mut out.state, &mut out.scratch);
            }
            data.ready.sleep = true;
            Ok(())
        }
    }

    /// 复用设备顶点与质心。
    /// qvel写入只废弃本阶段。
    pub fn update_flex_edges(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        if data.layout.edges.is_some_and(|l| !l.is_empty()) {
            self.require_input(data, data.ready.com, "com")?;
            self.require_input(data, data.ready.flex, "flex_positions")?;
        }
        data.ready.edges = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let (Some(device), Some(out), Some(flex)) =
                (&self.edges, &mut data.edges, &data.flex)
            {
                // 类型化包装核对证明、精确跨度、同会话与别名。
                device.kernel.launch(
                    &device.metadata,
                    &out.qvel,
                    &data.com,
                    flex,
                    &mut out.values,
                    data.layout.qpos.worlds() as u32,
                )?;
            }
            data.ready.edges = true;
            Ok(())
        }
    }

    /// 面阶段只读取设备节点。
    /// 它不依赖质心或qvel。
    pub fn update_flex_faces(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.check_data(data)?;
        if data.layout.faces.is_some_and(|l| !l.is_empty()) {
            self.require_input(data, data.ready.flex, "flex_positions")?;
        }
        data.ready.faces = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            if let (Some(device), Some(nodes), Some(out)) =
                (&self.faces, &data.flex, &mut data.faces)
            {
                // 类型化包装核对证明、精确跨度、同会话与别名。
                device.kernel.launch(
                    &device.metadata,
                    &device.parameters,
                    nodes,
                    out,
                    data.layout.qpos.worlds() as u32,
                )?;
            }
            data.ready.faces = true;
            Ok(())
        }
    }

    /// 依次执行六个基础子集。
    /// 混合入口另做GPU合并。
    /// 柔体入口另加位置更新。
    /// 可选边面紧随位置更新。
    /// 不上传模型或编译内核。
    /// 不回读阶段间的宿主结果。
    /// 不提供完整等价阶段。
    pub fn update(&self, data: &mut KinematicsData) -> Result<(), TransferError> {
        self.update_rigid(data)?;
        self.update_attached(data)?;
        self.update_com(data)?;
        self.update_camlight(data)?;
        self.update_flex_positions(data)?;
        self.update_flex_edges(data)?;
        self.update_flex_faces(data)?;
        self.update_tendons(data)?;
        self.update_tendon_wake(data)
    }
}
