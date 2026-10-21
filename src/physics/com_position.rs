//! 质心位置世界视图、探针与设备上传。

use super::*;
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{BatchLayout, InertialModelInput};
use crate::runtime::TransferSession;

/// 四项质心结果的只读世界视图。
/// cdof按角向量、线向量排列。
#[derive(Clone, Copy, Debug)]
pub struct ComPositionWorld<'a> {
    /// 每体子树质量，单位kg。
    pub subtree_mass: &'a [f32],
    /// 每体世界坐标质心，单位m。
    pub subtree_com: &'a [f32],
    /// 每体根子树质心坐标惯量。
    pub cinert: &'a [f32],
    /// 每自由度空间运动映射。
    pub cdof: &'a [f32],
}

/// 同步GPU质心结果。
/// cinert每体包含十项。
/// 前六项为xx,yy,zz,xy,xz,yz。
/// 后四项为质量一阶矩与质量。
#[derive(Clone, Debug)]
pub struct ComPositionOutput {
    pub(in crate::physics) layout: ComPositionLayout,
    pub(in crate::physics) values: Vec<f32>,
}

impl ComPositionOutput {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nbody(&self) -> usize {
        self.layout.nbody
    }
    pub fn nv(&self) -> usize {
        self.layout.nv
    }
    pub fn world(&self, world: usize) -> Result<ComPositionWorld<'_>, InputError> {
        let r = self.layout.output.world_elements(world)?;
        let v = &self.values[r.start + 4..r.end + 4];
        let nb = self.nbody();
        Ok(ComPositionWorld {
            subtree_mass: &v[..nb],
            subtree_com: &v[nb..4 * nb],
            cinert: &v[4 * nb..14 * nb],
            cdof: &v[14 * nb..],
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(in crate::physics) struct ComPositionLayout {
    pub(in crate::physics) output: BatchLayout,
    pub(in crate::physics) nbody: usize,
    pub(in crate::physics) nv: usize,
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) metadata: usize,
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) parameters: usize,
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) guarded: usize,
}

impl ComPositionLayout {
    pub(in crate::physics) fn new(
        worlds: usize,
        nbody: usize,
        njnt: usize,
        nv: usize,
    ) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize || nbody == 0 {
            return Err(InputError::InvalidDimension {
                field: "com_position_dimensions",
            });
        }
        let overflow = || InputError::Overflow {
            field: "com_position_layout",
        };
        let metadata = njnt
            .checked_mul(3)
            .and_then(|n| n.checked_add(nbody))
            .ok_or_else(overflow)?;
        let parameters = nbody.checked_mul(4).ok_or_else(overflow)?;
        let stride = nbody
            .checked_mul(14)
            .and_then(|n| nv.checked_mul(6).and_then(|d| n.checked_add(d)))
            .ok_or_else(overflow)?;
        if [metadata, parameters, stride]
            .iter()
            .any(|&n| n > i32::MAX as usize)
        {
            return Err(overflow());
        }
        let output = BatchLayout::new(worlds, stride, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or_else(overflow)?;
        BatchLayout::new(1, guarded, 4)?;
        #[cfg(not(feature = "cuda-probe"))]
        let _ = (metadata, parameters, guarded);
        Ok(Self {
            output,
            nbody,
            nv,
            #[cfg(feature = "cuda-probe")]
            metadata,
            #[cfg(feature = "cuda-probe")]
            parameters,
            #[cfg(feature = "cuda-probe")]
            guarded,
        })
    }
}

pub(in crate::physics) fn check_com_position(
    model: &InertialModelInput,
    worlds: usize,
    qpos: &[f32],
) -> Result<(KinematicsLayout, ComPositionLayout), InputError> {
    let fk = check_kinematics(model, worlds, qpos)?;
    for (field, values) in [
        ("body_mass", &model.fields().body_mass[..1]),
        ("body_inertia", &model.fields().body_inertia[..3]),
    ] {
        if values.iter().any(|&v| v != 0.0) {
            return Err(InputError::InvalidTopology {
                field,
                index: 0,
                reason: "nonzero_world_inertia",
            });
        }
    }
    let k = model.kinematics();
    Ok((
        fk,
        ComPositionLayout::new(worlds, k.nbody(), k.njnt(), k.nv())?,
    ))
}

