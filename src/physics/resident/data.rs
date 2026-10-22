//! 数据设备缓冲与快照。

use super::checks::check_g01_qpos;
use super::checks::check_mocap_mode;
#[cfg(feature = "cuda-probe")]
use super::output::read_flex_validity;
#[cfg(feature = "cuda-probe")]
use super::output::read_output;
use super::*;

impl KinematicsData {
    /// A1：数据身份；模型身份与布局身份分开记录。
    pub fn data_identity(&self) -> DataIdentity {
        self.lifecycle.identity()
    }

    /// A1：生命周期记录；含会话、状态版本与证明代际。
    pub fn lifecycle(&self) -> &DataLifecycle<TransferSession> {
        &self.lifecycle
    }

    /// A1：状态写入代际；每次成功状态写入加一。
    pub fn state_version(&self) -> u64 {
        self.lifecycle.state_version()
    }

    /// A1：各状态字段的只读写入版本。
    /// 下游模块据此记录来源版本并核对同代。
    pub fn state_field_versions(&self) -> StateFieldVersions {
        self.lifecycle.state_field_versions()
    }

    /// A1：元数据写入代际。
    /// 常驻元数据缓冲在计划构造期上传，运行期不可改写。
    pub fn metadata_generation(&self) -> u64 {
        self.lifecycle.metadata_generation()
    }

    /// A1：当前源状态快照；供下游记录并核对来源代际。
    pub fn source_state(&self) -> SourceState {
        SourceState::new(
            self.lifecycle.identity(),
            self.lifecycle.state_version(),
            self.lifecycle.state_field_versions(),
        )
    }

    #[cfg(all(test, feature = "cuda-probe"))]
    pub(in crate::physics) fn tendon_velocity_source_for_test(
        &mut self,
    ) -> &mut TransferBuffer<f32> {
        &mut self.tendon.as_mut().unwrap().values
    }
    #[cfg(all(test, feature = "cuda-probe"))]
    pub(in crate::physics) fn state_values_for_test(&self) -> Result<Vec<f32>, TransferError> {
        let mut values = crate::runtime::host_staging::<f32>(self.state.len())?;
        self.state.read_range_into(0, &mut values)?;
        Ok(values)
    }
    /// CRB只借用新鲜质心工作区。
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn mass_matrix_input(
        &self,
    ) -> Result<&TransferBuffer<f32>, TransferError> {
        self.ready.require(self.ready.com, "com_position")?;
        Ok(&self.com)
    }

