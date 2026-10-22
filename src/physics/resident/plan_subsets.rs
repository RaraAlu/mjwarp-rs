//! 计划子集构造。

use super::checks::check_g01_qpos;
use super::checks::check_parameters;
#[cfg(feature = "cuda-probe")]
use super::pack::ATTACHED_PARAMETERS;
#[cfg(feature = "cuda-probe")]
use super::pack::COM_PARAMETERS;
#[cfg(feature = "cuda-probe")]
use super::pack::PackedModel;
#[cfg(feature = "cuda-probe")]
use super::pack::RIGID_PARAMETERS;
#[cfg(feature = "cuda-probe")]
use super::pack::pack_camlight;
#[cfg(feature = "cuda-probe")]
use super::pack::pack_flex;
#[cfg(feature = "cuda-probe")]
use super::pack::pack_flex_edges;
#[cfg(feature = "cuda-probe")]
use super::pack::pack_flex_faces;
#[cfg(feature = "cuda-probe")]
use super::pack::pack_flex_hessian;
#[cfg(feature = "cuda-probe")]
use super::pack::pack_sleep;
#[cfg(feature = "cuda-probe")]
use super::pack::pack_tendon;
#[cfg(feature = "cuda-probe")]
use super::pack_batches::pack_fixed_tendon;
#[cfg(feature = "cuda-probe")]
use super::pack_batches::pack_model;
#[cfg(feature = "cuda-probe")]
use super::pack_batches::pack_spatial_tendon;
#[cfg(feature = "cuda-probe")]
use super::sleep_copy::SleepCopyKernel;
#[cfg(feature = "cuda-probe")]
use super::sleep_copy::SleepCopyMetadataBuffer;
use super::*;

