//! 计划数据创建与检查。

#[cfg(feature = "cuda-probe")]
use super::output::allocate_integer_output;
#[cfg(feature = "cuda-probe")]
use super::output::allocate_output;
use super::*;

impl KinematicsPlan {
    /// 创建独立世界与设备输出。
    /// 全部尺寸通过后才申请显存。
    /// 初始派生结果保持未就绪。
    pub fn create_data(&self, worlds: usize) -> Result<KinematicsData, TransferError> {
        let mut layout = ResidentLayout::new(&self.model, worlds, self.nmocap)?;
        layout.camlight = CamLightLayout::new(worlds, self.ncam, self.nlight)?;
        layout.fixed_tendon = FixedTendonLayout::new(
            worlds,
            self.fixed_tendon_rows.ntendon(),
            self.fixed_tendon_rows.nnz(),
        )?;
        layout.spatial_tendon = SpatialTendonLayout::new(
            worlds,
            self.spatial_tendon_rows.ntendon(),
            self.spatial_tendon_rows.nnz(),
            self.spatial_tendon_rows.nwrap(),
        )?;
        layout.tendon = self
            .tendon_rows
            .as_ref()
            .map(|r| TendonLayout::new(worlds, r.ntendon(), r.nnz(), r.nwrap()))
            .transpose()?;
        layout.flex = self
            .flex_fields
            .as_ref()
            .map(|f| {
                FlexPositionLayout::new(
                    worlds,
                    f.flex_interp.len(),
                    f.flex_nodebodyid.len(),
                    f.flex_vertbodyid.len(),
                )
            })
            .transpose()?;
        layout.sleep = self
            .sleep_info
            .as_ref()
            .map(|i| SleepTreeLayout::new(worlds, i.ntree))
            .transpose()?;
        layout.edges = self
            .edge_fields
            .as_ref()
            .map(|f| {
                FlexEdgeLayout::new(
                    worlds,
                    f.nflexedge(),
                    f.nnz(),
                    self.model.rigid().kinematics().nv(),
                )
            })
            .transpose()?;
        layout.faces = self
            .face_fields
            .as_ref()
            .map(|f| FlexFaceLayout::new(worlds, f.nflexface()))
            .transpose()?;
        layout.hessian = self
            .hessian_fields
            .as_ref()
            .map(|_| {
                FlexHessianLayout::new(
                    worlds,
                    self.flex_fields
                        .as_ref()
                        .expect("checked flex fields")
                        .flex_vertbodyid
                        .len(),
                    self.edge_fields
                        .as_ref()
                        .expect("checked edge fields")
                        .nflexedge(),
                )
            })
            .transpose()?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = layout.state;
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            // Three contiguous field blocks preserve independent world writes.
            // Kernel wide offsets match these checked batch lengths exactly.
            let mut state =
                crate::runtime::host_staging::<f32>(layout.state.total_elements().max(1))?;
            let nq = layout.qpos.elements_per_world();
            if nq != 0 {
                for (world, row) in state[..layout.qpos.total_elements()]
                    .chunks_exact_mut(nq)
                    .enumerate()
                {
                    row.copy_from_slice(self.parameters.world(
                        KinematicsParameter::Qpos0,
                        &self.model,
                        world,
                    ));
                }
            }
            let pos_start = layout.qpos.total_elements();
            let quat_start = pos_start + layout.mocap_pos.total_elements();
            for world in 0..worlds {
                let bp = self
                    .parameters
                    .world(KinematicsParameter::BodyPos, &self.model, world);
                let bq = self
                    .parameters
                    .world(KinematicsParameter::BodyQuat, &self.model, world);
                for (body, &id) in self
                    .body_mocapid
                    .iter()
                    .enumerate()
                    .filter(|&(_, &id)| id >= 0)
                {
                    let p = pos_start + world * 3 * self.nmocap + id as usize * 3;
                    let q = quat_start + world * 4 * self.nmocap + id as usize * 4;
                    state[p..p + 3].copy_from_slice(&bp[body * 3..body * 3 + 3]);
                    state[q..q + 4].copy_from_slice(&bq[body * 4..body * 4 + 4]);
                }
            }
            let mut data = KinematicsData {
                equivalent: self.equivalent,
                model: Arc::clone(&self.model),
                fixed_tendon_rows: Arc::clone(&self.fixed_tendon_rows),
                spatial_tendon_rows: Arc::clone(&self.spatial_tendon_rows),
                tendon_rows: self.tendon_rows.clone(),
                edge_fields: self.edge_fields.clone(),
                face_fields: self.face_fields.clone(),
                hessian_fields: self.hessian_fields.clone(),
                // A1：状态缓冲区仍由本类型持有；身份与生命周期改记共享类型。
                lifecycle: DataLifecycle::new(
                    self.session.clone(),
                    DataIdentity::new(
                        ModelIdentity::of_model(&self.model),
                        LayoutIdentity::of_parts(&[
                            worlds,
                            layout.qpos.elements_per_world(),
                            layout.mocap_pos.elements_per_world(),
                            layout.state.elements_per_world(),
                        ]),
                    ),
                ),
                layout,
                ready: ReadyStages::default(),
                flex_validity: layout
                    .flex
                    .map(|l| {
                        let mut values = crate::runtime::host_staging::<i32>(l.guarded_validity)?;
                        values.fill(-131072);
                        values[4..l.guarded_validity - 4].fill(0);
                        self.session.upload(&values)
                    })
                    .transpose()?,
                state: self.session.upload(&state)?,
                rigid: allocate_output(&self.session, layout.rigid.guarded)?,
                com: allocate_output(&self.session, layout.com.guarded)?,
                attached: allocate_output(&self.session, layout.attached.guarded)?,
                camlight: allocate_output(&self.session, layout.camlight.guarded)?,
                fixed_tendon: allocate_output(&self.session, layout.fixed_tendon.guarded)?,
                spatial_tendon: allocate_output(&self.session, layout.spatial_tendon.guarded)?,
                spatial_wrap: allocate_integer_output(
                    &self.session,
                    layout.spatial_tendon.guarded_indices,
                )?,
                tendon: layout
                    .tendon
                    .map(|l| -> Result<_, TransferError> {
                        Ok(ResidentTendon {
                            values: allocate_output(&self.session, l.guarded)?,
                            indices: allocate_integer_output(&self.session, l.guarded_indices)?,
                        })
                    })
                    .transpose()?,
                flex: layout
                    .flex
                    .map(|l| allocate_output(&self.session, l.guarded))
                    .transpose()?,
                faces: layout
                    .faces
                    .map(|l| allocate_output(&self.session, l.guarded))
                    .transpose()?,
                hessian: layout
                    .hessian
                    .map(|l| allocate_output(&self.session, l.guarded))
                    .transpose()?,
                edges: layout
                    .edges
                    .map(|l| -> Result<_, TransferError> {
                        let mut qvel = crate::runtime::host_staging::<f32>(l.guarded_qvel)?;
                        qvel.fill(-131072.0);
                        qvel[4..l.guarded_qvel - 4].fill(0.0);
                        Ok(ResidentFlexEdges {
                            qvel: self.session.upload(&qvel)?,
                            values: allocate_output(&self.session, l.guarded)?,
                        })
                    })
                    .transpose()?,
                sleep: layout
                    .sleep
                    .map(|l| -> Result<_, TransferError> {
                        let k = self.model.rigid().kinematics();
                        let initial = SleepTreeState {
                            tree_asleep: vec![crate::model::TREE_FULLY_AWAKE; l.ntree],
                            nbody_awake: k.nbody() as i32,
                            nv_awake: k.nv() as i32,
                        }
                        .packed();
                        let mut values = crate::runtime::host_staging::<i32>(l.guarded)?;
                        values.fill(-131072);
                        for row in values[4..l.guarded - 4].chunks_exact_mut(initial.len()) {
                            row.copy_from_slice(&initial);
                        }
                        Ok(ResidentSleep {
                            state: self.session.upload(&values)?,
                            scratch: allocate_integer_output(&self.session, l.guarded)?,
                        })
                    })
                    .transpose()?,
            };
            if self.equivalent {
                for world in 0..worlds {
                    let start = 4 + world * layout.rigid.output.elements_per_world();
                    data.rigid.write_range(start, &[0.0; 3])?;
                    data.rigid
                        .write_range(start + 3 * layout.rigid.nbody, &[1.0, 0.0, 0.0, 0.0])?;
                }
            }
            self.update_rigid(&mut data)?;
            // 初始化入口与附着入口共用同一冻结ABI与缓冲。
            self.initialize_kernel.launch(
                &self.attached.metadata,
                &self.attached.parameters,
                &data.rigid,
                &mut data.attached,
                worlds as u32,
            )?;
            data.ready = ReadyStages::default();
            Ok(data)
        }
    }

    pub(in crate::physics) fn check_data(
        &self,
        data: &KinematicsData,
    ) -> Result<(), TransferError> {
        if Arc::ptr_eq(&self.model, &data.model) {
            Ok(())
        } else {
            Err(TransferError::ModelMismatch)
        }
    }
}
