//! 计划构造入口。

use super::checks::check_finite_field;
use super::checks::check_g01_qpos;
use super::checks::check_mocap_mode;
use super::checks::check_size;
use super::*;

impl KinematicsPlan {
    /// 检查模型及qpos0后上传。
    /// 本辅助不接受非法默认姿态。
    /// 需要cuda-probe与NVRTC。
    pub fn new(
        session: &TransferSession,
        model: AttachedModelInput,
    ) -> Result<Self, TransferError> {
        Self::with_parameters(session, model, KinematicsParameters::default())
    }

    /// 先检查全部参数行，再上传。
    /// 缺省字段使用原模型共享行。
    /// 参数周期无需整除世界数。
    /// 计划创建后不允许改写参数。
    pub fn with_parameters(
        session: &TransferSession,
        model: AttachedModelInput,
        parameters: KinematicsParameters,
    ) -> Result<Self, TransferError> {
        let ids = vec![-1; model.rigid().kinematics().nbody()];
        Self::with_mocap(session, MocapModelInput::new(model, ids)?, parameters)
    }

    /// 先检查映射与全部参数行。
    /// 初始mocap使用体参数行。
    /// 静态geom使用GPU初始化。
    /// 模型参数与拓扑保持只读。
    pub fn with_mocap(
        session: &TransferSession,
        model: MocapModelInput,
        parameters: KinematicsParameters,
    ) -> Result<Self, TransferError> {
        Self::with_camlight(
            session,
            CamLightModelInput::new(model, CamLightFields::default())?,
            parameters,
            CamLightParameters::default(),
        )
    }

    /// 接收已编译的跟踪常量。
    /// 十项参数独立按世界取模。
    /// 相机与光源只计算位姿。
    pub fn with_camlight(
        session: &TransferSession,
        model: CamLightModelInput,
        parameters: KinematicsParameters,
        camlight_parameters: CamLightParameters,
    ) -> Result<Self, TransferError> {
        Self::with_fixed_tendons(
            session,
            FixedTendonModelInput::new(model, FixedTendonFields::default())?,
            parameters,
            camlight_parameters,
        )
    }

    /// 接收关节项与原生CSR。
    /// 系数与拓扑共享一份。
    /// 长度直接使用当前qpos。
    pub fn with_fixed_tendons(
        session: &TransferSession,
        model: FixedTendonModelInput,
        parameters: KinematicsParameters,
        camlight_parameters: CamLightParameters,
    ) -> Result<Self, TransferError> {
        Self::with_spatial_tendons(
            session,
            SpatialTendonModelInput::new(model, SpatialTendonFields::default())?,
            parameters,
            camlight_parameters,
        )
    }

    /// 接收site、滑轮与球柱路径。
    /// 两种肌腱集合各用局部编号。
    pub fn with_spatial_tendons(
        session: &TransferSession,
        model: SpatialTendonModelInput,
        parameters: KinematicsParameters,
        camlight_parameters: CamLightParameters,
    ) -> Result<Self, TransferError> {
        Self::with_tendon_subsets(
            session,
            model,
            parameters,
            camlight_parameters,
            OptionalSubsets::default(),
        )
    }

    /// 保留原生全局编号与CSR。
    /// GPU合并长度、力臂与包裹。
    /// 旧局部接口保持原编号。
    pub fn with_tendons(
        session: &TransferSession,
        model: TendonModelInput,
        parameters: KinematicsParameters,
        camlight_parameters: CamLightParameters,
    ) -> Result<Self, TransferError> {
        let (model, rows) = model.into_parts();
        Self::with_tendon_subsets(
            session,
            model,
            parameters,
            camlight_parameters,
            OptionalSubsets {
                tendon_rows: Some(rows),
                ..Default::default()
            },
        )
    }

    /// 另加柔体节点与顶点位置。
    /// 可选边复用质心与qvel。
    /// 可选面只复用设备节点。
    /// 位置阶段清除Hessian标志。
    /// 保留混合肌腱全局编号。
    pub fn with_flex_positions(
        session: &TransferSession,
        model: FlexPositionModelInput,
        parameters: KinematicsParameters,
        camlight_parameters: CamLightParameters,
    ) -> Result<Self, TransferError> {
        let (model, fields, edges, faces, hessian) = model.into_parts();
        let (model, rows) = model.into_parts();
        Self::with_tendon_subsets(
            session,
            model,
            parameters,
            camlight_parameters,
            OptionalSubsets {
                tendon_rows: Some(rows),
                flex_fields: Some(fields),
                edge_fields: edges,
                face_fields: faces,
                hessian_fields: hessian,
                sleep_info: None,
                equivalent: false,
            },
        )
    }

    /// 另加肌腱唤醒副作用。
    /// 不实现完整休眠状态机。
    pub fn with_tendon_wake(
        session: &TransferSession,
        model: TendonWakeModelInput,
        parameters: KinematicsParameters,
        camlight_parameters: CamLightParameters,
    ) -> Result<Self, TransferError> {
        let (model, flex, edges, faces, hessian, info) = model.into_parts();
        let (model, rows) = model.into_parts();
        Self::with_tendon_subsets(
            session,
            model,
            parameters,
            camlight_parameters,
            OptionalSubsets {
                tendon_rows: Some(rows),
                flex_fields: flex,
                edge_fields: edges,
                face_fields: faces,
                hessian_fields: hessian,
                sleep_info: Some(info),
                equivalent: false,
            },
        )
    }

