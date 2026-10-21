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
                let dimensions = [0, 0, 0, data.layout.qpos.worlds() as u32];
                // SAFETY: 映射覆盖全部CSR行。
                // 空间内核限定动态点数量。
                // 同一会话持有全部缓冲。
                // 布局限定各缓冲的世界偏移。
                // 输入与输出独立分配。
                // 各线程只写一个世界。
                // 浮点段采用i32索引参数。
                // 整数段读取i32空间结果。
                // 同步调用保留全部缓冲寿命。
                unsafe {
                    device.values_kernel.launch(
                        &device.model.metadata,
                        &data.spatial_wrap,
                        &data.fixed_tendon,
                        &mut output.values,
                        dimensions,
                    )?;
                    device.values_kernel.launch(
                        &device.model.metadata,
                        &data.spatial_wrap,
                        &data.spatial_tendon,
                        &mut output.values,
                        [1, 0, 0, dimensions[3]],
                    )?;
                    device.indices_kernel.launch(
                        &device.model.metadata,
                        &device.model.parameters,
                        &data.spatial_wrap,
                        &mut output.indices,
                        dimensions,
                    )?;
                }
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
                // SAFETY: This private entry shares the checked flex metadata.
                // It ignores float inputs and writes only world*nf i32 slots.
                // Guarded validity owns the same session and checked dimensions.
                // Clear before position work, even when positions later fail.
                unsafe {
                    device.invalidate_kernel.launch(
                        &device.model.metadata,
                        &device.model.parameters,
                        &data.rigid,
                        data.flex_validity.as_mut().expect("checked flex validity"),
                        [0, 0, 0, data.layout.qpos.worlds() as u32],
                    )?;
                }
                // SAFETY: Checked immutable fields partition nodes and vertices.
                // Each synchronized thread reads an initialized rigid world and owns
                // its guarded output; wide strides match checked batch layouts.
                unsafe {
                    device.kernel.launch(
                        &device.model.metadata,
                        &device.model.parameters,
                        &data.rigid,
                        out,
                        [0, 0, 0, data.layout.qpos.worlds() as u32],
                    )?;
                }
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
                // SAFETY: Validated elements, edge pairs and 21-value ranges
                // bound all accesses. Checked node offsets select vertices.
                // Buffers share one session and disjoint guarded allocations.
                // Synchronization finishes blocks before publishing valid flags.
                unsafe {
                    device.kernel.launch_flex_hessian(
                        (&device.model.metadata, &device.model.parameters),
                        data.flex.as_ref().expect("checked flex positions"),
                        &data.edges.as_ref().expect("checked flex edges").values,
                        data.flex_validity.as_ref().expect("checked flex validity"),
                        output,
                        data.layout.qpos.worlds() as u32,
                    )?;
                    device.validate_kernel.launch(
                        &device.model.metadata,
                        &device.model.parameters,
                        output,
                        data.flex_validity.as_mut().expect("checked flex validity"),
                        [0, 0, 0, data.layout.qpos.worlds() as u32],
                    )?;
                }
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
                let dims = [0, 0, 0, data.layout.qpos.worlds() as u32];
                // SAFETY: Checked tree cycles and global paths bound all accesses.
                // Separate guarded buffers never alias. Synchronized copy seeds
                // scratch before each world-owned wake pass; failures keep state.
                device.copy_kernel.launch(
                    &device.metadata,
                    &device.parameters,
                    &out.state,
                    &mut out.scratch,
                    dims[3],
                )?;
                unsafe {
                    device.wake_kernel.launch(
                        device.metadata.as_buffer(),
                        &device.parameters,
                        &tendon.values,
                        &mut out.scratch,
                        dims,
                    )?;
                }
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
                // SAFETY: Checked endpoints, disjoint CSR rows, ancestry and
                // guarded wide strides bound every read/write. Inputs retain
                // this model/session. The private f32 ABI waits before reuse.
                unsafe {
                    device.kernel.launch_flex_edges(
                        &device.metadata,
                        &out.qvel,
                        &data.com,
                        flex,
                        &mut out.values,
                        data.layout.qpos.worlds() as u32,
                    )?;
                }
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
                // SAFETY: Canonical checked face rows bound all node references.
                // Guarded layouts bound each world. Buffers share this session.
                // The private f32 ABI synchronizes before output reuse.
                unsafe {
                    device.kernel.launch(
                        &device.model.metadata,
                        &device.model.parameters,
                        nodes,
                        out,
                        [0, 0, 0, data.layout.qpos.worlds() as u32],
                    )?;
                }
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
