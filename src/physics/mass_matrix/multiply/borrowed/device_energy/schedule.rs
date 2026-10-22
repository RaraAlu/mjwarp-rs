//! 常驻 CRB→M→M·qvel→默认32线程动能的整链调度。
//!
//! 调用方只写 qpos 与显式势能；本入口在同一调用内依次执行
//! 位置/CRB（含 M）→ 宿主 qvel 写入乘积缓冲 → M·qvel → 动能归约，
//! 并在返回快照上携带可复算的 A1 来源快照。
//! 任一阶段失败都保留各自旧发布状态，不部分发布能量。

use super::*;
use crate::physics::{MassMatrixData, MassMatrixMultiplyData, MassMatrixPlan};

/// 执行一次完整 CRB→M→M·qvel→动能 调度。
///
/// `qvel` 与 `potential` 均为世界级宿主输入：qvel 长度 `worlds*nv`，
/// potential 长度 `worlds`。来源代际在位置更新前后必须一致，否则拒绝。
#[allow(clippy::too_many_arguments)]
pub fn schedule_energy_chain(
    producer: &MassMatrixPlan,
    source: &mut MassMatrixData,
    multiply: &MassMatrixMultiplyPlan<'_>,
    multiply_data: &mut MassMatrixMultiplyData,
    energy: &MassMatrixEnergyPlan<'_, '_>,
    energy_data: &mut MassMatrixEnergyData,
    qvel: &[f32],
    potential: &[f32],
) -> Result<MassMatrixEnergySnapshot, TransferError> {
    producer.check_data(source)?;
    let before = source.source_state();
    producer.update(source)?;
    // CRB/矩阵更新只消费位置与质心，不改写常驻状态；若来源代际被改写，
    // 说明调用方混用了别处写过的数据，整链不可信。
    if source.source_state() != before {
        return Err(TransferError::StageNotReady {
            stage: "device_energy_chain_source",
        });
    }
    multiply_data.write_vector(qvel)?;
    let matrix_view = multiply.update(multiply_data, source, None)?;
    energy_data.write_potential(potential)?;
    let view = energy.update(energy_data, &matrix_view)?;
    view.snapshot()
}
