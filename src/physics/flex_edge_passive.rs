//! 柔体边弹簧与阻尼被动力。
//! 本模块不代表完整被动力阶段。

use crate::diagnostics::InputError;
#[cfg(feature = "cuda-probe")]
use crate::diagnostics::TransferError;
use crate::model::{BatchLayout, FlexEdgeFields};
#[cfg(feature = "cuda-probe")]
use crate::runtime::{TransferBuffer, TransferSession};
use std::sync::Arc;

use super::{KinematicsData, KinematicsPlan};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FlexEdgePassiveOptions {
    pub disable_spring: bool,
    pub disable_damper: bool,
}

/// 对应冻结上游的四项模型字段。
/// 刚度与阻尼按柔体编号索引。
#[derive(Clone, Debug)]
pub struct FlexEdgePassiveInput {
    pub length0: Vec<f32>,
    pub stiffness: Vec<f32>,
    pub damping: Vec<f32>,
    pub rigid: Vec<bool>,
}

// 默认配置无设备路径；仅cuda-probe读取。
#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
struct Spec {
    nv: usize,
    ne: usize,
    nf: usize,
    nnz: usize,
    stride: usize,
    fields: Arc<FlexEdgeFields>,
    input: FlexEdgePassiveInput,
    flexid: Vec<i32>,
}

// 默认配置无设备路径；仅cuda-probe读取。
#[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
fn spec(
    nv: usize,
    fields: Arc<FlexEdgeFields>,
    input: FlexEdgePassiveInput,
) -> Result<Spec, InputError> {
    let ne = fields.nflexedge();
    let nf = fields.flex_edgeadr.len();
    for (field, actual, expected) in [
        ("flexedge_length0", input.length0.len(), ne),
        ("flex_edgestiffness", input.stiffness.len(), nf),
        ("flex_edgedamping", input.damping.len(), nf),
        ("flexedge_rigid", input.rigid.len(), ne),
    ] {
        if actual != expected {
            return Err(InputError::LengthMismatch {
                field,
                expected,
                actual,
            });
        }
    }
    if nv == 0 || ne == 0 || nf == 0 {
        return Err(InputError::InvalidDimension {
            field: "flex_edge_passive_dimensions",
        });
    }
    let nnz = fields.nnz();
    let stride = ne
        .checked_mul(2)
        .and_then(|n| n.checked_add(nnz))
        .filter(|&n| n <= i32::MAX as usize)
        .ok_or(InputError::Overflow {
            field: "flex_edge_passive_stride",
        })?;
    let mut flexid = vec![-1; ne];
    for f in 0..nf {
        let start = fields.flex_edgeadr[f] as usize;
        let count = fields.flex_edgenum[f] as usize;
        flexid[start..start + count].fill(f as i32);
    }
    if flexid.iter().any(|&f| f < 0) {
        return Err(InputError::InvalidTopology {
            field: "flex_edgeflexid",
            index: 0,
            reason: "unclaimed_edge",
        });
    }
    Ok(Spec {
        nv,
        ne,
        nf,
        nnz,
        stride,
        fields,
        input,
        flexid,
    })
}

fn layout(worlds: usize, nv: usize) -> Result<BatchLayout, InputError> {
    if worlds == 0
        || worlds
            .checked_mul(nv)
            .and_then(|n| n.checked_mul(2))
            .is_none_or(|n| n > (u32::MAX - 255) as usize)
    {
        return Err(InputError::InvalidDimension {
            field: "flex_edge_passive_work",
        });
    }
    BatchLayout::new(worlds, nv, 4)
}

pub struct FlexEdgePassivePlan<'p> {
    source: &'p KinematicsPlan,
    spec: Arc<Spec>,
    options: FlexEdgePassiveOptions,
    #[cfg(feature = "cuda-probe")]
    session: TransferSession,
    #[cfg(feature = "cuda-probe")]
    metadata: launcher_abi::FlexEdgePassiveMetadataBuffer,
    #[cfg(feature = "cuda-probe")]
    parameters: TransferBuffer<f32>,
    #[cfg(feature = "cuda-probe")]
    kernels: kernels::FlexEdgePassiveKernels,
}

pub struct FlexEdgePassiveData {
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

pub struct FlexEdgePassiveView<'a, 'p> {
    // 默认配置无设备路径；仅cuda-probe读取。
    #[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
    plan: &'a FlexEdgePassivePlan<'p>,
    #[cfg_attr(not(feature = "cuda-probe"), allow(dead_code))]
    source: &'a KinematicsData,
    data: &'a FlexEdgePassiveData,
}

#[derive(Clone, Debug)]
pub struct FlexEdgePassiveSnapshot {
    layout: BatchLayout,
    values: Vec<f32>,
}

impl FlexEdgePassiveSnapshot {
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

#[path = "flex_edge_passive/cuda.rs"]
mod cuda;
#[path = "flex_edge_passive/data.rs"]
mod data;
#[cfg(feature = "cuda-probe")]
#[path = "flex_edge_passive/kernels.rs"]
mod kernels;
#[cfg(feature = "cuda-probe")]
#[path = "flex_edge_passive/launcher_abi.rs"]
mod launcher_abi;
#[cfg(all(test, feature = "cuda-probe"))]
#[path = "flex_edge_passive/launcher_tests.rs"]
mod launcher_tests;
#[path = "flex_edge_passive/plan.rs"]
mod plan;
#[cfg(test)]
#[path = "flex_edge_passive/tests.rs"]
mod tests;
