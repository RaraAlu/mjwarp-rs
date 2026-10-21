//! 计划上下文与借用来源。

use super::*;

impl KinematicsPlan {
    pub(in crate::physics) fn tendon_velocity_context(
        &self,
    ) -> Result<(TransferSession, usize, Arc<TendonRows>), TransferError> {
        let rows = self
            .tendon_rows
            .clone()
            .ok_or(InputError::InvalidDimension {
                field: "velocity_requires_global_tendons",
            })?;
        Ok((
            self.session.clone(),
            self.model.rigid().kinematics().nv(),
            rows,
        ))
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn tendon_velocity_source<'a>(
        &self,
        data: &'a KinematicsData,
    ) -> Result<&'a TransferBuffer<f32>, TransferError> {
        self.check_data(data)?;
        if !self
            .tendon_rows
            .as_ref()
            .zip(data.tendon_rows.as_ref())
            .is_some_and(|(a, b)| Arc::ptr_eq(a, b))
        {
            return Err(TransferError::ModelMismatch);
        }
        // 等价入口也必须检查就绪。
        data.ready
            .require(data.ready.tendon, "tendon_velocity_source")?;
        data.tendon.as_ref().map(|t| &t.values).ok_or(
            InputError::InvalidDimension {
                field: "velocity_requires_global_tendons",
            }
            .into(),
        )
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn flex_edge_passive_context(
        &self,
    ) -> Result<(TransferSession, usize, Arc<FlexEdgeFields>), TransferError> {
        let fields = self
            .edge_fields
            .clone()
            .ok_or(InputError::InvalidDimension {
                field: "passive_requires_flex_edges",
            })?;
        Ok((
            self.session.clone(),
            self.model.rigid().kinematics().nv(),
            fields,
        ))
    }

    // 默认构建的唯一调用者同样无设备路径。
    #[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
    pub(in crate::physics) fn passive_sleep_context(&self) -> Option<(usize, &[i32], bool, bool)> {
        self.sleep_info.as_ref().map(|info| {
            (
                info.ntree,
                info.fields.body_treeid.as_slice(),
                info.fields.sleep_enabled,
                info.fields.island_disabled,
            )
        })
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn gravity_compensation_source<'a>(
        &self,
        data: &'a KinematicsData,
    ) -> Result<(&'a TransferBuffer<f32>, &'a TransferBuffer<f32>), TransferError> {
        self.check_data(data)?;
        data.ready
            .require(data.ready.rigid, "gravity_compensation_rigid")?;
        data.ready
            .require(data.ready.com, "gravity_compensation_com")?;
        Ok((&data.rigid, &data.com))
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn fluid_force_qvel_source<'a>(
        &self,
        data: &'a KinematicsData,
    ) -> Result<&'a TransferBuffer<f32>, TransferError> {
        self.check_data(data)?;
        data.edges.as_ref().map(|edges| &edges.qvel).ok_or(
            InputError::InvalidDimension {
                field: "fluid_force_requires_qvel",
            }
            .into(),
        )
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn fluid_force_attached_source<'a>(
        &self,
        data: &'a KinematicsData,
    ) -> Result<&'a TransferBuffer<f32>, TransferError> {
        self.check_data(data)?;
        data.ready
            .require(data.ready.attached, "fluid_force_attached")?;
        Ok(&data.attached)
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn passive_sleep_source<'a>(
        &self,
        data: &'a KinematicsData,
    ) -> Result<&'a TransferBuffer<i32>, TransferError> {
        self.check_data(data)?;
        let expected = self.sleep_info.as_ref().ok_or(InputError::DisabledOutput)?;
        let layout = data.layout.sleep.ok_or(InputError::DisabledOutput)?;
        if layout.ntree != expected.ntree {
            return Err(TransferError::ModelMismatch);
        }
        data.ready
            .require(data.ready.sleep, "passive_force_sleep_source")?;
        data.sleep
            .as_ref()
            .map(|sleep| &sleep.state)
            .ok_or(InputError::DisabledOutput.into())
    }

    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn flex_edge_passive_source<'a>(
        &self,
        data: &'a KinematicsData,
    ) -> Result<&'a TransferBuffer<f32>, TransferError> {
        self.check_data(data)?;
        if !self
            .edge_fields
            .as_ref()
            .zip(data.edge_fields.as_ref())
            .is_some_and(|(a, b)| Arc::ptr_eq(a, b))
        {
            return Err(TransferError::ModelMismatch);
        }
        data.ready
            .require(data.ready.edges, "flex_edge_passive_source")?;
        data.edges.as_ref().map(|edges| &edges.values).ok_or(
            InputError::InvalidDimension {
                field: "passive_requires_flex_edges",
            }
            .into(),
        )
    }
}
