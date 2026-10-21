//! 质量求解探针入口与设备求解。

#[cfg(feature = "cuda-probe")]
use super::buffer::RigidMassSolveMetadataBuffer;
#[cfg(feature = "cuda-probe")]
use super::errors::rigid_mass_solve_launch_error;
#[cfg(feature = "cuda-probe")]
use super::kernel::RigidMassSolveKernel;
#[cfg(feature = "cuda-probe")]
use super::validate::pack_rigid_mass_solve_blocks;
use super::*;

/// 同步GPU分块因子与多右端项求解。
/// rhs采用`[world,rhs,nv]`布局。
/// GPU复用已检查的质量矩阵。
/// 主元必须严格为正且有限。
/// 接口不修补主元或回退CPU。
/// 零自由度允许空向量结果。
/// 单个失败世界阻止整个结果发布。
/// 单自由度采用倒数，至64维采用Cholesky。
/// 更大树保留稠密反向LDL辅助。
/// 不复制上游tile调度或多自由度紧凑优化。
/// 不替代U060、U065与完整G03。
pub fn probe_mass_solve(
    session: &TransferSession,
    model: &InertialModelInput,
    worlds: usize,
    qpos: &[f32],
    rhs_count: usize,
    rhs: &[f32],
) -> Result<MassSolveOutput, TransferError> {
    // Complete input checks precede any device submission.
    crate::physics::check_com_position(model, worlds, qpos)?;
    let layout = MassSolveLayout::new(worlds, model.kinematics().nv(), rhs_count)?;
    layout.check_rhs(rhs)?;
    #[cfg(not(feature = "cuda-probe"))]
    {
        let _ = session;
        Err(ProbeError::FeatureDisabled("cuda-probe").into())
    }
    #[cfg(feature = "cuda-probe")]
    {
        let blocks = factor_blocks(&model.fields().dof_parentid);
        check_block_intervals(&model.fields().dof_parentid, &blocks)?;
        let (state, checkpoint) =
            crate::physics::mass_matrix::mass_matrix_device(session, model, worlds, qpos)?;
        drop(checkpoint);
        solve_device(
            session,
            model.kinematics().nbody(),
            layout,
            &state,
            rhs,
            blocks,
        )
    }
}

#[cfg(feature = "cuda-probe")]
pub(super) fn solve_device(
    session: &TransferSession,
    nbody: usize,
    layout: MassSolveLayout,
    state: &crate::runtime::TransferBuffer<f32>,
    rhs: &[f32],
    blocks: Vec<MassFactorBlock>,
) -> Result<MassSolveOutput, TransferError> {
    let packed =
        pack_rigid_mass_solve_blocks(&blocks, layout.nv).map_err(rigid_mass_solve_launch_error)?;
    let abi = [
        u32::try_from(nbody).map_err(|_| InputError::Overflow {
            field: "mass_solve_nbody",
        })?,
        u32::try_from(layout.nv).map_err(|_| InputError::Overflow {
            field: "mass_solve_nv",
        })?,
        u32::try_from(layout.rhs_count).map_err(|_| InputError::Overflow {
            field: "mass_solve_rhs_count",
        })?,
        u32::try_from(layout.output.worlds()).map_err(|_| InputError::Overflow {
            field: "mass_solve_worlds",
        })?,
    ];
    let metadata = RigidMassSolveMetadataBuffer::upload(session, packed, abi)?;
    // Preserve a real pointer for the empty zero-DOF vector.
    let parameters = session.upload(if rhs.is_empty() { &[0.0] } else { rhs })?;
    let mut values = crate::runtime::host_staging::<f32>(layout.guarded)?;
    values.fill(-131072.0);
    values[4..layout.guarded - 4].fill(f32::NAN);
    let mut output = session.upload(&values)?;
    let kernel = RigidMassSolveKernel::compile(session)?;
    let launch = kernel.preflight(session, &metadata, &parameters, state, &output)?;
    kernel.launch(&launch, &metadata, &parameters, state, &mut output)?;
    output.read_range_into(0, &mut values)?;
    check_result(layout, &values)?;
    Ok(MassSolveOutput {
        layout,
        values,
        blocks,
    })
}

#[cfg(any(feature = "cuda-probe", test))]
pub(super) fn check_result<T: Copy + Into<f64>>(
    layout: MassSolveLayout,
    values: &[T],
) -> Result<(), TransferError> {
    // Failure statuses may precede uninitialized payload. Check guards first,
    // then decode explicit statuses before requiring the whole payload finite.
    crate::physics::check_device_guards(values)?;
    for world in 0..layout.output.worlds() {
        let r = layout.output.world_elements(world)?;
        let s: [f64; 3] = [
            values[r.start + 4].into(),
            values[r.start + 5].into(),
            values[r.start + 6].into(),
        ];
        if s[0] == 0.0 && s[1] == -1.0 && s[2] == 0.0 {
            continue;
        }
        let dof = s[1] as usize;
        if !s[1].is_finite() || s[1] != dof as f64 || dof >= layout.nv {
            return Err(ProbeError::InvalidArgument("GPU求解状态无效").into());
        }
        if s[0] == 1.0 {
            return Err(TransferError::InvalidPivot {
                world,
                dof,
                value: s[2],
            });
        }
        if s[0] == 2.0 && (!s[2].is_finite() || s[2] > f64::from(f32::MAX)) {
            return Err(InputError::NonFinite {
                field: "mass_solve_diagonal_inverse",
                index: world * layout.nv + dof,
            }
            .into());
        }
        return Err(ProbeError::InvalidArgument("GPU求解状态无效").into());
    }
    crate::physics::check_device_values(values, "mass_solve_output")
}

// 冻结smooth.py提供Cholesky与反向LDL公式。
// 本辅助按树分块；每线程处理一个世界。
// Copyright 2025 The Newton Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