    /// 完整G01保留等价数值路径。
    /// 模型必须包含全部边面字段。
    /// 空集合也采用显式空字段。
    /// 旧构造入口继续严格检查。
    pub fn for_g01(
        session: &TransferSession,
        model: TendonWakeModelInput,
        parameters: KinematicsParameters,
        camlight_parameters: CamLightParameters,
    ) -> Result<Self, TransferError> {
        if model.flex_positions().is_none()
            || model.flex_edges().is_none()
            || model.flex_faces().is_none()
        {
            return Err(InputError::InvalidDimension {
                field: "g01_requires_complete_flex_fields",
            }
            .into());
        }
        let (model, flex, edges, faces, hessian, info) = model.into_parts();
        let (model, rows) = model.into_parts();
        Self::with_tendon_subsets(
            session,
            model,
            parameters,
            camlight_parameters,
            OptionalSubsets {
                equivalent: true,
                tendon_rows: Some(rows),
                flex_fields: flex,
                edge_fields: edges,
                face_fields: faces,
                hessian_fields: hessian,
                sleep_info: Some(info),
            },
        )
    }

    /// 先检查全部长度与引用。
    /// 输入错误不改写设备状态。
    /// 传输失败废弃就绪票据。
    /// 低层回读仍反映当前缓冲。
    pub fn import_g01_state(
        &self,
        data: &mut KinematicsData,
        state: &G01State,
    ) -> Result<(), TransferError> {
        self.check_data(data)?;
        if !self.equivalent {
            return Err(InputError::InvalidDimension {
                field: "g01_requires_equivalent_plan",
            }
            .into());
        }
        check_g01_qpos(self.model.rigid(), data.worlds(), &state.qpos)?;
        let e = data.edge_layout()?;
        check_finite_field("qvel", e.qvel.total_elements(), &state.qvel)?;
        check_mocap_mode(
            data.layout.mocap_pos.total_elements(),
            data.layout.mocap_quat.total_elements(),
            &state.mocap_pos,
            &state.mocap_quat,
            false,
        )?;
        for (cache, field, expected) in [
            (
                &state.rigid_cache,
                "rigid_cache",
                data.layout.rigid.output.total_elements(),
            ),
            (
                &state.com_cache,
                "com_cache",
                data.layout.com.output.total_elements(),
            ),
        ] {
            if let Some(cache) = cache {
                check_finite_field(field, expected, cache)?;
            }
        }
        if let Some(poses) = &state.world_pose {
            check_size("world_pose", data.worlds(), poses.len())?;
            for pose in poses {
                check_finite_field("world_pose", 7, pose)?;
            }
        }
        if let Some(cache) = &state.attached_cache {
            check_finite_field(
                "attached_cache",
                data.layout.attached.output.total_elements(),
                cache,
            )?;
        }
        if let Some(flags) = &state.flex_hessian_valid {
            check_size(
                "flex_hessian_valid",
                data.layout
                    .flex
                    .expect("complete G01 flex")
                    .validity
                    .total_elements(),
                flags.len(),
            )?;
        }
        if let Some(sleep) = &state.sleep {
            check_size("sleep", data.worlds(), sleep.len())?;
            let l = data.layout.sleep.expect("complete G01 sleep");
            let k = self.model.rigid().kinematics();
            for row in sleep {
                row.validate(l.ntree, k.nbody(), k.nv())?;
            }
        }
        #[cfg(not(feature = "cuda-probe"))]
        {
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            data.ready = ReadyStages::default();
            data.write_checked_qpos(0, &state.qpos)?;
            data.write_qvel(&state.qvel)?;
            data.write_checked_mocap(0, 0, &state.mocap_pos, &state.mocap_quat)?;
            if let Some(cache) = &state.rigid_cache {
                data.rigid.write_range(4, cache)?;
            }
            if let Some(cache) = &state.com_cache {
                data.com.write_range(4, cache)?;
            }
            if let Some(poses) = &state.world_pose {
                for (world, pose) in poses.iter().enumerate() {
                    let start = 4 + world * data.layout.rigid.output.elements_per_world();
                    data.rigid.write_range(start, &pose[..3])?;
                    data.rigid
                        .write_range(start + 3 * data.layout.rigid.nbody, &pose[3..])?;
                }
            }
            if let Some(cache) = &state.attached_cache {
                data.attached.write_range(4, cache)?;
            }
            if let Some(flags) = &state.flex_hessian_valid {
                let values: Vec<i32> = flags.iter().map(|&v| i32::from(v)).collect();
                data.flex_validity
                    .as_mut()
                    .expect("complete G01 flex")
                    .write_range(4, &values)?;
            }
            if let Some(sleep) = &state.sleep {
                let l = data.layout.sleep.expect("complete G01 sleep");
                for (world, row) in sleep.iter().enumerate() {
                    let start = 4 + l.output.world_elements(world)?.start;
                    data.sleep
                        .as_mut()
                        .expect("complete G01 sleep")
                        .state
                        .write_range(start, &row.packed())?;
                }
            }
            Ok(())
        }
    }
}
