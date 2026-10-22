//! G01运动学世界视图、探针与设备上传。

#[cfg(feature = "cuda-probe")]
use super::*;
#[cfg(not(feature = "cuda-probe"))]
use crate::diagnostics::ProbeError;
use crate::diagnostics::{InputError, TransferError};
use crate::model::{BatchLayout, InertialModelInput};
use crate::runtime::TransferSession;

/// 七项GPU结果的只读世界视图。
/// 矩阵按行展开，四元数用wxyz。
#[derive(Clone, Copy, Debug)]
pub struct KinematicsWorld<'a> {
    pub xpos: &'a [f32],
    pub xquat: &'a [f32],
    pub xmat: &'a [f32],
    pub xipos: &'a [f32],
    pub ximat: &'a [f32],
    pub xanchor: &'a [f32],
    pub xaxis: &'a [f32],
}

/// 同步完成后的宿主结果。
/// 宿主不计算任何运动学。
#[derive(Clone, Debug)]
pub struct KinematicsOutput {
    pub(in crate::physics) layout: KinematicsLayout,
    pub(in crate::physics) values: Vec<f32>,
}

impl KinematicsOutput {
    pub fn worlds(&self) -> usize {
        self.layout.output.worlds()
    }
    pub fn nbody(&self) -> usize {
        self.layout.nbody
    }
    pub fn njnt(&self) -> usize {
        self.layout.njnt
    }
    pub fn world(&self, world: usize) -> Result<KinematicsWorld<'_>, InputError> {
        let r = self.layout.output.world_elements(world)?;
        let v = &self.values[r.start + 4..r.end + 4];
        let nb = self.nbody();
        let nj = self.njnt();
        Ok(KinematicsWorld {
            xpos: &v[..3 * nb],
            xquat: &v[3 * nb..7 * nb],
            xmat: &v[7 * nb..16 * nb],
            xipos: &v[16 * nb..19 * nb],
            ximat: &v[19 * nb..28 * nb],
            xanchor: &v[28 * nb..28 * nb + 3 * nj],
            xaxis: &v[28 * nb + 3 * nj..],
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(in crate::physics) struct KinematicsLayout {
    pub(in crate::physics) output: BatchLayout,
    pub(in crate::physics) nbody: usize,
    pub(in crate::physics) njnt: usize,
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) nq: usize,
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) metadata: usize,
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) parameters: usize,
    #[cfg(feature = "cuda-probe")]
    pub(in crate::physics) guarded: usize,
}

impl KinematicsLayout {
    pub(in crate::physics) fn new(
        worlds: usize,
        nq: usize,
        nbody: usize,
        njnt: usize,
    ) -> Result<Self, InputError> {
        if worlds == 0 || worlds > (u32::MAX - 255) as usize || nbody == 0 {
            return Err(InputError::InvalidDimension {
                field: "kinematics_dimensions",
            });
        }
        fn packed(terms: &[(usize, usize)]) -> Result<usize, InputError> {
            let n = terms.iter().try_fold(0usize, |n, &(count, width)| {
                count
                    .checked_mul(width)
                    .and_then(|k| n.checked_add(k))
                    .ok_or(InputError::Overflow {
                        field: "kinematics_layout",
                    })
            })?;
            if n > i32::MAX as usize {
                return Err(InputError::Overflow {
                    field: "kinematics_kernel_index",
                });
            }
            Ok(n)
        }
        let metadata = packed(&[(nbody, 3), (njnt, 2)])?;
        let parameters = packed(&[(nq, 1), (nbody, 14), (njnt, 6)])?;
        let stride = packed(&[(nbody, 28), (njnt, 6)])?;
        let output = BatchLayout::new(worlds, stride, 4)?;
        BatchLayout::new(worlds, nq, 4)?;
        let guarded = output
            .total_elements()
            .checked_add(8)
            .ok_or(InputError::Overflow {
                field: "kinematics_guards",
            })?;
        BatchLayout::new(1, guarded, 4)?;
        #[cfg(not(feature = "cuda-probe"))]
        let _ = (metadata, parameters, guarded);
        Ok(Self {
            output,
            nbody,
            njnt,
            #[cfg(feature = "cuda-probe")]
            nq,
            #[cfg(feature = "cuda-probe")]
            metadata,
            #[cfg(feature = "cuda-probe")]
            parameters,
            #[cfg(feature = "cuda-probe")]
            guarded,
        })
    }
}

pub(in crate::physics) fn norm_squared(values: &[f32]) -> f64 {
    values.iter().map(|&x| f64::from(x).powi(2)).sum()
}