/// G01质心子集的同步GPU探针。
/// 复用设备中的刚体运动学结果。
/// GPU推导子树质量与根体索引。
/// 零质量子树质心保持零。
/// 不采用原生CPU的阈值回退。
/// 世界体质量与惯量必须为零。
/// 惯量单位为kg*m^2。
/// 不处理休眠、柔性体或mocap。
/// 不替代U057完整等价入口。
/// 每次调用上传并同步回读。
/// 需要cuda-probe及NVRTC。
pub fn probe_com_position(
    session: &TransferSession,
    model: &InertialModelInput,
    worlds: usize,
    qpos: &[f32],
) -> Result<ComPositionOutput, TransferError> {
    let (fk, layout) = check_com_position(model, worlds, qpos)?;
    #[cfg(not(feature = "cuda-probe"))]
    {
        let _ = (session, fk, layout);
        Err(ProbeError::FeatureDisabled("cuda-probe").into())
    }
    #[cfg(feature = "cuda-probe")]
    {
        com_position_device(session, model, fk, layout, qpos)
            .map(|(_, values)| ComPositionOutput { layout, values })
    }
}

#[cfg(feature = "cuda-probe")]
pub(in crate::physics) fn com_position_device(
    session: &TransferSession,
    model: &InertialModelInput,
    fk: KinematicsLayout,
    layout: ComPositionLayout,
    qpos: &[f32],
) -> Result<(crate::runtime::TransferBuffer<f32>, Vec<f32>), TransferError> {
    // The validated FK output remains allocated on the same GPU session.
    // Its host checkpoint proves finite values; no CPU transform feeds COM.
    let (state, checkpoint) = kinematics_device(session, model, fk, qpos)?;
    drop(checkpoint);
    let k = model.kinematics();
    let device = upload_com_model(session, model, layout)?;
    let mut values = crate::runtime::host_staging::<f32>(layout.guarded)?;
    values.fill(-131072.0);
    values[4..layout.guarded - 4].fill(f32::NAN);
    let mut output = session.upload(&values)?;
    let kernel = crate::runtime::SynchronousKernel::compile(
        session,
        COM_POSITION_CUDA,
        "rigid_com_position",
    )?;
    // SAFETY: Checked layouts bound all packed offsets by i32::MAX.
    // The fixed shader and all physical buffers use f32.
    // Validated topology proves parent order and joint/body/DOF addresses.
    // FK state has precisely 28*nbody+6*njnt elements per world plus guards.
    // Each thread owns one output world, with no cross-thread reads/writes.
    // The same-session adapter waits before any buffer or module can drop.
    unsafe {
        kernel.launch(
            &device.metadata,
            &device.parameters,
            &state,
            &mut output,
            [
                k.nbody() as u32,
                k.njnt() as u32,
                k.nv() as u32,
                layout.output.worlds() as u32,
            ],
        )?;
    }
    output.read_range_into(0, &mut values)?;
    check_device_values(&values, "com_position_output")?;
    Ok((output, values))
}

#[cfg(feature = "cuda-probe")]
pub(in crate::physics) fn upload_com_model(
    session: &TransferSession,
    model: &InertialModelInput,
    layout: ComPositionLayout,
) -> Result<DeviceModel, TransferError> {
    let k = model.kinematics();
    let f = k.fields();
    let i = model.fields();
    let mut metadata = crate::runtime::host_staging::<i32>(layout.metadata)?;
    let mut cursor = 0;
    for field in [&f.body_parentid, &f.jnt_type, &f.jnt_bodyid, &f.jnt_dofadr] {
        metadata[cursor..cursor + field.len()].copy_from_slice(field);
        cursor += field.len();
    }
    let mut parameters = crate::runtime::host_staging::<f32>(layout.parameters)?;
    parameters[..k.nbody()].copy_from_slice(&i.body_mass);
    parameters[k.nbody()..].copy_from_slice(&i.body_inertia);
    Ok(DeviceModel {
        metadata: session.upload(&metadata)?,
        parameters: session.upload(&parameters)?,
    })
}