    /// 显式回读只检查CRB必需前置。
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) fn check_mass_matrix_inputs(&self) -> Result<(), TransferError> {
        self.ready.require(self.ready.com, "com_position")?;
        read_output(
            &self.rigid,
            self.layout.rigid.output.total_elements() + 8,
            "crb_rigid_input",
        )?;
        read_output(
            &self.com,
            self.layout.com.output.total_elements() + 8,
            "crb_com_input",
        )?;
        Ok(())
    }

    pub fn flex_hessian_fields(&self) -> Option<&FlexHessianFields> {
        self.hessian_fields.as_deref()
    }
    pub fn flex_face_fields(&self) -> Option<&FlexFaceFields> {
        self.face_fields.as_deref()
    }
    pub fn flex_edge_fields(&self) -> Option<&FlexEdgeFields> {
        self.edge_fields.as_deref()
    }
    /// 只写入启用边计算的状态。
    /// 输入错误保留已有结果。
    pub fn write_qvel(&mut self, qvel: &[f32]) -> Result<(), TransferError> {
        let l = self.edge_layout()?;
        self.write_checked_qvel(0, l.qvel.total_elements(), qvel)
    }
    pub fn write_world_qvel(&mut self, world: usize, qvel: &[f32]) -> Result<(), TransferError> {
        let r = self.edge_layout()?.qvel.world_elements(world)?;
        self.write_checked_qvel(r.start, r.len(), qvel)
    }
    pub(super) fn edge_layout(&self) -> Result<FlexEdgeLayout, TransferError> {
        self.layout.edges.ok_or(
            InputError::InvalidDimension {
                field: "flex_edges_disabled",
            }
            .into(),
        )
    }
    pub(super) fn write_checked_qvel(
        &mut self,
        offset: usize,
        expected: usize,
        qvel: &[f32],
    ) -> Result<(), TransferError> {
        if qvel.len() != expected {
            return Err(InputError::LengthMismatch {
                field: "qvel",
                expected,
                actual: qvel.len(),
            }
            .into());
        }
        if let Some(index) = qvel.iter().position(|v| !v.is_finite()) {
            return Err(InputError::NonFinite {
                field: "qvel",
                index,
            }
            .into());
        }
        self.ready.edges = false;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = offset;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.edges
                .as_mut()
                .expect("checked edge state")
                .qvel
                .write_range(4 + offset, qvel)?;
            self.lifecycle.record_state_write(StateField::Qvel)?;
            Ok(())
        }
    }
    /// 检查全部状态后写入一个世界。
    /// 它只废弃树副作用结果。
    pub fn write_world_sleep(
        &mut self,
        world: usize,
        state: &SleepTreeState,
    ) -> Result<(), TransferError> {
        let l = self.layout.sleep.ok_or(InputError::DisabledOutput)?;
        let range = l.output.world_elements(world)?;
        let k = self.model.rigid().kinematics();
        state.validate(l.ntree, k.nbody(), k.nv())?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = range;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.sleep
                .as_mut()
                .expect("checked sleep layout")
                .state
                .write_range(4 + range.start, &state.packed())?;
            self.ready.sleep = false;
            self.lifecycle.record_state_write(StateField::Sleep)?;
            Ok(())
        }
    }
    pub fn tendon_rows(&self) -> Option<&TendonRows> {
        self.tendon_rows.as_deref()
    }
    pub fn spatial_tendon_rows(&self) -> &SpatialTendonRows {
        &self.spatial_tendon_rows
    }
    pub fn fixed_tendon_rows(&self) -> &FixedTendonRows {
        &self.fixed_tendon_rows
    }
    pub fn worlds(&self) -> usize {
        self.layout.qpos.worlds()
    }

    /// 完整检查后改写全部qpos。
    /// 输入错误保留已有设备结果。
    pub fn write_qpos(&mut self, qpos: &[f32]) -> Result<(), TransferError> {
        if self.equivalent {
            check_g01_qpos(self.model.rigid(), self.worlds(), qpos)?;
        } else {
            check_kinematics(self.model.rigid(), self.worlds(), qpos)?;
        }
        self.write_checked_qpos(0, qpos)
    }

    /// 只改写指定世界的qpos。
    /// 它废弃整组派生结果。
    pub fn write_world_qpos(&mut self, world: usize, qpos: &[f32]) -> Result<(), TransferError> {
        let range = self.layout.qpos.world_elements(world)?;
        if self.equivalent {
            check_g01_qpos(self.model.rigid(), 1, qpos)?;
        } else {
            check_kinematics(self.model.rigid(), 1, qpos)?;
        }
        self.write_checked_qpos(range.start, qpos)
    }

    pub(super) fn write_checked_qpos(
        &mut self,
        offset: usize,
        qpos: &[f32],
    ) -> Result<(), TransferError> {
        self.ready = ReadyStages::default();
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (offset, qpos);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            self.state.write_range(offset, qpos)?;
            self.lifecycle.record_state_write(StateField::Qpos)?;
            Ok(())
        }
    }

    pub fn nmocap(&self) -> usize {
        self.layout.mocap_pos.elements_per_world() / 3
    }

    /// 检查后写入全部mocap状态。
    /// 四元数顺序采用wxyz。
    /// GPU更新归一化四元数。
    /// 输入错误保留已有结果。
    pub fn write_mocap(&mut self, pos: &[f32], quat: &[f32]) -> Result<(), TransferError> {
        check_mocap_mode(
            self.layout.mocap_pos.total_elements(),
            self.layout.mocap_quat.total_elements(),
            pos,
            quat,
            !self.equivalent,
        )?;
        self.write_checked_mocap(0, 0, pos, quat)
    }

    /// 只写入指定世界的mocap。
    /// 它废弃整组派生结果。
    pub fn write_world_mocap(
        &mut self,
        world: usize,
        pos: &[f32],
        quat: &[f32],
    ) -> Result<(), TransferError> {
        let p = self.layout.mocap_pos.world_elements(world)?;
        let q = self.layout.mocap_quat.world_elements(world)?;
        check_mocap_mode(p.len(), q.len(), pos, quat, !self.equivalent)?;
        self.write_checked_mocap(p.start, q.start, pos, quat)
    }

    pub(super) fn write_checked_mocap(
        &mut self,
        p: usize,
        q: usize,
        pos: &[f32],
        quat: &[f32],
    ) -> Result<(), TransferError> {
        self.ready = ReadyStages::default();
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (p, q, pos, quat);
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let start = self.layout.qpos.total_elements();
            self.state.write_range(start + p, pos)?;
            self.state
                .write_range(start + self.layout.mocap_pos.total_elements() + q, quat)?;
            self.lifecycle.record_state_write(StateField::Mocap)?;
            Ok(())
        }
    }

    /// 回读并检查全部子集结果。
    /// 失败不发布部分宿主快照。
    /// 结果未就绪时明确返回错误。
    pub fn readback(&self) -> Result<KinematicsSnapshot, TransferError> {
        self.ready.require_all()?;
        self.ready.require(
            self.layout.camlight.is_empty() || self.ready.camlight,
            "camlight",
        )?;
        self.ready.require(
            self.layout.fixed_tendon.is_empty() || self.ready.fixed_tendon,
            "fixed_tendon",
        )?;
        self.ready.require(
            self.layout.spatial_tendon.is_empty() || self.ready.spatial_tendon,
            "spatial_tendon",
        )?;
        self.ready.require(
            self.layout.tendon.is_none_or(|l| l.is_empty()) || self.ready.tendon,
            "tendon",
        )?;
        self.ready.require(
            self.layout.flex.is_none_or(|l| l.is_empty()) || self.ready.flex,
            "flex_positions",
        )?;
        self.ready.require(
            self.layout.sleep.is_none() || self.ready.sleep,
            "tendon_wake",
        )?;
        self.ready.require(
            self.layout.edges.is_none_or(|l| l.is_empty()) || self.ready.edges,
            "flex_edges",
        )?;
        self.ready.require(
            self.layout.faces.is_none_or(|l| l.is_empty()) || self.ready.faces,
            "flex_faces",
        )?;
        self.readback_fields()
    }

    /// 回读G01当前字段与缓存。
    /// 本入口不要求新鲜度票据。
    /// 未计算字段保留已有设备值。
    /// 它仍检查哨兵与有限性。
    pub fn readback_g01(&self) -> Result<KinematicsSnapshot, TransferError> {
        if !self.equivalent {
            return Err(InputError::InvalidDimension {
                field: "g01_requires_equivalent_plan",
            }
            .into());
        }
        self.readback_fields()
    }

    pub(super) fn readback_fields(&self) -> Result<KinematicsSnapshot, TransferError> {
        let lengths = [
            self.layout.rigid.output.total_elements() + 8,
            self.layout.com.output.total_elements() + 8,
            self.layout.attached.output.total_elements() + 8,
            self.layout.camlight.output.total_elements() + 8,
            self.layout.fixed_tendon.output.total_elements() + 8,
            self.layout.spatial_tendon.output.total_elements() + 8,
        ];
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = lengths;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let rigid = read_output(&self.rigid, lengths[0], "resident_rigid_output")?;
            let com = read_output(&self.com, lengths[1], "resident_com_output")?;
            let attached = read_output(&self.attached, lengths[2], "resident_attached_output")?;
            let camlight = read_output(&self.camlight, lengths[3], "resident_camlight_output")?;
            let fixed_tendon = read_output(
                &self.fixed_tendon,
                lengths[4],
                "resident_fixed_tendon_output",
            )?;
            let spatial_tendon = read_output(
                &self.spatial_tendon,
                lengths[5],
                "resident_spatial_tendon_output",
            )?;
            let spatial_wrap = read_output(
                &self.spatial_wrap,
                self.layout.spatial_tendon.indices.total_elements() + 8,
                "resident_spatial_wrap_output",
            )?;
            let tendon = match (&self.tendon, self.layout.tendon, &self.tendon_rows) {
                (Some(out), Some(layout), Some(rows)) => Some(TendonOutput {
                    layout,
                    values: read_output(&out.values, layout.guarded, "resident_tendon_output")?,
                    indices: read_output(
                        &out.indices,
                        layout.guarded_indices,
                        "resident_tendon_indices",
                    )?,
                    rows: Arc::clone(rows),
                }),
                _ => None,
            };
            Ok(KinematicsSnapshot {
                hessian: match (&self.hessian, self.layout.hessian) {
                    (Some(out), Some(layout)) if self.ready.hessian => Some(FlexHessianOutput {
                        layout,
                        fields: Arc::clone(
                            self.hessian_fields
                                .as_ref()
                                .expect("checked hessian fields"),
                        ),
                        edges: Arc::clone(self.edge_fields.as_ref().expect("checked edge fields")),
                        values: read_output(out, layout.guarded, "resident_flex_hessian_output")?,
                    }),
                    _ => None,
                },
                faces: match (&self.faces, self.layout.faces) {
                    (Some(out), Some(layout)) => Some(FlexFaceOutput {
                        layout,
                        fields: Arc::clone(self.face_fields.as_ref().expect("checked face fields")),
                        values: read_output(out, layout.guarded, "resident_flex_face_output")?,
                    }),
                    _ => None,
                },
                edges: match (&self.edges, self.layout.edges) {
                    (Some(out), Some(layout)) => {
                        read_output(&out.qvel, layout.guarded_qvel, "resident_qvel_input")?;
                        Some(FlexEdgeOutput {
                            layout,
                            fields: Arc::clone(
                                self.edge_fields.as_ref().expect("checked edge fields"),
                            ),
                            values: read_output(
                                &out.values,
                                layout.guarded,
                                "resident_flex_edge_output",
                            )?,
                        })
                    }
                    _ => None,
                },
                sleep: match (&self.sleep, self.layout.sleep) {
                    (Some(out), Some(layout)) => Some(SleepTreeOutput {
                        layout,
                        values: read_output(
                            &out.state,
                            layout.guarded,
                            "resident_sleep_tree_output",
                        )?,
                    }),
                    _ => None,
                },
                flex: match (&self.flex, self.layout.flex) {
                    (Some(out), Some(layout)) => Some(FlexPositionOutput {
                        layout,
                        values: read_output(out, layout.guarded, "resident_flex_position_output")?,
                        validity: read_flex_validity(
                            self.flex_validity.as_ref().expect("checked flex validity"),
                            layout.guarded_validity,
                        )?,
                    }),
                    _ => None,
                },
                attached: AttachedKinematicsOutput {
                    rigid: KinematicsOutput {
                        layout: self.layout.rigid,
                        values: rigid,
                    },
                    layout: self.layout.attached,
                    values: attached,
                },
                com: ComPositionOutput {
                    layout: self.layout.com,
                    values: com,
                },
                camlight: CamLightOutput {
                    layout: self.layout.camlight,
                    values: camlight,
                },
                fixed_tendon: FixedTendonOutput {
                    layout: self.layout.fixed_tendon,
                    values: fixed_tendon,
                    rows: Arc::clone(&self.fixed_tendon_rows),
                },
                spatial_tendon: SpatialTendonOutput {
                    layout: self.layout.spatial_tendon,
                    values: spatial_tendon,
                    indices: spatial_wrap,
                    rows: Arc::clone(&self.spatial_tendon_rows),
                },
                tendon,
            })
        }
    }
}
