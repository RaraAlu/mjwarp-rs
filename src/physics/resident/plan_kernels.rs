//! 计划内核编译：G01基础内核与可选设备上传。

#[cfg(feature = "cuda-probe")]
use super::camlight_launcher::CamLightDims;
#[cfg(feature = "cuda-probe")]
use super::camlight_launcher::CamLightMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::camlight_launcher::CamLightStaticDims;
#[cfg(feature = "cuda-probe")]
use super::camlight_launcher::ResidentCamLightKernel;
#[cfg(feature = "cuda-probe")]
use super::camlight_launcher::camlight_launch_error;
#[cfg(feature = "cuda-probe")]
use super::field_batch::FieldMetadataBuffer;
#[cfg(feature = "cuda-probe")]
#[cfg(feature = "cuda-probe")]
use super::flex_edges_launcher::FlexEdgesDims;
#[cfg(feature = "cuda-probe")]
use super::flex_edges_launcher::FlexEdgesKernel;
#[cfg(feature = "cuda-probe")]
use super::flex_edges_launcher::FlexEdgesMetadata;
#[cfg(feature = "cuda-probe")]
use super::flex_edges_launcher::FlexEdgesMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::flex_edges_launcher::FlexEdgesStaticDims;
#[cfg(feature = "cuda-probe")]
use super::flex_edges_launcher::flex_edges_launch_error;
#[cfg(feature = "cuda-probe")]
use super::flex_faces_launcher::FlexFacesDims;
#[cfg(feature = "cuda-probe")]
use super::flex_faces_launcher::FlexFacesKernel;
#[cfg(feature = "cuda-probe")]
use super::flex_faces_launcher::FlexFacesMetadata;
#[cfg(feature = "cuda-probe")]
use super::flex_faces_launcher::FlexFacesMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::flex_faces_launcher::FlexFacesStaticDims;
#[cfg(feature = "cuda-probe")]
use super::flex_faces_launcher::flex_faces_launch_error;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher::FlexDims;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher::FlexStaticDims;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher_specs::FlexInvalidateKernel;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher_specs::FlexMetadata;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher_specs::FlexMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher_specs::FlexPositionsKernel;
#[cfg(feature = "cuda-probe")]
use super::flex_positions_launcher_specs::flex_launch_error;
#[cfg(feature = "cuda-probe")]
use super::pack::PackedModel;
#[cfg(feature = "cuda-probe")]
use super::pack_batches::FIELD_BATCH_CUDA;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher::SpatialTendonDims;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher::SpatialTendonDimsInput;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher::SpatialTendonStaticDims;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher_specs::ResidentSpatialTendonKernel;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher_specs::SpatialTendonMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::spatial_tendon_launcher_specs::spatial_tendon_launch_error;
#[cfg(feature = "cuda-probe")]
use super::tendon_launcher::ResidentTendonKernel;
#[cfg(feature = "cuda-probe")]
use super::tendon_launcher::TendonDims;
#[cfg(feature = "cuda-probe")]
use super::tendon_launcher::TendonMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::tendon_launcher::TendonStaticDims;
#[cfg(feature = "cuda-probe")]
use super::tendon_launcher::tendon_launch_error;
#[cfg(feature = "cuda-probe")]
use super::*;

/// G01基础内核集合。
#[cfg(feature = "cuda-probe")]
pub(super) struct BaseKernels {
    pub(super) rigid: crate::physics::rigid_kinematics::RigidKernel,
    pub(super) com: ResidentComKernel,
    pub(super) attached: ResidentAttachedKernel,
    pub(super) initialize_attached: ResidentInitializeAttachedKernel,
}

/// 编译G01基础内核；等价模式只在源码前缀加冻结宏。
#[cfg(feature = "cuda-probe")]
pub(super) fn compile_base_kernels(
    session: &TransferSession,
    equivalent: bool,
) -> Result<BaseKernels, TransferError> {
    let mode = if equivalent {
        "#define MJWARP_G01_EQUIVALENT\n"
    } else {
        ""
    };
    let prefix = format!("{FIELD_BATCH_CUDA}\n#define MJWARP_MOCAP\n{mode}");
    let source = format!("{prefix}{}", crate::physics::KINEMATICS_CUDA);
    let rigid = crate::physics::rigid_kinematics::RigidKernel::new(
        session,
        SynchronousKernel::compile(session, &source, "rigid_kinematics")?,
    );
    let source = format!("{FIELD_BATCH_CUDA}\n{}", crate::physics::COM_POSITION_CUDA);
    let com = ResidentComKernel::compile(session, &source)?;
    let source = format!(
        "{prefix}{}\n{}",
        crate::physics::KINEMATICS_CUDA,
        crate::physics::attached::ATTACHED_CUDA
    );
    let attached = ResidentAttachedKernel::compile(session, &source)?;
    let initialize_attached = ResidentInitializeAttachedKernel::compile(session, &source)?;
    Ok(BaseKernels {
        rigid,
        com,
        attached,
        initialize_attached,
    })
}

