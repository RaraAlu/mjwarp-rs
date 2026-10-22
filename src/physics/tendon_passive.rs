//! 肌腱弹簧与阻尼被动力。
//! 本模块不代表完整被动力阶段。

use crate::diagnostics::InputError;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::TransferError;
use crate::model::{BatchLayout, ParameterBatch};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{TransferBuffer, TransferSession};
use std::sync::Arc;

use super::{TendonVelocityPlan, TendonVelocityView};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TendonPassiveOptions {
    pub disable_spring: bool,
    pub disable_damper: bool,
}

#[derive(Clone, Debug)]
pub struct TendonPassiveParameters {
    pub stiffness: ParameterBatch,
    pub stiffness_poly: ParameterBatch,
    pub damping: ParameterBatch,
    pub damping_poly: ParameterBatch,
    pub length_spring: ParameterBatch,
}

impl TendonPassiveParameters {
    // 默认配置无设备路径；仅cuda-probe读取。
    #[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
    fn fields(&self) -> [&ParameterBatch; 5] {
        [
            &self.stiffness,
            &self.stiffness_poly,
            &self.damping,
            &self.damping_poly,
            &self.length_spring,
        ]
    }
}

// 默认配置无设备路径；仅cuda-probe读取。
#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
struct Spec {
    nv: usize,
    nt: usize,
    max_row: usize,
    offsets: [usize; 5],
    parameter_len: usize,
    parameters: TendonPassiveParameters,
}

// 默认配置无设备路径；仅cuda-probe读取。
#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
fn spec(
    context: (usize, usize, usize, usize, usize),
    parameters: TendonPassiveParameters,
) -> Result<Spec, InputError> {
    let (nv, nt, _nnz, _stride, max_row) = context;
    if nv == 0 || nt == 0 || max_row == 0 {
        return Err(InputError::InvalidDimension {
            field: "tendon_passive_dimensions",
        });
    }
    let widths = [nt, 2 * nt, nt, 2 * nt, 2 * nt];
    let mut offsets = [0; 5];
    let mut parameter_len = 0usize;
    for (index, field) in parameters.fields().iter().enumerate() {
        if field.row_elements() != widths[index] {
            return Err(InputError::LengthMismatch {
                field: "tendon_passive_parameter",
                expected: widths[index],
                actual: field.row_elements(),
            });
        }
        offsets[index] = parameter_len;
        parameter_len = parameter_len
            .checked_add(field.values().len())
            .filter(|&n| n <= i32::MAX as usize)
            .ok_or(InputError::Overflow {
                field: "tendon_passive_parameters",
            })?;
    }
    Ok(Spec {
        nv,
        nt,
        max_row,
        offsets,
        parameter_len,
        parameters,
    })
}

fn layout(worlds: usize, nv: usize) -> Result<BatchLayout, InputError> {
    if worlds == 0
        || nv == 0
        || worlds
            .checked_mul(nv)
            .and_then(|n| n.checked_mul(2))
            .is_none_or(|n| n > (u32::MAX - 255) as usize)
    {
        return Err(InputError::InvalidDimension {
            field: "tendon_passive_work",
        });
    }
    BatchLayout::new(worlds, nv, 4)
}

pub struct TendonPassivePlan<'v, 'p> {
    velocity: &'v TendonVelocityPlan<'p>,
    spec: Arc<Spec>,
    options: TendonPassiveOptions,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    parameters: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    metadata: launcher_abi::TendonPassiveMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    kernels: kernels::TendonPassiveKernels,
}

pub struct TendonPassiveData {
    spec: Arc<Spec>,
    layout: BatchLayout,
    ready: bool,
    #[cfg(feature = "cuda-probe")]
    current: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    staged: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    status: TransferBuffer<f32>,
}

pub struct TendonPassiveView<'a, 'v, 'p> {
    plan: &'a TendonPassivePlan<'v, 'p>,
    velocity: &'a TendonVelocityView<'a, 'p>,
    data: &'a TendonPassiveData,
}

#[derive(Clone, Debug)]
pub struct TendonPassiveSnapshot {
    layout: BatchLayout,
    values: Vec<f32>,
}

impl TendonPassiveSnapshot {
    pub fn worlds(&self) -> usize {
        self.layout.worlds()
    }
    pub fn spring(&self, world: usize) -> Result<&[f32], InputError> {
        let range = self.layout.world_elements(world)?;
        Ok(&self.values[4 + range.start..4 + range.end])
    }
    pub fn damper(&self, world: usize) -> Result<&[f32], InputError> {
        let range = self.layout.world_elements(world)?;
        let offset = 4 + self.layout.total_elements();
        Ok(&self.values[offset + range.start..offset + range.end])
    }
}

#[cfg(feature = "cuda-probe")]
fn read(
    buffer: &TransferBuffer<f32>,
    count: usize,
    field: &'static str,
) -> Result<Vec<f32>, TransferError> {
    let mut values = crate::runtime::host_staging::<f32>(count + 8)?;
    buffer.read_range_into(0, &mut values)?;
    crate::physics::check_device_values(&values, field)?;
    Ok(values)
}

#[path = "tendon_passive/cuda.rs"]
mod cuda;
#[path = "tendon_passive/data.rs"]
mod data;
#[cfg(feature = "cuda-probe")]
#[path = "tendon_passive/kernels.rs"]
mod kernels;
#[cfg(feature = "cuda-probe")]
#[path = "tendon_passive/launcher_abi.rs"]
mod launcher_abi;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "tendon_passive/launcher_tests.rs"]
mod launcher_tests;
#[path = "tendon_passive/plan.rs"]
mod plan;
#[cfg(test)]
#[path = "tendon_passive/tests.rs"]
mod tests;
