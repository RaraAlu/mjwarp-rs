//! 冻结内核上传、启动与回读。

use super::receipt::validate_input;
use super::{
    FrozenWarpGravityArtifact, FrozenWarpGravityInput, FrozenWarpGravityReadback, WarpArray,
    WarpBounds3, artifact, cuda_error,
};
use crate::diagnostics::ProbeError;
use cudarc::driver::{DevicePtr, DevicePtrMut, LaunchConfig, PushKernelArg};

impl FrozenWarpGravityArtifact {
    pub fn launch(&self, input: &FrozenWarpGravityInput) -> Result<Vec<f32>, ProbeError> {
        self.launch_diagnostic(input)?.into_finite()
    }

    /// 仅供诊断先归档原始位模式。
    /// 调用方随后必须调用`into_finite`。
    pub fn launch_diagnostic(
        &self,
        input: &FrozenWarpGravityInput,
    ) -> Result<FrozenWarpGravityReadback, ProbeError> {
        validate_input(input)?;
        let stream = &self.stream;
        let gravity = stream
            .clone_htod(&input.opt_gravity)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let parent = stream
            .clone_htod(&input.body_parentid)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let roots = stream
            .clone_htod(&input.body_rootid)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let mass = stream
            .clone_htod(&input.body_mass)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let gravcomp = stream
            .clone_htod(&input.body_gravcomp)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let dof = stream
            .clone_htod(&input.dof_bodyid)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let ancestors = stream
            .clone_htod(&input.body_isdofancestor)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let xipos = stream
            .clone_htod(&input.xipos)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let subtree = stream
            .clone_htod(&input.subtree_com)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let cdof = stream
            .clone_htod(&input.cdof)
            .map_err(cuda_error("frozen-warp-gravity-upload"))?;
        let mut output = stream
            .alloc_zeros::<f32>(input.worlds * input.nv)
            .map_err(cuda_error("frozen-warp-gravity-output"))?;

        let (gravity_ptr, gravity_record) = gravity.device_ptr(stream);
        let (parent_ptr, parent_record) = parent.device_ptr(stream);
        let (roots_ptr, roots_record) = roots.device_ptr(stream);
        let (mass_ptr, mass_record) = mass.device_ptr(stream);
        let (gravcomp_ptr, gravcomp_record) = gravcomp.device_ptr(stream);
        let (dof_ptr, dof_record) = dof.device_ptr(stream);
        let (ancestor_ptr, ancestor_record) = ancestors.device_ptr(stream);
        let (xipos_ptr, xipos_record) = xipos.device_ptr(stream);
        let (subtree_ptr, subtree_record) = subtree.device_ptr(stream);
        let (cdof_ptr, cdof_record) = cdof.device_ptr(stream);
        let (output_ptr, output_record) = output.device_ptr_mut(stream);
        let mass_batches = input.body_mass.len() / input.nbody;
        let gravcomp_batches = input.body_gravcomp.len() / input.nbody;
        let descriptors = [
            contiguous(gravity_ptr, &[input.opt_gravity.len() / 3], 12)?,
            contiguous(parent_ptr, &[input.nbody], 4)?,
            contiguous(roots_ptr, &[input.nbody], 4)?,
            contiguous(mass_ptr, &[mass_batches, input.nbody], 4)?,
            contiguous(gravcomp_ptr, &[gravcomp_batches, input.nbody], 4)?,
            contiguous(dof_ptr, &[input.nv], 4)?,
            contiguous(ancestor_ptr, &[input.nbody, input.nv], 4)?,
            contiguous(xipos_ptr, &[input.worlds, input.nbody], 12)?,
            contiguous(subtree_ptr, &[input.worlds, input.nbody], 12)?,
            contiguous(cdof_ptr, &[input.worlds, input.nv], 24)?,
            contiguous(output_ptr, &[input.worlds, input.nv], 4)?,
        ];
        let dynamic_bodies = input.nbody - 1;
        let size = input
            .worlds
            .checked_mul(dynamic_bodies)
            .and_then(|value| value.checked_mul(input.nv))
            .ok_or_else(|| artifact("input", "launch size overflow"))?;
        let work = u32::try_from(size).map_err(|_| artifact("input", "launch overflow"))?;
        let bounds = WarpBounds3 {
            shape: [
                i32::try_from(input.worlds).map_err(|_| artifact("input", "world overflow"))?,
                i32::try_from(dynamic_bodies).map_err(|_| artifact("input", "body overflow"))?,
                i32::try_from(input.nv).map_err(|_| artifact("input", "dof overflow"))?,
            ],
            padding: 0,
            size: u64::from(work),
            coord_mult: 1,
        };
        let mut launch = stream.launch_builder(&self.function);
        launch.arg(&bounds);
        for descriptor in &descriptors {
            launch.arg(descriptor);
        }
        // SAFETY: 验证后的描述符覆盖全部输入。
        // 每线程只原子累加一个输出槽。
        unsafe {
            launch
                .launch(LaunchConfig {
                    grid_dim: (work.div_ceil(256), 1, 1),
                    block_dim: (256, 1, 1),
                    shared_mem_bytes: 0,
                })
                .map_err(cuda_error("frozen-warp-gravity-launch"))?;
        }
        drop(output_record);
        drop(cdof_record);
        drop(subtree_record);
        drop(xipos_record);
        drop(ancestor_record);
        drop(dof_record);
        drop(gravcomp_record);
        drop(mass_record);
        drop(roots_record);
        drop(parent_record);
        drop(gravity_record);
        let values = stream
            .clone_dtoh(&output)
            .map_err(cuda_error("frozen-warp-gravity-readback"))?;
        let bits = values.iter().map(|value| value.to_bits()).collect();
        Ok(FrozenWarpGravityReadback { values, bits })
    }
}

fn contiguous(data: u64, shape: &[usize], element_bytes: usize) -> Result<WarpArray, ProbeError> {
    if data == 0 || !(1..=4).contains(&shape.len()) || shape.contains(&0) {
        return Err(artifact("input", "invalid array shape"));
    }
    let mut array = WarpArray {
        data,
        grad: 0,
        shape: [0; 4],
        strides: [0; 4],
        ndim: shape.len() as u16,
        flags: 0,
        padding: 0,
    };
    let mut stride = element_bytes;
    for axis in (0..shape.len()).rev() {
        array.shape[axis] =
            i32::try_from(shape[axis]).map_err(|_| artifact("input", "shape overflow"))?;
        array.strides[axis] =
            i32::try_from(stride).map_err(|_| artifact("input", "stride overflow"))?;
        stride = stride
            .checked_mul(shape[axis])
            .ok_or_else(|| artifact("input", "stride overflow"))?;
    }
    Ok(array)
}
