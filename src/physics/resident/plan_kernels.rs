//! 计划内核编译：G01基础内核与相机光源设备上传。

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
use super::pack::PackedModel;
#[cfg(feature = "cuda-probe")]
use super::pack_batches::FIELD_BATCH_CUDA;
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
