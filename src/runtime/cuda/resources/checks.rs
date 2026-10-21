//! 租约拒绝路径、模式枚举与结果校验。

use super::foreign::ForeignContext;
use crate::{
    diagnostics::{ProbeError, ResourceError},
    runtime::{
        lease::{Access, DeviceIdentity, LeasedBuffer, ViewRequest},
        samples::validate_float_values,
    },
};
use cudarc::driver::{CudaSlice, CudaStream};
use std::sync::Arc;

pub(super) fn reject<T>(
    result: Result<T, ProbeError>,
    expected: ResourceError,
) -> Result<(), ProbeError> {
    match result {
        Err(ProbeError::Resource(actual)) if actual == expected => Ok(()),
        _ => Err(ProbeError::InvalidArgument("资源拒绝结果不符")),
    }
}

pub(super) fn check_rejections(
    input: &LeasedBuffer<CudaSlice<f32>>,
    output: &LeasedBuffer<CudaSlice<f32>>,
    view: ViewRequest,
    capacity: usize,
    foreign: &ForeignContext,
) -> Result<usize, ProbeError> {
    let requests = [
        (
            ViewRequest {
                target: DeviceIdentity {
                    device: view.target.device ^ 1,
                    ..view.target
                },
                ..view
            },
            ResourceError::DeviceMismatch,
        ),
        (
            ViewRequest {
                target: DeviceIdentity {
                    context: foreign.raw,
                    ..view.target
                },
                ..view
            },
            ResourceError::ContextMismatch,
        ),
        (
            ViewRequest { bytes: 0, ..view },
            ResourceError::InvalidRange {
                offset: view.offset,
                bytes: 0,
                capacity,
            },
        ),
        (
            ViewRequest {
                offset: usize::MAX,
                bytes: 4,
                ..view
            },
            ResourceError::InvalidRange {
                offset: usize::MAX,
                bytes: 4,
                capacity,
            },
        ),
        (
            ViewRequest {
                offset: capacity,
                bytes: 4,
                ..view
            },
            ResourceError::InvalidRange {
                offset: capacity,
                bytes: 4,
                capacity,
            },
        ),
        (
            ViewRequest {
                offset: view.offset + 1,
                ..view
            },
            ResourceError::Misaligned,
        ),
        (ViewRequest { bytes: 3, ..view }, ResourceError::Misaligned),
        (
            ViewRequest { layout: 0, ..view },
            ResourceError::StaleLayout {
                expected: 1,
                actual: 0,
            },
        ),
        (view, ResourceError::Conflict),
        (
            ViewRequest {
                access: Access::Read,
                ..view
            },
            ResourceError::Conflict,
        ),
    ];
    let count = requests.len() + 2;
    for (request, expected) in requests {
        reject(output.lease(request), expected)?;
    }
    reject(input.lease(view), ResourceError::ReadOnly)?;
    reject(output.invalidate_layout(), ResourceError::LayoutBusy)?;
    Ok(count)
}

#[derive(Clone, Copy)]
pub(super) enum Mode {
    Wait,
    Query,
    DropToken,
    ForgetToken,
    DropQueue,
    Graph,
    HostFailure,
}

pub(super) fn check_alive(
    weak: &std::sync::Weak<CudaSlice<f32>>,
    alive: bool,
) -> Result<(), ProbeError> {
    if weak.upgrade().is_some() == alive {
        Ok(())
    } else {
        Err(ProbeError::InvalidArgument("资源所有者寿命不符"))
    }
}

pub(super) fn validate(
    consumer: &Arc<CudaStream>,
    input: &CudaSlice<f32>,
    output: &CudaSlice<f32>,
    expected_input: &[f32],
    expected_output: &[f32],
) -> Result<(), ProbeError> {
    let input = consumer
        .clone_dtoh(input)
        .map_err(super::super::cuda_error("lease-download-input"))?;
    let output = consumer
        .clone_dtoh(output)
        .map_err(super::super::cuda_error("lease-download-output"))?;
    consumer
        .synchronize()
        .map_err(super::super::cuda_error("lease-download-wait"))?;
    validate_float_values(expected_input, &input)?;
    validate_float_values(expected_output, &output)
}