impl KinematicsPlan {
    pub(super) fn with_tendon_subsets(
        session: &TransferSession,
        model: SpatialTendonModelInput,
        parameters: KinematicsParameters,
        camlight_parameters: CamLightParameters,
        subsets: OptionalSubsets,
    ) -> Result<Self, TransferError> {
        let OptionalSubsets {
            equivalent,
            tendon_rows,
            flex_fields,
            edge_fields,
            face_fields,
            hessian_fields,
            sleep_info,
        } = subsets;
        if let Some(rows) = &tendon_rows {
            TendonLayout::new(1, rows.ntendon(), rows.nnz(), rows.nwrap())?;
        }
        let (model, spatial_fields, spatial_rows, spatial_geometry) = model.into_parts();
        let spatial_layout = SpatialTendonLayout::new(
            1,
            spatial_rows.ntendon(),
            spatial_rows.nnz(),
            spatial_rows.nwrap(),
        )?;
        let (model, fixed_tendon_fields, fixed_tendon_rows) = model.into_parts();
        let fixed_tendon_layout =
            FixedTendonLayout::new(1, fixed_tendon_rows.ntendon(), fixed_tendon_rows.nnz())?;
        if equivalent {
            camlight_parameters.validate_mode(&model, false)?;
        } else {
            camlight_parameters.validate(&model)?;
        }
        let ncam = model.ncam();
        let nlight = model.nlight();
        let camlight_layout = CamLightLayout::new(1, ncam, nlight)?;
        let (model, camlight_fields) = model.into_parts();
        let (model, body_mocapid, nmocap, static_geom) = model.into_parts();
        let k = model.rigid().kinematics();
        let (fk, com) = if equivalent {
            (
                check_g01_qpos(model.rigid(), 1, &k.fields().qpos0)?,
                ComPositionLayout::new(1, k.nbody(), k.njnt(), k.nv())?,
            )
        } else {
            check_com_position(model.rigid(), 1, &k.fields().qpos0)?
        };
        let attached = AttachedLayout::new(1, model.ngeom(), model.nsite())?;
        if equivalent {
            parameters.validate(&model)?;
        } else {
            check_parameters(&model, &parameters)?;
        }
        let flex_layout = flex_fields
            .as_ref()
            .map(|f| {
                FlexPositionLayout::new(
                    1,
                    f.flex_interp.len(),
                    f.flex_nodebodyid.len(),
                    f.flex_vertbodyid.len(),
                )
            })
            .transpose()?;
        let sleep_layout = sleep_info
            .as_ref()
            .map(|i| SleepTreeLayout::new(1, i.ntree))
            .transpose()?;
        let edge_layout = edge_fields
            .as_ref()
            .map(|f| {
                FlexEdgeLayout::new(1, f.nflexedge(), f.nnz(), model.rigid().kinematics().nv())
            })
            .transpose()?;
        let face_layout = face_fields
            .as_ref()
            .map(|f| FlexFaceLayout::new(1, f.nflexface()))
            .transpose()?;
        let hessian_layout = hessian_fields
            .as_ref()
            .map(|_| {
                FlexHessianLayout::new(
                    1,
                    flex_fields
                        .as_ref()
                        .expect("checked flex fields")
                        .flex_vertbodyid
                        .len(),
                    edge_fields
                        .as_ref()
                        .expect("checked edge fields")
                        .nflexedge(),
                )
            })
            .transpose()?;
        #[cfg(not(feature = "cuda-probe"))]
        {
            let _ = (
                session,
                fk,
                com,
                attached,
                body_mocapid,
                nmocap,
                static_geom,
                camlight_fields,
                camlight_layout,
                fixed_tendon_layout,
                fixed_tendon_fields,
                fixed_tendon_rows,
                spatial_fields,
                spatial_rows,
                spatial_layout,
                spatial_geometry,
                tendon_rows,
                flex_fields,
                flex_layout,
                sleep_info,
                sleep_layout,
                edge_fields,
                edge_layout,
                face_fields,
                face_layout,
                hessian_fields,
                hessian_layout,
            );
            Err(ProbeError::FeatureDisabled("cuda-probe").into())
        }
        #[cfg(feature = "cuda-probe")]
        {
            let com_stride = com.output.elements_per_world();
            let _ = (attached, sleep_layout);
            let k = model.rigid().kinematics().fields();
            let a = model.fields();
            // Pack and check all descriptors before any device allocation.
            let rigid = pack_model(
                &model,
                &parameters,
                &[
                    &k.body_parentid,
                    &k.body_jntadr,
                    &k.body_jntnum,
                    &k.jnt_type,
                    &k.jnt_qposadr,
                    &[nmocap as i32],
                    &body_mocapid,
                ],
                &RIGID_PARAMETERS,
            )?;
            let com = pack_model(
                &model,
                &parameters,
                &[&k.body_parentid, &k.jnt_type, &k.jnt_bodyid, &k.jnt_dofadr],
                &COM_PARAMETERS,
            )?;
            let static_geom: Vec<i32> = static_geom.into_iter().map(i32::from).collect();
            let attached = pack_model(
                &model,
                &parameters,
                &[
                    &[fk.output.elements_per_world() as i32],
                    &a.geom_bodyid,
                    &a.site_bodyid,
                    &static_geom,
                ],
                &ATTACHED_PARAMETERS,
            )?;
            let camlight = pack_camlight(
                &camlight_fields,
                &camlight_parameters,
                fk.output.elements_per_world(),
                com_stride,
            )?;
            let fixed_tendon = pack_fixed_tendon(&model, &fixed_tendon_fields)?;
            let spatial_tendon =
                pack_spatial_tendon(&model, &spatial_fields, spatial_geometry.as_ref())?;
            let tendon = tendon_rows
                .as_ref()
                .map(|rows| pack_tendon(rows, &fixed_tendon_rows, &spatial_rows))
                .transpose()?;
            let flex = flex_fields
                .as_ref()
                .map(|f| pack_flex(f, fk.output.elements_per_world(), fk.nbody))
                .transpose()?;
            let sleep = sleep_info
                .as_ref()
                .map(|i| pack_sleep(i, tendon_rows.as_ref().expect("checked global tendons")))
                .transpose()?;
            let edges = edge_fields
                .as_ref()
                .map(|f| {
                    pack_flex_edges(
                        &model,
                        flex_fields.as_ref().expect("checked flex positions"),
                        f,
                        com_stride,
                        flex_layout
                            .expect("checked flex layout")
                            .output
                            .elements_per_world(),
                    )
                })
                .transpose()?;
            let faces = face_fields
                .as_ref()
                .map(|f| {
                    pack_flex_faces(
                        flex_fields.as_ref().expect("checked flex positions"),
                        f,
                        flex_layout
                            .expect("checked flex layout")
                            .output
                            .elements_per_world(),
                    )
                })
                .transpose()?;
            let hessian = hessian_fields
                .as_ref()
                .map(|h| {
                    pack_flex_hessian(
                        flex_fields.as_ref().expect("checked flex fields"),
                        edge_fields.as_ref().expect("checked edge fields"),
                        h,
                        flex_layout
                            .expect("checked flex layout")
                            .output
                            .elements_per_world(),
                        edge_layout
                            .expect("checked edge layout")
                            .output
                            .elements_per_world(),
                    )
                })
                .transpose()?;
            let rigid =
                rigid.upload_rigid(session, equivalent, model.rigid().kinematics(), nmocap)?;
            let kin = model.rigid().kinematics();
            let (com_metadata, com_parameters) = ResidentComMetadataBuffer::from_packed(
                session,
                com,
                kin.nbody(),
                kin.njnt(),
                kin.nv(),
            )?;
            let com = DeviceCom {
                metadata: com_metadata,
                parameters: com_parameters,
            };
            let attached = {
                let (metadata, parameters) = ResidentAttachedMetadataBuffer::from_packed(
                    session,
                    attached,
                    kin.nbody(),
                    kin.njnt(),
                    model.ngeom(),
                    model.nsite(),
                )?;
                DeviceAttached {
                    metadata,
                    parameters,
                }
            };
            let BaseKernels {
                rigid: rigid_kernel,
                com: com_kernel,
                attached: attached_kernel,
                initialize_attached: initialize_kernel,
            } = compile_base_kernels(session, equivalent)?;
            let camlight = if camlight_layout.is_empty() {
                None
            } else {
                let k = model.rigid().kinematics();
                let dims = (
                    k.nbody(),
                    fk.output.elements_per_world(),
                    com_stride,
                    ncam,
                    nlight,
                );
                let (metadata, parameters, kernel) = upload_camlight(session, camlight, dims)?;
                Some(DeviceCamLight {
                    metadata,
                    parameters,
                    kernel,
                })
            };
            let fixed_tendon = if fixed_tendon_layout.is_empty() {
                None
            } else {
                let k = model.rigid().kinematics();
                let dims = (
                    k.nq(),
                    fixed_tendon_rows.ntendon(),
                    fixed_tendon_rows.nnz(),
                    fixed_tendon_fields.wrap_objid.len(),
                    nmocap,
                );
                let (metadata, parameters, kernel) =
                    upload_fixed_tendon(session, fixed_tendon, dims)?;
                Some(DeviceFixedTendon {
                    metadata,
                    parameters,
                    kernel,
                })
            };
            let spatial_tendon = if spatial_layout.is_empty() {
                None
            } else {
                let (metadata, parameters, kernel) = upload_spatial_tendon(
                    session,
                    spatial_tendon,
                    &model,
                    &spatial_rows,
                    &spatial_fields,
                )?;
                Some(DeviceSpatialTendon {
                    metadata,
                    parameters,
                    kernel,
                })
            };
            let tendon = if let Some(tendon) = tendon {
                Some(DeviceTendon {
                    model: tendon.upload(session)?,
                    values_kernel: SynchronousKernel::compile(
                        session,
                        crate::physics::tendon::TENDON_CUDA,
                        "tendon_values",
                    )?,
                    indices_kernel: SynchronousKernel::compile(
                        session,
                        crate::physics::tendon::TENDON_CUDA,
                        "tendon_indices",
                    )?,
                })
            } else {
                None
            };
            let flex = if flex_layout.is_some_and(|l| !l.is_empty()) {
                Some(DeviceFlexPositions {
                    model: flex.expect("checked flex fields").upload(session)?,
                    invalidate_kernel: SynchronousKernel::compile(
                        session,
                        crate::physics::flex::FLEX_POSITION_CUDA,
                        "flex_invalidate",
                    )?,
                    kernel: SynchronousKernel::compile(
                        session,
                        crate::physics::flex::FLEX_POSITION_CUDA,
                        "flex_positions",
                    )?,
                })
            } else {
                None
            };
            let sleep = if let Some(sleep) = sleep {
                let PackedModel {
                    metadata,
                    parameters,
                } = sleep;
                Some(DeviceSleep {
                    metadata: SleepCopyMetadataBuffer::upload(session, metadata)?,
                    parameters: session.upload(&parameters)?,
                    copy_kernel: SleepCopyKernel::compile(session)?,
                    wake_kernel: SynchronousKernel::compile(
                        session,
                        crate::physics::sleep::TENDON_WAKE_CUDA,
                        "tendon_wake",
                    )?,
                })
            } else {
                None
            };
            let edges = if edge_layout.is_some_and(|l| !l.is_empty()) {
                Some(DeviceFlexEdges {
                    metadata: session.upload(&edges.expect("checked edge fields"))?,
                    kernel: SynchronousKernel::compile(
                        session,
                        crate::physics::flex_edge::FLEX_EDGE_CUDA,
                        "flex_edges",
                    )?,
                })
            } else {
                None
            };
            let faces = if face_layout.is_some_and(|l| !l.is_empty()) {
                Some(DeviceFlex {
                    model: faces.expect("checked face fields").upload(session)?,
                    kernel: SynchronousKernel::compile(
                        session,
                        crate::physics::flex_face::FLEX_FACE_CUDA,
                        "flex_faces",
                    )?,
                })
            } else {
                None
            };
            let hessian = if hessian_layout.is_some_and(|l| l.output.elements_per_world() != 0) {
                Some(DeviceFlexHessian {
                    model: hessian.expect("checked hessian fields").upload(session)?,
                    kernel: SynchronousKernel::compile(
                        session,
                        crate::physics::flex_hessian::FLEX_HESSIAN_CUDA,
                        "flex_hessian",
                    )?,
                    validate_kernel: SynchronousKernel::compile(
                        session,
                        crate::physics::flex_hessian::FLEX_HESSIAN_CUDA,
                        "flex_hessian_validate",
                    )?,
                })
            } else {
                None
            };
            Ok(Self {
                equivalent,
                model: Arc::new(model),
                parameters,
                body_mocapid,
                nmocap,
                ncam,
                nlight,
                camlight_parameters,
                fixed_tendon_rows: Arc::new(fixed_tendon_rows),
                spatial_tendon_rows: Arc::new(spatial_rows),
                tendon_rows: tendon_rows.map(Arc::new),
                flex_fields: flex_fields.map(Arc::new),
                edge_fields: edge_fields.map(Arc::new),
                face_fields: face_fields.map(Arc::new),
                hessian_fields: hessian_fields.map(Arc::new),
                sleep_info: sleep_info.map(Arc::new),
                session: session.clone(),
                rigid,
                com,
                attached,
                rigid_kernel,
                com_kernel,
                attached_kernel,
                initialize_kernel,
                camlight,
                fixed_tendon,
                spatial_tendon,
                tendon,
                flex,
                edges,
                faces,
                hessian,
                sleep,
            })
        }
    }
}
