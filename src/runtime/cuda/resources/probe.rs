//! 租约资源探针主流程。

use super::super::{LoadedKernel, cuda_error, prepare};
use super::affine::AffineResources;
use super::checks::{Mode, check_alive, check_rejections, reject, validate};
use super::fence::CudaFence;
use super::foreign::{ForeignContext, identity, wrap};
use crate::{
    diagnostics::{ProbeError, ResourceError},
    runtime::{
        GUARD_ELEMENTS, GUARD_VALUE, ProbeBackend, ResourceProbeConfig, ResourceProbeReport,
        completion::{CompletionQueue, CompletionStatus},
        input_values,
        lease::{Access, ViewRequest},
        samples::validate_float_values,
    },
};

pub(in crate::runtime) fn run(
    config: ResourceProbeConfig,
) -> Result<ResourceProbeReport, ProbeError> {
    let (info, program) = prepare(config.kernel_config())?;
    let mut program = Some(program);
    let producer = info
        .context
        .new_stream()
        .map_err(cuda_error("lease-producer"))?;
    let execute = info
        .context
        .new_stream()
        .map_err(cuda_error("lease-execute"))?;
    let consumer = info
        .context
        .new_stream()
        .map_err(cuda_error("lease-consumer"))?;
    let foreign = ForeignContext::new(&info.context)?;
    let view_bytes = config.elements * size_of::<f32>();
    let capacity = (config.elements + 2 * GUARD_ELEMENTS) * size_of::<f32>();
    let view = ViewRequest {
        target: identity(&info.context),
        offset: GUARD_ELEMENTS * size_of::<f32>(),
        bytes: view_bytes,
        layout: 1,
        access: Access::Write,
    };
    let modes = [
        Mode::Wait,
        Mode::Query,
        Mode::DropToken,
        Mode::ForgetToken,
        Mode::DropQueue,
        Mode::HostFailure,
        Mode::Graph,
    ];
    let mut rejected_requests = 0;
    for (epoch, mode) in modes.into_iter().enumerate() {
        let values = input_values(config.elements, epoch);
        let expected_input: Vec<_> = [GUARD_VALUE; GUARD_ELEMENTS]
            .into_iter()
            .chain(values.iter().copied())
            .chain([GUARD_VALUE; GUARD_ELEMENTS])
            .collect();
        let expected_output: Vec<_> = [GUARD_VALUE; GUARD_ELEMENTS]
            .into_iter()
            .chain(values.iter().map(|x| x * 2.0 + 1.0))
            .chain([GUARD_VALUE; GUARD_ELEMENTS])
            .collect();
        let input = wrap(
            producer
                .clone_htod(&expected_input)
                .map_err(cuda_error("lease-upload"))?,
            false,
        )?;
        let output = wrap(
            producer
                .clone_htod(&vec![GUARD_VALUE; expected_output.len()])
                .map_err(cuda_error("lease-output"))?,
            true,
        )?;
        let input_weak = input.owner_weak();
        let output_weak = output.owner_weak();
        let metadata = if config.backend == ProbeBackend::NativePtx {
            None
        } else {
            Some(
                producer
                    .clone_htod(&[config.elements as u32; 2])
                    .map_err(cuda_error("lease-metadata"))?,
            )
        };
        let warmup_output = if matches!(mode, Mode::Graph) {
            let owner = producer
                .clone_htod(&vec![GUARD_VALUE; expected_output.len()])
                .map_err(cuda_error("lease-warmup-output"))?;
            Some(wrap(owner, true)?.lease(view)?)
        } else {
            None
        };
        let ready = producer
            .record_event(None)
            .map_err(cuda_error("lease-producer-ready"))?;
        execute
            .wait(&ready)
            .map_err(cuda_error("lease-consumer-ready"))?;
        let resources = AffineResources {
            graph: None,
            input: input.lease(ViewRequest {
                access: Access::Read,
                ..view
            })?,
            output: output.lease(view)?,
            warmup_output,
            metadata,
            kernel: LoadedKernel {
                function: program.as_ref().unwrap().main.function.clone(),
                shared_memory: program.as_ref().unwrap().main.shared_memory,
            },
            elements: config.elements,
        };
        if matches!(mode, Mode::Wait) {
            rejected_requests += check_rejections(&input, &output, view, capacity, &foreign)?;
        }
        let mut queue = CompletionQueue::new();
        // 队列先接管租约、内核与元数据。
        let token = queue.retain(
            resources,
            CudaFence {
                stream: execute.clone(),
                event: None,
            },
        );
        // 调用方在内核提交前释放句柄。
        drop(input);
        let mut output = Some(output);
        if matches!(mode, Mode::Wait | Mode::Query | Mode::Graph) {
            drop(output.take());
        }
        if matches!(mode, Mode::Graph) {
            // 最后一个任务独立保留内核模块。
            drop(program.take());
        }
        check_alive(&input_weak, true)?;
        check_alive(&output_weak, true)?;
        if queue.query(&token)? {
            return Err(ProbeError::InvalidArgument("未提交任务提前完成"));
        }
        let submitted = (|| {
            let (resources, fence) = queue.retained(&token)?;
            if matches!(mode, Mode::Graph) {
                resources.capture(&execute, config.backend)?;
                resources.graph.as_ref().unwrap().launch()?;
            } else {
                resources.launch(&execute)?;
            }
            if matches!(mode, Mode::HostFailure) {
                // 模拟提交后、记录事件前的宿主错误。
                return Err(ProbeError::InvalidArgument("模拟宿主提交错误"));
            }
            fence.arm()?;
            // 下游流只等待显式完成事件。
            consumer
                .wait(fence.event.as_ref().unwrap())
                .map_err(cuda_error("lease-downstream-wait"))
        })();
        if let Err(error) = submitted {
            queue.fail(&token, error.clone())?;
            if !matches!(mode, Mode::HostFailure)
                || error != ProbeError::InvalidArgument("模拟宿主提交错误")
            {
                return Err(error);
            }
            drop(queue);
            if token.status() != CompletionStatus::Failed(error) {
                return Err(ProbeError::InvalidArgument("提交失败状态丢失"));
            }
            let output = output.take().unwrap();
            reject(output.lease(view), ResourceError::Quarantined)?;
            rejected_requests += 1;
            drop(output);
            check_alive(&input_weak, false)?;
            check_alive(&output_weak, false)?;
            continue;
        }
        if matches!(mode, Mode::Wait | Mode::Query | Mode::Graph) {
            if matches!(mode, Mode::Query) {
                execute
                    .synchronize()
                    .map_err(cuda_error("lease-query-ready"))?;
                if !queue.query(&token)? {
                    return Err(ProbeError::InvalidArgument("已完成事件仍待定"));
                }
                if queue.retained(&token).is_ok() {
                    return Err(ProbeError::InvalidArgument("完成任务再次提交"));
                }
                check_alive(&output_weak, true)?;
            }
            let resources = queue.wait(&token)?;
            validate(
                &consumer,
                resources.input.owner(),
                resources.output.owner(),
                &expected_input,
                &expected_output,
            )?;
            if token.status() != CompletionStatus::Completed {
                return Err(ProbeError::InvalidArgument("完成状态不符"));
            }
            drop(resources);
        } else {
            let output = output.take().unwrap();
            let observer = token.clone();
            match mode {
                Mode::DropToken => drop(token),
                Mode::ForgetToken => std::mem::forget(token),
                _ => (),
            }
            drop(queue);
            if observer.status() != CompletionStatus::Completed {
                return Err(ProbeError::InvalidArgument("释放队列未等待完成"));
            }
            check_alive(&input_weak, false)?;
            let read = output.lease(ViewRequest {
                access: Access::Read,
                ..view
            })?;
            let downloaded = consumer
                .clone_dtoh(read.owner())
                .map_err(cuda_error("lease-drop-download"))?;
            consumer
                .synchronize()
                .map_err(cuda_error("lease-drop-download-wait"))?;
            validate_float_values(&expected_output, &downloaded)?;
            drop(read);
            // 完成后才允许切换布局版本。
            let layout = output.invalidate_layout()?;
            reject(
                output.lease(view),
                ResourceError::StaleLayout {
                    expected: layout,
                    actual: view.layout,
                },
            )?;
            rejected_requests += 1;
            drop(output);
        }
        check_alive(&input_weak, false)?;
        check_alive(&output_weak, false)?;
    }
    drop(foreign);
    drop(program);
    info.context
        .check_err()
        .map_err(cuda_error("lease-final-status"))?;
    Ok(ResourceProbeReport {
        backend: config.backend,
        compiler_revision: match config.backend {
            ProbeBackend::NativePtx => None,
            #[cfg(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe"))]
            _ => Some(crate::runtime::cubecl::REVISION),
            #[cfg(not(any(feature = "cubecl-cpp-probe", feature = "cubecl-llvm-probe")))]
            _ => None,
        },
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        device: config.device,
        gpu_name: info.gpu_name,
        compute_capability: info.capability,
        driver_api_version: info.driver_api,
        elements: config.elements,
        view_bytes,
        output_owner_bytes: capacity,
        completed_submissions: modes.len() - 1,
        failed_submissions: 1,
        rejected_requests,
        graph_replays: 1,
    })
}
