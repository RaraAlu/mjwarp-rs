//! 冻结内核上传、启动与回读。

use super::receipt::validate_input;
use super::{
    FrozenWarpGravityArtifact, FrozenWarpGravityInput, FrozenWarpGravityReadback, cuda_error,
    launcher_error,
};
use crate::diagnostics::ProbeError;
use crate::runtime::launcher::{LaunchBounds3, WarpBounds, WarpLaunch, f32_arg, i32_arg};
use cudarc::driver::{DevicePtr, DevicePtrMut};

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
        let output_capacity = u64::try_from(output.len()).unwrap_or(u64::MAX) * 4;
        let (output_ptr, output_record) = output.device_ptr_mut(stream);
        let mass_batches = input.body_mass.len() / input.nbody;
        let gravcomp_batches = input.body_gravcomp.len() / input.nbody;
        let cap = |elements: usize| u64::try_from(elements).unwrap_or(u64::MAX) * 4;

        let dynamic_bodies = input.nbody - 1;
        let bounds = WarpBounds::Three(
            LaunchBounds3::new([input.worlds, dynamic_bodies, input.nv])
                .map_err(launcher_error("frozen-warp-gravity-bounds"))?,
        );
        let f32a = |data: u64, shape: &[usize], width: u64, capacity: u64| {
            f32_arg(data, shape, width, capacity)
                .map_err(launcher_error("frozen-warp-gravity-array"))
        };
        let i32a = |data: u64, shape: &[usize], capacity: u64| {
            i32_arg(data, shape, capacity).map_err(launcher_error("frozen-warp-gravity-array"))
        };
        let args = [
            f32a(
                gravity_ptr,
                &[input.opt_gravity.len() / 3],
                12,
                cap(gravity.len()),
            )?,
            i32a(parent_ptr, &[input.nbody], cap(parent.len()))?,
            i32a(roots_ptr, &[input.nbody], cap(roots.len()))?,
            f32a(mass_ptr, &[mass_batches, input.nbody], 4, cap(mass.len()))?,
            f32a(
                gravcomp_ptr,
                &[gravcomp_batches, input.nbody],
                4,
                cap(gravcomp.len()),
            )?,
            i32a(dof_ptr, &[input.nv], cap(dof.len()))?,
            i32a(ancestor_ptr, &[input.nbody, input.nv], cap(ancestors.len()))?,
            f32a(
                xipos_ptr,
                &[input.worlds, input.nbody],
                12,
                cap(xipos.len()),
            )?,
            f32a(
                subtree_ptr,
                &[input.worlds, input.nbody],
                12,
                cap(subtree.len()),
            )?,
            f32a(cdof_ptr, &[input.worlds, input.nv], 24, cap(cdof.len()))?,
            f32a(output_ptr, &[input.worlds, input.nv], 4, output_capacity)?,
        ];

        self.handle
            .launch_l2(
                stream,
                &WarpLaunch {
                    bounds,
                    args: &args,
                },
            )
            .map_err(launcher_error("frozen-warp-gravity-launch"))?;
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