pub(in crate::physics) fn check_kinematics(
    model: &InertialModelInput,
    worlds: usize,
    qpos: &[f32],
) -> Result<KinematicsLayout, InputError> {
    let k = model.kinematics();
    let f = k.fields();
    let layout = KinematicsLayout::new(worlds, k.nq(), k.nbody(), k.njnt())?;
    let expected = worlds.checked_mul(k.nq()).ok_or(InputError::Overflow {
        field: "kinematics_qpos",
    })?;
    if qpos.len() != expected {
        return Err(InputError::LengthMismatch {
            field: "kinematics_qpos",
            expected,
            actual: qpos.len(),
        });
    }
    for (index, value) in qpos.iter().enumerate() {
        if !value.is_finite() {
            return Err(InputError::NonFinite {
                field: "kinematics_qpos",
                index,
            });
        }
    }
    // This strict helper does not silently reinterpret noncanonical world input.
    for (field, values, required) in [
        ("body_pos", &f.body_pos[..3], &[0.0, 0.0, 0.0][..]),
        ("body_quat", &f.body_quat[..4], &[1.0, 0.0, 0.0, 0.0][..]),
        (
            "body_ipos",
            &model.fields().body_ipos[..3],
            &[0.0, 0.0, 0.0][..],
        ),
        (
            "body_iquat",
            &model.fields().body_iquat[..4],
            &[1.0, 0.0, 0.0, 0.0][..],
        ),
    ] {
        if values != required {
            return Err(InputError::InvalidTopology {
                field,
                index: 0,
                reason: "noncanonical_world",
            });
        }
    }
    for (field, values, width) in [
        ("body_quat", &f.body_quat, 4),
        ("body_iquat", &model.fields().body_iquat, 4),
    ] {
        for (index, rotation) in values.chunks_exact(width).enumerate() {
            if (norm_squared(rotation) - 1.0).abs() > 2e-6 {
                return Err(InputError::InvalidTopology {
                    field,
                    index,
                    reason: "nonunit_model_rotation",
                });
            }
        }
    }
    for joint in 0..k.njnt() {
        let ty = f.jnt_type[joint];
        let a = f.jnt_qposadr[joint] as usize;
        if ty >= 2 {
            if (norm_squared(&f.jnt_axis[3 * joint..3 * joint + 3]) - 1.0).abs() > 2e-6 {
                return Err(InputError::InvalidTopology {
                    field: "jnt_axis",
                    index: joint,
                    reason: "nonunit_joint_axis",
                });
            }
        } else {
            let offset = a + if ty == 0 { 3 } else { 0 };
            for world in 0..worlds {
                let start = world * k.nq() + offset;
                let squared = norm_squared(&qpos[start..start + 4]);
                if !(1e-12..=1e12).contains(&squared) {
                    return Err(InputError::InvalidTopology {
                        field: "kinematics_qpos",
                        index: start,
                        reason: "unusable_state_quaternion",
                    });
                }
            }
        }
    }
    Ok(layout)
}

/// G01刚体子集的同步GPU探针。
/// 参数仅共享一份，状态按世界展开。
/// 支持四种关节与静态祖先。
/// 非零状态四元数在GPU归一化。
/// 位置用米，角度用弧度。
/// 只计算七项体与关节字段。
/// 不处理mocap、休眠或柔性体。
/// 不输出geom、site或相机姿态。
/// 不替代U018、U062等价入口。
/// 每次调用上传并同步回读。
/// 需要cuda-probe及NVRTC。
pub fn probe_kinematics(
    session: &TransferSession,
    model: &InertialModelInput,
    worlds: usize,
    qpos: &[f32],
) -> Result<KinematicsOutput, TransferError> {
    let layout = check_kinematics(model, worlds, qpos)?;
    #[cfg(not(feature = "cuda-probe"))]
    {
        let _ = (session, layout);
        Err(ProbeError::FeatureDisabled("cuda-probe").into())
    }
    #[cfg(feature = "cuda-probe")]
    {
        kinematics_device(session, model, layout, qpos)
            .map(|(_, values)| KinematicsOutput { layout, values })
    }
}

#[cfg(feature = "cuda-probe")]
pub(in crate::physics) fn kinematics_device(
    session: &TransferSession,
    model: &InertialModelInput,
    layout: KinematicsLayout,
    qpos: &[f32],
) -> Result<(crate::runtime::TransferBuffer<f32>, Vec<f32>), TransferError> {
    let device = upload_kinematics_model(session, model, layout)?;
    // A real one-element allocation supplies a valid ABI pointer for nq=0.
    let state = session.upload(if qpos.is_empty() { &[0.0] } else { qpos })?;
    let mut values = crate::runtime::host_staging::<f32>(layout.guarded)?;
    values.fill(-131072.0);
    values[4..layout.guarded - 4].fill(f32::NAN);
    let mut output = session.upload(&values)?;
    let kernel =
        crate::runtime::SynchronousKernel::compile(session, KINEMATICS_CUDA, "rigid_kinematics")?;
    rigid_kinematics::RigidKernel::new(session, kernel).launch(
        &device,
        &state,
        &mut output,
        layout.output.worlds() as u32,
    )?;
    output.read_range_into(0, &mut values)?;
    check_device_values(&values, "kinematics_output")?;
    Ok((output, values))
}

#[cfg(feature = "cuda-probe")]
pub(in crate::physics) fn upload_kinematics_model(
    session: &TransferSession,
    model: &InertialModelInput,
    layout: KinematicsLayout,
) -> Result<rigid_kinematics::RigidModel, TransferError> {
    let f = model.kinematics().fields();
    let i = model.fields();
    let mut metadata = crate::runtime::host_staging::<i32>(layout.metadata)?;
    let mut parameters = crate::runtime::host_staging::<f32>(layout.parameters)?;
    let mut cursor = 0;
    for field in [
        &f.body_parentid,
        &f.body_jntadr,
        &f.body_jntnum,
        &f.jnt_type,
        &f.jnt_qposadr,
    ] {
        metadata[cursor..cursor + field.len()].copy_from_slice(field);
        cursor += field.len();
    }
    cursor = 0;
    for field in [
        &f.qpos0,
        &f.body_pos,
        &f.body_quat,
        &i.body_ipos,
        &i.body_iquat,
        &f.jnt_pos,
        &f.jnt_axis,
    ] {
        parameters[cursor..cursor + field.len()].copy_from_slice(field);
        cursor += field.len();
    }
    rigid_kinematics::RigidModel::upload(
        session,
        metadata,
        parameters,
        rigid_kinematics::RigidVariant::Probe,
        [layout.nq as u32, layout.nbody as u32, layout.njnt as u32, 0],
    )
}