/// 上传相机光源元数据、参数并编译两段内核。
#[cfg(feature = "cuda-probe")]
pub(super) fn upload_camlight(
    session: &TransferSession,
    packed: PackedModel,
    dims: (usize, usize, usize, usize, usize),
) -> Result<
    (
        CamLightMetadataBuffer,
        TransferBuffer<f32>,
        ResidentCamLightKernel,
    ),
    TransferError,
> {
    let (nb, rigid_stride, com_stride, ncam, nlight) = dims;
    let static_dims = CamLightStaticDims::new(
        nb,
        rigid_stride,
        com_stride,
        ncam,
        nlight,
        packed.parameters.len(),
    )
    .map_err(camlight_launch_error)?;
    let dims = CamLightDims {
        static_dims,
        worlds: 1,
    };
    let (metadata, parameters) =
        CamLightMetadataBuffer::from_packed(session, packed, dims, camlight_launch_error)?;
    let source = format!(
        "{FIELD_BATCH_CUDA}\n{}\n{}",
        crate::physics::KINEMATICS_CUDA,
        crate::physics::camlight::CAMLIGHT_CUDA
    );
    Ok((
        metadata,
        parameters,
        ResidentCamLightKernel::compile(session, &source)?,
    ))
}

/// 上传空间肌腱元数据、参数并编译三项内核。
#[cfg(feature = "cuda-probe")]
pub(super) fn upload_spatial_tendon(
    session: &TransferSession,
    packed: PackedModel,
    model: &AttachedModelInput,
    rows: &SpatialTendonRows,
    fields: &SpatialTendonFields,
) -> Result<
    (
        SpatialTendonMetadataBuffer,
        TransferBuffer<f32>,
        ResidentSpatialTendonKernel,
    ),
    TransferError,
> {
    let k = model.rigid().kinematics();
    let attached_stride = AttachedLayout::new(1, model.ngeom(), model.nsite())
        .map_err(TransferError::from)?
        .output
        .elements_per_world();
    let static_dims = SpatialTendonStaticDims::new(SpatialTendonDimsInput {
        nb: k.nbody(),
        nv: k.nv(),
        nt: rows.ntendon(),
        nnz: rows.nnz(),
        nw: fields.wrap_type.len(),
        ngeom: model.ngeom(),
        nsite: model.nsite(),
        attached_stride,
        parameters_len: packed.parameters.len(),
    })
    .map_err(spatial_tendon_launch_error)?;
    let dims = SpatialTendonDims {
        static_dims,
        worlds: 1,
    };
    let (metadata, parameters) = SpatialTendonMetadataBuffer::from_packed(
        session,
        packed,
        dims,
        spatial_tendon_launch_error,
    )?;
    let source = format!(
        "{}\n{}",
        crate::physics::tendon_wrap::TENDON_WRAP_CUDA,
        crate::physics::spatial_tendon::SPATIAL_TENDON_CUDA
    );
    Ok((
        metadata,
        parameters,
        ResidentSpatialTendonKernel::compile(session, &source)?,
    ))
}
/// 上传肌腱合并元数据、参数并编译合并与索引内核。
#[cfg(feature = "cuda-probe")]
pub(super) fn upload_tendon(
    session: &TransferSession,
    packed: PackedModel,
    rows: &TendonRows,
    fixed: &FixedTendonRows,
    spatial: &SpatialTendonRows,
) -> Result<
    (
        TendonMetadataBuffer,
        TransferBuffer<f32>,
        ResidentTendonKernel,
    ),
    TransferError,
> {
    let static_dims = TendonStaticDims::new(
        (rows.ntendon(), rows.nnz(), rows.nwrap()),
        (fixed.ntendon(), fixed.nnz()),
        (spatial.ntendon(), spatial.nnz(), spatial.nwrap()),
        packed.parameters.len(),
    )
    .map_err(tendon_launch_error)?;
    let dims = TendonDims {
        static_dims,
        worlds: 1,
        kind: 0,
    };
    let (metadata, parameters) =
        TendonMetadataBuffer::from_packed(session, packed, dims, tendon_launch_error)?;
    Ok((
        metadata,
        parameters,
        ResidentTendonKernel::compile(session)?,
    ))
}
/// 上传柔体位置元数据、参数并编译失效与位置内核。
#[cfg(feature = "cuda-probe")]
pub(super) fn upload_flex_positions(
    session: &TransferSession,
    packed: PackedModel,
    model: &AttachedModelInput,
    fields: &FlexPositionFields,
) -> Result<
    (
        FlexMetadataBuffer,
        TransferBuffer<f32>,
        FlexInvalidateKernel,
        FlexPositionsKernel,
    ),
    TransferError,
> {
    let k = model.rigid().kinematics();
    let rigid_stride = KinematicsLayout::new(1, k.nq(), k.nbody(), k.njnt())
        .map_err(TransferError::from)?
        .output
        .elements_per_world();
    let static_dims = FlexStaticDims::new(
        fields.flex_interp.len(),
        fields.flex_nodebodyid.len(),
        fields.flex_vertbodyid.len(),
        k.nbody(),
        rigid_stride,
        packed.parameters.len(),
    )
    .map_err(flex_launch_error)?;
    let dims = FlexDims {
        static_dims,
        worlds: 1,
    };
    let (metadata, parameters) =
        FieldMetadataBuffer::<FlexMetadata>::from_packed(session, packed, dims, flex_launch_error)?;
    Ok((
        metadata,
        parameters,
        FlexInvalidateKernel::compile(session)?,
        FlexPositionsKernel::compile(session)?,
    ))
}
/// 上传柔体边元数据并编译边内核。
#[cfg(feature = "cuda-probe")]
pub(super) fn upload_flex_edges(
    session: &TransferSession,
    metadata: Vec<i32>,
    model: &AttachedModelInput,
    edges: &FlexEdgeFields,
    flex: &FlexPositionFields,
    com_stride: usize,
    flex_stride: usize,
) -> Result<(FlexEdgesMetadataBuffer, FlexEdgesKernel), TransferError> {
    let k = model.rigid().kinematics();
    let static_dims = FlexEdgesStaticDims::new(
        k.nbody(),
        k.nv(),
        edges.nflexedge(),
        edges.nnz(),
        flex.flex_vertbodyid.len(),
        com_stride,
        flex_stride,
    )
    .map_err(flex_edges_launch_error)?;
    let dims = FlexEdgesDims {
        static_dims,
        worlds: 1,
    };
    let packed = PackedModel {
        metadata,
        parameters: vec![0.0],
    };
    let (metadata, parameters) = FieldMetadataBuffer::<FlexEdgesMetadata>::from_packed(
        session,
        packed,
        dims,
        flex_edges_launch_error,
    )?;
    drop(parameters);
    Ok((metadata, FlexEdgesKernel::compile(session)?))
}

/// 上传柔体面元数据、参数并编译面内核。
#[cfg(feature = "cuda-probe")]
pub(super) fn upload_flex_faces(
    session: &TransferSession,
    packed: PackedModel,
    flex: &FlexPositionFields,
    faces: &FlexFaceFields,
    node_stride: usize,
) -> Result<
    (
        FlexFacesMetadataBuffer,
        TransferBuffer<f32>,
        FlexFacesKernel,
    ),
    TransferError,
> {
    let static_dims = FlexFacesStaticDims::new(
        faces.nflexface(),
        flex.flex_nodebodyid.len(),
        node_stride,
        packed.parameters.len(),
    )
    .map_err(flex_faces_launch_error)?;
    let dims = FlexFacesDims {
        static_dims,
        worlds: 1,
    };
    let (metadata, parameters) = FieldMetadataBuffer::<FlexFacesMetadata>::from_packed(
        session,
        packed,
        dims,
        flex_faces_launch_error,
    )?;
    Ok((metadata, parameters, FlexFacesKernel::compile(session)?))
}
